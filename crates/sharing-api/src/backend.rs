//! The [`SharingBackend`] port: the narrow backend interface the sharing business
//! logic calls.
//!
//! All sharing *semantics* — share/schema/table discovery, the kernel-backed
//! NDJSON query path, and the storage-backed asset (volume / agent skill) surface
//! — live in [`crate::handler`] and are expressed in terms of this trait. A server
//! implements only these operations over its own share store, table/volume
//! resolution, credential vending, and authorization; it never re-implements any
//! sharing semantics.
//!
//! The trait is generic over a context `Cx` (mangrove's `RequestContext`, …)
//! threaded from the axum router. All types exchanged here are the crate's own
//! coordinate/spec structs below plus the `unitycatalog_sharing_client` protocol
//! types — never a server's resource model.
//!
//! # Authorization contract
//!
//! Authorization goes through **one** hook, [`SharingBackend::authorize`], which
//! the handler calls with a [`SharingAction`] before every user-facing operation.
//! The data methods below (`get_share`, `resolve_table_location`,
//! `vend_read_credential`, …) are pure data access and must not perform their own
//! caller-facing authorization — the handler has already authorized the operation.

use async_trait::async_trait;

use unitycatalog_sharing_client::models::open_sharing::v1::{
    Share as SharingShare, SharingTemporaryCredentials,
};

use crate::error::Error;
#[cfg(doc)]
use crate::kernel::ObjectStoreFactory;

/// Result of a [`SharingBackend`] operation.
pub type BackendResult<T> = Result<T, Error>;

/// A fully-qualified shared-table coordinate (`share.schema.table`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharingTableReference {
    pub share: String,
    pub schema: String,
    pub table: String,
}

impl SharingTableReference {
    pub fn new(
        share: impl Into<String>,
        schema: impl Into<String>,
        table: impl Into<String>,
    ) -> Self {
        Self {
            share: share.into(),
            schema: schema.into(),
            table: table.into(),
        }
    }

    /// A stable, unique name for this table in the kernel session's log-replay
    /// schema (`share__schema__table`).
    pub fn system_table_name(&self) -> String {
        format!("{}__{}__{}", self.share, self.schema, self.table)
    }
}

/// A reference to a shared storage-backed asset (volume or agent skill) within a
/// share/schema. Both asset kinds resolve through the backing volume primitive,
/// so they share one reference type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharingVolumeReference {
    pub share: String,
    pub schema: String,
    pub name: String,
}

impl SharingVolumeReference {
    pub fn new(
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        Self {
            share: share.into(),
            schema: schema.into(),
            name: name.into(),
        }
    }
}

/// The kind of shared asset backed by a volume primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SharedAssetKind {
    Volume,
    AgentSkill,
}

/// One object listed under a share, in the crate's portable shape. Derived from a
/// share's data objects; `schema` / `name` come from the object's shared-as name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShareObject {
    /// The schema the object is shared under.
    pub schema: String,
    /// The object's shared name (within its schema).
    pub name: String,
}

/// The kind of object listed under a share.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShareObjectKind {
    Table,
    Volume,
    AgentSkill,
}

/// A share resolved by the backend, with its identity fields. The objects are
/// fetched separately via [`SharingBackend::list_share_objects`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ResolvedShare {
    pub name: String,
    pub id: Option<String>,
    pub comment: Option<String>,
}

impl From<ResolvedShare> for SharingShare {
    fn from(s: ResolvedShare) -> Self {
        SharingShare {
            name: s.name,
            id: s.id,
            comment: s.comment,
            ..Default::default()
        }
    }
}

/// A storage location resolved for a shared table or asset. Carries both the
/// canonical URL string and — since the query path needs a parsed [`url::Url`] —
/// the parsed form.
#[derive(Debug, Clone)]
pub struct ResolvedLocation {
    pub url: url::Url,
    pub raw: String,
}

impl ResolvedLocation {
    pub fn parse(raw: impl Into<String>) -> Result<Self, Error> {
        let raw = raw.into();
        let url = url::Url::parse(&raw)?;
        Ok(Self { url, raw })
    }
}

/// The action the handler authorizes before an operation. Scoped to the share;
/// the concrete table/asset is authorized when its location is resolved.
#[derive(Debug, Clone)]
pub enum SharingAction<'a> {
    /// List shares accessible to the caller.
    ListShares,
    /// Read a specific share (discovery, schema/table listing, table query).
    ReadShare { share: &'a str },
}

/// Optional sharing operations a backend may or may not support. Reserved for
/// future capability negotiation (e.g. `responseformat=delta`, async queries);
/// all flags default to `false`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SharingCapabilities {
    /// Whether the backend can serve `responseformat=delta` query responses.
    pub delta_response_format: bool,
    /// Whether the backend serves the change-data-feed (`/changes`) endpoint.
    pub change_data_feed: bool,
    /// Whether the backend serves asynchronous queries (`asyncquery=true`).
    pub async_query: bool,
}

/// The backend port. See the module docs.
#[async_trait]
pub trait SharingBackend<Cx = crate::DefaultContext>: Send + Sync + 'static {
    /// The optional operations this backend serves. Defaults to all-off.
    fn capabilities(&self) -> SharingCapabilities {
        SharingCapabilities::default()
    }

    /// The kernel-backed DataFusion session that serves the NDJSON query path.
    ///
    /// A backend builds this once (via [`KernelSession::new`] over its
    /// [`ObjectStoreFactory`]) and hands the handler a reference; the handler owns
    /// no DataFusion state itself.
    ///
    /// [`KernelSession::new`]: crate::session::KernelSession::new
    fn kernel_session(&self) -> &crate::session::KernelSession;

    /// Authorize an action before the handler performs it.
    async fn authorize(&self, action: SharingAction<'_>, cx: &Cx) -> BackendResult<()>;

    /// List the shares accessible to the caller, filtered to those the caller may
    /// read, honoring the page token / max-results. Returns the shares plus the
    /// next page token.
    async fn list_shares(
        &self,
        max_results: Option<usize>,
        page_token: Option<String>,
        cx: &Cx,
    ) -> BackendResult<(Vec<ResolvedShare>, Option<String>)>;

    /// Resolve a share by name. Missing → [`Error::NotFound`].
    async fn get_share(&self, share: &str, cx: &Cx) -> BackendResult<ResolvedShare>;

    /// List the objects of a given kind under a share (each as its
    /// `(schema, name)`), in share order.
    async fn list_share_objects(
        &self,
        share: &str,
        kind: ShareObjectKind,
        cx: &Cx,
    ) -> BackendResult<Vec<ShareObject>>;

    /// Resolve the storage location of a shared table. Missing → [`Error::NotFound`].
    async fn resolve_table_location(
        &self,
        table: &SharingTableReference,
        cx: &Cx,
    ) -> BackendResult<ResolvedLocation>;

    /// Resolve the storage location of a shared volume or agent skill.
    /// Missing → [`Error::NotFound`].
    async fn resolve_asset_location(
        &self,
        asset: &SharingVolumeReference,
        kind: SharedAssetKind,
        cx: &Cx,
    ) -> BackendResult<ResolvedLocation>;

    /// Vend read-only credentials for an already-resolved storage location, mapped
    /// into the Open Sharing credential envelope. Open Sharing only grants read
    /// access to shared assets.
    async fn vend_read_credential(
        &self,
        location: &ResolvedLocation,
        cx: &Cx,
    ) -> BackendResult<SharingTemporaryCredentials>;
}
