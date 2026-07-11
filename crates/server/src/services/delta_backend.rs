//! The mangrove [`DeltaBackend`] adapter.
//!
//! Implements [`unitycatalog_delta_api::DeltaBackend`] for
//! [`ServerHandler<RequestContext>`], expressing the narrow backend port over the
//! server's existing handler traits (`TableHandler`, `StagingTableHandler`,
//! `TemporaryCredentialHandler`), the `Policy` authorization surface, the resource
//! store, and the commit coordinator. All Delta *semantics* — the managed-table
//! contract, the `updateTable` action dispatcher, `loadTable` construction — live
//! in [`unitycatalog_delta_api`]; this adapter only maps types and errors.
//!
//! The impl is on `ServerHandler<RequestContext>` directly (not a wrapper) so the
//! router's `RequestContext: FromRequestParts<ServerHandler>` extraction is
//! preserved, exactly as the previous blanket `impl DeltaApiHandler for T` was.

use async_trait::async_trait;

use unitycatalog_common::models::staging_tables::v1::{CreateStagingTableRequest, StagingTable};
use unitycatalog_common::models::tables::v1::{
    Column as UcColumn, CreateTableRequest, DataSourceFormat, DeleteTableRequest, GetTableRequest,
    Table, TableType,
};
use unitycatalog_common::models::temporary_credentials::v1::{
    GenerateTemporaryPathCredentialsRequest, GenerateTemporaryTableCredentialsRequest,
    TemporaryCredential, generate_temporary_path_credentials_request::Operation as PathOp,
    generate_temporary_table_credentials_request::Operation as TableOp,
    temporary_credential::Credentials,
};
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};

use unitycatalog_delta_api::authz::DeltaAction;
use unitycatalog_delta_api::backend::{
    BackendResult, CreateTableSpec, CredentialAccess, DeltaBackend, ResolvedTable, SchemaRef,
    StagingReservation, TableRef, UpdateTableSpec, VendedCredential, VendedCredentialKind, etag_of,
};
use unitycatalog_delta_api::column::{
    Column as CrateColumn, ColumnTypeName as CrateColumnTypeName,
};
use unitycatalog_delta_api::coordinator::{CommitCoordinator, ProvidesCommitCoordinator};
use unitycatalog_delta_api::error::DeltaBackendError;
use unitycatalog_delta_api::models::DeltaTableType;

use crate::api::RequestContext;
use crate::api::staging_tables::find_staging_table_by_location;
use crate::api::tables::TableHandler;
use crate::codegen::staging_tables::StagingTableHandler;
use crate::codegen::temporary_credentials::TemporaryCredentialHandler;
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
            "RESOURCE_ALREADY_EXISTS" => DeltaBackendError::AlreadyExists(e.to_string()),
            "INVALID_PARAMETER_VALUE" => DeltaBackendError::InvalidArgument(e.to_string()),
            "PERMISSION_DENIED" => DeltaBackendError::PermissionDenied(e.to_string()),
            "COMMIT_VERSION_CONFLICT" => DeltaBackendError::CommitVersionConflict(e.to_string()),
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
        Error::NotImplemented(w) => DeltaBackendError::NotImplemented(w),
        _ => DeltaBackendError::Internal(e.to_string()),
    }
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
        type_name: CrateColumnTypeName::from(c.type_name),
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
        type_name: c.type_name as i32,
        comment: c.comment,
        nullable: c.nullable,
        partition_index: c.partition_index,
        ..Default::default()
    }
}

// ===================================================================
// Table / staging / credential mapping
// ===================================================================

fn to_uc_table_type(t: DeltaTableType) -> TableType {
    match t {
        DeltaTableType::Managed => TableType::Managed,
        DeltaTableType::External => TableType::External,
    }
}

