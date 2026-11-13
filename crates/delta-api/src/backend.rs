//! The `DeltaBackend` port: the narrow backend interface the Delta business logic
//! calls.
//!
//! All Delta *semantics* — the managed-table contract, the `updateTable` action
//! dispatcher, `loadTable` list construction, credential→config mapping, and
//! commit arbitration — live in [`crate::handler`] and are expressed in terms of
//! this trait. A server implements only these operations over its own storage,
//! credential vending, and authorization; it never re-implements any Delta
//! semantics.
//!
//! The trait is generic over a context `Cx` (mangrove's `RequestContext`,
//! lakekeeper's `RequestMetadata`, …) threaded from the axum router. All types
//! exchanged here are the crate's own — the Delta wire DTOs, [`crate::column`],
//! and the coordinate/spec structs below — never a server's resource model.
//!
//! # Authorization contract
//!
//! Authorization goes through **one** hook, [`DeltaBackend::authorize`], which the
//! handler calls with a [`DeltaAction`] before every user-facing operation —
//! *before* resolving the target, so an unauthorized caller cannot tell a missing
//! table from a forbidden one. The data methods below (`resolve_table`,
//! `create_table_row`, `apply_update`, `delete_table`, `vend_*`,
//! `resolve_staging_*`, …) are **pure data access and must not perform their own
//! authorization** — the handler has already authorized the operation. (An
//! implementor is of course free to enforce coarse storage-level guards, but the
//! caller-facing access decision belongs in `authorize`.) See [`crate::authz`].
//!
//! # Constructing port types
//!
//! The structs a backend builds ([`ResolvedTable`], [`StagingReservation`],
//! [`VendedCredential`], [`DeltaCapabilities`]) and the specs the handler hands it
//! are `#[non_exhaustive]`, so fields can be added without a breaking release.
//! Build them with their constructor, then assign the public fields:
//!
//! ```
//! # #[cfg(feature = "server")] {
//! use unitycatalog_delta_api::backend::ResolvedTable;
//! use unitycatalog_delta_api::models::{DeltaDataSourceFormat, DeltaTableType};
//!
//! let mut table = ResolvedTable::new("s3://bucket/tables/t");
//! table.table_id = Some("0190d3b2-7c1e-7a35-8f7e-2d3f4a5b6c7d".into());
//! table.table_type = Some(DeltaTableType::Managed);
//! table.data_source_format = Some(DeltaDataSourceFormat::Delta);
//! table.version = 3;
//! # }
//! ```

use std::collections::BTreeMap;

use async_trait::async_trait;

use serde::Deserialize;

use crate::authz::DeltaAction;
use crate::column::Column;
use crate::coordinator::{CommitCoordinator, CommitInfo};
use crate::error::DeltaBackendError;
use crate::models::{DeltaDataSourceFormat, DeltaTableType};

/// Result of a [`DeltaBackend`] operation.
pub type BackendResult<T> = Result<T, DeltaBackendError>;

/// A fully-qualified table coordinate.
///
/// Deserializes from the router's `{catalog}/{schema}/{table}` path parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct TableRef {
    pub catalog: String,
    pub schema: String,
    pub table: String,
}

impl TableRef {
    /// The dotted `catalog.schema.table` full name.
    pub fn full_name(&self) -> String {
        format!("{}.{}.{}", self.catalog, self.schema, self.table)
    }
}

/// A schema coordinate (the parent of table / staging-table creation).
///
/// Deserializes from the router's `{catalog}/{schema}` path parameters.
#[derive(Debug, Clone, Deserialize)]
pub struct SchemaRef {
    pub catalog: String,
    pub schema: String,
}

