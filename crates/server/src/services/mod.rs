use std::sync::Arc;

use crate::Result;
use crate::grants::Grants;
use crate::policy::{Decision, Facts, Policy, ProvidesPolicy, Securable, ServerAction};
use crate::store::{ProvidesResourceStore, ResourceStore};
use unitycatalog_delta_api::coordinator::{
    CommitCoordinator, InMemoryCommitCoordinator, ProvidesCommitCoordinator,
};
use unitycatalog_storage_access::{
    InMemoryStorageAccess, ProvidesStorageAccessStore, StorageAccessStore,
};

mod delta_backend;
pub mod location;
pub mod location_policy;
pub(crate) mod object_store;
pub(crate) mod vending;
// The storage byte-proxy local arm impl. Gated on `storage-proxy`, where the
// `unitycatalog-storage-proxy` dependency (and the serve stack that mounts it) is
// available.
#[cfg(feature = "storage-proxy")]
mod storage_proxy_backend;

pub use location_policy::LocalStoragePolicy;
pub use vending::{AUDIT_TARGET, NeonVending};

/// Access to the server's [`LocalStoragePolicy`].
///
/// Implemented by the server handler so request handlers (and helpers generic
/// over `T`) can reach the server-wide allowlist that governs
/// which host paths may back a `file://` storage location. Mirrors
/// [`ProvidesCommitCoordinator`].
pub trait ProvidesLocalStoragePolicy {
    fn local_storage_policy(&self) -> &LocalStoragePolicy;
}

/// Access to the server's metastore-level managed storage root.
///
/// This is the default managed storage location for the metastore as a whole.
/// A managed catalog created without an explicit `storage_root` inherits this
/// root, mirroring the Unity Catalog metastore → catalog → schema hierarchy
/// ("if the metastore has no managed storage set, you must set one at the
/// catalog level"). `None` means no metastore default is configured, in which
/// case every managed catalog must carry its own `storage_root`.
pub trait ProvidesManagedStorageRoot {
    fn managed_storage_root(&self) -> Option<&str>;
}

#[derive(Clone)]
pub struct ServerHandler<Cx> {
    handler: Arc<ServerHandlerInner<Cx>>,
}

impl<Cx: Send + Sync + 'static> ServerHandler<Cx> {
    /// Construct a handler keeping credentials, external locations and Delta
    /// commits in memory, for tests and ephemeral servers.
    pub fn try_new_tokio(
        policy: Arc<dyn Policy<Cx>>,
        store: Arc<dyn ResourceStore>,
    ) -> Result<Self> {
        Self::try_new_tokio_with_coordinator(
            policy,
            store,
            Arc::new(InMemoryStorageAccess::new()),
            Arc::new(InMemoryCommitCoordinator::default()),
        )
    }

    /// Construct a handler over specific stores.
    ///
    /// Use this to wire a durable backend: its [`StorageAccessStore`] and its
    /// persistent [`CommitCoordinator`] (e.g. the Postgres-backed ones).
    pub fn try_new_tokio_with_coordinator(
        policy: Arc<dyn Policy<Cx>>,
        store: Arc<dyn ResourceStore>,
        storage_access: Arc<dyn StorageAccessStore>,
        commit_coordinator: Arc<dyn CommitCoordinator>,
    ) -> Result<Self> {
        let handler = Arc::new(
            ServerHandlerInner::new(policy.clone(), store.clone(), storage_access)
                .with_commit_coordinator(commit_coordinator),
        );
        Ok(Self { handler })
    }
}

