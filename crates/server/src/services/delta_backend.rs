//! The mangrove [`DeltaBackend`] adapter.
//!
//! Implements [`unitycatalog_delta_api::DeltaBackend`] for
//! [`ServerHandler<RequestContext>`], expressing the narrow backend port over the
//! server's `Policy` authorization surface, the resource store, the credential
//! vendor, and the commit coordinator. All Delta *semantics* — the managed-table
//! contract, the `updateTable` action dispatcher, `loadTable` construction — live
//! in [`unitycatalog_delta_api`]; this adapter only maps types and errors.
//!
//! It honours the port's authorization contract: every permission check is in
//! [`DeltaBackend::authorize`] (by table name, before resolution), and the data
//! methods are pure data access, sharing the non-authorizing cores of the UC REST
//! handlers (`allocate_staging_table`, `vend_location`, `vend_table`).
//!
//! The impl is on `ServerHandler<RequestContext>` directly (not a wrapper) so the
//! router's `RequestContext: FromRequestParts<ServerHandler>` extraction is
//! preserved, exactly as the previous blanket `impl DeltaApiHandler for T` was.

use async_trait::async_trait;
use buffa::Enumeration;

use unitycatalog_common::models::staging_tables::v1::{CreateStagingTableRequest, StagingTable};
use unitycatalog_common::models::tables::v1::{
    Column as UcColumn, CreateTableRequest, DataSourceFormat, Table, TableType,
};
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};
use unitycatalog_common::store::Precondition;

use unitycatalog_delta_api::authz::DeltaAction;
use unitycatalog_delta_api::backend::{
    BackendResult, CreateTableSpec, CredentialAccess, DeltaBackend, DeltaCapabilities,
    ResolvedTable, SchemaRef, StagingReservation, TableRef, UpdateTableSpec, VendedCredential,
};
use unitycatalog_delta_api::column::{
    Column as CrateColumn, ColumnTypeName as CrateColumnTypeName,
};
use unitycatalog_delta_api::coordinator::{CommitCoordinator, ProvidesCommitCoordinator};
use unitycatalog_delta_api::error::DeltaBackendError;
use unitycatalog_delta_api::models::DeltaTableType;

use crate::api::RequestContext;
use crate::api::staging_tables::{allocate_staging_table, find_staging_table_by_location};
use crate::api::temporary_credentials::{
    authorize_path_vend, required_permission, vend_location, vend_table,
};
use crate::policy::{Permission, Policy, Principal};
use crate::services::ServerHandler;
use crate::services::location::StorageLocationUrl;
use crate::services::object_store::validate_external_storage_location;
use crate::store::{ResourceStore, ResourceStoreReader};
use crate::{Error, Result};

// ===================================================================
// Error mapping
// ===================================================================