/// A table resolved by the backend, in the crate's portable shape.
///
/// Build with [`ResolvedTable::new`] and assign fields (the struct is
/// `#[non_exhaustive]`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ResolvedTable {
    /// The table UUID (as a string). `None` for tables the backend does not
    /// assign an id to (which the Delta API rejects downstream).
    pub table_id: Option<String>,
    /// The table's storage root location.
    pub location: String,
    /// The Delta table type, or `None` for table types the Delta API cannot
    /// serve (views, metric views, …), which `loadTable` rejects with 400.
    pub table_type: Option<DeltaTableType>,
    /// The table's data source format, if known. Commit-coordinator state is
    /// only attached to MANAGED tables whose format is
    /// [`DeltaDataSourceFormat::Delta`].
    pub data_source_format: Option<DeltaDataSourceFormat>,
    pub columns: Vec<Column>,
    pub properties: BTreeMap<String, String>,
    /// The table comment, if any.
    pub comment: Option<String>,
    /// Creation time, epoch milliseconds.
    pub created_at_ms: Option<i64>,
    /// Last-update time, epoch milliseconds. Surfaced as `updated_time`.
    pub updated_at_ms: Option<i64>,
    /// The backend's monotonic per-object version. Drives the etag and the
    /// `assert-etag` compare-and-swap (see [`etag_of`]). A backend that does not
    /// track versions leaves this `0`.
    pub version: u64,
}

impl ResolvedTable {
    /// A table at `location` with every other field empty / `None` / `0`.
    pub fn new(location: impl Into<String>) -> Self {
        Self {
            table_id: None,
            location: location.into(),
            table_type: None,
            data_source_format: None,
            columns: Vec::new(),
            properties: BTreeMap::new(),
            comment: None,
            created_at_ms: None,
            updated_at_ms: None,
            version: 0,
        }
    }
}

/// The etag for a resolved table: `etag-<version>`.
///
/// This is the single definition of the table etag, shared by the handler (which
/// hands it to the client on `loadTable`) and by any backend enforcing the
/// `assert-etag` compare-and-swap in [`DeltaBackend::update_table_row`], so the
/// asserted etag and the compared etag are always derived identically. The
/// version is the backend's monotonic per-object counter, bumped on every
/// mutation — a purpose-built etag that maps directly to the store's
/// version-precondition CAS.
pub fn etag_of(table: &ResolvedTable) -> String {
    etag_of_version(table.version)
}

/// The etag formula over the version it depends on, for backends that hold the
/// version without a full [`ResolvedTable`]. See [`etag_of`].
pub fn etag_of_version(version: u64) -> String {
    format!("etag-{version}")
}

/// A staging-table reservation (uuid + managed location) allocated before a
/// managed `createTable`.
///
/// Build with [`StagingReservation::new`] and assign the optional fields (the
/// struct is `#[non_exhaustive]`).
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct StagingReservation {
    /// The reservation UUID the created table adopts.
    pub table_id: String,
    /// The reservation's name, as the backend keys it.
    ///
    /// Carried so a backend that consumes the reservation during
    /// [`create_table_row`](DeltaBackend::create_table_row) can delete it by its
    /// store key without a second lookup.
    pub name: String,
    /// The managed location under which the client writes the initial commit.
    pub location: String,
    /// The principal that created the reservation, for the creator-match check.
    pub created_by: Option<String>,
    /// Whether the reservation has already been finalized into a table.
    pub stage_committed: bool,
}

impl StagingReservation {
    /// An uncommitted reservation with no recorded creator.
    pub fn new(
        table_id: impl Into<String>,
        name: impl Into<String>,
        location: impl Into<String>,
    ) -> Self {
        Self {
            table_id: table_id.into(),
            name: name.into(),
            location: location.into(),
            created_by: None,
            stage_committed: false,
        }
    }
}

/// The access level for a vended storage credential.
pub use olai_storage_credentials::AccessLevel as CredentialAccess;

/// A credential vended by the backend: the catalog-independent model, plus the
/// location it covers as the catalog spells it, which the handler sends as the
/// wire `prefix`.
///
/// The prefix is the catalog's own spelling of the table / path location (no
/// trailing slash), matching the UC spec examples and the Java reference server
/// (`DeltaCredentialsMapper`), rather than the credential scope's canonical
/// directory form.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct VendedCredential {
    /// The prefix / URL the credential applies to.
    pub url: String,
    /// The credential.
    pub credential: olai_storage_credentials::VendedCredential,
}

impl VendedCredential {
    /// A credential for `url`.
    pub fn new(
        url: impl Into<String>,
        credential: olai_storage_credentials::VendedCredential,
    ) -> Self {
        Self {
            url: url.into(),
            credential,
        }
    }
}

