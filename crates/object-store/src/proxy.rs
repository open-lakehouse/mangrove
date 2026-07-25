//! Same-origin storage byte-proxy client (wasm-only).
//!
//! When a Unity Catalog server announces `storageAccess: "proxy"` from its
//! `/capabilities` endpoint, the browser engine must not read/write cloud
//! storage directly (that path is CORS-gated on the storage host and cannot
//! speak S3/GCP protocols from wasm). Instead it routes every byte through the
//! server's `/storage-proxy/{securable}/{key}` surface, which authorizes,
//! vends a scoped credential, and streams the object server-side.
//!
//! The client store is the stock [`object_store::http::HttpStore`] pointed at
//! the proxy base, with a Bearer token attached via
//! [`ClientOptions::with_default_headers`]. No cloud protocol runs in the
//! browser, so S3/GCP work through the proxy on wasm even though their native
//! `object_store` backends are gated out (`Azure-first` on wasm).
//!
//! ## Key mapping ([`StripPrefixStore`])
//!
//! The server's proxy handler returns a store already **prefixed at the
//! securable root**, so it expects object keys *relative to the table/volume
//! root* (e.g. `_delta_log/00.json`). But the read path registers the store on
//! a [`RoutingObjectStore`](../../olai_delta_df) that forwards the **full
//! bucket-rooted path** unchanged (e.g. `container/prefix/_delta_log/00.json`).
//! [`StripPrefixStore`] bridges the two: it strips the known securable prefix
//! from every incoming location before delegating to the inner `HttpStore`, so
//! the wire request is `/storage-proxy/{securable}/_delta_log/00.json`. The
//! write path (`files/engine.rs`) already addresses relative to the volume
//! root, so its prefix is empty and the wrapper is a passthrough.

use std::sync::Arc;

use futures::StreamExt;
use object_store::path::Path;
use object_store::{
    CopyOptions, GetOptions, GetResult, ListResult, MultipartUpload, ObjectMeta, ObjectStore,
    PutMultipartOptions, PutOptions, PutPayload, PutResult, Result as OsResult,
};
use url::Url;

#[cfg(target_arch = "wasm32")]
use crate::error::Error;

/// The storage-access posture the server announces, resolved once when the
/// factory is built.
///
/// Only constructed on `wasm32` (native storage access is always direct), but
/// the type is defined unconditionally so `lib.rs`'s wasm-gated field and this
/// module stay in one place.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum StorageAccess {
    /// The browser reads/writes cloud storage directly with a vended
    /// credential (CORS-gated on the storage host). The historical default.
    #[default]
    Direct,
    /// The browser routes every byte through the server's storage byte-proxy
    /// at `base` (a fully-qualified `.../storage-proxy` URL, no trailing slash).
    Proxy { base: Url },
}

/// Discover the server's storage-access posture from its `/capabilities`
/// endpoint.
///
/// `uc_uri` is the UC REST base (e.g.
/// `https://host/api/2.1/unity-catalog/`); `/capabilities` is served at the
/// server root, so we resolve it against the origin. `token`, when set, is
/// sent as a Bearer credential (the endpoint may be behind auth).
///
/// Any failure — network error, non-2xx, unparseable body — resolves to
/// [`StorageAccess::Direct`]: an older server without the endpoint, or one that
/// does not run the proxy, keeps the direct behavior. Discovery never blocks
/// table access on a capabilities hiccup.
#[cfg(target_arch = "wasm32")]
pub(crate) async fn discover_storage_access(uc_uri: &Url, token: Option<&str>) -> StorageAccess {
    match try_discover(uc_uri, token).await {
        Ok(access) => access,
        // Treat any discovery failure as "direct": never fail table access on a
        // capabilities probe.
        Err(_) => StorageAccess::Direct,
    }
}