impl<Cx: Send + Sync + 'static> ServerHandler<Cx> {
    /// Keep grants in `grants` instead of in memory. Call at construction time,
    /// before the handler is cloned/shared.
    pub fn with_grants(mut self, grants: Grants) -> Self {
        self.handler = Arc::new(self.handler.rebuild(|inner| inner.grants = grants));
        self
    }

    /// The grants of the securables.
    pub fn grants(&self) -> &Grants {
        &self.handler.grants
    }

    /// Set the allowlist governing `file://` storage locations.
    ///
    /// Rebuilds the inner handler with the policy attached. Call at construction
    /// time, before the handler is cloned/shared. When unset, all local storage
    /// is denied.
    pub fn with_local_storage_policy(mut self, policy: impl Into<Arc<LocalStoragePolicy>>) -> Self {
        let policy = policy.into();
        self.handler = Arc::new(
            self.handler
                .rebuild(|inner| inner.local_storage_policy = policy),
        );
        self
    }

    /// Set the metastore-level managed storage root.
    ///
    /// Rebuilds the inner handler with the root attached. Call at construction
    /// time, before the handler is cloned/shared. When unset, managed catalogs
    /// must each supply their own `storage_root`.
    pub fn with_managed_storage_root(mut self, root: Option<impl Into<Arc<str>>>) -> Self {
        let root = root.map(Into::into);
        self.handler = Arc::new(
            self.handler
                .rebuild(|inner| inner.managed_storage_root = root),
        );
        self
    }

    /// Name the registered storage credential that governs the metastore's managed
    /// storage root.
    ///
    /// Managed tables created under the root (a managed catalog without its own
    /// `storage_root`) are vended from this credential when no external location
    /// covers them. When unset, such paths must be covered by an external
    /// location. Call at construction time, before the handler is shared.
    pub fn with_managed_storage_root_credential(
        mut self,
        credential_name: Option<impl Into<Arc<str>>>,
    ) -> Self {
        let name = credential_name.map(Into::into);
        self.handler = Arc::new(
            self.handler
                .rebuild(|inner| inner.managed_storage_root_credential = name),
        );
        self
    }

    /// Set trusted Neon vending (see [`NeonVending`]): the defaults registered
    /// Neon credentials are completed from, and whether they mint.
    ///
    /// Rebuilds the inner handler. Call at construction time, before the handler
    /// is cloned/shared. When unset, Neon credentials register but never mint.
    pub fn with_neon_vending(mut self, neon: NeonVending) -> Self {
        let neon = Arc::new(neon);
        self.handler = Arc::new(self.handler.rebuild(|inner| inner.neon_vending = neon));
        self
    }

    /// The trusted Neon vending settings.
    pub(crate) fn neon_vending(&self) -> &NeonVending {
        &self.handler.neon_vending
    }

    /// The vendor every temporary credential is minted through.
    pub(crate) fn vendor(&self) -> &Arc<vending::CatalogVendor> {
        &self.handler.vendor
    }

    /// The storage-access store, shared.
    #[cfg(test)]
    pub(crate) fn storage_access_arc(&self) -> Arc<dyn StorageAccessStore> {
        self.handler.storage_access.clone()
    }

    /// The configured managed root and its credential, if both are set.
    #[cfg(test)]
    pub(crate) fn managed_root_credential(&self) -> Option<vending::ManagedRootCredential> {
        self.handler.managed_root()
    }

    /// Drop every cached minter and vended credential, so that a changed or
    /// deleted storage credential's secrets and credentials are not reused. Call
    /// after any storage-credential update or delete.
    pub(crate) fn forget_vended_credentials(&self) {
        self.handler.vendor.clear();
    }
}

#[derive(Clone)]
pub struct ServerHandlerInner<Cx> {
    policy: Arc<dyn Policy<Cx>>,
    store: Arc<dyn ResourceStore>,
    /// Credentials and the external locations they govern.
    storage_access: Arc<dyn StorageAccessStore>,
    /// Delta catalog-managed commit coordinator (in-memory by default).
    commit_coordinator: Arc<dyn CommitCoordinator>,
    /// Allowlist governing which host paths may back a `file://` storage
    /// location. Deny-all by default (see [`LocalStoragePolicy`]).
    local_storage_policy: Arc<LocalStoragePolicy>,
    /// Metastore-level managed storage root inherited by managed catalogs that
    /// omit `storage_root`. `None` ⇒ no metastore default (see
    /// [`ProvidesManagedStorageRoot`]).
    managed_storage_root: Option<Arc<str>>,
    /// The registered storage credential governing `managed_storage_root`.
    managed_storage_root_credential: Option<Arc<str>>,
    /// Trusted Neon vending: registration defaults and whether Neon credentials
    /// mint. Disabled by default.
    neon_vending: Arc<NeonVending>,
    /// The grants of the securables and the directory naming their principals
    /// (in memory by default).
    grants: Grants,
    /// The vendor temporary credentials are minted through; derived from
    /// `storage_access`, the managed-root settings and `neon_vending` (see
    /// [`Self::rebuild`]).
    vendor: Arc<vending::CatalogVendor>,
}