/// Map a stored data source format onto the crate's wire format enum. Formats
/// the wire enum does not carry (`Unspecified`, unknown) map to `None`.
fn to_delta_format(f: i32) -> Option<unitycatalog_delta_api::models::DeltaDataSourceFormat> {
    use unitycatalog_delta_api::models::DeltaDataSourceFormat as F;
    match DataSourceFormat::try_from(f).ok()? {
        DataSourceFormat::Delta => Some(F::Delta),
        DataSourceFormat::Iceberg => Some(F::Iceberg),
        DataSourceFormat::Hudi => Some(F::Hudi),
        DataSourceFormat::Parquet => Some(F::Parquet),
        DataSourceFormat::Csv => Some(F::Csv),
        DataSourceFormat::Json => Some(F::Json),
        DataSourceFormat::Orc => Some(F::Orc),
        DataSourceFormat::Avro => Some(F::Avro),
        DataSourceFormat::Text => Some(F::Text),
        DataSourceFormat::UnityCatalog => Some(F::UnityCatalog),
        DataSourceFormat::Deltasharing => Some(F::Deltasharing),
        DataSourceFormat::Unspecified => None,
    }
}

/// Map a stored [`Table`] into the crate's portable [`ResolvedTable`].
///
/// View-like table types (views, metric views, …) map to `table_type: None`,
/// which the shared handler rejects with the spec's "not a Delta table" 400.
fn table_to_resolved(table: Table) -> ResolvedTable {
    let table_type = match TableType::try_from(table.table_type) {
        Ok(TableType::Managed) => Some(DeltaTableType::Managed),
        Ok(TableType::External) => Some(DeltaTableType::External),
        _ => None,
    };
    ResolvedTable {
        table_id: table.table_id,
        location: table.storage_location.unwrap_or_default(),
        table_type,
        data_source_format: to_delta_format(table.data_source_format),
        columns: table.columns.into_iter().map(uc_column_to_crate).collect(),
        properties: table.properties.into_iter().collect(),
        created_at_ms: table.created_at,
        updated_at_ms: table.updated_at,
    }
}

fn staging_to_reservation(st: StagingTable) -> StagingReservation {
    StagingReservation {
        table_id: st.id,
        name: st.name,
        location: st.staging_location,
        created_by: st.created_by,
        stage_committed: st.stage_committed,
    }
}

/// Map a vended [`TemporaryCredential`] onto the crate's [`VendedCredential`].
fn to_vended_credential(creds: &TemporaryCredential, url: String) -> VendedCredential {
    let kind = match &creds.credentials {
        Some(Credentials::AwsTempCredentials(aws)) => VendedCredentialKind::S3 {
            access_key_id: aws.access_key_id.clone(),
            secret_access_key: aws.secret_access_key.clone(),
            session_token: (!aws.session_token.is_empty()).then(|| aws.session_token.clone()),
        },
        Some(Credentials::AzureUserDelegationSas(az)) => VendedCredentialKind::AzureSas {
            sas_token: az.sas_token.clone(),
        },
        Some(Credentials::GcpOauthToken(gcp)) => VendedCredentialKind::GcsOauth {
            oauth_token: gcp.oauth_token.clone(),
        },
        // R2 reuses the S3-shaped fields.
        Some(Credentials::R2TempCredentials(r2)) => VendedCredentialKind::S3 {
            access_key_id: r2.access_key_id.clone(),
            secret_access_key: r2.secret_access_key.clone(),
            session_token: (!r2.session_token.is_empty()).then(|| r2.session_token.clone()),
        },
        _ => VendedCredentialKind::None,
    };
    VendedCredential {
        url,
        expiration_time_ms: creds.expiration_time,
        kind,
    }
}

fn to_table_op(access: CredentialAccess) -> i32 {
    match access {
        CredentialAccess::Read => TableOp::Read as i32,
        CredentialAccess::ReadWrite => TableOp::ReadWrite as i32,
    }
}

