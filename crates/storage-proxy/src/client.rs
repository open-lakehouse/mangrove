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
    ProxyCapabilities, ProxyReq, Securable, StorageProxyBackend, reject_key_traversal,
};
use crate::error::{ProxyError, ProxyResult};

/// A [`StorageProxyBackend`] that resolves + vends through a
/// [`UnityObjectStoreFactory`]. Portable against any UC server.
#[derive(Clone)]
pub struct UnityFactoryProxyBackend {
    factory: UnityObjectStoreFactory,
}

impl UnityFactoryProxyBackend {
    /// Connect to a Unity Catalog server at `base_uri` (e.g.
    /// `https://host/api/2.1/unity-catalog/`) with an optional bearer `token`.
    pub async fn connect(base_uri: impl Into<String>, token: Option<String>) -> ProxyResult<Self> {
        let allow_unauthenticated = token.is_none();
        let factory = UnityObjectStoreFactory::builder()
            .with_uri(base_uri)
            .with_token(token)
            .with_allow_unauthenticated(allow_unauthenticated)
            .build()
            .await
            .map_err(|e| ProxyError::Internal(format!("connect UC factory: {e}")))?;
        Ok(Self { factory })
    }

    /// Build from an already-constructed factory (e.g. one sharing a client pool).
    pub fn from_factory(factory: UnityObjectStoreFactory) -> Self {
        Self { factory }
    }
}

#[async_trait::async_trait]
impl<Cx: Send + Sync + 'static> StorageProxyBackend<Cx> for UnityFactoryProxyBackend {
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

    async fn open(&self, req: &ProxyReq, _cx: &Cx) -> ProxyResult<Arc<DynObjectStore>> {
        let write = req.verb.is_write();
        let store = match &req.securable {
            Securable::Table { full_name } => {
                let op = if write {
                    TableOperation::ReadWrite
                } else {
                    TableOperation::Read
                };
                self.factory.for_table(full_name.clone(), op).await
            }
            Securable::Volume { full_name } => {
                let op = if write {
                    VolumeOperation::ReadWrite
                } else {
                    VolumeOperation::Read
                };
                self.factory.for_volume(full_name.clone(), op).await
            }
            Securable::Path { url } => {
                let op = if write {
                    PathOperation::ReadWrite
                } else {
                    PathOperation::Read
                };
                self.factory.for_path(url, op).await
            }
        }
        .map_err(ProxyError::Storage)?;

        // `as_dyn()` returns the store prefixed at the credential-scoped root, so
        // `req.key` addresses relative to the securable.
        Ok(store.as_dyn())
    }
}
