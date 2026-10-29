//! Recipient authentication: the [`RecipientAuthenticator`] port and the bearer
//! token layer that runs it in front of the sharing routes.
//!
//! Delta Sharing / Open Sharing recipients authenticate with a bearer token from
//! their sharing profile. [`require_recipient`] wraps a router so every request
//! must carry `Authorization: Bearer <token>`; the token is resolved to a
//! [`Recipient`] by the host's authenticator and stored in the request
//! extensions, where the [`Recipient`] extractor — the request context `Cx` of
//! the sharing handlers — picks it up. Requests without a valid token get `401`.

use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::{FromRequestParts, Request, State};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::error::{Error, Result};

/// An authenticated sharing recipient.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recipient {
    /// The recipient's stable identifier.
    pub id: String,
    /// The recipient's name.
    pub name: String,
}

/// Resolves a bearer token to the [`Recipient`] it belongs to.
#[async_trait]
pub trait RecipientAuthenticator: Send + Sync + 'static {
    /// The recipient `token` was issued to. An unknown, expired or revoked token
    /// → [`Error::Unauthenticated`].
    async fn authenticate(&self, token: &str) -> Result<Recipient>;
}

/// Require a valid recipient bearer token on every route of `router`.
pub fn require_recipient(
    router: axum::Router,
    authenticator: Arc<dyn RecipientAuthenticator>,
) -> axum::Router {
    router.layer(axum::middleware::from_fn_with_state(
        authenticator,
        authenticate,
    ))
}

async fn authenticate(
    State(authenticator): State<Arc<dyn RecipientAuthenticator>>,
    mut request: Request,
    next: Next,
) -> Response {
    let token = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(bearer_token);
    let Some(token) = token else {
        return Error::Unauthenticated.into_response();
    };
    match authenticator.authenticate(token).await {
        Ok(recipient) => {
            request.extensions_mut().insert(recipient);
            next.run(request).await
        }
        Err(err) => err.into_response(),
    }
}

/// The token of an `Authorization: Bearer <token>` header value.
fn bearer_token(value: &str) -> Option<&str> {
    let (scheme, token) = value.split_once(' ')?;
    let token = token.trim();
    (scheme.eq_ignore_ascii_case("bearer") && !token.is_empty()).then_some(token)
}

impl<S: Send + Sync> FromRequestParts<S> for Recipient {
    type Rejection = Error;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self> {
        parts
            .extensions
            .get::<Recipient>()
            .cloned()
            .ok_or(Error::Unauthenticated)
    }
}

#[cfg(test)]
mod tests {
    use super::bearer_token;

    #[test]
    fn parses_bearer_tokens() {
        assert_eq!(bearer_token("Bearer abc"), Some("abc"));
        assert_eq!(bearer_token("bearer  abc "), Some("abc"));
        assert_eq!(bearer_token("Basic abc"), None);
        assert_eq!(bearer_token("Bearer "), None);
        assert_eq!(bearer_token("Bearer"), None);
    }
}
