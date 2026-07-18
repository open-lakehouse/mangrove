//! Axum router for the UC Delta REST API (`/delta/v1/...`).
//!
//! The router is **state-agnostic and host-composable**. It is generic over the
//! host's axum state `S` and returns an *unstated* [`Router<S>`], so a host can
//! `.nest`/`.merge` it into its own `Router<S>` tree *before* calling
//! `.with_state` — even when the host applies auth / request-metadata middleware
//! at the outer layer. The handler is captured (type-erased as
//! `Arc<dyn DeltaApiHandler<Cx>>`) rather than being made the axum state, and the
//! per-request context `Cx` is produced by a host-supplied [`ContextExtractor`]
//! rather than an axum `FromRequestParts` impl. This removes both the
//! `Cx: FromRequestParts<T>` coupling and the fully-stated-`Router<()>` composition
//! barrier (see issue #135).
//!
//! Three layers of ergonomics, all building on the same primitive:
//!
//! - [`router_with_context_at`] / [`router_with_context`] — the primitive: you
//!   supply the handler and a [`ContextExtractor`] closure.
//! - [`router_from_extension_at`] / [`router_from_extension`] — convenience for the
//!   common case where the host's middleware installs the context as a request
//!   extension; the extractor just clones it back out. Zero glue.
//! - [`get_router`] — the simplest entry point: extension-sourced context, fixed
//!   `/delta/v1` base, and a fully-stated `Router<()>` for hosts whose whole tree
//!   is `Router<()>`.
//!
//! Compose with plain `.nest`/`.merge` — **not** `.nest_service`, which mounts a
//! `Service` behind an implicit `{*rest}` wildcard that can conflict with a host's
//! sibling wildcard nests.

// The `ContextExtractor` error variant is an axum `Response` by design (a
// short-circuit HTTP response, e.g. 401). It is deliberately un-boxed to keep the
// public `ContextExtractor<Cx>` alias — which hosts also name — simple; the
// `Response` is moved at most once per request, never in a hot loop, so the size
// lint does not apply meaningfully here.
#![allow(clippy::result_large_err)]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use axum::extract::{FromRequest, FromRequestParts, Path, Query, Request};
use axum::http::StatusCode;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use axum::routing::{Router, get, post};
use serde::Deserialize;
use serde::de::DeserializeOwned;

use crate::backend::{SchemaRef, TableRef};
use crate::handler::{DeltaApiHandler, GetConfigQuery};
use crate::models::*;

/// Produces the per-request context `Cx` from the request head.
///
/// Called once per request with the request [`Parts`]; the returned future
/// resolves to `Ok(cx)` to proceed or `Err(response)` to short-circuit with an
/// HTTP response (e.g. 401). The closure is stored in an [`Arc`] and cloned once
/// per request, so it must be `Send + Sync + 'static`.
///
/// The extractor is **async** so a host can build `Cx` from data that is only
/// reachable through an async axum extractor — most notably a matched URL path
/// segment via [`Path`](axum::extract::Path) /
/// [`RawPathParams`](axum::extract::RawPathParams), whose `FromRequestParts`
/// impls are `async`. Extractors sourcing `Cx` from a request extension can
/// return a ready future (`async move { Ok(..) }`). See issue #142.
pub type ContextExtractor<Cx> = Arc<
    dyn for<'a> Fn(&'a mut Parts) -> Pin<Box<dyn Future<Output = Result<Cx, Response>> + Send + 'a>>
        + Send
        + Sync
        + 'static,
>;