/// Specification for persisting a new Delta table row. The handler has already
/// validated the contract and derived the UC-shaped columns + stored properties.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct CreateTableSpec {
    pub at: SchemaRef,
    pub name: String,
    pub table_type: DeltaTableType,
    pub location: String,
    pub comment: Option<String>,
    pub columns: Vec<Column>,
    pub properties: BTreeMap<String, String>,
    /// For MANAGED tables, the adopted staging reservation id; `None` for EXTERNAL
    /// (the backend assigns the id).
    pub table_id: Option<String>,
    /// For MANAGED tables, the staging reservation being adopted, already resolved
    /// and authorized by the handler.
    ///
    /// When `Some`, `create_table_row` must **atomically consume this reservation
    /// and create the table** adopting its id (`table_id` above equals
    /// `adopt_staging.table_id`) — a transactional backend does both in one
    /// transaction, closing the orphaned-reservation race. `None` for EXTERNAL
    /// tables, which have no reservation.
    pub adopt_staging: Option<StagingReservation>,
}

/// Specification for persisting metadata changes to an existing table (the
/// non-commit half of `updateTable`). The handler has already applied the
/// canonical-order action dispatch to produce the new columns/properties/comment.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct UpdateTableSpec {
    pub table_id: String,
    pub columns: Vec<Column>,
    pub properties: BTreeMap<String, String>,
    /// The new comment. Unlike `columns`/`properties` (full replacement
    /// snapshots), this is a delta: `None` leaves the stored comment unchanged
    /// (the wire has no clear-comment action); `Some(c)` sets it.
    pub comment: Option<String>,
    /// The `assert-etag` precondition, when the client sent one.
    ///
    /// `Some(etag)` means the write is a **compare-and-swap**: the backend must
    /// verify the table's current etag still equals `etag` *at write time* and
    /// return [`DeltaBackendError::UpdateRequirementConflict`] on mismatch —
    /// closing the read-modify-write race. `None` means no `assert-etag` was
    /// asserted and the write is unconditional. (`assert-table-uuid` is checked in
    /// the handler and never reaches the port.)
    ///
    /// [`DeltaBackendError::UpdateRequirementConflict`]: crate::error::DeltaBackendError::UpdateRequirementConflict
    pub expected_etag: Option<String>,
}

impl UpdateTableSpec {
    /// A metadata write for `table_id` with no etag precondition and no comment
    /// change.
    pub fn new(
        table_id: impl Into<String>,
        columns: Vec<Column>,
        properties: BTreeMap<String, String>,
    ) -> Self {
        Self {
            table_id: table_id.into(),
            columns,
            properties,
            comment: None,
            expected_etag: None,
        }
    }
}

/// One `updateTable` request, fully validated and applied in memory by the
/// handler, for the backend to persist **atomically** via
/// [`DeltaBackend::apply_update`].
///
/// The spec requires the actions of one request to apply atomically: a 409 tells
/// the client the commit "definitively did not land", and it then rebases. So the
/// metadata write (with its `assert-etag` compare-and-swap) and the commit
/// ratification must succeed or fail together.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct TableUpdate {
    /// The table as resolved (and authorized) before the update — the snapshot the
    /// update was computed from. The default `apply_update` uses it to undo the
    /// metadata write when the commit is rejected.
    pub base: ResolvedTable,
    /// The metadata change, or `None` when the request changes no metadata (a pure
    /// `add-commit` / backfill). Its `expected_etag` carries any `assert-etag`.
    pub metadata: Option<UpdateTableSpec>,
    /// The `add-commit` to ratify, if any.
    pub commit: Option<CommitInfo>,
    /// The `set-latest-backfilled-version` to record, if any.
    pub latest_backfilled_version: Option<i64>,
}

impl TableUpdate {
    /// Whether the update touches the commit log.
    pub fn has_commit_log_change(&self) -> bool {
        self.commit.is_some() || self.latest_backfilled_version.is_some()
    }
}

/// Optional Delta operations a backend may or may not support.
///
/// Reported by [`DeltaBackend::capabilities`] and used by `getConfig` to advertise
/// only the endpoints the backend actually serves — so a discovery-driven client is
/// never pointed at an operation the backend will `501` on. All flags default to
/// the conservative `false`; a backend opts in by overriding `capabilities()`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct DeltaCapabilities {
    /// Whether `renameTable` is served. The port models rename as optionally
    /// [`NotImplemented`](crate::error::DeltaBackendError::NotImplemented) (see
    /// [`DeltaBackend::rename_table`]); a backend whose `rename_table` returns
    /// `NotImplemented` must leave this `false` so `getConfig` does not advertise an
    /// endpoint that 501s.
    pub rename: bool,
}