/// Map the server's internal [`Error`] onto the crate's [`DeltaBackendError`].
///
/// Reproduces the previous server-side `DeltaError::parts` dispatch exactly (see
/// the deleted `crates/server/src/rest/routers/delta/models.rs`), so response
/// status codes and error types are unchanged by the extraction.
fn to_backend_err(e: Error) -> DeltaBackendError {
    match &e {
        Error::NotFound | Error::ResourceStore { .. } => DeltaBackendError::NotFound(e.to_string()),
        // The wrapped common error carries its own semantics; dispatch on its
        // machine-readable code so e.g. an "already exists" doesn't surface as 404.
        Error::Common { source } => match source.error_code() {
            "RESOURCE_NOT_FOUND" => DeltaBackendError::NotFound(e.to_string()),
            "RESOURCE_ALREADY_EXISTS" => DeltaBackendError::AlreadyExists(e.to_string()),
            "INVALID_PARAMETER_VALUE" => DeltaBackendError::InvalidArgument(e.to_string()),
            "PERMISSION_DENIED" => DeltaBackendError::PermissionDenied(e.to_string()),
            "COMMIT_VERSION_CONFLICT" => DeltaBackendError::CommitVersionConflict(e.to_string()),
            // A store CAS mismatch (`Precondition::Version`) surfaces as a generic
            // conflict; for the Delta write path that is an `assert-etag` failure.
            "RESOURCE_CONFLICT" => DeltaBackendError::UpdateRequirementConflict(e.to_string()),
            "RESOURCE_EXHAUSTED" => DeltaBackendError::ResourceExhausted(e.to_string()),
            _ => DeltaBackendError::NotFoundGeneric(e.to_string()),
        },
        Error::NotAllowed => DeltaBackendError::PermissionDenied(e.to_string()),
        Error::Unauthenticated => DeltaBackendError::Unauthenticated(e.to_string()),
        Error::AlreadyExists => DeltaBackendError::AlreadyExists(e.to_string()),
        Error::CommitVersionConflict(m) => DeltaBackendError::CommitVersionConflict(m.clone()),
        Error::UpdateRequirementConflict(m) => {
            DeltaBackendError::UpdateRequirementConflict(m.clone())
        }
        Error::ResourceExhausted(m) => DeltaBackendError::ResourceExhausted(m.clone()),
        Error::InvalidArgument(m) => DeltaBackendError::InvalidArgument(m.clone()),
        Error::InvalidIdentifier(_) | Error::MissingRecipient => {
            DeltaBackendError::InvalidArgument(e.to_string())
        }
        Error::NotImplemented(w) => DeltaBackendError::NotImplemented((*w).into()),
        _ => DeltaBackendError::Internal(e.to_string()),
    }
}

/// Parse the version out of an etag string produced by
/// [`unitycatalog_delta_api::backend::etag_of`] (`etag-<version>`).
///
/// Returns `None` for a malformed etag, which the caller treats as an
/// `assert-etag` conflict.
fn parse_etag_version(etag: &str) -> Option<u64> {
    etag.strip_prefix("etag-").and_then(|v| v.parse().ok())
}

// ===================================================================
// Column mapping
// ===================================================================

/// Map a common UC column onto the crate's portable [`Column`](CrateColumn).
fn uc_column_to_crate(c: UcColumn) -> CrateColumn {
    CrateColumn {
        name: c.name,
        type_text: c.type_text,
        type_json: c.type_json,
        position: c.position,
        type_name: CrateColumnTypeName::from(c.type_name.to_i32()),
        comment: c.comment,
        nullable: c.nullable,
        partition_index: c.partition_index,
    }
}

/// Map a crate column back onto the common UC column.
///
/// The Delta contract only produces the fields the crate `Column` carries; the
/// remaining generated fields (`type_precision`, `type_scale`,
/// `type_interval_type`, `column_id`) are left at their defaults, mirroring the
/// columns the old contract-derived `CreateTableRequest` persisted.
fn crate_column_to_uc(c: CrateColumn) -> UcColumn {
    UcColumn {
        name: c.name,
        type_text: c.type_text,
        type_json: c.type_json,
        position: c.position,
        type_name: (c.type_name as i32).into(),
        comment: c.comment,
        nullable: c.nullable,
        partition_index: c.partition_index,
        ..Default::default()
    }
}

// ===================================================================
// Table / staging / credential mapping
// ===================================================================

/// The handler only ever passes `MANAGED` / `EXTERNAL` (it rejects any other
/// `table-type` up front); anything else maps to unspecified, which the UC
/// handlers reject.
fn to_uc_table_type(t: DeltaTableType) -> TableType {
    match t {
        DeltaTableType::Managed => TableType::Managed,
        DeltaTableType::External => TableType::External,
        _ => TableType::TableTypeUnspecified,
    }
}