#[cfg(target_arch = "wasm32")]
async fn try_discover(uc_uri: &Url, token: Option<&str>) -> Result<StorageAccess, Error> {
    // `/capabilities` is a server-root endpoint, not under the UC API path.
    let caps_url = uc_uri
        .join("/capabilities")
        .map_err(|e| Error::invalid_url(format!("capabilities url: {e}")))?;

    let mut req = reqwest::Client::new().get(caps_url.clone());
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let resp = req
        .send()
        .await
        .map_err(|e| Error::invalid_config(format!("capabilities request failed: {e}")))?;
    if !resp.status().is_success() {
        return Err(Error::invalid_config(format!(
            "capabilities returned {}",
            resp.status()
        )));
    }
    let body: CapabilitiesBody = resp
        .json()
        .await
        .map_err(|e| Error::invalid_config(format!("capabilities body: {e}")))?;
    Ok(body.into_storage_access(uc_uri))
}

/// The `/capabilities` response the server emits (see
/// `crates/server/src/run.rs`): `{"storageAccess":"proxy",
/// "storageProxy":{"basePath":"/storage-proxy","conditionalWrites":true}}` or
/// `{"storageAccess":"direct"}`.
#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize)]
struct CapabilitiesBody {
    #[serde(rename = "storageAccess", default)]
    storage_access: String,
    #[serde(rename = "storageProxy", default)]
    storage_proxy: Option<StorageProxyBody>,
}

#[cfg(target_arch = "wasm32")]
#[derive(serde::Deserialize)]
struct StorageProxyBody {
    #[serde(rename = "basePath", default)]
    base_path: Option<String>,
}

#[cfg(target_arch = "wasm32")]
impl CapabilitiesBody {
    fn into_storage_access(self, uc_uri: &Url) -> StorageAccess {
        if self.storage_access != "proxy" {
            return StorageAccess::Direct;
        }
        // Default base path matches the server's mount; honor an override.
        let base_path = self
            .storage_proxy
            .and_then(|p| p.base_path)
            .unwrap_or_else(|| "/storage-proxy".to_string());
        // Resolve against the origin (the mount is server-root, not under the
        // UC API path). Trim a trailing slash so callers append `/{securable}/`.
        match uc_uri.join(&base_path) {
            Ok(base) => StorageAccess::Proxy {
                base: trim_trailing_slash(base),
            },
            // A base path that will not resolve is unusable; fall back to direct.
            Err(_) => StorageAccess::Direct,
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn trim_trailing_slash(mut url: Url) -> Url {
    let trimmed = url.path().trim_end_matches('/').to_string();
    url.set_path(&trimmed);
    url
}

/// Percent-encode a raw cloud URL into the `path:` securable identifier the
/// proxy router expects. Matches `Securable::parse` in
/// `crates/storage-proxy/src/backend.rs`.
///
/// Called from the wasm `for_path` proxy branch; exercised by native tests.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn path_securable(url: &Url) -> String {
    // The whole URL becomes one path segment; encode `/`, `:`, and other
    // reserved characters so it stays a single segment through routing.
    format!("path:{}", encode_segment(url.as_str()))
}

/// Percent-encode a string for use as a single URL path segment.
///
/// Encodes everything that is not an unreserved character (`A-Za-z0-9-._~`),
/// including `/` and `:`, so a cloud URL survives as one `{securable}` segment.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn encode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// An [`ObjectStore`] that strips a fixed path `prefix` from every request
/// location before delegating to `inner`.
///
/// This is the inverse of [`object_store::prefix::PrefixStore`] (which *adds* a
/// prefix). The proxy's inner [`HttpStore`](object_store::http::HttpStore) is
/// rooted at `.../storage-proxy/{securable}/`, whose keys are relative to the
/// securable root — but the read path forwards bucket-rooted paths
/// (`<prefix>/_delta_log/x`). Stripping `prefix` yields the securable-relative
/// key the proxy endpoint expects.
///
/// An empty `prefix` makes this a passthrough (the write path already addresses
/// relative to the volume root).
///
/// Constructed only on `wasm32` (by `build_proxy_store`); exercised by native
/// unit tests, so it is defined unconditionally.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
#[derive(Debug)]
pub(crate) struct StripPrefixStore {
    prefix: Path,
    inner: Arc<dyn ObjectStore>,
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
impl StripPrefixStore {
    /// Wrap `inner`, stripping `prefix` from every location. When `prefix` is
    /// empty the returned store forwards locations unchanged.
    pub(crate) fn new(prefix: Path, inner: Arc<dyn ObjectStore>) -> Self {
        Self { prefix, inner }
    }

    /// Strip `self.prefix` from `location`. A location outside the prefix is
    /// returned unchanged — the routing layer only ever forwards in-prefix
    /// paths, so this is defensive rather than a silent escape.
    fn strip(&self, location: &Path) -> Path {
        strip_path(&self.prefix, location)
    }
}

impl std::fmt::Display for StripPrefixStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "StripPrefixStore({}, {})", self.prefix, self.inner)
    }
}

