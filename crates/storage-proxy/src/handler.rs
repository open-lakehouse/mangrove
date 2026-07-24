//! The streaming HTTP handler: a blanket [`StorageProxyHandler`] over any
//! [`StorageProxyBackend`].
//!
//! This is where all the wire semantics live — `Range` → `206`/`Content-Range`,
//! `ETag`/`Accept-Ranges` relay, `If-Match` → conditional `PUT` → `412`, and
//! **bounded-memory streaming** in both directions. A backend implementer never
//! touches any of it; they only authorize and open a store.
//!
//! # Memory
//! - `GET`/`HEAD` stream the object store's [`GetResult`] straight into the axum
//!   response body via [`axum::body::Body::from_stream`]; the whole object is
//!   never materialized.
//! - `PUT` with **no** `If-Match` streams the request body through a
//!   [`WriteMultipart`], flushing fixed-size parts — bounded regardless of object
//!   size.
//! - `PUT` **with** `If-Match` is an inherently single-request conditional write
//!   on the cloud side, so the body is buffered under a hard cap
//!   ([`MAX_BUFFERED_PUT`]); this path is for the small `_delta_log` commit files
//!   the wasm write path conditionally writes.

use axum::body::Body;
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use object_store::{
    GetOptions, GetRange, ObjectStoreExt, PutMode, PutOptions, PutPayload, UpdateVersion,
    WriteMultipart,
};

use crate::backend::{ProxyReq, StorageProxyBackend};
use crate::error::{ProxyError, ProxyResult};

/// Hard cap on the body buffered for a **conditional** (`If-Match`) `PUT`. A
/// conditional write is a single cloud request, so the body cannot be streamed as
/// multipart; the cap bounds proxy memory. Sized generously for Delta commit files
/// (the only conditional writes the wasm path issues) while rejecting a runaway
/// upload with `413`.
pub const MAX_BUFFERED_PUT: usize = 256 * 1024 * 1024;

/// The multipart part size for streaming (unconditional) `PUT`s. Matches
/// `object_store`'s default; the proxy never buffers more than one part.
const MULTIPART_CHUNK_SIZE: usize = 5 * 1024 * 1024;

/// The HTTP-facing surface produced by the blanket impl below. A host router
/// dispatches to these three methods; each returns a fully-formed axum
/// [`Response`] (streaming for `GET`), or a [`ProxyError`] the router turns into a
/// response.
#[async_trait::async_trait]
pub trait StorageProxyHandler<Cx>: Send + Sync + 'static {
    /// Serve a (optionally ranged) body read.
    async fn get(&self, req: ProxyReq, cx: Cx) -> ProxyResult<Response>;
    /// Serve a metadata-only read.
    async fn head(&self, req: ProxyReq, cx: Cx) -> ProxyResult<Response>;
    /// Serve a whole-object write, consuming the request `body`.
    async fn put(&self, req: ProxyReq, body: Body, cx: Cx) -> ProxyResult<Response>;
}