/// Map a stored data source format onto the crate's wire format enum. Formats
/// the wire enum does not carry (`Unspecified`, unknown) map to `None`.
fn to_delta_format(f: i32) -> Option<unitycatalog_delta_api::models::DeltaDataSourceFormat> {
    use unitycatalog_delta_api::models::DeltaDataSourceFormat as F;
    match DataSourceFormat::from_i32(f)? {
        // UniForm tables are Delta tables that also publish Iceberg/Hudi metadata.
        DataSourceFormat::DELTA
        | DataSourceFormat::DELTA_UNIFORM_ICEBERG
        | DataSourceFormat::DELTA_UNIFORM_HUDI => Some(F::Delta),
        DataSourceFormat::ICEBERG => Some(F::Iceberg),
        DataSourceFormat::HUDI => Some(F::Hudi),
        DataSourceFormat::PARQUET => Some(F::Parquet),
        DataSourceFormat::CSV => Some(F::Csv),
        DataSourceFormat::JSON => Some(F::Json),
        DataSourceFormat::ORC => Some(F::Orc),
        DataSourceFormat::AVRO => Some(F::Avro),
        DataSourceFormat::TEXT => Some(F::Text),
        DataSourceFormat::UNITY_CATALOG => Some(F::UnityCatalog),
        DataSourceFormat::DELTASHARING => Some(F::Deltasharing),
        DataSourceFormat::HIVE | DataSourceFormat::DATA_SOURCE_FORMAT_UNSPECIFIED => None,
    }
}

/// Map a stored [`Table`] into the crate's portable [`ResolvedTable`].
///
/// `version` is the store's per-object version, which drives the etag and the
/// `assert-etag` compare-and-swap; callers read it via
/// [`ResourceStoreReader::get_versioned`].
///
/// View-like table types (views, metric views, …) map to `table_type: None`,
/// which the shared handler rejects with the spec's "not a Delta table" 400.
fn table_to_resolved(table: Table, version: u64) -> ResolvedTable {
    let table_type = match table.table_type.as_known() {
        Some(TableType::MANAGED) => Some(DeltaTableType::Managed),
        Some(TableType::EXTERNAL) => Some(DeltaTableType::External),
        _ => None,
    };
    let mut resolved = ResolvedTable::new(table.storage_location.unwrap_or_default());
    resolved.table_id = table.table_id;
    resolved.table_type = table_type;
    resolved.data_source_format = to_delta_format(table.data_source_format.to_i32());
    resolved.columns = table.columns.into_iter().map(uc_column_to_crate).collect();
    resolved.properties = table.properties.into_iter().collect();
    resolved.comment = table.comment;
    resolved.created_at_ms = table.created_at;
    resolved.updated_at_ms = table.updated_at;
    resolved.version = version;
    resolved
}

fn staging_to_reservation(st: StagingTable) -> StagingReservation {
    let mut reservation = StagingReservation::new(st.id, st.name, st.staging_location);
    reservation.created_by = st.created_by;
    reservation.stage_committed = st.stage_committed;
    reservation
}

/// The store identity of a three-part table name.
fn table_ident(table: &TableRef) -> ResourceIdent {
    ResourceIdent::table(ResourceName::new([
        table.catalog.as_str(),
        table.schema.as_str(),
        table.table.as_str(),
    ]))
}

// ===================================================================
// The adapter
// ===================================================================

impl ServerHandler<RequestContext> {
    /// Resolve a staging reservation by uuid via the resource store.
    async fn get_staging_by_id(&self, table_id: &str) -> Result<StagingTable> {
        let uuid = uuid::Uuid::parse_str(table_id)
            .map_err(|_| Error::invalid_argument("table_id is not a valid UUID"))?;
        let ident = ResourceIdent::StagingTable(ResourceRef::Uuid(uuid));
        let staging: StagingTable = self.get(&ident).await?.0.try_into()?;
        Ok(staging)
    }
}

#[async_trait]
impl DeltaBackend<RequestContext> for ServerHandler<RequestContext> {
    fn capabilities(&self) -> DeltaCapabilities {
        // The store provides a native, versioned rename, so advertise renameTable.
        DeltaCapabilities::default().with_rename(true)
    }

    async fn catalog_exists(&self, catalog: &str, _cx: &RequestContext) -> BackendResult<()> {
        let ident = ResourceIdent::catalog(ResourceName::new([catalog]));
        self.get(&ident)
            .await
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        Ok(())
    }

