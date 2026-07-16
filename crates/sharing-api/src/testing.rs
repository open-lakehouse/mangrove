//! An in-memory [`SharingBackend`] for exercising the sharing logic without a
//! real server.
//!
//! Enabled by the `testing` feature. The crate's own tests use it to drive the
//! discovery, asset, and credential-vending surface end-to-end; downstream
//! servers can enable it to test their own port wiring.
//!
//! It is deliberately permissive: authorization allows everything, credential
//! vending returns a fixed fake AWS credential, and the object-store factory
//! hands back an in-memory store. The point is to exercise the *sharing* logic
//! (discovery iteration, asset resolution, the router wiring), not access control
//! or live Delta-log serving — the kernel query path needs a real Delta table and
//! is covered by the server's integration tests.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use object_store::DynObjectStore;
use object_store::memory::InMemory;
use url::Url;

use unitycatalog_sharing_client::models::open_sharing::v1::{
    SharingAwsCredentials, SharingTemporaryCredentials, sharing_temporary_credentials::Credentials,
};

use crate::backend::{
    BackendResult, ResolvedLocation, ResolvedShare, ShareObject, ShareObjectKind, SharedAssetKind,
    SharingAction, SharingBackend, SharingCapabilities, SharingTableReference,
    SharingVolumeReference,
};
use crate::error::{Error, Result};
use crate::kernel::ObjectStoreFactory;
use crate::session::KernelSession;

/// One shared object in the fixture: its kind, `schema.name`, and storage location.
#[derive(Debug, Clone)]
pub struct FixtureObject {
    pub kind: ShareObjectKind,
    pub schema: String,
    pub name: String,
    pub location: String,
}

/// A fixture share: identity + its objects.
#[derive(Debug, Clone, Default)]
pub struct FixtureShare {
    pub name: String,
    pub id: Option<String>,
    pub comment: Option<String>,
    pub objects: Vec<FixtureObject>,
}

/// An [`ObjectStoreFactory`] that always hands back a single shared in-memory store.
struct InMemoryFactory {
    store: Arc<DynObjectStore>,
}

#[async_trait]
impl ObjectStoreFactory for InMemoryFactory {
    async fn create_object_store(&self, _url: &Url) -> Result<Arc<DynObjectStore>> {
        Ok(self.store.clone())
    }
}

/// An in-memory [`SharingBackend`] (see the module docs).
pub struct InMemorySharingBackend {
    shares: BTreeMap<String, FixtureShare>,
    session: KernelSession,
}

impl InMemorySharingBackend {
    /// Build a backend over the given fixture shares.
    pub fn new(shares: Vec<FixtureShare>) -> Self {
        let store: Arc<DynObjectStore> = Arc::new(InMemory::new());
        let factory = Arc::new(InMemoryFactory { store });
        let session = KernelSession::new(factory).expect("kernel session builds in-memory");
        Self {
            shares: shares.into_iter().map(|s| (s.name.clone(), s)).collect(),
            session,
        }
    }

    /// A convenience constructor with one share containing one table.
    pub fn with_one_table(share: &str, schema: &str, table: &str, location: &str) -> Self {
        Self::new(vec![FixtureShare {
            name: share.to_string(),
            id: Some(format!("{share}-id")),
            comment: None,
            objects: vec![FixtureObject {
                kind: ShareObjectKind::Table,
                schema: schema.to_string(),
                name: table.to_string(),
                location: location.to_string(),
            }],
        }])
    }

    fn share(&self, name: &str) -> BackendResult<&FixtureShare> {
        self.shares.get(name).ok_or(Error::NotFound)
    }
}

impl Clone for InMemorySharingBackend {
    fn clone(&self) -> Self {
        // Cloning rebuilds a fresh session over the same fixtures; the axum router
        // only needs `Clone` to hand state to each request, and the in-memory
        // fixtures are immutable, so a fresh session is equivalent.
        Self::new(self.shares.values().cloned().collect())
    }
}

fn matches(kind: ShareObjectKind, want: ShareObjectKind) -> bool {
    kind == want
}

#[async_trait]
impl<Cx: Clone + Send + Sync + 'static> SharingBackend<Cx> for InMemorySharingBackend {
    fn capabilities(&self) -> SharingCapabilities {
        SharingCapabilities::default()
    }

    fn kernel_session(&self) -> &KernelSession {
        &self.session
    }

    async fn authorize(&self, _action: SharingAction<'_>, _cx: &Cx) -> BackendResult<()> {
        Ok(())
    }

    async fn list_shares(
        &self,
        max_results: Option<usize>,
        _page_token: Option<String>,
        _cx: &Cx,
    ) -> BackendResult<(Vec<ResolvedShare>, Option<String>)> {
        let mut shares: Vec<ResolvedShare> = self
            .shares
            .values()
            .map(|s| ResolvedShare {
                name: s.name.clone(),
                id: s.id.clone(),
                comment: s.comment.clone(),
            })
            .collect();
        if let Some(limit) = max_results {
            shares.truncate(limit);
        }
        Ok((shares, None))
    }

    async fn get_share(&self, share: &str, _cx: &Cx) -> BackendResult<ResolvedShare> {
        let s = self.share(share)?;
        Ok(ResolvedShare {
            name: s.name.clone(),
            id: s.id.clone(),
            comment: s.comment.clone(),
        })
    }

    async fn list_share_objects(
        &self,
        share: &str,
        kind: ShareObjectKind,
        _cx: &Cx,
    ) -> BackendResult<Vec<ShareObject>> {
        let s = self.share(share)?;
        Ok(s.objects
            .iter()
            .filter(|o| matches(o.kind, kind))
            .map(|o| ShareObject {
                schema: o.schema.clone(),
                name: o.name.clone(),
            })
            .collect())
    }

    async fn resolve_table_location(
        &self,
        table: &SharingTableReference,
        _cx: &Cx,
    ) -> BackendResult<ResolvedLocation> {
        let s = self.share(&table.share)?;
        let obj = s
            .objects
            .iter()
            .find(|o| {
                matches(o.kind, ShareObjectKind::Table)
                    && o.schema == table.schema
                    && o.name == table.table
            })
            .ok_or(Error::NotFound)?;
        ResolvedLocation::parse(&obj.location)
    }

    async fn resolve_asset_location(
        &self,
        asset: &SharingVolumeReference,
        kind: SharedAssetKind,
        _cx: &Cx,
    ) -> BackendResult<ResolvedLocation> {
        let want = match kind {
            SharedAssetKind::Volume => ShareObjectKind::Volume,
            SharedAssetKind::AgentSkill => ShareObjectKind::AgentSkill,
        };
        let s = self.share(&asset.share)?;
        let obj = s
            .objects
            .iter()
            .find(|o| matches(o.kind, want) && o.schema == asset.schema && o.name == asset.name)
            .ok_or(Error::NotFound)?;
        ResolvedLocation::parse(&obj.location)
    }

    async fn vend_read_credential(
        &self,
        location: &ResolvedLocation,
        _cx: &Cx,
    ) -> BackendResult<SharingTemporaryCredentials> {
        Ok(SharingTemporaryCredentials {
            expiration_time: 1_700_000_000_000,
            url: Some(location.raw.clone()),
            credentials: Some(Credentials::AwsTempCredentials(Box::new(
                SharingAwsCredentials {
                    access_key_id: "AKIAFAKE".to_string(),
                    secret_access_key: "fake-secret".to_string(),
                    session_token: "fake-token".to_string(),
                    ..Default::default()
                },
            ))),
            ..Default::default()
        })
    }
}