#[async_trait::async_trait]
impl<B, Cx> StorageProxyHandler<Cx> for B
where
    B: StorageProxyBackend<Cx>,
    Cx: Send + 'static,
{
    async fn get(&self, req: ProxyReq, cx: Cx) -> ProxyResult<Response> {
        self.authorize(&req, &cx).await?;
        let store = self.open(&req, &cx).await?;

        let opts = GetOptions {
            range: req.range.clone(),
            ..Default::default()
        };
        let path = object_store::path::Path::from(req.key.as_str());
        let res = store.get_opts(&path, opts).await?;

        // Read metadata + effective range *before* consuming the payload stream.
        let total = res.meta.size;
        let etag = res.meta.e_tag.clone();
        let effective = res.range.clone();
        let content_len = effective.end.saturating_sub(effective.start);
        let ranged = req.range.is_some();

        let mut headers = base_read_headers(content_len, etag.as_deref());
        let status = if ranged {
            headers.insert(
                header::CONTENT_RANGE,
                header::HeaderValue::from_str(&format!(
                    "bytes {}-{}/{}",
                    effective.start,
                    effective.end.saturating_sub(1),
                    total
                ))
                .expect("ascii content-range"),
            );
            StatusCode::PARTIAL_CONTENT
        } else {
            StatusCode::OK
        };

        // Stream the object bytes lazily — the whole object is never buffered.
        let stream = res
            .into_stream()
            .map(|chunk| chunk.map_err(std::io::Error::other));
        let body = Body::from_stream(stream);
        Ok((status, headers, body).into_response())
    }

    async fn head(&self, req: ProxyReq, cx: Cx) -> ProxyResult<Response> {
        self.authorize(&req, &cx).await?;
        let store = self.open(&req, &cx).await?;

        // `head: true` fetches metadata only.
        let opts = GetOptions {
            head: true,
            ..Default::default()
        };
        let path = object_store::path::Path::from(req.key.as_str());
        let res = store.get_opts(&path, opts).await?;

        let headers = base_read_headers(res.meta.size, res.meta.e_tag.as_deref());
        // No body on HEAD.
        Ok((StatusCode::OK, headers, Body::empty()).into_response())
    }

    async fn put(&self, req: ProxyReq, body: Body, cx: Cx) -> ProxyResult<Response> {
        self.authorize(&req, &cx).await?;
        let store = self.open(&req, &cx).await?;
        let path = object_store::path::Path::from(req.key.as_str());

        let put_result = match &req.if_match {
            // Conditional write: an inherently single-request cloud operation.
            // `If-Match: *` has no `object_store` equivalent — reject explicitly.
            Some(v) if v.trim() == "*" => {
                return Err(ProxyError::InvalidArgument(
                    "`If-Match: *` is not supported; send a concrete ETag".into(),
                ));
            }
            Some(etag) => {
                let payload = buffer_body(body, MAX_BUFFERED_PUT).await?;
                let opts = PutOptions {
                    mode: PutMode::Update(UpdateVersion {
                        e_tag: Some(etag.clone()),
                        version: None,
                    }),
                    ..Default::default()
                };
                // A mismatch surfaces as `object_store::Error::Precondition` →
                // `ProxyError::Storage` → 412 (see error.rs).
                store.put_opts(&path, payload, opts).await?
            }
            // Unconditional write: stream through multipart, bounded memory.
            None => {
                let upload = store.put_multipart(&path).await?;
                let mut writer = WriteMultipart::new_with_chunk_size(upload, MULTIPART_CHUNK_SIZE);
                let mut stream = body.into_data_stream();
                while let Some(chunk) = stream.next().await {
                    let chunk =
                        chunk.map_err(|e| ProxyError::Internal(format!("body read: {e}")))?;
                    writer.write(&chunk);
                }
                writer.finish().await.map_err(ProxyError::Storage)?
            }
        };

        let mut headers = HeaderMap::new();
        if let Some(etag) = put_result.e_tag
            && let Ok(v) = header::HeaderValue::from_str(&etag)
        {
            headers.insert(header::ETAG, v);
        }
        // `object_store` does not distinguish created-vs-overwritten, so return 200
        // uniformly; the wasm `HttpStore` only reads the ETag.
        Ok((StatusCode::OK, headers).into_response())
    }
}

/// The headers common to `GET` (full or ranged) and `HEAD`: `Content-Length`,
/// `Accept-Ranges: bytes`, and `ETag` when present.
fn base_read_headers(content_length: u64, etag: Option<&str>) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_LENGTH,
        header::HeaderValue::from_str(&content_length.to_string()).expect("ascii content-length"),
    );
    headers.insert(
        header::ACCEPT_RANGES,
        header::HeaderValue::from_static("bytes"),
    );
    if let Some(etag) = etag
        && let Ok(v) = header::HeaderValue::from_str(etag)
    {
        headers.insert(header::ETAG, v);
    }
    headers
}

/// Drain a request body into a [`PutPayload`], failing with `413` if it exceeds
/// `cap`. Used only for conditional (`If-Match`) writes, which cannot be streamed.
async fn buffer_body(body: Body, cap: usize) -> ProxyResult<PutPayload> {
    let mut stream = body.into_data_stream();
    let mut buf = BytesMut::new();
    while let Some(chunk) = stream.next().await {
        let chunk: Bytes = chunk.map_err(|e| ProxyError::Internal(format!("body read: {e}")))?;
        if buf.len() + chunk.len() > cap {
            return Err(ProxyError::PayloadTooLarge(format!(
                "conditional PUT body exceeds the {cap}-byte cap"
            )));
        }
        buf.extend_from_slice(&chunk);
    }
    Ok(PutPayload::from(buf.freeze()))
}