    async fn resolve_table(
        &self,
        table: &TableRef,
        _cx: &RequestContext,
    ) -> BackendResult<ResolvedTable> {
        // Pure data access (the handler authorized the action already): one
        // versioned read by name, so the etag reflects exactly the row returned
        // (the `assert-etag` CAS keys on this same version).
        let (resource, _, version) = self
            .get_versioned(&table_ident(table))
            .await
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        let t: Table = resource
            .try_into()
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        Ok(table_to_resolved(t, version))
    }

    async fn authorize(&self, action: DeltaAction<'_>, cx: &RequestContext) -> BackendResult<()> {
        // The single authorization seam: every check happens here, and the data
        // methods below are pure data access. Table actions arrive before the
        // table is resolved and are checked by name, like the UC REST table
        // endpoints, so a missing table is denied rather than reported missing.
        let result = match action {
            DeltaAction::ReadCatalog { catalog } => {
                let ident = ResourceIdent::catalog(ResourceName::new([catalog]));
                self.authorize_checked(&ident, &Permission::Read, cx).await
            }
            DeltaAction::CreateTable {
                at,
                name,
                table_type,
            } => {
                // CREATE on the target table via the same SecuredAction the
                // UC-REST createTable uses.
                let create_action = CreateTableRequest {
                    name: name.to_string(),
                    catalog_name: at.catalog.clone(),
                    schema_name: at.schema.clone(),
                    table_type: to_uc_table_type(table_type).into(),
                    data_source_format: DataSourceFormat::Delta.into(),
                    ..Default::default()
                };
                self.check_required(&create_action, cx).await
            }
            DeltaAction::ReadTable { table } => {
                self.authorize_checked(&table_ident(table), &Permission::Read, cx)
                    .await
            }
            DeltaAction::WriteTable { table } => {
                self.authorize_checked(&table_ident(table), &Permission::Write, cx)
                    .await
            }
            // Delete and rename change the securable itself: MANAGE, as the UC
            // REST deleteTable requires.
            DeltaAction::DeleteTable { table } | DeltaAction::RenameTable { from: table, .. } => {
                self.authorize_checked(&table_ident(table), &Permission::Manage, cx)
                    .await
            }
            DeltaAction::VendTableCredential { table, access } => {
                self.authorize_checked(&table_ident(table), &required_permission(access), cx)
                    .await
            }
            DeltaAction::VendPathCredential { location, access } => {
                let url = StorageLocationUrl::parse(location).map_err(to_backend_err)?;
                authorize_path_vend(self, &url, access, cx).await
            }
            DeltaAction::CreateStaging { at, name } => {
                let request = CreateStagingTableRequest {
                    name: name.to_string(),
                    catalog_name: at.catalog.clone(),
                    schema_name: at.schema.clone(),
                    ..Default::default()
                };
                self.check_required(&request, cx).await
            }
            DeltaAction::AdoptStaging { reservation } => {
                // The creator-match, in mangrove's identity terms: the caller's
                // principal name must equal the reservation's `created_by`
                // (anonymous reservations, `created_by == None`, are usable by
                // any anonymous caller — the pre-crate behavior).
                let principal = match cx.recipient() {
                    Principal::User(name) => Some(name.clone()),
                    Principal::Anonymous => None,
                };
                if reservation.created_by.as_deref() != principal.as_deref() {
                    return Err(DeltaBackendError::PermissionDenied(
                        "caller is not the creator of the staging table".to_string(),
                    ));
                }
                Ok(())
            }
            // `DeltaAction` is `#[non_exhaustive]`. Fail closed on an action this
            // adapter has not been taught: a newly added operation must not slip
            // through unauthorized until its arm is written.
            _ => {
                return Err(DeltaBackendError::PermissionDenied(
                    "unrecognized Delta action".to_string(),
                ));
            }
        };
        result.map_err(to_backend_err)
    }

    async fn validate_external_location(
        &self,
        location: &str,
        _cx: &RequestContext,
    ) -> BackendResult<()> {
        let parsed = StorageLocationUrl::parse(location).map_err(to_backend_err)?;
        validate_external_storage_location(self, &parsed)
            .await
            .map_err(to_backend_err)
    }

