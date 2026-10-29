//! In-memory port implementations for exercising the sharing logic without a
//! real host.
//!
//! Enabled by the `testing` feature. The crate's own tests use them to drive the
//! discovery, asset, query and credential surface end-to-end; downstream hosts
//! can enable it to test their own wiring.
//!
//! They are deliberately permissive: authorization allows everything and
//! credential vending returns a fixed fake AWS credential. [`LocalStorageAccess`]
//! reads `file://` locations from the local filesystem (so real Delta tables can
//! be queried) and hands back their file URLs unsigned.

use std::collections::BTreeMap;
use std::sync::Arc;

use async_trait::async_trait;
use delta_kernel::Version;
use object_store::DynObjectStore;
use object_store::local::LocalFileSystem;
use object_store::memory::InMemory;

use open_sharing_common::models::open_sharing::v1::{
    SharingAwsCredentials, SharingTemporaryCredentials, sharing_temporary_credentials::Credentials,
};

use crate::access::{StorageAccess, TableStorage, UnsignedUrls};
use crate::catalog::{
    AssetKind, AssetRef, DataFormat, Page, PageRequest, ResolvedLocation, ShareCatalog, ShareInfo,
    SharedAsset, SharedObject, SharingAction,
};
use crate::error::{Error, Result};
use crate::service::{SharingService, paginate};

/// One shared object in a fixture share.
#[derive(Debug, Clone)]
pub struct FixtureObject {
    pub kind: AssetKind,
    pub schema: String,
    pub name: String,
    pub location: String,
    /// Whether the table's history is shared (time travel allowed).
    pub history_shared: bool,
    /// The lowest shared table version.
    pub start_version: Option<Version>,
}

impl FixtureObject {
    /// A Delta table sharing its full history.
    pub fn table(schema: &str, name: &str, location: &str) -> Self {
        Self::new(AssetKind::Table, schema, name, location)
    }

    /// An object of `kind`; tables share their full history.
    pub fn new(kind: AssetKind, schema: &str, name: &str, location: &str) -> Self {
        Self {
            kind,
            schema: schema.to_string(),
            name: name.to_string(),
            location: location.to_string(),
            history_shared: kind == AssetKind::Table,
            start_version: None,
        }
    }
}

/// A fixture share: identity plus its objects.
#[derive(Debug, Clone, Default)]
pub struct FixtureShare {
    pub name: String,
    pub id: Option<String>,
    pub comment: Option<String>,
    pub objects: Vec<FixtureObject>,
}

/// A [`ShareCatalog`] over fixed, in-memory shares.
#[derive(Debug, Clone, Default)]
pub struct InMemoryShareCatalog {
    shares: BTreeMap<String, FixtureShare>,
}

impl InMemoryShareCatalog {
    pub fn new(shares: Vec<FixtureShare>) -> Self {
        Self {
            shares: shares.into_iter().map(|s| (s.name.clone(), s)).collect(),
        }
    }

    fn share(&self, name: &str) -> Result<&FixtureShare> {
        self.shares.get(name).ok_or(Error::NotFound)
    }
}

fn info(share: &FixtureShare) -> ShareInfo {
    ShareInfo {
        name: share.name.clone(),
        id: share.id.clone(),
        comment: share.comment.clone(),
    }
}

#[async_trait]
impl<Cx: Send + Sync + 'static> ShareCatalog<Cx> for InMemoryShareCatalog {
    async fn authorize(&self, _action: SharingAction<'_>, _cx: &Cx) -> Result<()> {
        Ok(())
    }

    async fn list_shares(&self, page: PageRequest, _cx: &Cx) -> Result<Page<ShareInfo>> {
        paginate(
            self.shares.values().map(info).collect(),
            page.max_results.map(|n| n as i32),
            page.page_token.as_deref(),
        )
    }

    async fn get_share(&self, share: &str, _cx: &Cx) -> Result<ShareInfo> {
        Ok(info(self.share(share)?))
    }

    async fn list_objects(&self, share: &str, _cx: &Cx) -> Result<Vec<SharedObject>> {
        Ok(self
            .share(share)?
            .objects
            .iter()
            .map(|o| SharedObject {
                schema: o.schema.clone(),
                name: o.name.clone(),
                kind: o.kind,
                id: Some(format!("{share}.{}.{}", o.schema, o.name)),
            })
            .collect())
    }

    async fn resolve(&self, asset: &AssetRef, _cx: &Cx) -> Result<SharedAsset> {
        let object = self
            .share(&asset.share)?
            .objects
            .iter()
            .find(|o| o.kind == asset.kind && o.schema == asset.schema && o.name == asset.name)
            .ok_or(Error::NotFound)?;
        Ok(SharedAsset {
            reference: asset.clone(),
            id: Some(format!("{}.{}.{}", asset.share, asset.schema, asset.name)),
            location: ResolvedLocation::parse(&object.location)?,
            format: (object.kind == AssetKind::Table).then_some(DataFormat::Delta),
            history_shared: object.history_shared,
            start_version: object.start_version,
            source_name: None,
        })
    }
}

/// A [`StorageAccess`] for tests and local development.
///
/// `file://` locations are read from the local filesystem; anything else gets an
/// empty in-memory store (enough for discovery-only fixtures). File URLs are
/// returned unsigned, and vending returns a fixed fake AWS credential.
#[derive(Debug, Clone, Default)]
pub struct LocalStorageAccess;

#[async_trait]
impl<Cx: Send + Sync + 'static> StorageAccess<Cx> for LocalStorageAccess {
    async fn table_storage(&self, asset: &SharedAsset, _cx: &Cx) -> Result<TableStorage> {
        let store: Arc<DynObjectStore> = if asset.location.url.scheme() == "file" {
            Arc::new(LocalFileSystem::new())
        } else {
            Arc::new(InMemory::new())
        };
        Ok(TableStorage {
            store,
            root: asset.location.as_directory(),
            signer: Arc::new(UnsignedUrls),
        })
    }

    async fn vend_read_credential(
        &self,
        asset: &SharedAsset,
        _cx: &Cx,
    ) -> Result<SharingTemporaryCredentials> {
        Ok(SharingTemporaryCredentials {
            expiration_time: 1_700_000_000_000,
            url: Some(asset.location.raw.clone()),
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

/// A [`SharingService`] over fixture shares, with [`LocalStorageAccess`].
pub fn in_memory_service<Cx: Clone + Send + Sync + 'static>(
    shares: Vec<FixtureShare>,
) -> SharingService<Cx> {
    SharingService::new(
        Arc::new(InMemoryShareCatalog::new(shares)),
        Arc::new(LocalStorageAccess),
    )
}
