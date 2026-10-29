//! The Open Sharing server as a Node-API addon.
//!
//! The Neon Function's handler (`function/index.ts`) calls [`start`] once,
//! forwards every fetch `Request` through [`handle`], and calls [`shutdown`] on
//! SIGINT. Requests never touch a socket: [`handle`] drives the in-process axum
//! router (`olai-sharing-server`'s `embed::start`) with `oneshot`.
//!
//! Sharing is pure request/response, so unlike a service with background work
//! nothing needs settling between requests; an isolate can be evicted at any
//! time.

use std::sync::{Mutex, OnceLock};

use axum::Router;
use axum::body::Body;
use http_body_util::BodyExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use open_sharing_server::Config;
use open_sharing_server::embed;
use tower::ServiceExt;

/// The router, `None` before [`start`] and after [`shutdown`].
static ROUTER: Mutex<Option<Router>> = Mutex::new(None);
static TRACING: OnceLock<()> = OnceLock::new();

#[napi(object)]
pub struct Header {
    pub name: String,
    pub value: String,
}

#[napi(object)]
pub struct JsRequest {
    pub method: String,
    /// Path and query, e.g. `/api/v1/delta-sharing/shares?maxResults=10`.
    pub url: String,
    pub headers: Vec<Header>,
    pub body: Buffer,
}

#[napi(object)]
pub struct JsResponse {
    pub status: u16,
    pub headers: Vec<Header>,
    pub body: Buffer,
}

fn to_napi(e: impl std::fmt::Display) -> napi::Error {
    napi::Error::from_reason(e.to_string())
}

/// Load the configuration, connect the store and verify its schema.
///
/// Configuration comes from the environment (`SHARING_SERVER_CONFIG_YAML` or
/// `SHARING_SERVER_CONFIG`, plus `DATABASE_URL` / `DATABASE_URL_UNPOOLED`, which
/// Neon sets for the branch: the pooled URL serves requests, the direct one the
/// schema check). Migrations are never applied here. Idempotent.
#[napi]
pub async fn start() -> napi::Result<()> {
    TRACING.get_or_init(|| {
        let filter =
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    });
    if ROUTER.lock().map_err(to_napi)?.is_some() {
        return Ok(());
    }
    let config = Config::from_env().map_err(to_napi)?;
    let service = embed::start(&config).await.map_err(to_napi)?;
    *ROUTER.lock().map_err(to_napi)? = Some(service.router);
    tracing::info!("sharing server started");
    Ok(())
}

/// Serve one request through the router and buffer the whole response.
#[napi]
pub async fn handle(req: JsRequest) -> napi::Result<JsResponse> {
    let router = ROUTER
        .lock()
        .map_err(to_napi)?
        .clone()
        .ok_or_else(|| to_napi("the sharing server is not started"))?;

    let mut builder = axum::http::Request::builder()
        .method(req.method.as_str())
        .uri(req.url.as_str());
    for h in &req.headers {
        builder = builder.header(h.name.as_str(), h.value.as_str());
    }
    let request = builder
        .body(Body::from(req.body.to_vec()))
        .map_err(to_napi)?;

    let response = router.oneshot(request).await.map_err(to_napi)?;
    let (parts, body) = response.into_parts();
    let body = body.collect().await.map_err(to_napi)?.to_bytes();
    let headers = parts
        .headers
        .iter()
        .filter_map(|(name, value)| {
            Some(Header {
                name: name.to_string(),
                value: value.to_str().ok()?.to_string(),
            })
        })
        .collect();
    Ok(JsResponse {
        status: parts.status.as_u16(),
        headers,
        body: body.to_vec().into(),
    })
}

/// Stop serving. Dropping the router closes the store's connection pool.
#[napi]
pub async fn shutdown() -> napi::Result<()> {
    ROUTER.lock().map_err(to_napi)?.take();
    Ok(())
}