impl DeltaCapabilities {
    /// Advertise (or not) `renameTable`.
    pub fn with_rename(mut self, rename: bool) -> Self {
        self.rename = rename;
        self
    }
}

/// The backend port. See the module docs.
#[async_trait]
pub trait DeltaBackend<Cx: Send + Sync = ()>: Send + Sync + 'static {
    /// The optional operations this backend serves, used by `getConfig` to advertise
    /// a capability-accurate endpoint list. Defaults to [`DeltaCapabilities::default`]
    /// (all off), so a backend advertises an optional operation only by opting in.
    fn capabilities(&self) -> DeltaCapabilities {
        DeltaCapabilities::default()
    }

    /// Authorize an action before the handler performs it.
    ///
    /// The single authorization seam (see the [module authz contract](self)):
    /// the handler calls this with a [`DeltaAction`] at the top of every
    /// user-facing operation. Deny by returning
    /// [`DeltaBackendError::PermissionDenied`] (→ 403) or
    /// [`DeltaBackendError::Unauthenticated`] (→ 401).
    async fn authorize(&self, action: DeltaAction<'_>, cx: &Cx) -> BackendResult<()>;

    /// Confirm the catalog exists (for `getConfig`). Missing →
    /// [`DeltaBackendError::NotFound`] (reported as
    /// [`NoSuchCatalog`](DeltaBackendError::NoSuchCatalog)).
    async fn catalog_exists(&self, catalog: &str, cx: &Cx) -> BackendResult<()>;

    /// Resolve a table by 3-part name. Missing → [`DeltaBackendError::NotFound`].
    async fn resolve_table(&self, table: &TableRef, cx: &Cx) -> BackendResult<ResolvedTable>;

    /// Validate that an EXTERNAL table location lies within a registered external
    /// location.
    async fn validate_external_location(&self, location: &str, cx: &Cx) -> BackendResult<()>;

    /// Persist a new Delta table row and return it resolved.
    ///
    /// When [`spec.adopt_staging`](CreateTableSpec::adopt_staging) is `Some`, this
    /// call **atomically consumes that staging reservation and creates the table**
    /// adopting its id (a transactional backend does both in one transaction,
    /// closing the orphaned-reservation race; a non-transactional backend does the
    /// two steps back-to-back with the same window as before).
    async fn create_table_row(
        &self,
        spec: CreateTableSpec,
        cx: &Cx,
    ) -> BackendResult<ResolvedTable>;

    /// Persist one `updateTable` request — metadata change and/or commit-log
    /// change — **atomically**, and return the refreshed table.
    ///
    /// The contract (mirroring the Java reference's single transaction):
    ///
    /// 1. If `update.metadata` carries an `expected_etag`, the write is a
    ///    compare-and-swap against the table's *current* etag; on mismatch return
    ///    [`DeltaBackendError::UpdateRequirementConflict`] and change nothing.
    /// 2. Ratify `update.commit` / `update.latest_backfilled_version` through the
    ///    commit coordinator. On any rejection nothing persists, including the
    ///    metadata write. A retry of an already-ratified commit returns
    ///    [`DeltaBackendError::CommitAlreadyAccepted`] after undoing everything
    ///    (the handler then answers with the current table).
    /// 3. Return the table as it is after the update.
    ///
    /// A transactional backend overrides this to do both in one transaction
    /// holding a row lock. The default is a portable approximation for backends
    /// that cannot: it writes the metadata first (CAS), then ratifies the commit,
    /// and on rejection *compensates* by restoring `update.base`'s columns,
    /// properties and comment with another CAS. Concurrent readers can observe the
    /// metadata briefly ahead of the commit, a comment that was `None` before
    /// cannot be cleared again, and a failed compensation is logged. Ordering it
    /// this way means a rejected request never leaves a ratified commit behind a
    /// 409, which would make the client rebase and duplicate its write.
    ///
    /// [`DeltaBackendError::UpdateRequirementConflict`]: crate::error::DeltaBackendError::UpdateRequirementConflict
    /// [`DeltaBackendError::CommitAlreadyAccepted`]: crate::error::DeltaBackendError::CommitAlreadyAccepted
    async fn apply_update(&self, update: TableUpdate, cx: &Cx) -> BackendResult<ResolvedTable> {
        apply_update_sequential(self, update, cx).await
    }

    /// Persist metadata changes to an existing table and return it resolved.
    ///
    /// The metadata-write primitive behind the default
    /// [`apply_update`](Self::apply_update); a backend that overrides
    /// `apply_update` may implement this as a thin wrapper over the same code.
    ///
    /// When [`spec.expected_etag`](UpdateTableSpec::expected_etag) is `Some`, the
    /// write is a **compare-and-swap** against that etag (see the field docs):
    /// return [`DeltaBackendError::UpdateRequirementConflict`] if the table's
    /// current etag no longer matches. Returns the refreshed table so the handler
    /// need not re-resolve it.
    ///
    /// [`DeltaBackendError::UpdateRequirementConflict`]: crate::error::DeltaBackendError::UpdateRequirementConflict
    async fn update_table_row(
        &self,
        spec: UpdateTableSpec,
        cx: &Cx,
    ) -> BackendResult<ResolvedTable>;

    /// Delete a table.
    async fn delete_table(&self, table: &TableRef, cx: &Cx) -> BackendResult<()>;

    /// Rename a table within the same catalog + schema. Backends without rename
    /// support return [`DeltaBackendError::NotImplemented`].
    async fn rename_table(&self, from: &TableRef, to_name: &str, cx: &Cx) -> BackendResult<()>;

    /// Allocate a staging reservation (uuid + managed location) under a schema.
    async fn allocate_staging(
        &self,
        at: &SchemaRef,
        name: &str,
        cx: &Cx,
    ) -> BackendResult<StagingReservation>;

    /// Resolve a staging reservation by its managed location.
    async fn resolve_staging_by_location(
        &self,
        location: &str,
        cx: &Cx,
    ) -> BackendResult<StagingReservation>;

    /// Resolve a staging reservation by its uuid.
    async fn resolve_staging_by_id(
        &self,
        table_id: &str,
        cx: &Cx,
    ) -> BackendResult<StagingReservation>;

    /// Vend a credential for a table's location at the given access level.
    async fn vend_table_credential(
        &self,
        table_id: &str,
        access: CredentialAccess,
        cx: &Cx,
    ) -> BackendResult<VendedCredential>;

    /// Vend a credential for an arbitrary path at the given access level.
    async fn vend_path_credential(
        &self,
        location: &str,
        access: CredentialAccess,
        cx: &Cx,
    ) -> BackendResult<VendedCredential>;

    /// The commit coordinator backing this backend's Delta commit log.
    fn commit_coordinator(&self) -> &dyn CommitCoordinator;
}