/// Parse an HTTP `Range` header value into an [`object_store::GetRange`].
///
/// Supports the single-range forms `bytes=a-b` ([`GetRange::Bounded`]),
/// `bytes=a-` ([`GetRange::Offset`]), and `bytes=-n` ([`GetRange::Suffix`]).
/// Returns `Ok(None)` for an absent header. A malformed or multi-range value is
/// rejected with [`ProxyError::InvalidArgument`].
pub fn parse_range(headers: &HeaderMap) -> ProxyResult<Option<GetRange>> {
    let Some(value) = headers.get(header::RANGE) else {
        return Ok(None);
    };
    let value = value
        .to_str()
        .map_err(|_| ProxyError::InvalidArgument("non-ASCII Range header".into()))?;
    let spec = value
        .strip_prefix("bytes=")
        .ok_or_else(|| ProxyError::InvalidArgument(format!("unsupported Range unit: {value}")))?;
    if spec.contains(',') {
        return Err(ProxyError::InvalidArgument(
            "multi-range requests are not supported".into(),
        ));
    }
    let (start, end) = spec
        .split_once('-')
        .ok_or_else(|| ProxyError::InvalidArgument(format!("malformed Range: {value}")))?;
    let parse_u64 = |s: &str| {
        s.trim()
            .parse::<u64>()
            .map_err(|_| ProxyError::InvalidArgument(format!("malformed Range number in: {value}")))
    };
    let range = match (start.trim(), end.trim()) {
        ("", "") => {
            return Err(ProxyError::InvalidArgument(format!("empty Range: {value}")));
        }
        // bytes=-n  -> last n bytes
        ("", n) => GetRange::Suffix(parse_u64(n)?),
        // bytes=a-  -> from offset a
        (a, "") => GetRange::Offset(parse_u64(a)?),
        // bytes=a-b -> [a, b] inclusive -> [a, b+1) half-open
        (a, b) => {
            let a = parse_u64(a)?;
            let b = parse_u64(b)?;
            if b < a {
                return Err(ProxyError::InvalidArgument(format!(
                    "Range end before start: {value}"
                )));
            }
            GetRange::Bounded(a..b + 1)
        }
    };
    Ok(Some(range))
}

/// Extract the `If-Match` header value (if any) for a conditional `PUT`.
pub fn parse_if_match(headers: &HeaderMap) -> ProxyResult<Option<String>> {
    match headers.get(header::IF_MATCH) {
        None => Ok(None),
        Some(v) => Ok(Some(
            v.to_str()
                .map_err(|_| ProxyError::InvalidArgument("non-ASCII If-Match header".into()))?
                .to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hm(name: header::HeaderName, value: &str) -> HeaderMap {
        let mut h = HeaderMap::new();
        h.insert(name, header::HeaderValue::from_str(value).unwrap());
        h
    }

    #[test]
    fn range_bounded() {
        let r = parse_range(&hm(header::RANGE, "bytes=2-5"))
            .unwrap()
            .unwrap();
        assert_eq!(r, GetRange::Bounded(2..6));
    }

    #[test]
    fn range_offset_and_suffix() {
        assert_eq!(
            parse_range(&hm(header::RANGE, "bytes=100-"))
                .unwrap()
                .unwrap(),
            GetRange::Offset(100)
        );
        assert_eq!(
            parse_range(&hm(header::RANGE, "bytes=-500"))
                .unwrap()
                .unwrap(),
            GetRange::Suffix(500)
        );
    }

    #[test]
    fn range_absent_is_none() {
        assert!(parse_range(&HeaderMap::new()).unwrap().is_none());
    }

    #[test]
    fn range_rejects_bad() {
        assert!(parse_range(&hm(header::RANGE, "items=0-1")).is_err());
        assert!(parse_range(&hm(header::RANGE, "bytes=0-1,4-5")).is_err());
        assert!(parse_range(&hm(header::RANGE, "bytes=5-2")).is_err());
        assert!(parse_range(&hm(header::RANGE, "bytes=--")).is_err());
    }

    #[test]
    fn if_match_roundtrip() {
        assert_eq!(
            parse_if_match(&hm(header::IF_MATCH, "\"etag123\"")).unwrap(),
            Some("\"etag123\"".to_string())
        );
        assert!(parse_if_match(&HeaderMap::new()).unwrap().is_none());
    }
}
