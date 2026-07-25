//! Authentication middleware for Delta Sharing server.
use std::marker::PhantomData;
use std::task::{Context, Poll};

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use futures_util::{FutureExt, future::BoxFuture};
use tower::{Layer, Service};
use unitycatalog_common::{Error, Result};

use crate::api::RequestContext;
use crate::policy::Principal;

/// The default header a reverse proxy uses to forward the authenticated user.
pub const DEFAULT_FORWARDED_USER_HEADER: &str = "x-forwarded-user";

/// Extract a [`RequestContext`] from the [`Principal`] that the
/// [`AuthenticationMiddleware`] inserted into the request extensions.
///
/// This is the transport-side counterpart to that middleware: the middleware
/// writes the `Principal`, this reads it back so handlers can receive a
/// `RequestContext`. It lives here (rather than next to the `RequestContext`
/// definition in `crate::api`) to keep the `api` module free of any axum
/// coupling.
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for RequestContext {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        let recipient = parts
            .extensions
            .get::<Principal>()
            .cloned()
            .unwrap_or_else(Principal::anonymous);
        Ok(RequestContext { recipient })
    }
}

/// Authenticator for authenticating requests to a sharing server.
///
/// `I` is the identity type inserted into request extensions. It must be
/// `Clone + Send + Sync + 'static` so it can be stored in axum extensions.
pub trait Authenticator<I: Clone + Send + Sync + 'static>: Send + Sync + 'static {
    /// Authenticate a request and return an identity value.
    ///
    /// This method should return the identity of the caller, or an error if the
    /// request is not authenticated or the identity cannot be determined.
    fn authenticate(&self, request: &Request) -> Result<I>;
}

/// Authenticator that always marks the recipient as anonymous.
///
/// This is the default authenticator used when no authentication is configured.
/// The server is designed to run behind a reverse proxy (e.g., nginx, Envoy) that
/// handles authentication and injects the authenticated identity (e.g., via an
/// `X-Forwarded-User` header). A production `Authenticator` implementation should
/// read that header and return `Principal::User(name)` — see
/// [`ReverseProxyAuthenticator`].
#[derive(Clone)]
pub struct AnonymousAuthenticator;

impl Authenticator<Principal> for AnonymousAuthenticator {
    fn authenticate(&self, _: &Request) -> Result<Principal> {
        Ok(Principal::anonymous())
    }
}

/// Authenticator that reads the caller's identity from a header set by a
/// trusted upstream reverse proxy (e.g. nginx / Envoy / an OAuth2 proxy that
/// forwards `X-Forwarded-User`).
///
/// The proxy is responsible for actually authenticating the request; this
/// authenticator only *trusts and extracts* the forwarded identity, so it must
/// only be enabled when the server is not directly reachable — otherwise a
/// client could forge the header. It exists so that features which vend
/// credentials on the caller's behalf (notably the same-origin storage
/// byte-proxy) attribute access to a real user instead of the over-privileged
/// [`Principal::anonymous`].
///
/// When the header is absent, [`on_missing`](Self::with_on_missing) decides
/// whether to reject the request (`401`) or fall back to anonymous.
#[derive(Clone)]
pub struct ReverseProxyAuthenticator {
    /// Header carrying the forwarded user (matched case-insensitively, as HTTP
    /// header names are). Defaults to [`DEFAULT_FORWARDED_USER_HEADER`].
    header: String,
    /// What to do when the header is absent.
    on_missing: OnMissingIdentity,
}

/// Behavior when the forwarded-identity header is absent from a request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnMissingIdentity {
    /// Reject the request with an authentication error (`401`). The safe
    /// default for a deployment that expects every request to carry identity.
    Reject,
    /// Treat the request as [`Principal::anonymous`]. Useful for a mixed
    /// deployment where some routes are intentionally unauthenticated.
    Anonymous,
}

impl ReverseProxyAuthenticator {
    /// Build an authenticator reading [`DEFAULT_FORWARDED_USER_HEADER`],
    /// rejecting requests that lack it.
    pub fn new() -> Self {
        Self {
            header: DEFAULT_FORWARDED_USER_HEADER.to_string(),
            on_missing: OnMissingIdentity::Reject,
        }
    }

    /// Override the header the identity is read from.
    pub fn with_header(mut self, header: impl Into<String>) -> Self {
        self.header = header.into();
        self
    }

    /// Set the behavior when the header is absent (default:
    /// [`OnMissingIdentity::Reject`]).
    pub fn with_on_missing(mut self, on_missing: OnMissingIdentity) -> Self {
        self.on_missing = on_missing;
        self
    }
}

impl Default for ReverseProxyAuthenticator {
    fn default() -> Self {
        Self::new()
    }
}

impl Authenticator<Principal> for ReverseProxyAuthenticator {
    fn authenticate(&self, request: &Request) -> Result<Principal> {
        match request.headers().get(&self.header) {
            Some(value) => {
                let name = value.to_str().map_err(|_| {
                    Error::unauthenticated(format!("`{}` header is not valid UTF-8", self.header))
                })?;
                let name = name.trim();
                if name.is_empty() {
                    return Err(Error::unauthenticated(format!(
                        "`{}` header is empty",
                        self.header
                    )));
                }
                Ok(Principal::user(name))
            }
            None => match self.on_missing {
                OnMissingIdentity::Anonymous => Ok(Principal::anonymous()),
                OnMissingIdentity::Reject => Err(Error::unauthenticated(format!(
                    "missing `{}` header",
                    self.header
                ))),
            },
        }
    }
}