/// The default [`DeltaBackend::apply_update`]: metadata CAS first, then the commit,
/// compensating the metadata write when the commit is rejected. See the trait
/// method docs for its guarantees and limits.
pub async fn apply_update_sequential<B, Cx>(
    backend: &B,
    update: TableUpdate,
    cx: &Cx,
) -> BackendResult<ResolvedTable>
where
    B: DeltaBackend<Cx> + ?Sized,
    Cx: Send + Sync,
{
    let has_commit_log_change = update.has_commit_log_change();
    let TableUpdate {
        base,
        metadata,
        commit,
        latest_backfilled_version,
    } = update;
    let table_id = base
        .table_id
        .clone()
        .ok_or_else(|| DeltaBackendError::Internal("table has no id".into()))?;

    let written = match metadata {
        Some(spec) => Some(backend.update_table_row(spec, cx).await?),
        None => None,
    };

    if has_commit_log_change
        && let Err(err) = backend
            .commit_coordinator()
            .commit(&table_id, commit, latest_backfilled_version)
            .await
    {
        if let Some(written) = &written {
            let mut restore =
                UpdateTableSpec::new(&table_id, base.columns.clone(), base.properties.clone());
            restore.comment = base.comment.clone();
            restore.expected_etag = Some(etag_of(written));
            if let Err(undo) = backend.update_table_row(restore, cx).await {
                tracing::error!(
                    table_id = %table_id,
                    commit_error = %err,
                    undo_error = %undo,
                    "commit rejected and the metadata write could not be undone; \
                     the catalog metadata is ahead of the commit log"
                );
            }
        }
        return Err(err.into());
    }

    Ok(written.unwrap_or(base))
}