#[async_trait::async_trait]
impl ObjectStore for StripPrefixStore {
    async fn put_opts(
        &self,
        location: &Path,
        payload: PutPayload,
        opts: PutOptions,
    ) -> OsResult<PutResult> {
        self.inner
            .put_opts(&self.strip(location), payload, opts)
            .await
    }

    async fn put_multipart_opts(
        &self,
        location: &Path,
        opts: PutMultipartOptions,
    ) -> OsResult<Box<dyn MultipartUpload>> {
        self.inner
            .put_multipart_opts(&self.strip(location), opts)
            .await
    }

    async fn get_opts(&self, location: &Path, options: GetOptions) -> OsResult<GetResult> {
        self.inner.get_opts(&self.strip(location), options).await
    }

    fn delete_stream(
        &self,
        locations: futures::stream::BoxStream<'static, OsResult<Path>>,
    ) -> futures::stream::BoxStream<'static, OsResult<Path>> {
        // Strip on the way in (so the inner store deletes the right key) and
        // re-prefix on the way out (so callers see the location they passed).
        // Only the owned `prefix` is captured, so both streams are `'static`.
        let prefix = self.prefix.clone();
        let stripped = locations
            .map(move |loc| loc.map(|p| strip_path(&prefix, &p)))
            .boxed();
        let prefix_out = self.prefix.clone();
        self.inner
            .delete_stream(stripped)
            .map(move |res| res.map(|p| prepend_path(&prefix_out, &p)))
            .boxed()
    }

    fn list(
        &self,
        prefix: Option<&Path>,
    ) -> futures::stream::BoxStream<'static, OsResult<ObjectMeta>> {
        // Listing is not part of the proxy contract (the Delta read path never
        // lists; volume browsing uses the ConnectRPC FilesService). Delegate
        // with the stripped prefix so the store is still a well-formed
        // ObjectStore; the underlying HttpStore returns an error for the
        // WebDAV path the proxy does not serve.
        let mapped = prefix.map(|p| self.strip(p));
        self.inner.list(mapped.as_ref())
    }

    async fn list_with_delimiter(&self, prefix: Option<&Path>) -> OsResult<ListResult> {
        let mapped = prefix.map(|p| self.strip(p));
        self.inner.list_with_delimiter(mapped.as_ref()).await
    }

    async fn copy_opts(&self, from: &Path, to: &Path, options: CopyOptions) -> OsResult<()> {
        self.inner
            .copy_opts(&self.strip(from), &self.strip(to), options)
            .await
    }
}

/// Strip `prefix` from `location` (free function form used inside stream
/// closures where `&self` cannot be captured across the `'static` boundary).
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn strip_path(prefix: &Path, location: &Path) -> Path {
    if prefix.parts().count() == 0 {
        return location.clone();
    }
    match location.prefix_match(prefix) {
        Some(rest) => Path::from_iter(rest),
        None => location.clone(),
    }
}