    async fn create_table_row(
        &self,
        spec: CreateTableSpec,
        _cx: &RequestContext,
    ) -> BackendResult<ResolvedTable> {
        let adopt_ident = spec.adopt_staging.as_ref().map(|reservation| {
            ResourceIdent::staging_table(ResourceName::new([reservation.name.as_str()]))
        });
        let table = Table {
            name: spec.name,
            catalog_name: spec.at.catalog,
            schema_name: spec.at.schema,
            table_type: to_uc_table_type(spec.table_type).into(),
            data_source_format: DataSourceFormat::Delta.into(),
            columns: spec.columns.into_iter().map(crate_column_to_uc).collect(),
            storage_location: Some(spec.location),
            comment: spec.comment,
            properties: spec.properties.into_iter().collect(),
            table_id: spec.table_id,
            ..Default::default()
        };
        // Persist directly via the store (the request is already validated); the
        // UC-REST create path re-reads the snapshot for the managed branch, which
        // the Delta API does not want.
        //
        // Managed adoption is a *relabel* (StagingTable → Table at the same id):
        // both cannot exist at that id, so `replace_atomically` consumes the
        // reservation and creates the table in one transaction — closing the
        // orphaned-reservation window. EXTERNAL tables have no reservation and are
        // a plain create. Take the version the store assigned the new row so the
        // returned etag matches what a later `loadTable` reads, without assuming a
        // fixed initial version.
        let (resource, _, version) = match adopt_ident {
            Some(ident) => {
                self.replace_atomically_versioned(&ident, table.into())
                    .await
            }
            None => self.create_versioned(table.into()).await,
        }
        .map_err(Error::from)
        .map_err(to_backend_err)?;
        let stored: Table = resource
            .try_into()
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        Ok(table_to_resolved(stored, version))
    }

    async fn update_table_row(
        &self,
        spec: UpdateTableSpec,
        _cx: &RequestContext,
    ) -> BackendResult<ResolvedTable> {
        let uuid = uuid::Uuid::parse_str(&spec.table_id).map_err(|_| {
            DeltaBackendError::InvalidArgument("table id is not a valid UUID".into())
        })?;
        let ident = ResourceIdent::Table(ResourceRef::Uuid(uuid));
        // Read the row together with its version, so an `assert-etag` translates
        // into a real compare-and-swap at write time.
        let (resource, _, version) = self
            .get_versioned(&ident)
            .await
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        let mut table: Table = resource
            .try_into()
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        // assert-etag compare-and-swap: parse the expected etag back to the version
        // it encodes and pass it as a `Precondition::Version` to the update below,
        // so the store rejects the write atomically if the row advanced between
        // this read and the write (closing the read-modify-write race). A malformed
        // or non-matching etag fails fast here.
        let precondition = match &spec.expected_etag {
            Some(expected) => {
                let expected_version = parse_etag_version(expected).ok_or_else(|| {
                    DeltaBackendError::UpdateRequirementConflict(
                        "assert-etag failed: table has been modified".into(),
                    )
                })?;
                if expected_version != version {
                    return Err(DeltaBackendError::UpdateRequirementConflict(
                        "assert-etag failed: table has been modified".into(),
                    ));
                }
                Precondition::Version(expected_version)
            }
            None => Precondition::Any,
        };
        table.columns = spec.columns.into_iter().map(crate_column_to_uc).collect();
        table.properties = spec.properties.into_iter().collect();
        // `None` means "leave the stored comment unchanged" (see `UpdateTableSpec`):
        // the handler only sets it when a set-table-comment action is present.
        if let Some(comment) = spec.comment {
            table.comment = Some(comment);
        }
        let (updated_resource, _, new_version) = self
            .update_checked(&ident, table.into(), precondition)
            .await
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        let updated: Table = updated_resource
            .try_into()
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        Ok(table_to_resolved(updated, new_version))
    }

    async fn delete_table(&self, table: &TableRef, _cx: &RequestContext) -> BackendResult<()> {
        self.delete(&table_ident(table))
            .await
            .map_err(Error::from)
            .map_err(to_backend_err)
    }

