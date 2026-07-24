//! An in-memory [`StorageProxyBackend`] for exercising the wire contract without a
//! cloud.
//!
//! [`InMemoryStorageProxyBackend`] wraps an [`object_store::memory::InMemory`] and
//! authorizes exactly one securable + key-prefix, so the confused-deputy and
//! read-only-scope tests have something concrete to be rejected against. Every
//! handler path (ranged `get_opts`, conditional `put_opts`, streaming
//! `put_multipart`) is supported by `InMemory`, so the full contract is testable
//! here.

use std::sync::Arc;

use object_store::DynObjectStore;
use object_store::memory::InMemory;
use object_store::prefix::PrefixStore;

use crate::backend::{
    ProxyCapabilities, ProxyReq, Securable, StorageProxyBackend, reject_key_traversal,
};
use crate::error::{ProxyError, ProxyResult};

/// An in-memory backend authorizing a single securable and key prefix.
#[derive(Clone)]
pub struct InMemoryStorageProxyBackend {
    store: Arc<InMemory>,
    /// The one securable this backend authorizes.
    allowed: Securable,
    /// The key prefix within the securable the caller may address (`""` = any key
    /// under the securable root).
    prefix: String,
    /// Whether writes (`PUT`) are permitted.
    allow_write: bool,
}

impl InMemoryStorageProxyBackend {
    /// A backend authorizing the whole of the given `table:` securable, read+write,
    /// over a fresh empty store.
    pub fn table(full_name: impl Into<String>) -> Self {
        Self {
            store: Arc::new(InMemory::new()),
            allowed: Securable::Table {
                full_name: full_name.into(),
            },
            prefix: String::new(),
            allow_write: true,
        }
    }

    /// A backend authorizing the whole of the given `vol:` securable, read+write.
    pub fn volume(full_name: impl Into<String>) -> Self {
        Self {
            store: Arc::new(InMemory::new()),
            allowed: Securable::Volume {
                full_name: full_name.into(),
            },
            prefix: String::new(),
            allow_write: true,
        }
    }

    /// Restrict authorized keys to those under `prefix` (drives out-of-scope tests).
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = prefix.into();
        self
    }

    /// Make the backend read-only (drives read-only-scope `PUT` rejection tests).
    pub fn read_only(mut self) -> Self {
        self.allow_write = false;
        self
    }

    /// Borrow the backing store, e.g. to seed objects in a test.
    pub fn store(&self) -> Arc<InMemory> {
        self.store.clone()
    }
}

#[async_trait::async_trait]
impl<Cx: Send + Sync + 'static> StorageProxyBackend<Cx> for InMemoryStorageProxyBackend {
    fn capabilities(&self) -> ProxyCapabilities {
        ProxyCapabilities { enabled: true }
    }

    async fn authorize(&self, req: &ProxyReq, _cx: &Cx) -> ProxyResult<()> {
        reject_key_traversal(&req.key)?;
        if req.securable != self.allowed {
            return Err(ProxyError::PermissionDenied(format!(
                "securable {:?} is not authorized",
                req.securable
            )));
        }
        if !self.prefix.is_empty() && !req.key.starts_with(&self.prefix) {
            return Err(ProxyError::OutOfScope(format!(
                "key `{}` is outside the authorized prefix `{}`",
                req.key, self.prefix
            )));
        }
        if req.verb.is_write() && !self.allow_write {
            return Err(ProxyError::PermissionDenied(
                "write not permitted for this scope".into(),
            ));
        }
        Ok(())
    }

    async fn open(&self, _req: &ProxyReq, _cx: &Cx) -> ProxyResult<Arc<DynObjectStore>> {
        // A real backend prefixes at the securable's storage root; the in-memory
        // store's root already stands in for that root, so return it directly (the
        // authorized key prefix is enforced in `authorize`, not by re-prefixing).
        Ok(self.store.clone())
    }
}

/// A [`PrefixStore`]-wrapping helper, mirroring how a real arm roots `req.key`
/// under a securable prefix. Exposed for tests that want to assert prefix
/// confinement explicitly.
pub fn prefixed(store: Arc<InMemory>, prefix: &str) -> Arc<DynObjectStore> {
    Arc::new(PrefixStore::new(store, prefix))
}