fn to_path_op(access: CredentialAccess) -> i32 {
    match access {
        CredentialAccess::Read => PathOp::PathRead as i32,
        CredentialAccess::ReadWrite => PathOp::PathReadWrite as i32,
    }
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
        cx: &RequestContext,
    ) -> BackendResult<ResolvedTable> {
        let t = TableHandler::get_table(
            self,
            GetTableRequest {
                full_name: table.full_name(),
                include_delta_metadata: None,
                include_browse: None,
                include_manifest_capabilities: None,
            },
            cx.clone(),
        )
        .await
        .map_err(to_backend_err)?;
        Ok(table_to_resolved(t))
    }

    async fn authorize(&self, action: DeltaAction<'_>, cx: &RequestContext) -> BackendResult<()> {
        match action {
            DeltaAction::CreateTable {
                at,
                name,
                table_type,
            } => {
                // Authorize CREATE on the target table via the same SecuredAction
                // the UC-REST createTable uses.
                let create_action = CreateTableRequest {
                    name: name.to_string(),
                    catalog_name: at.catalog.clone(),
                    schema_name: at.schema.clone(),
                    table_type: to_uc_table_type(table_type) as i32,
                    data_source_format: DataSourceFormat::Delta as i32,
                    ..Default::default()
                };
                self.check_required(&create_action, cx)
                    .await
                    .map_err(to_backend_err)
            }
            DeltaAction::WriteTable { table_id, .. } => {
                let uuid = uuid::Uuid::parse_str(table_id).map_err(|_| {
                    DeltaBackendError::InvalidArgument("table id is not a valid UUID".into())
                })?;
                let ident = ResourceIdent::Table(ResourceRef::Uuid(uuid));
                self.authorize_checked(&ident, &Permission::Write, cx)
                    .await
                    .map_err(to_backend_err)
            }
            DeltaAction::AdoptStaging { reservation } => {
                // The creator-match, in mangrove's identity terms: the caller's
                // principal name must equal the reservation's `created_by`
                // (anonymous reservations, `created_by == None`, are adoptable by
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
            // Read / delete / rename / credential vending / staging creation are
            // authorized by the downstream handler traits these operations
            // delegate to (`TableHandler`, `StagingTableHandler`,
            // `TemporaryCredentialHandler`), each of which runs `check_required`
            // itself. Authorizing again here would double-check; matching the
            // pre-crate behavior, the handler-level hook is a no-op for them.
            DeltaAction::ReadTable { .. }
            | DeltaAction::DeleteTable { .. }
            | DeltaAction::RenameTable { .. }
            | DeltaAction::VendTableCredential { .. }
            | DeltaAction::VendPathCredential { .. }
            | DeltaAction::CreateStaging { .. } => Ok(()),
            // `DeltaAction` is `#[non_exhaustive]`. Fail closed on an action this
            // adapter has not been taught: a newly added operation must not slip
            // through unauthorized until its arm is written.
            _ => Err(DeltaBackendError::PermissionDenied(
                "unrecognized Delta action".to_string(),
            )),
        }
    }

    async fn validate_external_location(
        &self,
        location: &str,
        _cx: &RequestContext,
    ) -> BackendResult<()> {
        let parsed = StorageLocationUrl::parse(location)
            .map_err(Error::from)
            .map_err(to_backend_err)?;
        validate_external_storage_location(self, &parsed)
            .await
            .map_err(to_backend_err)
    }

    async fn create_table_row(
        &self,
        spec: CreateTableSpec,
        _cx: &RequestContext,
    ) -> BackendResult<ResolvedTable> {
        // Managed adoption: consume the reservation, then create the table adopting
        // its id. The store keys objects by a single id PRIMARY KEY, so the staging
        // object must be removed before the table can claim that id — the same
        // ordering (and the same non-atomic window) as before the port collapsed
        // finalize + create. `olai-store` has no cross-op transaction, so a failed
        // `create` after the `delete` still orphans the reservation; truly closing
        // this needs a transaction in olai-store (trestle follow-up:
        // https://github.com/open-lakehouse/trestle/issues/75 — CAS update + txn).
        // `StagingReservation` carries `name`, so the delete needs no re-read.
        if let Some(reservation) = &spec.adopt_staging {
            let ident =
                ResourceIdent::staging_table(ResourceName::new([reservation.name.as_str()]));
            self.delete(&ident)
                .await
                .map_err(Error::from)
                .map_err(to_backend_err)?;
        }
        let table = Table {
            name: spec.name,
            catalog_name: spec.at.catalog,
            schema_name: spec.at.schema,
            table_type: to_uc_table_type(spec.table_type) as i32,
            data_source_format: DataSourceFormat::Delta as i32,
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
        let stored: Table = self
            .create(table.into())
            .await
            .map_err(Error::from)
            .and_then(|(r, _)| r.try_into().map_err(Error::from))
            .map_err(to_backend_err)?;
        Ok(table_to_resolved(stored))
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
        let mut table: Table = self
            .get(&ident)
            .await
            .map_err(Error::from)
            .and_then(|(r, _)| r.try_into().map_err(Error::from))
            .map_err(to_backend_err)?;
        // assert-etag compare-and-swap: `ResourceStore::update` (backed by
        // `olai-store`) is an unconditional overwrite with no expected-version, so
        // this is a *best-effort* check against the row we just read — it narrows
        // but does not fully close the read-modify-write race. Truly closing it
        // needs a conditional UPDATE in olai-store (trestle follow-up:
        // https://github.com/open-lakehouse/trestle/issues/75 — CAS update + txn).
        if let Some(expected) = &spec.expected_etag
            && expected != &etag_of(&table_to_resolved(table.clone()))
        {
            return Err(DeltaBackendError::UpdateRequirementConflict(
                "assert-etag failed: table has been modified".into(),
            ));
        }
        table.columns = spec.columns.into_iter().map(crate_column_to_uc).collect();
        table.properties = spec.properties.into_iter().collect();
        // `None` means "leave the stored comment unchanged" (see `UpdateTableSpec`):
        // the handler only sets it when a set-table-comment action is present.
        if let Some(comment) = spec.comment {
            table.comment = Some(comment);
        }
        let updated: Table = self
            .update(&ident, table.into())
            .await
            .map_err(Error::from)
            .and_then(|(r, _)| r.try_into().map_err(Error::from))
            .map_err(to_backend_err)?;
        Ok(table_to_resolved(updated))
    }

    async fn delete_table(&self, table: &TableRef, cx: &RequestContext) -> BackendResult<()> {
        TableHandler::delete_table(
            self,
            DeleteTableRequest {
                full_name: table.full_name(),
            },
            cx.clone(),
        )
        .await
        .map_err(to_backend_err)
    }

    async fn rename_table(
        &self,
        _from: &TableRef,
        _to_name: &str,
        _cx: &RequestContext,
    ) -> BackendResult<()> {
        // The UC table store has no rename op yet (tracked as a follow-up).
        Err(DeltaBackendError::NotImplemented("Delta API: renameTable"))
    }

    async fn allocate_staging(
        &self,
        at: &SchemaRef,
        name: &str,
        cx: &RequestContext,
    ) -> BackendResult<StagingReservation> {
        let staging = StagingTableHandler::create_staging_table(
            self,
            CreateStagingTableRequest {
                name: name.to_string(),
                catalog_name: at.catalog.clone(),
                schema_name: at.schema.clone(),
            },
            cx.clone(),
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
        cx: &RequestContext,
    ) -> BackendResult<VendedCredential> {
        let creds = self
            .generate_temporary_table_credentials(
                GenerateTemporaryTableCredentialsRequest {
                    table_id: table_id.to_string(),
                    operation: to_table_op(access),
                },
                cx.clone(),
            )
            .await
            .map_err(to_backend_err)?;
        let url = creds.url.clone();
        Ok(to_vended_credential(&creds, url))
    }

    async fn vend_path_credential(
        &self,
        location: &str,
        access: CredentialAccess,
        cx: &RequestContext,
    ) -> BackendResult<VendedCredential> {
        let creds = self
            .generate_temporary_path_credentials(
                GenerateTemporaryPathCredentialsRequest {
                    url: location.to_string(),
                    operation: to_path_op(access),
                    dry_run: Some(false),
                },
                cx.clone(),
            )
            .await
            .map_err(to_backend_err)?;
        let url = creds.url.clone();
        Ok(to_vended_credential(&creds, url))
    }

    fn commit_coordinator(&self) -> &dyn CommitCoordinator {
        ProvidesCommitCoordinator::commit_coordinator(self)
    }
}
