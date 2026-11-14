//! The default request context of the handler traits.

#[cfg(feature = "olai-auth")]
use std::sync::Arc;

/// The identity a request is made on behalf of.
///
/// This is one possible authorization context (`Cx`) for a policy; a server that
/// extracts a username from a reverse-proxy header would authorize against a
/// `Principal`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Principal {
    /// No authenticated identity — used when no credentials are present, e.g.
    /// behind a proxy that does not forward an identity header.
    Anonymous,
    /// A named user identity, typically the value of a forwarded-user header.
    User(String),
}

impl Principal {
    /// Returns an [`Anonymous`](Principal::Anonymous) principal.
    pub fn anonymous() -> Self {
        Self::Anonymous
    }

    /// Returns a [`User`](Principal::User) principal with the given name.
    pub fn user(name: impl Into<String>) -> Self {
        Self::User(name.into())
    }
}

/// The context a handler method receives unless the trait is instantiated with
/// another `Cx`.
///
/// Build one with [`new`](Self::new) or [`anonymous`](Self::anonymous), or, in an
/// axum service, extract it from the request (feature `axum`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RequestContext {
    /// Who the request is made on behalf of.
    pub recipient: Principal,
    /// The verified caller, when the request was authenticated by `olai-auth`.
    ///
    /// [`recipient`](Self::recipient) is then [`Principal::User`] with the caller's
    /// [`subject`](olai_auth::Principal::subject).
    #[cfg(feature = "olai-auth")]
    pub authenticated: Option<Arc<olai_auth::Principal>>,
}

impl RequestContext {
    /// A context acting on behalf of `recipient`.
    pub fn new(recipient: Principal) -> Self {
        Self {
            recipient,
            #[cfg(feature = "olai-auth")]
            authenticated: None,
        }
    }

    /// A context with no identity.
    pub fn anonymous() -> Self {
        Self::new(Principal::Anonymous)
    }

    /// A context for a caller verified by `olai-auth`. The recipient is the caller's
    /// subject.
    #[cfg(feature = "olai-auth")]
    pub fn from_verified(principal: olai_auth::Principal) -> Self {
        Self {
            recipient: Principal::User(principal.subject().to_owned()),
            authenticated: Some(Arc::new(principal)),
        }
    }

    /// Who the request is made on behalf of.
    pub fn recipient(&self) -> &Principal {
        &self.recipient
    }

    /// The verified caller, when the request was authenticated by `olai-auth`.
    #[cfg(feature = "olai-auth")]
    pub fn verified_principal(&self) -> Option<&olai_auth::Principal> {
        self.authenticated.as_deref()
    }

    /// The context an authentication middleware left in the request extensions.
    ///
    /// An `olai_auth::Principal` (feature `olai-auth`) wins over a [`Principal`].
    /// With neither, the context is anonymous.
    #[cfg(feature = "axum")]
    pub fn from_extensions(extensions: &axum::http::Extensions) -> Self {
        #[cfg(feature = "olai-auth")]
        if let Some(principal) = extensions.get::<olai_auth::Principal>() {
            return Self::from_verified(principal.clone());
        }
        extensions
            .get::<Principal>()
            .cloned()
            .map_or_else(Self::anonymous, Self::new)
    }
}

impl AsRef<Principal> for RequestContext {
    fn as_ref(&self) -> &Principal {
        &self.recipient
    }
}

/// Extract a [`RequestContext`] from what an authentication middleware inserted
/// into the request extensions (see [`RequestContext::from_extensions`]), falling
/// back to [`Principal::Anonymous`] when there is nothing.
#[cfg(feature = "axum")]
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for RequestContext {
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> std::result::Result<Self, Self::Rejection> {
        Ok(Self::from_extensions(&parts.extensions))
    }
}

#[cfg(all(test, feature = "axum"))]
mod tests {
    use axum::http::Extensions;

    use super::*;

    #[test]
    fn empty_extensions_are_anonymous() {
        let cx = RequestContext::from_extensions(&Extensions::new());
        assert_eq!(cx.recipient, Principal::Anonymous);
    }

    #[test]
    fn a_principal_becomes_the_recipient() {
        let mut ext = Extensions::new();
        ext.insert(Principal::user("alice"));
        let cx = RequestContext::from_extensions(&ext);
        assert_eq!(cx.recipient, Principal::user("alice"));
    }

    #[cfg(feature = "olai-auth")]
    #[test]
    fn a_verified_principal_wins() {
        let verified = olai_auth::Principal {
            kind: olai_auth::PrincipalKind::User {
                sub: "user-1".into(),
                email: Some("a@example.com".into()),
                email_verified: true,
            },
            issuer: "https://auth.example.com".into(),
            scopes: vec![],
            client_id: None,
            auth_method: olai_auth::AuthMethod::Jwt,
            admin: false,
            issued_at: None,
        };
        let mut ext = Extensions::new();
        ext.insert(Principal::user("ignored"));
        ext.insert(verified.clone());
        let cx = RequestContext::from_extensions(&ext);
        assert_eq!(cx.recipient, Principal::user("user-1"));
        assert_eq!(cx.verified_principal(), Some(&verified));
    }
}
