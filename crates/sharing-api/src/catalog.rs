//! The [`ShareCatalog`] port: what a recipient can see and read.
//!
//! A share catalog answers the discovery questions of the sharing protocol —
//! which shares does this caller see, which objects does a share contain — and
//! resolves a shared object to a [`SharedAsset`]: its storage location, format
//! and the history the provider shares. How shares are stored, and which source
//! catalog an object's location ultimately comes from (Unity Catalog, an Iceberg
//! REST catalog, a static configuration, …), is entirely the implementation's
//! concern.
//!
//! # Authorization contract
//!
//! Authorization goes through **one** hook, [`ShareCatalog::authorize`], which the
//! service calls with a [`SharingAction`] before every recipient-facing operation.
//! The data methods are pure data access and must not perform their own
//! caller-facing authorization.

use async_trait::async_trait;
use delta_kernel::Version;

use open_sharing_common::models::open_sharing::v1::Share as SharingShare;

use crate::error::{Error, Result};

/// The kind of object shared in a share.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetKind {
    /// A table, read through its [`DataFormat`].
    Table,
    /// A volume: a storage prefix shared as files.
    Volume,
    /// An agent skill: a directory (`SKILL.md` + scripts/assets) shared on top of
    /// a volume.
    AgentSkill,
}

/// The on-storage format of a shared table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DataFormat {
    /// A Delta Lake table.
    Delta,
    /// An Apache Iceberg table.
    Iceberg,
}

/// A share, as listed to a recipient.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ShareInfo {
    pub name: String,
    pub id: Option<String>,
    pub comment: Option<String>,
}

impl From<ShareInfo> for SharingShare {
    fn from(s: ShareInfo) -> Self {
        SharingShare {
            name: s.name,
            id: s.id,
            comment: s.comment,
            ..Default::default()
        }
    }
}

/// One object listed under a share, under its shared (`schema`, `name`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedObject {
    /// The schema the object is shared under.
    pub schema: String,
    /// The object's shared name within its schema.
    pub name: String,
    /// What kind of object this is.
    pub kind: AssetKind,
    /// A stable identifier for the object, when the catalog has one.
    pub id: Option<String>,
}

/// A fully-qualified reference to a shared object (`share.schema.name`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssetRef {
    pub share: String,
    pub schema: String,
    pub name: String,
    pub kind: AssetKind,
}

impl AssetRef {
    pub fn new(
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
        kind: AssetKind,
    ) -> Self {
        Self {
            share: share.into(),
            schema: schema.into(),
            name: name.into(),
            kind,
        }
    }

    /// A table reference.
    pub fn table(
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self::new(share, schema, name, AssetKind::Table)
    }
}

/// A storage location. Carries the canonical URL string alongside its parsed form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedLocation {
    pub url: url::Url,
    pub raw: String,
}

impl ResolvedLocation {
    pub fn parse(raw: impl Into<String>) -> Result<Self> {
        let raw = raw.into();
        let url = url::Url::parse(&raw)?;
        Ok(Self { url, raw })
    }

    /// The location as a directory URL (with a trailing `/`), so that joining a
    /// relative path appends to it rather than replacing its last segment.
    pub fn as_directory(&self) -> url::Url {
        let mut url = self.url.clone();
        if !url.path().ends_with('/') {
            url.set_path(&format!("{}/", url.path()));
        }
        url
    }
}

/// A shared object resolved to everything needed to serve it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedAsset {
    /// The shared coordinates the recipient addressed.
    pub reference: AssetRef,
    /// A stable identifier for the asset, when the catalog has one.
    pub id: Option<String>,
    /// Where the asset's data lives.
    pub location: ResolvedLocation,
    /// The table format. `None` for volumes and agent skills.
    pub format: Option<DataFormat>,
    /// Whether the provider shares the table's history. When `false`, only the
    /// latest version can be read: version / timestamp queries are rejected.
    pub history_shared: bool,
    /// The lowest table version recipients may read, when the provider limits it.
    pub start_version: Option<Version>,
    /// The asset's name in the source catalog it was resolved from (for example
    /// `catalog.schema.table`), so a [`StorageAccess`](crate::StorageAccess) can
    /// ask that catalog for credentials. `None` when there is no source catalog.
    pub source_name: Option<String>,
}

impl SharedAsset {
    /// A Delta table at `location`, sharing its full history.
    pub fn delta_table(reference: AssetRef, location: ResolvedLocation) -> Self {
        Self {
            reference,
            id: None,
            location,
            format: Some(DataFormat::Delta),
            history_shared: true,
            start_version: None,
            source_name: None,
        }
    }

    /// A storage-backed asset (volume / agent skill) at `location`.
    pub fn storage(reference: AssetRef, location: ResolvedLocation) -> Self {
        Self {
            reference,
            id: None,
            location,
            format: None,
            history_shared: false,
            start_version: None,
            source_name: None,
        }
    }

    /// The asset's table format, or an error for a non-table asset.
    pub fn table_format(&self) -> Result<DataFormat> {
        self.format
            .ok_or_else(|| Error::invalid_argument("shared asset is not a table"))
    }
}

/// The action the service authorizes before an operation. Scoped to the share;
/// the concrete object is checked when it is resolved.
#[derive(Debug, Clone)]
pub enum SharingAction<'a> {
    /// List shares accessible to the caller.
    ListShares,
    /// Read a specific share (discovery, listing, querying its objects).
    ReadShare { share: &'a str },
}

/// A page request: the requested maximum size and the continuation token.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PageRequest {
    pub max_results: Option<usize>,
    pub page_token: Option<String>,
}

/// One page of results, plus the token for the next page (if any).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub next_page_token: Option<String>,
}

/// The recipient-facing share catalog port. See the module docs.
#[async_trait]
pub trait ShareCatalog<Cx = crate::DefaultContext>: Send + Sync + 'static {
    /// Authorize an action before the service performs it.
    async fn authorize(&self, action: SharingAction<'_>, cx: &Cx) -> Result<()>;

    /// The shares visible to the caller, one page at a time.
    async fn list_shares(&self, page: PageRequest, cx: &Cx) -> Result<Page<ShareInfo>>;

    /// A share by name. Missing → [`Error::NotFound`].
    async fn get_share(&self, share: &str, cx: &Cx) -> Result<ShareInfo>;

    /// Every object in a share, of every kind, in a stable order.
    ///
    /// The service derives schemas and per-kind listings from this, and pages
    /// over it, so the order must be stable across calls.
    async fn list_objects(&self, share: &str, cx: &Cx) -> Result<Vec<SharedObject>>;

    /// Resolve a shared object. Missing (or not of `asset.kind`) →
    /// [`Error::NotFound`].
    async fn resolve(&self, asset: &AssetRef, cx: &Cx) -> Result<SharedAsset>;
}
