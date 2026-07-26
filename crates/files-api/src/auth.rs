//! Self-contained request authentication for the standalone `files-api` binary.
//!
//! The Files API vends storage credentials on the caller's behalf, so in a shared
//! deployment access should be attributed to a real user rather than an
//! over-privileged anonymous principal. This module provides a small tower
//! [`Layer`] that establishes the caller identity two ways:
//!
//!   - [`AuthMode::Anonymous`] — every request is anonymous. Suitable when the
//!     upstream Unity Catalog itself uses anonymous auth, or for single-tenant /
//!     local development.
//!   - [`AuthMode::ReverseProxy`] — trust a forwarded-identity header set by an
//!     upstream reverse proxy that has *already* authenticated the caller (e.g.
//!     nginx / Envoy / an OAuth2 proxy forwarding `X-Forwarded-User`). Only safe
//!     when the proxy is not directly reachable, since a client could otherwise
//!     forge the header.
//!
//! The layer inserts a [`ForwardedIdentity`] into the request extensions. The
//! current client arm does not yet consult it — it is carried for request
//! attribution/logging and future per-user credential vending — but a rejected
//! reverse-proxy request short-circuits with `401` before reaching the service.
//! This mirrors the `storage-proxy` binary's auth layer without depending on its
//! error / backend types.

use std::task::{Context, Poll};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use futures::{FutureExt, future::BoxFuture};
use tower::{Layer, Service};

/// The default header a reverse proxy uses to forward the authenticated user.
pub const DEFAULT_FORWARDED_USER_HEADER: &str = "x-forwarded-user";

/// The caller identity established by [`AuthLayer`], stored in the request
/// extensions.
///
/// `None` means the request is anonymous; `Some(name)` carries the forwarded user
/// name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ForwardedIdentity(pub Option<String>);

impl ForwardedIdentity {
    /// The anonymous identity.
    pub fn anonymous() -> Self {
        Self(None)
    }

    /// An authenticated user identity.
    pub fn user(name: impl Into<String>) -> Self {
        Self(Some(name.into()))
    }

    /// The forwarded user name, if any.
    pub fn forwarded_user(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

/// How the standalone Files API authenticates incoming requests.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthMode {
    /// Every request is anonymous.
    Anonymous,
    /// Trust a forwarded-identity header set by a trusted upstream reverse proxy.
    ReverseProxy {
        /// Header carrying the forwarded user (matched case-insensitively, as
        /// HTTP header names are).
        header: String,
        /// When `true`, a request lacking the header is treated as anonymous
        /// instead of rejected with `401`.
        allow_missing: bool,
    },
}

impl AuthMode {
    /// Establish the identity for a request, or an `Err(message)` to reject it
    /// with `401`.
    fn authenticate(&self, request: &Request) -> Result<ForwardedIdentity, String> {
        match self {
            AuthMode::Anonymous => Ok(ForwardedIdentity::anonymous()),
            AuthMode::ReverseProxy {
                header,
                allow_missing,
            } => match request.headers().get(header) {
                Some(value) => {
                    let name = value
                        .to_str()
                        .map_err(|_| format!("`{header}` header is not valid UTF-8"))?;
                    let name = name.trim();
                    if name.is_empty() {
                        return Err(format!("`{header}` header is empty"));
                    }
                    Ok(ForwardedIdentity::user(name))
                }
                None if *allow_missing => Ok(ForwardedIdentity::anonymous()),
                None => Err(format!("missing `{header}` header")),
            },
        }
    }
}

/// Tower layer that authenticates requests and inserts a [`ForwardedIdentity`]
/// extension. On a rejected request it short-circuits with a `401` response.
#[derive(Clone)]
pub struct AuthLayer {
    mode: AuthMode,
}

impl AuthLayer {
    /// Build a layer for the given [`AuthMode`].
    pub fn new(mode: AuthMode) -> Self {
        Self { mode }
    }
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            mode: self.mode.clone(),
        }
    }
}