/// Re-prepend `prefix` to a securable-relative `location`.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn prepend_path(prefix: &Path, location: &Path) -> Path {
    if prefix.parts().count() == 0 {
        return location.clone();
    }
    Path::from_iter(prefix.parts().chain(location.parts()))
}

/// Build a proxy-backed [`ObjectStore`] for a securable, rooted so that
/// bucket-rooted `prefix`-relative keys map onto
/// `{base}/{securable}/{key}`.
///
/// - `base` is the resolved proxy base (`.../storage-proxy`, no trailing slash).
/// - `securable_seg` is the typed `{securable}` segment (`table:<fqn>`,
///   `vol:<fqn>`, or `path:<pct-encoded url>`).
/// - `prefix` is the securable's bucket-relative path, stripped from incoming
///   locations by [`StripPrefixStore`]. Pass an empty [`Path`] for a store
///   whose callers already address relative to the securable root.
/// - `token`, when set, is sent as a Bearer credential on every request.
#[cfg(target_arch = "wasm32")]
pub(crate) fn build_proxy_store(
    base: &Url,
    securable_seg: &str,
    prefix: Path,
    token: Option<&str>,
) -> OsResult<Arc<dyn ObjectStore>> {
    use object_store::ClientOptions;
    use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

    // `{base}/{securable}/` — the HttpStore treats its URL as the root; keys are
    // appended beneath it. The securable segment is one path segment (its inner
    // `/` and `:` are percent-encoded for `path:` securables).
    let store_url = format!("{}/{}/", base.as_str(), securable_seg);

    let mut client_options = ClientOptions::default();
    if let Some(token) = token {
        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(&format!("Bearer {token}")) {
            headers.insert(AUTHORIZATION, value);
        }
        client_options = client_options.with_default_headers(headers);
    }

    // The fork's default `ReqwestConnector` + its wasm `HttpService for
    // reqwest::Client` (which forces `fetch_cache_no_store`) handle the browser
    // Fetch transport; no custom connector needed. Retries are disabled: the
    // wasm build has no timer for the retry backoff sleep.
    let http = object_store::http::HttpBuilder::new()
        .with_url(store_url)
        .with_client_options(client_options)
        .with_retry(object_store::RetryConfig {
            max_retries: 0,
            ..Default::default()
        })
        .build()?;

    Ok(Arc::new(StripPrefixStore::new(prefix, Arc::new(http))))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_securable_encodes_url_as_one_segment() {
        let url = Url::parse("s3://bucket/prefix/").unwrap();
        // Matches `Securable::parse`'s `path:<percent-encoded url>`.
        assert_eq!(path_securable(&url), "path:s3%3A%2F%2Fbucket%2Fprefix%2F");
    }

    #[test]
    fn encode_segment_leaves_unreserved_and_escapes_reserved() {
        assert_eq!(encode_segment("aZ0-._~"), "aZ0-._~");
        assert_eq!(encode_segment("a/b:c"), "a%2Fb%3Ac");
    }

    #[test]
    fn strip_removes_known_prefix() {
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let store = StripPrefixStore::new(Path::from("container/tbl"), inner);
        assert_eq!(
            store.strip(&Path::from("container/tbl/_delta_log/00.json")),
            Path::from("_delta_log/00.json")
        );
    }

    #[test]
    fn strip_empty_prefix_is_passthrough() {
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let store = StripPrefixStore::new(Path::default(), inner);
        assert_eq!(
            store.strip(&Path::from("data/part-0.parquet")),
            Path::from("data/part-0.parquet")
        );
    }

    #[test]
    fn strip_leaves_out_of_prefix_path_unchanged() {
        let inner: Arc<dyn ObjectStore> = Arc::new(object_store::memory::InMemory::new());
        let store = StripPrefixStore::new(Path::from("container/tbl"), inner);
        // Defensive: a path that is not under the prefix is not mangled.
        assert_eq!(
            store.strip(&Path::from("other/thing")),
            Path::from("other/thing")
        );
    }
}