    async fn rename_table(
        &self,
        from: &TableRef,
        to_name: &str,
        _cx: &RequestContext,
    ) -> BackendResult<()> {
        // A table's name is 3-part `[catalog, schema, table]`; a rename changes only
        // the leaf, keeping catalog+schema. `ResourceStore::rename` re-keys the
        // object and rewrites the leaf name inside its properties in one transaction,
        // preserving the id, associations, and any secrets.
        let from_ident = table_ident(from);
        let new_name = ResourceName::new([from.catalog.as_str(), from.schema.as_str(), to_name]);
        self.rename(&from_ident, &new_name, Precondition::Any)
            .await
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        Ok(())
    }

    async fn allocate_staging(
        &self,
        at: &SchemaRef,
        name: &str,
        cx: &RequestContext,
    ) -> BackendResult<StagingReservation> {
        let created_by = match cx.recipient() {
            Principal::User(name) => Some(name.clone()),
            Principal::Anonymous => None,
        };
        let staging = allocate_staging_table(
            self,
            CreateStagingTableRequest {
                name: name.to_string(),
                catalog_name: at.catalog.clone(),
                schema_name: at.schema.clone(),
                ..Default::default()
            },
            created_by,
        )
        .await
        .map_err(to_backend_err)?;
        Ok(staging_to_reservation(staging))
    }

    async fn resolve_staging_by_location(
        &self,
        location: &str,
        _cx: &RequestContext,
    ) -> BackendResult<StagingReservation> {
        let staging = find_staging_table_by_location(self, location)
            .await
            .map_err(to_backend_err)?;
        Ok(staging_to_reservation(staging))
    }

    async fn resolve_staging_by_id(
        &self,
        table_id: &str,
        _cx: &RequestContext,
    ) -> BackendResult<StagingReservation> {
        let staging = self
            .get_staging_by_id(table_id)
            .await
            .map_err(to_backend_err)?;
        Ok(staging_to_reservation(staging))
    }

    async fn vend_table_credential(
        &self,
        table_id: &str,
        access: CredentialAccess,
        _cx: &RequestContext,
    ) -> BackendResult<VendedCredential> {
        let uuid = uuid::Uuid::parse_str(table_id).map_err(|_| {
            DeltaBackendError::InvalidArgument("table id is not a valid UUID".into())
        })?;
        let (credential, url) =
            vend_table(self, &ResourceIdent::Table(ResourceRef::Uuid(uuid)), access)
                .await
                .map_err(to_backend_err)?;
        Ok(VendedCredential::new(url.raw().to_string(), credential))
    }

    async fn vend_path_credential(
        &self,
        location: &str,
        access: CredentialAccess,
        _cx: &RequestContext,
    ) -> BackendResult<VendedCredential> {
        let url = StorageLocationUrl::parse(location).map_err(to_backend_err)?;
        let credential = vend_location(self, &url, access)
            .await
            .map_err(to_backend_err)?;
        Ok(VendedCredential::new(url.raw().to_string(), credential))
    }

