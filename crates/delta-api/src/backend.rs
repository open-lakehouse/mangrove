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
//! handler calls with a [`DeltaAction`] before every user-facing operation. The
//! data methods below (`resolve_table`, `create_table_row`, `delete_table`,
//! `vend_*`, `resolve_staging_*`, …) are **pure data access and must not perform
//! their own authorization** — the handler has already authorized the operation.
//! (An implementor is of course free to enforce coarse storage-level guards, but
//! the caller-facing access decision belongs in `authorize`.) See [`crate::authz`].

use std::collections::BTreeMap;

use async_trait::async_trait;

use serde::Deserialize;

use crate::authz::DeltaAction;
use crate::column::Column;
use crate::coordinator::CommitCoordinator;
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
#[derive(Debug, Clone)]
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
    /// Creation time, epoch milliseconds.
    pub created_at_ms: Option<i64>,
    /// Last-update time, epoch milliseconds. Drives the etag.
    pub updated_at_ms: Option<i64>,
}

/// The etag for a resolved table: `etag-<updated_ms>`, else `etag-<uuid>`.
///
/// This is the single definition of the table etag, shared by the handler (which
/// hands it to the client on `loadTable`) and by any backend enforcing the
/// `assert-etag` compare-and-swap in [`DeltaBackend::update_table_row`], so the
/// asserted etag and the compared etag are always derived identically.
pub fn etag_of(table: &ResolvedTable) -> String {
    match table.updated_at_ms {
        Some(ts) => format!("etag-{ts}"),
        None => format!("etag-{}", table.table_id.clone().unwrap_or_default()),
    }
}

/// A staging-table reservation (uuid + managed location) allocated before a
/// managed `createTable`.
#[derive(Debug, Clone)]
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

/// The access level for a vended storage credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialAccess {
    Read,
    ReadWrite,
}

/// A credential vended by the backend, in a cloud-neutral shape the handler maps
/// onto the wire `DeltaStorageCredential`.
#[derive(Debug, Clone)]
pub struct VendedCredential {
    /// The prefix / URL the credential applies to.
    pub url: String,
    /// Expiration, epoch milliseconds.
    pub expiration_time_ms: i64,
    pub kind: VendedCredentialKind,
}

/// Cloud-provider-specific credential material.
#[derive(Debug, Clone)]
pub enum VendedCredentialKind {
    /// AWS (and R2, which reuses the S3-shaped fields).
    S3 {
        access_key_id: String,
        secret_access_key: String,
        session_token: Option<String>,
    },
    /// Azure user-delegation SAS.
    AzureSas { sas_token: String },
    /// GCS OAuth token.
    GcsOauth { oauth_token: String },
    /// No recognized credential material (an empty config is vended).
    None,
}

/// Specification for persisting a new Delta table row. The handler has already
/// validated the contract and derived the UC-shaped columns + stored properties.
#[derive(Debug, Clone)]
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

/// The backend port. See the module docs.
#[async_trait]
pub trait DeltaBackend<Cx = ()>: Send + Sync + 'static {
    /// Authorize an action before the handler performs it.
    ///
    /// The single authorization seam (see the [module authz contract](self)):
    /// the handler calls this with a [`DeltaAction`] at the top of every
    /// user-facing operation. Deny by returning
    /// [`DeltaBackendError::PermissionDenied`] (→ 403) or
    /// [`DeltaBackendError::Unauthenticated`] (→ 401).
    async fn authorize(&self, action: DeltaAction<'_>, cx: &Cx) -> BackendResult<()>;

    /// Confirm the catalog exists (for `getConfig`). Missing → [`DeltaBackendError::NotFound`].
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

    /// Persist metadata changes to an existing table and return it resolved.
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
