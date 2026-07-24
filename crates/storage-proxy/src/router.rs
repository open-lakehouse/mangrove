//! Axum router for the storage byte-proxy (`/storage-proxy/{securable}/{*key}`).
//!
//! Mirrors the state-agnostic, host-composable shape of the Delta API router: the
//! router is generic over the host's axum state `S` and returns an *unstated*
//! [`Router<S>`], the handler is type-erased behind an [`Arc`], and the per-request
//! context `Cx` is produced by a host-supplied [`ContextExtractor`] rather than an
//! axum `FromRequestParts` impl — so a host can `.nest`/`.merge` the surface into
//! its own tree before `.with_state`, even under its own auth middleware.
//!
//! Unlike the Delta router (JSON bodies), the proxy emits **streaming** responses
//! and consumes a raw request body on `PUT`, so the route closures build the axum
//! [`Response`] directly rather than serializing a DTO.

#![allow(clippy::result_large_err)]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::body::Body;
use axum::extract::{FromRequestParts, Path, Request};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::{Router, get, head, put};

use crate::backend::{ProxyReq, ProxyVerb, Securable};
use crate::handler::{StorageProxyHandler, parse_if_match, parse_range};

/// Produces the per-request context `Cx` from the request head.
///
/// Called once per request with the request [`Parts`]; the returned future
/// resolves to `Ok(cx)` to proceed or `Err(response)` to short-circuit (e.g. a
/// 401). Stored in an [`Arc`] and cloned once per request, so it must be
/// `Send + Sync + 'static`. Async so a host can build `Cx` from an async axum
/// extractor.
pub type ContextExtractor<Cx> = Arc<
    dyn for<'a> Fn(&'a mut Parts) -> Pin<Box<dyn Future<Output = Result<Cx, Response>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

/// Build a [`Router<S>`] for the storage proxy over an arbitrary host state `S`.
///
/// The returned router is **not** stated — the host applies `.with_state` after
/// composing it. `base` is the path prefix every route is mounted under (e.g.
/// `"/storage-proxy"`); pass `""` for relative routes suitable for a host
/// `.nest("/storage-proxy", ..)`. `base` must not have a trailing slash.
pub fn router_with_context_at<S, Cx>(
    base: &str,
    handler: Arc<dyn StorageProxyHandler<Cx>>,
    extract_cx: ContextExtractor<Cx>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Cx: Send + 'static,
{
    // Each verb captures its own clones of the two `Arc`s. The prologue splits the
    // request into parts + body, runs the context extractor, parses the securable
    // and headers, and dispatches to the streaming handler method.
    macro_rules! route {
        ($verb:expr, $method:ident) => {{
            let handler = handler.clone();
            let extract_cx = extract_cx.clone();
            $method(move |req: Request| {
                let handler = handler.clone();
                let extract_cx = extract_cx.clone();
                async move {
                    let (mut parts, body) = req.into_parts();
                    let cx = match extract_cx(&mut parts).await {
                        Ok(cx) => cx,
                        Err(resp) => return resp,
                    };
                    dispatch(&*handler, $verb, &mut parts, body, cx).await
                }
            })
        }};
    }

    let p = |suffix: &str| format!("{base}{suffix}");
    Router::new().route(
        &p("/{securable}/{*key}"),
        route!(ProxyVerb::Get, get)
            .head(route!(ProxyVerb::Head, head))
            .put(route!(ProxyVerb::Put, put)),
    )
}

/// [`router_with_context_at`] with the default `"/storage-proxy"` base.
pub fn router_with_context<S, Cx>(
    handler: Arc<dyn StorageProxyHandler<Cx>>,
    extract_cx: ContextExtractor<Cx>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Cx: Send + 'static,
{
    router_with_context_at("/storage-proxy", handler, extract_cx)
}

/// [`router_with_context_at`] for hosts whose middleware installs the context as a
/// request extension. The context is read out of `parts.extensions` and cloned; a
/// request without it is rejected 401.
pub fn router_from_extension_at<S, Cx>(
    base: &str,
    handler: Arc<dyn StorageProxyHandler<Cx>>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Cx: Clone + Send + Sync + 'static,
{
    let extract_cx: ContextExtractor<Cx> = Arc::new(|parts: &mut Parts| {
        let cx = parts.extensions.get::<Cx>().cloned().ok_or_else(|| {
            crate::error::ProxyError::Unauthenticated("missing request context".into())
                .into_response()
        });
        Box::pin(async move { cx })
    });
    router_with_context_at(base, handler, extract_cx)
}

/// [`router_from_extension_at`] with the default `"/storage-proxy"` base.
pub fn router_from_extension<S, Cx>(handler: Arc<dyn StorageProxyHandler<Cx>>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Cx: Clone + Send + Sync + 'static,
{
    router_from_extension_at("/storage-proxy", handler)
}

/// Parse the path + headers into a [`ProxyReq`] and dispatch to the handler for the
/// given verb. Errors (bad securable, bad Range/If-Match) short-circuit to a
/// response.
async fn dispatch<Cx>(
    handler: &dyn StorageProxyHandler<Cx>,
    verb: ProxyVerb,
    parts: &mut Parts,
    body: Body,
    cx: Cx,
) -> Response
where
    Cx: Send + 'static,
{
    // `Path<(securable, key)>` — the `{*key}` wildcard preserves slashes in the key.
    let Path((securable_seg, key)) =
        match Path::<(String, String)>::from_request_parts(parts, &()).await {
            Ok(p) => p,
            Err(rej) => return rej.into_response(),
        };

    let securable = match Securable::parse(&securable_seg) {
        Ok(s) => s,
        Err(e) => return e.into_response(),
    };

    let (range, if_match) = match verb {
        ProxyVerb::Get => match parse_range(&parts.headers) {
            Ok(r) => (r, None),
            Err(e) => return e.into_response(),
        },
        ProxyVerb::Head => (None, None),
        ProxyVerb::Put => match parse_if_match(&parts.headers) {
            Ok(m) => (None, m),
            Err(e) => return e.into_response(),
        },
    };

    let req = ProxyReq {
        verb,
        securable,
        key,
        range,
        if_match,
    };

    let result = match verb {
        ProxyVerb::Get => handler.get(req, cx).await,
        ProxyVerb::Head => handler.head(req, cx).await,
        ProxyVerb::Put => handler.put(req, body, cx).await,
    };
    match result {
        Ok(resp) => resp,
        Err(e) => e.into_response(),
    }
}