impl<Cx: Send + Sync + 'static> ServerHandlerInner<Cx> {
    pub fn new(
        policy: Arc<dyn Policy<Cx>>,
        store: Arc<dyn ResourceStore>,
        storage_access: Arc<dyn StorageAccessStore>,
    ) -> Self {
        Self {
            vendor: vending::catalog_vendor(storage_access.clone(), None, &NeonVending::default()),
            neon_vending: Arc::new(NeonVending::default()),
            policy,
            store,
            storage_access,
            grants: Grants::in_memory(),
            commit_coordinator: Arc::new(InMemoryCommitCoordinator::default()),
            // Deny all local (file://) storage until a policy is configured.
            local_storage_policy: Arc::new(LocalStoragePolicy::deny_all()),
            // No metastore-level managed storage root by default.
            managed_storage_root: None,
            managed_storage_root_credential: None,
        }
    }

    /// A copy with `change` applied and the vendor rebuilt for the result. All
    /// fields are `Arc`s, so this is cheap; it is meant for construction time
    /// (the rebuilt vendor starts with empty caches).
    fn rebuild(&self, change: impl FnOnce(&mut Self)) -> Self {
        let mut inner = Self {
            policy: self.policy.clone(),
            store: self.store.clone(),
            storage_access: self.storage_access.clone(),
            grants: self.grants.clone(),
            commit_coordinator: self.commit_coordinator.clone(),
            local_storage_policy: self.local_storage_policy.clone(),
            managed_storage_root: self.managed_storage_root.clone(),
            managed_storage_root_credential: self.managed_storage_root_credential.clone(),
            neon_vending: self.neon_vending.clone(),
            vendor: self.vendor.clone(),
        };
        change(&mut inner);
        inner.vendor = vending::catalog_vendor(
            inner.storage_access.clone(),
            inner.managed_root(),
            &inner.neon_vending,
        );
        inner
    }

    /// The managed root and its credential, when both are configured and the
    /// root parses (it is validated at startup).
    fn managed_root(&self) -> Option<vending::ManagedRootCredential> {
        let root = self
            .managed_storage_root
            .as_deref()
            .filter(|r| !r.is_empty())?;
        let credential_name = self.managed_storage_root_credential.as_deref()?;
        let url = location::StorageLocationUrl::parse(root).ok()?;
        Some(vending::ManagedRootCredential {
            scope: url.scope(),
            credential_name: credential_name.to_string(),
        })
    }

    /// Set the allowlist governing `file://` storage locations.
    ///
    /// When unset, the handler denies every local storage path.
    pub fn with_local_storage_policy(mut self, policy: impl Into<Arc<LocalStoragePolicy>>) -> Self {
        self.local_storage_policy = policy.into();
        self
    }

    /// Set the metastore-level managed storage root.
    ///
    /// When unset, managed catalogs must each supply their own `storage_root`.
    pub fn with_managed_storage_root(self, root: Option<impl Into<Arc<str>>>) -> Self {
        let root = root.map(Into::into);
        self.rebuild(|inner| inner.managed_storage_root = root)
    }

    /// Override the Delta commit coordinator (e.g. a Postgres-backed one, or a
    /// custom unbackfilled cap).
    pub fn with_commit_coordinator(mut self, coordinator: Arc<dyn CommitCoordinator>) -> Self {
        self.commit_coordinator = coordinator;
        self
    }
}

impl<Cx: Send + Sync + 'static> ProvidesPolicy<Cx> for ServerHandlerInner<Cx> {
    fn policy(&self) -> &Arc<dyn Policy<Cx>> {
        &self.policy
    }
}

impl<Cx: Send + Sync + 'static> ProvidesPolicy<Cx> for ServerHandler<Cx> {
    fn policy(&self) -> &Arc<dyn Policy<Cx>> {
        &self.handler.policy
    }
}

#[async_trait::async_trait]
impl<Cx: Send + Sync + 'static> Policy<Cx> for ServerHandlerInner<Cx> {
    async fn authorize(
        &self,
        action: ServerAction,
        securable: &Securable,
        facts: &Facts,
        context: &Cx,
    ) -> Result<Decision> {
        self.policy()
            .authorize(action, securable, facts, context)
            .await
    }

    async fn authorize_many(
        &self,
        action: ServerAction,
        securables: &[Securable],
        facts: &Facts,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        self.policy()
            .authorize_many(action, securables, facts, context)
            .await
    }
}

#[async_trait::async_trait]
impl<Cx: Send + Sync + 'static> Policy<Cx> for ServerHandler<Cx> {
    async fn authorize(
        &self,
        action: ServerAction,
        securable: &Securable,
        facts: &Facts,
        context: &Cx,
    ) -> Result<Decision> {
        self.handler
            .policy
            .authorize(action, securable, facts, context)
            .await
    }

    async fn authorize_many(
        &self,
        action: ServerAction,
        securables: &[Securable],
        facts: &Facts,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        self.handler
            .policy
            .authorize_many(action, securables, facts, context)
            .await
    }
}

impl<Cx: Send + Sync + 'static> ProvidesResourceStore for ServerHandlerInner<Cx> {
    fn store(&self) -> &dyn ResourceStore {
        self.store.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesResourceStore for ServerHandler<Cx> {
    fn store(&self) -> &dyn ResourceStore {
        self.handler.store.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesStorageAccessStore for ServerHandlerInner<Cx> {
    fn storage_access(&self) -> &dyn StorageAccessStore {
        self.storage_access.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesStorageAccessStore for ServerHandler<Cx> {
    fn storage_access(&self) -> &dyn StorageAccessStore {
        self.handler.storage_access.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesCommitCoordinator for ServerHandlerInner<Cx> {
    fn commit_coordinator(&self) -> &dyn CommitCoordinator {
        self.commit_coordinator.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesCommitCoordinator for ServerHandler<Cx> {
    fn commit_coordinator(&self) -> &dyn CommitCoordinator {
        self.handler.commit_coordinator.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesLocalStoragePolicy for ServerHandlerInner<Cx> {
    fn local_storage_policy(&self) -> &LocalStoragePolicy {
        self.local_storage_policy.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesLocalStoragePolicy for ServerHandler<Cx> {
    fn local_storage_policy(&self) -> &LocalStoragePolicy {
        self.handler.local_storage_policy.as_ref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesManagedStorageRoot for ServerHandlerInner<Cx> {
    fn managed_storage_root(&self) -> Option<&str> {
        self.managed_storage_root.as_deref()
    }
}

impl<Cx: Send + Sync + 'static> ProvidesManagedStorageRoot for ServerHandler<Cx> {
    fn managed_storage_root(&self) -> Option<&str> {
        self.handler.managed_storage_root.as_deref()
    }
}
