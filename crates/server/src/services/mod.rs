use std::sync::Arc;

use crate::Result;
use crate::policy::{Decision, Permission, Policy, ProvidesPolicy};
use crate::store::{ProvidesResourceStore, ResourceStore};
use unitycatalog_common::models::ResourceIdent;
use unitycatalog_delta_api::coordinator::{
    CommitCoordinator, InMemoryCommitCoordinator, ProvidesCommitCoordinator,
};

pub mod credential_vending;
mod delta_backend;
pub mod location;
pub mod location_policy;
pub(crate) mod object_store;

pub use location_policy::LocalStoragePolicy;

/// Access to the server's [`LocalStoragePolicy`].
///
/// Implemented by the server handler so request handlers (defined as blanket
/// impls over a generic `T`) can reach the server-wide allowlist that governs
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
    pub fn try_new_tokio(
        policy: Arc<dyn Policy<Cx>>,
        store: Arc<dyn ResourceStore>,
    ) -> Result<Self> {
        Self::try_new_tokio_with_coordinator(
            policy,
            store,
            Arc::new(InMemoryCommitCoordinator::default()),
        )
    }

    /// Construct a handler backed by a specific [`CommitCoordinator`].
    ///
    /// Use this to wire a persistent coordinator (e.g. the Postgres-backed
    /// `GraphStore`) instead of the default in-memory one.
    pub fn try_new_tokio_with_coordinator(
        policy: Arc<dyn Policy<Cx>>,
        store: Arc<dyn ResourceStore>,
        commit_coordinator: Arc<dyn CommitCoordinator>,
    ) -> Result<Self> {
        let handler = Arc::new(
            ServerHandlerInner::new(policy.clone(), store.clone())
                .with_commit_coordinator(commit_coordinator),
        );
        Ok(Self { handler })
    }
}

impl<Cx: Send + Sync + 'static> ServerHandler<Cx> {
    /// Set the allowlist governing `file://` storage locations.
    ///
    /// Rebuilds the inner handler with the policy attached. Call at construction
    /// time, before the handler is cloned/shared. When unset, all local storage
    /// is denied.
    pub fn with_local_storage_policy(mut self, policy: impl Into<Arc<LocalStoragePolicy>>) -> Self {
        // Rebuild the inner handler with the policy attached. All inner fields
        // are `Arc`s, so this is a cheap reconstruction (the derived `Clone`
        // would require `Cx: Clone`, which we don't impose).
        let prev = &self.handler;
        let inner = ServerHandlerInner {
            policy: prev.policy.clone(),
            store: prev.store.clone(),
            commit_coordinator: prev.commit_coordinator.clone(),
            local_storage_policy: policy.into(),
            managed_storage_root: prev.managed_storage_root.clone(),
        };
        self.handler = Arc::new(inner);
        self
    }

    /// Set the metastore-level managed storage root.
    ///
    /// Rebuilds the inner handler with the root attached. Call at construction
    /// time, before the handler is cloned/shared. When unset, managed catalogs
    /// must each supply their own `storage_root`.
    pub fn with_managed_storage_root(mut self, root: Option<impl Into<Arc<str>>>) -> Self {
        let prev = &self.handler;
        let inner = ServerHandlerInner {
            policy: prev.policy.clone(),
            store: prev.store.clone(),
            commit_coordinator: prev.commit_coordinator.clone(),
            local_storage_policy: prev.local_storage_policy.clone(),
            managed_storage_root: root.map(Into::into),
        };
        self.handler = Arc::new(inner);
        self
    }
}

#[derive(Clone)]
pub struct ServerHandlerInner<Cx> {
    policy: Arc<dyn Policy<Cx>>,
    store: Arc<dyn ResourceStore>,
    /// Delta catalog-managed commit coordinator (in-memory by default).
    commit_coordinator: Arc<dyn CommitCoordinator>,
    /// Allowlist governing which host paths may back a `file://` storage
    /// location. Deny-all by default (see [`LocalStoragePolicy`]).
    local_storage_policy: Arc<LocalStoragePolicy>,
    /// Metastore-level managed storage root inherited by managed catalogs that
    /// omit `storage_root`. `None` ⇒ no metastore default (see
    /// [`ProvidesManagedStorageRoot`]).
    managed_storage_root: Option<Arc<str>>,
}

impl<Cx: Send + Sync + 'static> ServerHandlerInner<Cx> {
    pub fn new(policy: Arc<dyn Policy<Cx>>, store: Arc<dyn ResourceStore>) -> Self {
        Self {
            policy,
            store,
            commit_coordinator: Arc::new(InMemoryCommitCoordinator::default()),
            // Deny all local (file://) storage until a policy is configured.
            local_storage_policy: Arc::new(LocalStoragePolicy::deny_all()),
            // No metastore-level managed storage root by default.
            managed_storage_root: None,
        }
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
    pub fn with_managed_storage_root(mut self, root: Option<impl Into<Arc<str>>>) -> Self {
        self.managed_storage_root = root.map(Into::into);
        self
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
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<Decision> {
        self.policy().authorize(resource, permission, context).await
    }

    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        self.policy()
            .authorize_many(resources, permission, context)
            .await
    }
}

#[async_trait::async_trait]
impl<Cx: Send + Sync + 'static> Policy<Cx> for ServerHandler<Cx> {
    async fn authorize(
        &self,
        resource: &ResourceIdent,
        permission: &Permission,
        context: &Cx,
    ) -> Result<Decision> {
        self.handler
            .policy
            .authorize(resource, permission, context)
            .await
    }

    async fn authorize_many(
        &self,
        resources: &[ResourceIdent],
        permission: &Permission,
        context: &Cx,
    ) -> Result<Vec<Decision>> {
        self.handler
            .policy
            .authorize_many(resources, permission, context)
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
