//! Request validation middleware.
//!
//! Provides:
//! - [`ValidationLayer`] / [`ValidationMiddleware`]: Tower middleware that rejects requests with
//!   a `Content-Length` header exceeding a fixed maximum body size (1 MiB) with HTTP 413.
//! - [`max_results_clamp`]: Utility that clamps an optional `max_results` query parameter to a
//!   caller-supplied maximum, converting from `i32` to `usize`. Use this in list handlers instead
//!   of the raw `.map(|v| v as usize)` pattern to enforce per-endpoint upper bounds.
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures_util::FutureExt;
use futures_util::future::BoxFuture;
use tower::{Layer, Service};

/// Maximum allowed request body size (1 MiB).
const MAX_BODY_SIZE: usize = 1024 * 1024;

// ---------------------------------------------------------------------------
// Public utility
// ---------------------------------------------------------------------------

/// Clamp `max_results` to at most `max`, converting `i32` to `usize`.
///
/// Returns `None` when `value` is `None` (i.e. the caller did not supply the parameter).
/// Negative values of `value` are treated as `0` via the unsigned cast.
///
/// # Example
///
/// ```
/// use unitycatalog_server::rest::max_results_clamp;
///
/// assert_eq!(max_results_clamp(None, 200), None);
/// assert_eq!(max_results_clamp(Some(10), 200), Some(10));
/// assert_eq!(max_results_clamp(Some(500), 200), Some(200));
/// ```
pub fn max_results_clamp(value: Option<i32>, max: usize) -> Option<usize> {
    value.map(|v| (v as usize).min(max))
}

// ---------------------------------------------------------------------------
// Middleware
// ---------------------------------------------------------------------------

/// Middleware that rejects requests whose `Content-Length` header exceeds `MAX_BODY_SIZE`.
///
/// Only the `Content-Length` header is inspected — the body itself is not buffered. This makes
/// the check cheap but means a client that omits the header (or lies about it) will not be
/// caught here. Pair with a body-size limiting layer (e.g. `tower_http::limit::RequestBodyLimitLayer`)
/// for full protection.
#[derive(Clone)]
pub struct ValidationMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for ValidationMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        // Check Content-Length before forwarding the request.
        if let Some(content_length) = req
            .headers()
            .get(axum::http::header::CONTENT_LENGTH)
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<usize>().ok())
            && content_length > MAX_BODY_SIZE
        {
            let response = (
                StatusCode::PAYLOAD_TOO_LARGE,
                format!(
                    "Request body too large: {} bytes (limit: {} bytes)",
                    content_length, MAX_BODY_SIZE
                ),
            )
                .into_response();
            return async { Ok(response) }.boxed();
        }

        self.inner.call(req).boxed()
    }
}

// ---------------------------------------------------------------------------
// Layer
// ---------------------------------------------------------------------------

/// [`Layer`] that applies [`ValidationMiddleware`] to a service.
#[derive(Clone, Default)]
pub struct ValidationLayer;

impl ValidationLayer {
    /// Create a new [`ValidationLayer`].
    pub fn new() -> Self {
        Self
    }
}

impl<S> Layer<S> for ValidationLayer {
    type Service = ValidationMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ValidationMiddleware { inner }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use tower::{ServiceBuilder, ServiceExt};

    use super::*;

    async fn echo(_req: Request<Body>) -> Result<Response<Body>, std::convert::Infallible> {
        Ok(Response::new(Body::empty()))
    }

    #[tokio::test]
    async fn test_allows_small_body() {
        let mut svc = ServiceBuilder::new()
            .layer(ValidationLayer::new())
            .service_fn(echo);

        let req = Request::post("/")
            .header(header::CONTENT_LENGTH, "100")
            .body(Body::empty())
            .unwrap();

        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_rejects_oversized_body() {
        let mut svc = ServiceBuilder::new()
            .layer(ValidationLayer::new())
            .service_fn(echo);

        let req = Request::post("/")
            .header(header::CONTENT_LENGTH, (MAX_BODY_SIZE + 1).to_string())
            .body(Body::empty())
            .unwrap();

        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn test_allows_missing_content_length() {
        let mut svc = ServiceBuilder::new()
            .layer(ValidationLayer::new())
            .service_fn(echo);

        let req = Request::post("/").body(Body::empty()).unwrap();
        let resp = svc.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[test]
    fn test_max_results_clamp() {
        assert_eq!(max_results_clamp(None, 200), None);
        assert_eq!(max_results_clamp(Some(10), 200), Some(10));
        assert_eq!(max_results_clamp(Some(200), 200), Some(200));
        assert_eq!(max_results_clamp(Some(500), 200), Some(200));
        assert_eq!(max_results_clamp(Some(0), 200), Some(0));
    }
}