/// Build a [`Router<S>`] for the Delta REST API over an arbitrary host state `S`.
///
/// The returned router is **not** stated — the host applies `.with_state` after
/// composing it into its own tree. `base` is the path prefix every route is
/// mounted under (e.g. `"/delta/v1"`); pass `""` for relative routes suitable for
/// a host `.nest("/delta/v1", ..)`. `base` must not have a trailing slash.
///
/// Routes match `openapi/delta.yaml`.
pub fn router_with_context_at<S, Cx>(
    base: &str,
    handler: Arc<dyn DeltaApiHandler<Cx>>,
    extract_cx: ContextExtractor<Cx>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Cx: Send + 'static,
{
    // Each route captures its own clones of the two `Arc`s. `route!` builds a
    // closure handler that (1) extracts the context, (2) extracts the inputs named
    // by the arm (path / query / body, in axum-extraction order), (3) calls the
    // handler method, and (4) serializes via `IntoResponse`. The double clone (once
    // into the outer closure so it is `Clone`, once inside so each produced future
    // owns `'static` `Arc`s) is the canonical axum pattern for capturing shared
    // state in closure handlers without `State`.
    //
    // The arms are distinct token shapes (`cx`, `cx path`, `cx query`,
    // `cx path query`, `cx path body`) rather than optional fragments, to avoid
    // macro-parse ambiguity between the path and query captures.
    macro_rules! route {
        // Shared closure prologue: clone the Arcs, split the request, bind the
        // handler and context under the caller's own identifiers ($h, $cx, $parts,
        // $body) so the arm body (which references them) shares hygiene context.
        (@enter $method:expr; $h:ident; $cx:ident; $parts:ident; $body:ident; $build:block) => {{
            let handler = handler.clone();
            let extract_cx = extract_cx.clone();
            $method(move |req: Request| {
                let handler = handler.clone();
                let extract_cx = extract_cx.clone();
                async move {
                    #[allow(unused_mut, unused_variables)]
                    let (mut $parts, $body) = req.into_parts();
                    let $cx = match extract_cx(&mut $parts).await {
                        Ok(cx) => cx,
                        Err(resp) => return resp,
                    };
                    let $h = handler;
                    $build
                }
            })
        }};
        // context only
        ($method:expr; |$h:ident, $cx:ident| $call:expr) => {
            route!(@enter $method; $h; $cx; parts; body; { into_response($call.await) })
        };
        // context + query
        ($method:expr; query $query_ty:ty; |$h:ident, $cx:ident, $q:ident| $call:expr) => {
            route!(@enter $method; $h; $cx; parts; body; {
                let $q = match extract_query::<$query_ty>(&mut parts).await { Ok(q) => q, Err(resp) => return resp };
                into_response($call.await)
            })
        };
        // context + path
        ($method:expr; path $path_ty:ty; |$h:ident, $cx:ident, $p:ident| $call:expr) => {
            route!(@enter $method; $h; $cx; parts; body; {
                let $p = match extract_path::<$path_ty>(&mut parts).await { Ok(p) => p, Err(resp) => return resp };
                into_response($call.await)
            })
        };
        // context + path + query
        ($method:expr; path $path_ty:ty; query $query_ty:ty; |$h:ident, $cx:ident, $p:ident, $q:ident| $call:expr) => {
            route!(@enter $method; $h; $cx; parts; body; {
                let $p = match extract_path::<$path_ty>(&mut parts).await { Ok(p) => p, Err(resp) => return resp };
                let $q = match extract_query::<$query_ty>(&mut parts).await { Ok(q) => q, Err(resp) => return resp };
                into_response($call.await)
            })
        };
        // context + path + JSON body
        ($method:expr; path $path_ty:ty; body $body_ty:ty; |$h:ident, $cx:ident, $p:ident, $b:ident| $call:expr) => {
            route!(@enter $method; $h; $cx; parts; body; {
                let $p = match extract_path::<$path_ty>(&mut parts).await { Ok(p) => p, Err(resp) => return resp };
                let $b = match extract_json::<$body_ty>(Request::from_parts(parts, body)).await { Ok(b) => b, Err(resp) => return resp };
                into_response($call.await)
            })
        };
    }

    let p = |suffix: &str| format!("{base}{suffix}");

    // The `/tables/{table}` path fans out to GET/POST/DELETE/HEAD; build its
    // method router by chaining the four method handlers.
    let table_methods = {
        let load = route!(get; path TableRef; |h, cx, path| h.load_table(path, cx));
        load.post(
            route!(post; path TableRef; body DeltaUpdateTableRequest; |h, cx, path, body| h.update_table(path, body, cx)),
        )
        .delete(route!(axum::routing::delete; path TableRef; |h, cx, path| h.delete_table(path, cx)))
        .head(route!(axum::routing::head; path TableRef; |h, cx, path| h.table_exists(path, cx)))
    };

    Router::new()
        .route(
            &p("/config"),
            route!(get; query GetConfigParams; |h, cx, params| {
                let query = GetConfigQuery { catalog: params.catalog, protocol_versions: params.protocol_versions };
                h.get_config(query, cx)
            }),
        )
        .route(
            &p("/catalogs/{catalog}/schemas/{schema}/staging-tables"),
            route!(post; path SchemaRef; body DeltaCreateStagingTableRequest; |h, cx, path, body| h.create_staging_table(path, body, cx)),
        )
        .route(
            &p("/catalogs/{catalog}/schemas/{schema}/tables"),
            route!(post; path SchemaRef; body DeltaCreateTableRequest; |h, cx, path, body| h.create_table(path, body, cx)),
        )
        .route(&p("/catalogs/{catalog}/schemas/{schema}/tables/{table}"), table_methods)
        .route(
            &p("/catalogs/{catalog}/schemas/{schema}/tables/{table}/rename"),
            route!(post; path TableRef; body DeltaRenameTableRequest; |h, cx, path, body| h.rename_table(path, body, cx)),
        )
        .route(
            &p("/catalogs/{catalog}/schemas/{schema}/tables/{table}/credentials"),
            route!(get; path TableRef; query OperationParam; |h, cx, path, params| h.get_table_credentials(path, params.operation, cx)),
        )
        .route(
            &p("/catalogs/{catalog}/schemas/{schema}/tables/{table}/metrics"),
            route!(post; path TableRef; body DeltaReportMetricsRequest; |h, cx, path, body| h.report_metrics(path, body, cx)),
        )
        .route(
            &p("/staging-tables/{table_id}/credentials"),
            route!(get; path String; |h, cx, table_id| h.get_staging_table_credentials(table_id, cx)),
        )
        .route(
            &p("/temporary-path-credentials"),
            route!(get; query PathCredentialParams; |h, cx, params| h.get_temporary_path_credentials(params.location, params.operation, cx)),
        )
}

/// [`router_with_context_at`] with the default `"/delta/v1"` base.
pub fn router_with_context<S, Cx>(
    handler: Arc<dyn DeltaApiHandler<Cx>>,
    extract_cx: ContextExtractor<Cx>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Cx: Send + 'static,
{
    router_with_context_at("/delta/v1", handler, extract_cx)
}

/// [`router_with_context_at`] for hosts whose middleware installs the context as a
/// request extension.
///
/// The context is read out of `parts.extensions` and cloned; a request that
/// reaches the router without the extension installed is rejected with a 401. Use
/// this when a host layer inserts `Cx` (e.g. an authenticated `RequestMetadata`)
/// into the extensions before routing. `base` follows the same rules as
/// [`router_with_context_at`].
pub fn router_from_extension_at<S, Cx>(
    base: &str,
    handler: Arc<dyn DeltaApiHandler<Cx>>,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Cx: Clone + Send + Sync + 'static,
{
    let extract_cx: ContextExtractor<Cx> = Arc::new(|parts: &mut Parts| {
        let cx = parts.extensions.get::<Cx>().cloned().ok_or_else(|| {
            crate::error::DeltaApiError::unauthenticated("missing request context").into_response()
        });
        Box::pin(async move { cx })
    });
    router_with_context_at(base, handler, extract_cx)
}

/// [`router_from_extension_at`] with the default `"/delta/v1"` base.
pub fn router_from_extension<S, Cx>(handler: Arc<dyn DeltaApiHandler<Cx>>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
    Cx: Clone + Send + Sync + 'static,
{
    router_from_extension_at("/delta/v1", handler)
}

/// Create a fully-stated [`Router`] for the Delta REST API, mounted under
/// `/delta/v1`, with the per-request context read from the request extensions.
///
/// This is the simplest entry point, for a host whose entire router tree is
/// `Router<()>` (so it can `.merge` a stated router directly). It is equivalent to
/// `router_from_extension::<(), Cx>(Arc::new(handler)).with_state(())`. A host that
/// carries its own axum state, or that builds `Cx` from something other than a
/// single extension, should use [`router_with_context`] /
/// [`router_from_extension`] instead.
pub fn get_router<T, Cx>(handler: T) -> Router
where
    T: DeltaApiHandler<Cx> + 'static,
    Cx: Clone + Send + Sync + 'static,
{
    router_from_extension::<(), Cx>(Arc::new(handler)).with_state(())
}

// ----- Extraction helpers ----------------------------------------------------

/// Extract typed path parameters. `Path`/`Query`/`Json` ignore the state type, so
/// we always pass `&()` regardless of the router's host state `S`.
async fn extract_path<P: DeserializeOwned + Send>(parts: &mut Parts) -> Result<P, Response> {
    Path::<P>::from_request_parts(parts, &())
        .await
        .map(|Path(p)| p)
        .map_err(IntoResponse::into_response)
}

async fn extract_query<Q: DeserializeOwned + Send>(parts: &mut Parts) -> Result<Q, Response> {
    Query::<Q>::from_request_parts(parts, &())
        .await
        .map(|Query(q)| q)
        .map_err(IntoResponse::into_response)
}

async fn extract_json<B: DeserializeOwned>(req: Request) -> Result<B, Response> {
    axum::Json::<B>::from_request(req, &())
        .await
        .map(|axum::Json(b)| b)
        .map_err(IntoResponse::into_response)
}

/// Map a handler `Result` to an axum [`Response`]. `Ok(())` (void operations)
/// becomes `204 No Content`; other `Ok(v)` values are JSON-serialized; `Err` uses
/// [`DeltaApiError`](crate::error::DeltaApiError)'s `IntoResponse`.
fn into_response<T: DeltaResponse>(result: crate::error::DeltaApiResult<T>) -> Response {
    match result {
        Ok(v) => v.into_delta_response(),
        Err(e) => e.into_response(),
    }
}

/// How a handler's `Ok` payload maps to an HTTP response body.
trait DeltaResponse {
    fn into_delta_response(self) -> Response;
}

impl DeltaResponse for () {
    fn into_delta_response(self) -> Response {
        StatusCode::NO_CONTENT.into_response()
    }
}

macro_rules! json_response {
    ($($ty:ty),+ $(,)?) => {$(
        impl DeltaResponse for $ty {
            fn into_delta_response(self) -> Response {
                axum::Json(self).into_response()
            }
        }
    )+};
}

json_response!(
    DeltaCatalogConfig,
    DeltaStagingTableResponse,
    DeltaLoadTableResponse,
    DeltaCredentialsResponse,
);

// ----- Query parameter deserialization helpers -------------------------------

#[derive(Debug, Deserialize)]
struct GetConfigParams {
    catalog: String,
    #[serde(rename = "protocol-versions")]
    protocol_versions: String,
}

#[derive(Debug, Deserialize)]
struct OperationParam {
    operation: DeltaCredentialOperation,
}

#[derive(Debug, Deserialize)]
struct PathCredentialParams {
    location: String,
    operation: DeltaCredentialOperation,
}