/// Middleware that authenticates requests using the given [`Authenticator`].
#[derive(Clone)]
pub struct AuthenticationMiddleware<S, T, I = Principal> {
    inner: S,
    authenticator: T,
    _identity: PhantomData<I>,
}

#[allow(unused)]
impl<S, T, I> AuthenticationMiddleware<S, T, I> {
    /// Create new [`AuthenticationMiddleware`].
    pub fn new(inner: S, authenticator: T) -> Self {
        Self {
            inner,
            authenticator,
            _identity: PhantomData,
        }
    }

    /// Create a new [`AuthenticationLayer`] with the given [`Authenticator`].
    ///
    /// This is a convenience method that is equivalent to calling [`AuthenticationLayer::new`].
    pub fn layer(authenticator: T) -> AuthenticationLayer<T, I> {
        AuthenticationLayer::new(authenticator)
    }
}

impl<S, T, I> Service<Request> for AuthenticationMiddleware<S, T, I>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
    T: Authenticator<I>,
    I: Clone + Send + Sync + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: Request) -> Self::Future {
        match self.authenticator.authenticate(&req) {
            Ok(identity) => {
                req.extensions_mut().insert(identity);
                self.inner.call(req).boxed()
            }
            Err(e) => async { Ok(crate::Error::from(e).into_response()) }.boxed(),
        }
    }
}

/// Layer that applies the [`AuthenticationMiddleware`].
#[derive(Clone)]
pub struct AuthenticationLayer<T, I = Principal> {
    authenticator: T,
    _identity: PhantomData<I>,
}

impl<T, I> AuthenticationLayer<T, I> {
    /// Create a new [`AuthenticationLayer`] with the provided [`Authenticator`].
    pub fn new(authenticator: T) -> Self {
        Self {
            authenticator,
            _identity: PhantomData,
        }
    }
}

impl<S, T, I> Layer<S> for AuthenticationLayer<T, I>
where
    T: Clone + Send + Sync + 'static,
    I: Clone + Send + Sync + 'static,
{
    type Service = AuthenticationMiddleware<S, T, I>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthenticationMiddleware {
            inner,
            authenticator: self.authenticator.clone(),
            _identity: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::extract::Request;
    use axum::http::{StatusCode, header};
    use tower::{ServiceBuilder, ServiceExt};

    use super::*;

    async fn check_recipient(req: Request) -> Result<Response<Body>> {
        assert!(matches!(
            req.extensions().get::<Principal>(),
            Some(Principal::Anonymous) | Some(Principal::User(_))
        ));
        Ok(Response::new(req.into_body()))
    }

    #[tokio::test]
    async fn test_authentication_middleware() {
        let authenticator = AnonymousAuthenticator {};
        let mut service = ServiceBuilder::new()
            .layer(AuthenticationLayer::new(authenticator))
            .service_fn(check_recipient);

        let request = Request::get("/")
            .header(header::AUTHORIZATION, "Bearer foo")
            .body(Body::empty())
            .unwrap();

        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let request = Request::get("/").body(Body::empty()).unwrap();
        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[test]
    fn reverse_proxy_extracts_forwarded_user() {
        let auth = ReverseProxyAuthenticator::new();
        let req = Request::get("/")
            .header(DEFAULT_FORWARDED_USER_HEADER, "alice")
            .body(Body::empty())
            .unwrap();
        assert_eq!(auth.authenticate(&req).unwrap(), Principal::user("alice"));
    }

    #[test]
    fn reverse_proxy_honors_custom_header_and_trims() {
        let auth = ReverseProxyAuthenticator::new().with_header("x-user");
        let req = Request::get("/")
            .header("x-user", "  bob  ")
            .body(Body::empty())
            .unwrap();
        assert_eq!(auth.authenticate(&req).unwrap(), Principal::user("bob"));
    }

    #[test]
    fn reverse_proxy_rejects_missing_header_by_default() {
        let auth = ReverseProxyAuthenticator::new();
        let req = Request::get("/").body(Body::empty()).unwrap();
        assert!(auth.authenticate(&req).is_err());
    }

    #[test]
    fn reverse_proxy_rejects_empty_header() {
        let auth = ReverseProxyAuthenticator::new();
        let req = Request::get("/")
            .header(DEFAULT_FORWARDED_USER_HEADER, "   ")
            .body(Body::empty())
            .unwrap();
        assert!(auth.authenticate(&req).is_err());
    }

    #[test]
    fn reverse_proxy_falls_back_to_anonymous_when_configured() {
        let auth = ReverseProxyAuthenticator::new().with_on_missing(OnMissingIdentity::Anonymous);
        let req = Request::get("/").body(Body::empty()).unwrap();
        assert_eq!(auth.authenticate(&req).unwrap(), Principal::anonymous());
    }

    #[tokio::test]
    async fn reverse_proxy_missing_identity_maps_to_401() {
        let mut service = ServiceBuilder::new()
            .layer(AuthenticationLayer::new(ReverseProxyAuthenticator::new()))
            .service_fn(check_recipient);
        let request = Request::get("/").body(Body::empty()).unwrap();
        let response = service.ready().await.unwrap().call(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }
}
