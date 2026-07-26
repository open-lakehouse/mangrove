//! The portable **client arm**: a [`StorageProxyBackend`] backed by a
//! [`UnityObjectStoreFactory`].
//!
//! Given only `{baseUrl, token}` this arm serves the proxy against **any** Unity
//! Catalog server — the factory resolves the securable and vends a scoped
//! credential through the UC REST API, so there is zero coupling to any particular
//! server implementation. Authorization is delegated to the upstream UC server
//! (a vend for an unauthorized securable fails), plus the local traversal guard.
//!
//! Requires the `client-arm` feature.

use std::sync::Arc;

use object_store::DynObjectStore;
use unitycatalog_object_store::{
    PathOperation, TableOperation, UnityObjectStoreFactory, VolumeOperation,
};

use crate::backend::{
    ForwardedUser, ProxyCapabilities, ProxyReq, Securable, StorageProxyBackend,
    reject_key_traversal,
};
use crate::error::{ProxyError, ProxyResult};

/// Default header the client arm re-emits the forwarded end-user identity under
/// on upstream Unity Catalog requests. Matches the reverse-proxy auth layer's
/// [`DEFAULT_FORWARDED_USER_HEADER`](crate::auth::DEFAULT_FORWARDED_USER_HEADER)
/// so the incoming and outgoing header names line up by default. Duplicated here
/// (as a literal) so the client arm does not depend on the `bin`-gated `auth`
/// module.
pub const DEFAULT_FORWARDED_USER_HEADER: &str = "x-forwarded-user";

/// A [`StorageProxyBackend`] that resolves + vends through a
/// [`UnityObjectStoreFactory`]. Portable against any UC server.
#[derive(Clone)]
pub struct UnityFactoryProxyBackend {
    factory: UnityObjectStoreFactory,
    /// Header name the forwarded end-user identity (from the request context) is
    /// re-emitted under on upstream UC calls. Defaults to
    /// [`DEFAULT_FORWARDED_USER_HEADER`].
    forwarded_header: String,
}

impl UnityFactoryProxyBackend {
    /// Connect to a Unity Catalog server at `base_uri` (e.g.
    /// `https://host/api/2.1/unity-catalog/`) with an optional bearer `token`.
    ///
    /// Forwards a per-request end-user identity to upstream UC under
    /// [`DEFAULT_FORWARDED_USER_HEADER`]. Use
    /// [`connect_with_forwarded_header`](Self::connect_with_forwarded_header) to
    /// re-emit the identity under a different header name.
    pub async fn connect(base_uri: impl Into<String>, token: Option<String>) -> ProxyResult<Self> {
        Self::connect_with_forwarded_header(base_uri, token, DEFAULT_FORWARDED_USER_HEADER).await
    }

    /// Like [`connect`](Self::connect), but re-emits the forwarded end-user
    /// identity under `forwarded_header` on upstream UC requests (typically the
    /// same header name the proxy read the incoming identity from).
    pub async fn connect_with_forwarded_header(
        base_uri: impl Into<String>,
        token: Option<String>,
        forwarded_header: impl Into<String>,
    ) -> ProxyResult<Self> {
        let allow_unauthenticated = token.is_none();
        let factory = UnityObjectStoreFactory::builder()
            .with_uri(base_uri)
            .with_token(token)
            .with_allow_unauthenticated(allow_unauthenticated)
            .build()
            .await
            .map_err(|e| ProxyError::Internal(format!("connect UC factory: {e}")))?;
        Ok(Self {
            factory,
            forwarded_header: forwarded_header.into(),
        })
    }

    /// Build from an already-constructed factory (e.g. one sharing a client pool).
    /// Forwards identity under [`DEFAULT_FORWARDED_USER_HEADER`].
    pub fn from_factory(factory: UnityObjectStoreFactory) -> Self {
        Self {
            factory,
            forwarded_header: DEFAULT_FORWARDED_USER_HEADER.to_string(),
        }
    }
}

#[async_trait::async_trait]
impl<Cx: ForwardedUser + Send + Sync + 'static> StorageProxyBackend<Cx>
    for UnityFactoryProxyBackend
{
    fn capabilities(&self) -> ProxyCapabilities {
        ProxyCapabilities { enabled: true }
    }

    async fn authorize(&self, req: &ProxyReq, _cx: &Cx) -> ProxyResult<()> {
        // The confused-deputy prefix confinement is intrinsic here: the vend
        // returns a credential scoped to the securable root, and `open` returns a
        // store prefixed there, so a key cannot address a sibling. We add the
        // traversal guard (layer 1) and defer permission checks to the vend itself.
        reject_key_traversal(&req.key)
    }

    async fn open(&self, req: &ProxyReq, cx: &Cx) -> ProxyResult<Arc<DynObjectStore>> {
        // Forward the validated end-user identity (if any) to the upstream UC
        // vend + metadata calls, so UC attributes the credential to the real user
        // rather than to the proxy's own service principal. An anonymous request
        // (`None`) reuses the base factory unchanged (its static token / auth).
        let factory = self
            .factory
            .with_forwarded_user(&self.forwarded_header, cx.forwarded_user())
            .map_err(ProxyError::Storage)?;

        let write = req.verb.is_write();
        let store = match &req.securable {
            Securable::Table { full_name } => {
                let op = if write {
                    TableOperation::ReadWrite
                } else {
                    TableOperation::Read
                };
                factory.for_table(full_name.clone(), op).await
            }
            Securable::Volume { full_name } => {
                let op = if write {
                    VolumeOperation::ReadWrite
                } else {
                    VolumeOperation::Read
                };
                factory.for_volume(full_name.clone(), op).await
            }
            Securable::Path { url } => {
                let op = if write {
                    PathOperation::ReadWrite
                } else {
                    PathOperation::Read
                };
                factory.for_path(url, op).await
            }
        }
        .map_err(ProxyError::Storage)?;

        // `as_dyn()` returns the store prefixed at the credential-scoped root, so
        // `req.key` addresses relative to the securable.
        Ok(store.as_dyn())
    }
}
