//! The default request context of the handler traits.

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
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub recipient: Principal,
}

impl RequestContext {
    pub fn recipient(&self) -> &Principal {
        &self.recipient
    }
}

impl AsRef<Principal> for RequestContext {
    fn as_ref(&self) -> &Principal {
        &self.recipient
    }
}

/// Extract a [`RequestContext`] from the [`Principal`] an authentication
/// middleware inserted into the request extensions, falling back to
/// [`Principal::Anonymous`] when there is none.
#[cfg(feature = "axum")]
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