    fn commit_coordinator(&self) -> &dyn CommitCoordinator {
        ProvidesCommitCoordinator::commit_coordinator(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_not_found_is_a_delta_not_found() {
        // Stores report a missing entity as a common error; it must surface as
        // `NotFound` (404 `NoSuchTableException`), not the generic fallback.
        let err = to_backend_err(Error::from(unitycatalog_common::Error::NotFound));
        assert!(matches!(err, DeltaBackendError::NotFound(_)), "{err:?}");
        let err = to_backend_err(Error::NotFound);
        assert!(matches!(err, DeltaBackendError::NotFound(_)), "{err:?}");
    }

    #[cfg(feature = "memory")]
    fn handler(decision: crate::policy::Decision) -> ServerHandler<RequestContext> {
        use std::sync::Arc;

        use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

        use crate::memory::InMemoryResourceStore;
        use crate::policy::ConstantPolicy;

        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::new(decision));
        ServerHandler::try_new_tokio(policy, store).unwrap()
    }

    #[cfg(feature = "memory")]
    fn ctx() -> RequestContext {
        RequestContext::anonymous()
    }

    /// Every action is checked in `authorize` (the data methods do not
    /// authorize), so a deny-all policy denies all of them, including rename,
    /// which used to reach the store unchecked.
    #[cfg(feature = "memory")]
    #[tokio::test]
    async fn deny_all_policy_denies_every_action() {
        let h = handler(crate::policy::Decision::Deny);
        let table = TableRef {
            catalog: "c".into(),
            schema: "s".into(),
            table: "t".into(),
        };
        let schema = SchemaRef {
            catalog: "c".into(),
            schema: "s".into(),
        };
        let actions = [
            DeltaAction::ReadCatalog { catalog: "c" },
            DeltaAction::CreateTable {
                at: &schema,
                name: "t",
                table_type: DeltaTableType::External,
            },
            DeltaAction::ReadTable { table: &table },
            DeltaAction::WriteTable { table: &table },
            DeltaAction::DeleteTable { table: &table },
            DeltaAction::RenameTable {
                from: &table,
                to: "t2",
            },
            DeltaAction::VendTableCredential {
                table: &table,
                access: CredentialAccess::Read,
            },
            DeltaAction::CreateStaging {
                at: &schema,
                name: "t",
            },
        ];
        for action in actions {
            let label = format!("{action:?}");
            let result = DeltaBackend::authorize(&h, action, &ctx()).await;
            assert!(
                matches!(result, Err(DeltaBackendError::PermissionDenied(_))),
                "{label}: {result:?}"
            );
        }
    }

    /// A managed catalog on the metastore root (no own `storage_root`, no
    /// external location over the root) gets staging credentials from the root's
    /// configured credential. An Azure storage key signs SAS tokens locally, so
    /// no network is involved.
    #[cfg(feature = "memory")]
    #[tokio::test]
    async fn managed_catalog_on_metastore_root_vends_staging_credentials() {
        use unitycatalog_common::models::catalogs::v1::CreateCatalogRequest;
        use unitycatalog_common::models::credentials::v1::{
            AzureStorageKey, CreateCredentialRequest, Purpose,
        };
        use unitycatalog_common::models::schemas::v1::CreateSchemaRequest;
        use unitycatalog_delta_api::DeltaApiHandler;
        use unitycatalog_delta_api::models::DeltaCreateStagingTableRequest;

        use crate::api::{CatalogHandler, CredentialHandler, SchemaHandler};

        let root = "abfss://meta@acct.dfs.core.windows.net/root";
        let h = handler(crate::policy::Decision::Allow)
            .with_managed_storage_root(Some(root))
            .with_managed_storage_root_credential(Some("root-cred"));
        h.create_credential(
            CreateCredentialRequest {
                name: "root-cred".into(),
                purpose: Purpose::Storage.into(),
                azure_storage_key: Some(AzureStorageKey {
                    account_name: "acct".into(),
                    // base64 of a dummy key: SAS signing needs valid base64.
                    account_key: "a2V5a2V5a2V5a2V5a2V5a2V5".into(),
                    ..Default::default()
                })
                .into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_catalog(
            CreateCatalogRequest {
                name: "cat".into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_schema(
            CreateSchemaRequest {
                name: "sch".into(),
                catalog_name: "cat".into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();

        let staging = DeltaApiHandler::create_staging_table(
            &h,
            SchemaRef {
                catalog: "cat".into(),
                schema: "sch".into(),
            },
            DeltaCreateStagingTableRequest { name: "t".into() },
            ctx(),
        )
        .await
        .unwrap();
        assert!(staging.location.starts_with(root), "{}", staging.location);
        assert!(!staging.storage_credentials.is_empty());

        let creds = DeltaApiHandler::get_staging_table_credentials(&h, staging.table_id, ctx())
            .await
            .unwrap();
        let cred = &creds.storage_credentials[0];
        assert_eq!(cred.prefix, staging.location);
        assert!(
            cred.config.keys().any(|k| k.starts_with("azure.")),
            "{:?}",
            cred.config.keys().collect::<Vec<_>>()
        );
    }
}