/// The service produced by [`AuthLayer`].
#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    mode: AuthMode,
}

impl<S> Service<Request> for AuthMiddleware<S>
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

    fn call(&mut self, mut req: Request) -> Self::Future {
        match self.mode.authenticate(&req) {
            Ok(identity) => {
                req.extensions_mut().insert(identity);
                self.inner.call(req).boxed()
            }
            Err(message) => {
                async move { Ok((StatusCode::UNAUTHORIZED, message).into_response()) }.boxed()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use tower::{ServiceBuilder, ServiceExt};

    use super::*;

    fn reverse_proxy(header: &str, allow_missing: bool) -> AuthMode {
        AuthMode::ReverseProxy {
            header: header.to_string(),
            allow_missing,
        }
    }

    #[test]
    fn anonymous_always_anonymous() {
        let req = Request::get("/").body(Body::empty()).unwrap();
        assert_eq!(
            AuthMode::Anonymous.authenticate(&req).unwrap(),
            ForwardedIdentity::anonymous()
        );
    }

    #[test]
    fn reverse_proxy_extracts_forwarded_user() {
        let mode = reverse_proxy(DEFAULT_FORWARDED_USER_HEADER, false);
        let req = Request::get("/")
            .header(DEFAULT_FORWARDED_USER_HEADER, "alice")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            mode.authenticate(&req).unwrap(),
            ForwardedIdentity::user("alice")
        );
    }

    #[test]
    fn reverse_proxy_honors_custom_header_and_trims() {
        let mode = reverse_proxy("x-user", false);
        let req = Request::get("/")
            .header("x-user", "  bob  ")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            mode.authenticate(&req).unwrap(),
            ForwardedIdentity::user("bob")
        );
    }

    #[test]
    fn reverse_proxy_rejects_missing_header_by_default() {
        let mode = reverse_proxy(DEFAULT_FORWARDED_USER_HEADER, false);
        let req = Request::get("/").body(Body::empty()).unwrap();
        assert!(mode.authenticate(&req).is_err());
    }

    #[test]
    fn reverse_proxy_rejects_empty_header() {
        let mode = reverse_proxy(DEFAULT_FORWARDED_USER_HEADER, false);
        let req = Request::get("/")
            .header(DEFAULT_FORWARDED_USER_HEADER, "   ")
            .body(Body::empty())
            .unwrap();
        assert!(mode.authenticate(&req).is_err());
    }

    #[test]
    fn reverse_proxy_falls_back_to_anonymous_when_configured() {
        let mode = reverse_proxy(DEFAULT_FORWARDED_USER_HEADER, true);
        let req = Request::get("/").body(Body::empty()).unwrap();
        assert_eq!(
            mode.authenticate(&req).unwrap(),
            ForwardedIdentity::anonymous()
        );
    }

    #[tokio::test]
    async fn layer_inserts_identity_extension() {
        async fn echo(req: Request) -> Result<Response, std::convert::Infallible> {
            assert!(req.extensions().get::<ForwardedIdentity>().is_some());
            Ok(Response::new(Body::empty()))
        }
        let mut service = ServiceBuilder::new()
            .layer(AuthLayer::new(AuthMode::Anonymous))
            .service_fn(echo);
        let req = Request::get("/").body(Body::empty()).unwrap();
        let resp = service.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn layer_missing_identity_maps_to_401() {
        async fn ok(_req: Request) -> Result<Response, std::convert::Infallible> {
            Ok(Response::new(Body::empty()))
        }
        let mut service = ServiceBuilder::new()
            .layer(AuthLayer::new(reverse_proxy(
                DEFAULT_FORWARDED_USER_HEADER,
                false,
            )))
            .service_fn(ok);
        let req = Request::get("/").body(Body::empty()).unwrap();
        let resp = service.ready().await.unwrap().call(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }
}
