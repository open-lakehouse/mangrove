//! Handler trait for the UC Delta REST API (`/delta/v1/...`).
//!
//! This is the hand-written counterpart to the generated resource handlers: the
//! Delta API is a standalone REST protocol (mirrored from the Unity Catalog Java
//! reference `DeltaApiService` / `openapi/delta.yaml`), so the trait, router, and
//! models are all maintained by hand. See the Delta Sharing handler for the
//! precedent.
//!
//! Phase 1 ships the trait + a stubbed default implementation surface; the router
//! wires every operation. Behavior (contract validation, the `updateTable` action
//! dispatcher, commit coordination) lands in Phase 2.

use std::collections::BTreeMap;

use async_trait::async_trait;

use unitycatalog_common::models::staging_tables::v1::CreateStagingTableRequest;
use unitycatalog_common::models::tables::v1::{
    Column, CreateTableRequest, DataSourceFormat, DeleteTableRequest, GetTableRequest, Table,
    TableType,
};
use unitycatalog_common::models::temporary_credentials::v1::{
    GenerateTemporaryPathCredentialsRequest, GenerateTemporaryTableCredentialsRequest,
    TemporaryCredential, generate_temporary_path_credentials_request::Operation as PathOp,
    generate_temporary_table_credentials_request::Operation as TableOp,
    temporary_credential::Credentials,
};
use unitycatalog_common::models::{ResourceIdent, ResourceRef};
use unitycatalog_delta_api::coordinator::ProvidesCommitCoordinator;

use crate::api::RequestContext;
use crate::api::credentials::CredentialHandlerExt;
use crate::api::staging_tables::find_staging_table_by_location;
use crate::api::tables::{TableHandler, TableManager};
use crate::api::temporary_credentials::TemporaryCredentialHandler;
use crate::codegen::staging_tables::StagingTableHandler;
use crate::policy::{Permission, Policy, Principal};
use crate::rest::routers::delta::models::*;
use crate::services::ProvidesLocalStoragePolicy;
use crate::services::location::StorageLocationUrl;
use crate::services::managed_delta_contract as contract;
use crate::services::object_store::validate_external_storage_location;
use crate::store::ResourceStore;
use crate::{Error, Result};

/// A fully-qualified table coordinate parsed from the request path.
#[derive(Debug, Clone)]
pub struct TablePath {
    pub catalog: String,
    pub schema: String,
    pub table: String,
}

/// A schema coordinate (parent of staging-tables / tables creation).
#[derive(Debug, Clone)]
pub struct SchemaPath {
    pub catalog: String,
    pub schema: String,
}

/// Query parameters for `getConfig`.
#[derive(Debug, Clone)]
pub struct GetConfigQuery {
    pub catalog: String,
    /// Comma-separated list of highest protocol versions the client supports.
    pub protocol_versions: String,
}

/// Handler for the Delta REST API. One method per `delta.yaml` operation.
///
/// Method names match the spec `operationId`s. Path/query parameters are passed
/// as typed structs; request bodies use the hand-written model types.
#[async_trait]
pub trait DeltaApiHandler<Cx = RequestContext>: Send + Sync + 'static {
    /// `GET /delta/v1/config`
    async fn get_config(&self, query: GetConfigQuery, context: Cx) -> Result<DeltaCatalogConfig>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/staging-tables`
    async fn create_staging_table(
        &self,
        path: SchemaPath,
        request: DeltaCreateStagingTableRequest,
        context: Cx,
    ) -> Result<DeltaStagingTableResponse>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables`
    async fn create_table(
        &self,
        path: SchemaPath,
        request: DeltaCreateTableRequest,
        context: Cx,
    ) -> Result<DeltaLoadTableResponse>;

    /// `GET /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn load_table(&self, path: TablePath, context: Cx) -> Result<DeltaLoadTableResponse>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn update_table(
        &self,
        path: TablePath,
        request: DeltaUpdateTableRequest,
        context: Cx,
    ) -> Result<DeltaLoadTableResponse>;

    /// `DELETE /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn delete_table(&self, path: TablePath, context: Cx) -> Result<()>;

    /// `HEAD /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    ///
    /// Returns `Ok(())` if the table exists; the router maps a not-found error to 404.
    async fn table_exists(&self, path: TablePath, context: Cx) -> Result<()>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/rename`
    async fn rename_table(
        &self,
        path: TablePath,
        request: DeltaRenameTableRequest,
        context: Cx,
    ) -> Result<()>;

    /// `GET /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/credentials`
    async fn get_table_credentials(
        &self,
        path: TablePath,
        operation: DeltaCredentialOperation,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/metrics`
    async fn report_metrics(
        &self,
        path: TablePath,
        request: DeltaReportMetricsRequest,
        context: Cx,
    ) -> Result<()>;

    /// `GET /delta/v1/staging-tables/{table_id}/credentials`
    async fn get_staging_table_credentials(
        &self,
        table_id: String,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse>;

    /// `GET /delta/v1/temporary-path-credentials`
    async fn get_temporary_path_credentials(
        &self,
        location: String,
        operation: DeltaCredentialOperation,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse>;
}

/// The static endpoint list `getConfig` advertises (the spec's `DeltaCatalogConfig.endpoints`).
const ENDPOINTS: &[&str] = &[
    "POST /v1/catalogs/{catalog}/schemas/{schema}/staging-tables",
    "POST /v1/catalogs/{catalog}/schemas/{schema}/tables",
    "GET /v1/catalogs/{catalog}/schemas/{schema}/tables",
    "GET /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
    "POST /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
    "DELETE /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
    "HEAD /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}",
    "POST /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/rename",
    "GET /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/credentials",
    "POST /v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/metrics",
    "GET /v1/staging-tables/{table_id}/credentials",
    "GET /v1/temporary-path-credentials",
];

/// Real implementation of the Delta API.
///
/// Delegates resource lifecycle, permission checks, and credential vending to the
/// existing handler traits (`StagingTableHandler`, `TableHandler`,
/// `TemporaryCredentialHandler`, `CredentialHandlerExt`) and the commit coordinator;
/// the Delta-specific logic (the catalog-managed contract, the kebab-case wire
/// mapping, the `updateTable` action dispatcher) lives here. `ServerHandler`
/// satisfies all these bounds, so the CLI wiring is unaffected.
#[async_trait]
impl<T> DeltaApiHandler<RequestContext> for T
where
    T: ResourceStore
        + Policy<RequestContext>
        + StagingTableHandler<RequestContext>
        + TableHandler<RequestContext>
        + TemporaryCredentialHandler<RequestContext>
        + CredentialHandlerExt
        + TableManager
        + ProvidesCommitCoordinator
        + ProvidesLocalStoragePolicy,
{
    async fn get_config(
        &self,
        query: GetConfigQuery,
        _context: RequestContext,
    ) -> Result<DeltaCatalogConfig> {
        // Confirm the catalog exists (mirrors the reference, which loads it).
        if query.catalog.is_empty() {
            return Err(Error::invalid_argument(
                "catalog query parameter is required",
            ));
        }
        let catalog_ident =
            ResourceIdent::catalog(unitycatalog_common::models::ResourceName::new([query
                .catalog
                .as_str()]));
        self.get(&catalog_ident).await?;
        Ok(DeltaCatalogConfig {
            endpoints: ENDPOINTS.iter().map(|s| s.to_string()).collect(),
            protocol_version: "1.0".to_string(),
        })
    }

    async fn create_staging_table(
        &self,
        path: SchemaPath,
        request: DeltaCreateStagingTableRequest,
        context: RequestContext,
    ) -> Result<DeltaStagingTableResponse> {
        // Reuse the existing staging flow (allocates uuid + managed location and
        // enforces CREATE on the schema).
        let staging = StagingTableHandler::create_staging_table(
            self,
            CreateStagingTableRequest {
                name: request.name,
                catalog_name: path.catalog,
                schema_name: path.schema,
            },
            context.clone(),
        )
        .await?;

        // Vend READ_WRITE credentials for the staging location so the client can
        // write the initial commit.
        let creds = self
            .generate_temporary_path_credentials(
                GenerateTemporaryPathCredentialsRequest {
                    url: staging.staging_location.clone(),
                    operation: PathOp::PathReadWrite as i32,
                    dry_run: Some(false),
                },
                context,
            )
            .await?;

        Ok(DeltaStagingTableResponse {
            table_id: staging.id.clone(),
            table_type: DeltaTableType::Managed,
            location: staging.staging_location.clone(),
            storage_credentials: vec![to_storage_credential(
                &staging.staging_location,
                &creds,
                DeltaCredentialOperation::ReadWrite,
            )],
            required_protocol: DeltaProtocol {
                min_reader_version: contract::REQUIRED_MIN_READER_VERSION,
                min_writer_version: contract::REQUIRED_MIN_WRITER_VERSION,
                reader_features: Some(
                    contract::REQUIRED_READER_FEATURES
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ),
                writer_features: Some(
                    contract::REQUIRED_WRITER_FEATURES
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ),
            },
            suggested_protocol: Some(DeltaSuggestedProtocol {
                reader_features: Some(
                    contract::SUGGESTED_READER_FEATURES
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ),
                writer_features: Some(
                    contract::SUGGESTED_WRITER_FEATURES
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ),
            }),
            required_properties: contract::required_properties(&staging.id),
            suggested_properties: Some(contract::suggested_properties()),
        })
    }

    async fn create_table(
        &self,
        path: SchemaPath,
        request: DeltaCreateTableRequest,
        context: RequestContext,
    ) -> Result<DeltaLoadTableResponse> {
        // Required-field + shape checks (mirrors DeltaCreateTableMapper).
        if request.name.is_empty() {
            return Err(Error::invalid_argument("Table name is required."));
        }
        if request.location.is_empty() {
            return Err(Error::invalid_argument("Table location is required."));
        }

        // Authorize CREATE on the target table. We persist via the store directly
        // (below) rather than `TableHandler::create_table` — whose managed branch
        // reads the snapshot — so the permission check is done explicitly here via
        // the same `SecuredAction` the UC-REST createTable uses.
        let create_action = CreateTableRequest {
            name: request.name.clone(),
            catalog_name: path.catalog.clone(),
            schema_name: path.schema.clone(),
            table_type: to_uc_table_type(request.table_type) as i32,
            data_source_format: DataSourceFormat::Delta as i32,
            ..Default::default()
        };
        self.check_required(&create_action, &context).await?;

        // MANAGED-only: the full catalog-managed contract validates against the
        // request's declared protocol/properties (the client wrote 0.json).
        if request.table_type == DeltaTableType::Managed {
            contract::validate(
                &request.protocol,
                request.domain_metadata.as_ref(),
                &request.properties,
            )?;
        }

        let columns =
            contract::delta_columns_to_uc(&request.columns, request.partition_columns.as_deref())?;
        let stored_properties = contract::build_stored_properties(&request);

        // For MANAGED, finalize the staging reservation: enforce creator-match,
        // tableId identity, mark committed, and adopt the staging uuid.
        let table_id = if request.table_type == DeltaTableType::Managed {
            let staging = find_staging_table_by_location(self, &request.location).await?;
            match context.recipient() {
                Principal::User(name) if staging.created_by.as_deref() == Some(name.as_str()) => {}
                Principal::Anonymous if staging.created_by.is_none() => {}
                _ => return Err(Error::NotAllowed),
            }
            if staging.stage_committed {
                return Err(Error::invalid_argument(format!(
                    "staging table at '{}' has already been committed",
                    request.location
                )));
            }
            contract::validate_table_id_property(&request.properties, &staging.id)?;

            // Consume the staging reservation: the table adopts its id, and the
            // backing object store keys objects by id (a single PRIMARY KEY), so
            // the staging object must be removed before the table claims that id —
            // otherwise the table insert collides ("Entity already exists").
            // Address it by the identity the store keys it under —
            // `StagingTable::resource_name()` is a bare `[name]`.
            let staging_ident =
                ResourceIdent::staging_table(unitycatalog_common::models::ResourceName::new([
                    staging.name.as_str(),
                ]));
            self.delete(&staging_ident).await?;
            Some(staging.id)
        } else {
            // EXTERNAL: the location must live inside a registered external location.
            let location = StorageLocationUrl::parse(&request.location)?;
            validate_external_storage_location(self, &location).await?;
            None
        };

        // Persist via the existing TableHandler so its CREATE authorization and
        // store conventions apply. We pass the Delta-derived columns/properties.
        let create_req = CreateTableRequest {
            name: request.name,
            catalog_name: path.catalog,
            schema_name: path.schema,
            table_type: to_uc_table_type(request.table_type) as i32,
            data_source_format: DataSourceFormat::Delta as i32,
            columns,
            storage_location: Some(request.location),
            comment: request.comment,
            properties: stored_properties.into_iter().collect(),
            // The Delta API only creates Delta tables, never view-like types.
            view_definition: None,
            view_dependencies: None,
        };
        // The TableHandler create_table reads the snapshot for the managed branch;
        // for the Delta API we have already validated and want to trust the request,
        // so we persist directly via the store instead of re-reading.
        let table = build_table_for_create(&create_req, table_id);
        let stored: Table = self.create(table.into()).await?.0.try_into()?;

        build_load_table_response(self, stored).await
    }

    async fn load_table(
        &self,
        path: TablePath,
        context: RequestContext,
    ) -> Result<DeltaLoadTableResponse> {
        let table = TableHandler::get_table(
            self,
            GetTableRequest {
                full_name: format!("{}.{}.{}", path.catalog, path.schema, path.table),
                include_delta_metadata: None,
                include_browse: None,
                include_manifest_capabilities: None,
            },
            context,
        )
        .await?;
        build_load_table_response(self, table).await
    }

    async fn update_table(
        &self,
        path: TablePath,
        request: DeltaUpdateTableRequest,
        context: RequestContext,
    ) -> Result<DeltaLoadTableResponse> {
        update_table_impl(self, path, request, context).await
    }

    async fn delete_table(&self, path: TablePath, context: RequestContext) -> Result<()> {
        TableHandler::delete_table(
            self,
            DeleteTableRequest {
                full_name: format!("{}.{}.{}", path.catalog, path.schema, path.table),
            },
            context,
        )
        .await
    }

    async fn table_exists(&self, path: TablePath, context: RequestContext) -> Result<()> {
        // Map "does not exist" to NotFound (the router renders 404 / the HEAD 204).
        TableHandler::get_table(
            self,
            GetTableRequest {
                full_name: format!("{}.{}.{}", path.catalog, path.schema, path.table),
                include_delta_metadata: None,
                include_browse: None,
                include_manifest_capabilities: None,
            },
            context,
        )
        .await
        .map(|_| ())
    }

    async fn rename_table(
        &self,
        _path: TablePath,
        _request: DeltaRenameTableRequest,
        _context: RequestContext,
    ) -> Result<()> {
        // The UC table store has no rename op yet; surface as not-implemented until
        // an UpdateTable/rename path exists (tracked as a follow-up).
        Err(Error::NotImplemented("Delta API: renameTable"))
    }

    async fn get_table_credentials(
        &self,
        path: TablePath,
        operation: DeltaCredentialOperation,
        context: RequestContext,
    ) -> Result<DeltaCredentialsResponse> {
        // Resolve the table uuid, then vend via the existing table-credential flow
        // (which authorizes the operation and downscopes the credential).
        let table = TableHandler::get_table(
            self,
            GetTableRequest {
                full_name: format!("{}.{}.{}", path.catalog, path.schema, path.table),
                include_delta_metadata: None,
                include_browse: None,
                include_manifest_capabilities: None,
            },
            context.clone(),
        )
        .await?;
        let table_id = table
            .table_id
            .ok_or_else(|| Error::invalid_argument("table has no id"))?;
        let creds = self
            .generate_temporary_table_credentials(
                GenerateTemporaryTableCredentialsRequest {
                    table_id,
                    operation: match operation {
                        DeltaCredentialOperation::Read => TableOp::Read as i32,
                        DeltaCredentialOperation::ReadWrite => TableOp::ReadWrite as i32,
                    },
                },
                context,
            )
            .await?;
        Ok(DeltaCredentialsResponse {
            storage_credentials: vec![to_storage_credential(&creds.url, &creds, operation)],
        })
    }

    async fn report_metrics(
        &self,
        path: TablePath,
        request: DeltaReportMetricsRequest,
        context: RequestContext,
    ) -> Result<()> {
        // Validate the table exists and that the body table-id matches the path.
        let table = TableHandler::get_table(
            self,
            GetTableRequest {
                full_name: format!("{}.{}.{}", path.catalog, path.schema, path.table),
                include_delta_metadata: None,
                include_browse: None,
                include_manifest_capabilities: None,
            },
            context,
        )
        .await?;
        if table.table_id.as_deref() != Some(request.table_id.as_str()) {
            return Err(Error::invalid_argument(
                "report table-id does not match the table identified by the path",
            ));
        }
        if let Some(cv) = request
            .report
            .as_ref()
            .and_then(|r| r.commit_report.as_ref())
            .and_then(|c| c.file_size_histogram.as_ref())
            .and_then(|h| h.commit_version)
            && cv < 0
        {
            return Err(Error::invalid_argument(
                "commit-version must be non-negative",
            ));
        }
        // Accept-and-ack: we have no maintenance scheduler yet (follow-up issue).
        Ok(())
    }

    async fn get_staging_table_credentials(
        &self,
        table_id: String,
        context: RequestContext,
    ) -> Result<DeltaCredentialsResponse> {
        // Resolve the staging reservation by uuid, then vend READ_WRITE creds for
        // its location.
        let uuid = uuid::Uuid::parse_str(&table_id)
            .map_err(|_| Error::invalid_argument("table_id is not a valid UUID"))?;
        let ident = ResourceIdent::StagingTable(ResourceRef::Uuid(uuid));
        let staging: unitycatalog_common::models::staging_tables::v1::StagingTable =
            self.get(&ident).await?.0.try_into()?;
        let creds = self
            .generate_temporary_path_credentials(
                GenerateTemporaryPathCredentialsRequest {
                    url: staging.staging_location.clone(),
                    operation: PathOp::PathReadWrite as i32,
                    dry_run: Some(false),
                },
                context,
            )
            .await?;
        Ok(DeltaCredentialsResponse {
            storage_credentials: vec![to_storage_credential(
                &staging.staging_location,
                &creds,
                DeltaCredentialOperation::ReadWrite,
            )],
        })
    }

    async fn get_temporary_path_credentials(
        &self,
        location: String,
        operation: DeltaCredentialOperation,
        context: RequestContext,
    ) -> Result<DeltaCredentialsResponse> {
        let creds = self
            .generate_temporary_path_credentials(
                GenerateTemporaryPathCredentialsRequest {
                    url: location.clone(),
                    operation: match operation {
                        DeltaCredentialOperation::Read => PathOp::PathRead as i32,
                        DeltaCredentialOperation::ReadWrite => PathOp::PathReadWrite as i32,
                    },
                    dry_run: Some(false),
                },
                context,
            )
            .await?;
        Ok(DeltaCredentialsResponse {
            storage_credentials: vec![to_storage_credential(&creds.url, &creds, operation)],
        })
    }
}

// ===================================================================
// Helpers
// ===================================================================

fn to_uc_table_type(t: DeltaTableType) -> TableType {
    match t {
        DeltaTableType::Managed => TableType::Managed,
        DeltaTableType::External => TableType::External,
    }
}

/// Assemble the persisted [`Table`] for a Delta createTable. For MANAGED the uuid is
/// the staging reservation's; for EXTERNAL the store assigns one.
fn build_table_for_create(req: &CreateTableRequest, table_id: Option<String>) -> Table {
    Table {
        name: req.name.clone(),
        catalog_name: req.catalog_name.clone(),
        schema_name: req.schema_name.clone(),
        table_type: req.table_type,
        data_source_format: req.data_source_format,
        columns: req.columns.clone(),
        storage_location: req.storage_location.clone(),
        comment: req.comment.clone(),
        properties: req.properties.clone(),
        table_id,
        ..Default::default()
    }
}

/// Build a `DeltaLoadTableResponse` from a stored [`Table`], appending unbackfilled
/// commits + `latest_table_version` for MANAGED+DELTA tables.
async fn build_load_table_response<T>(handler: &T, table: Table) -> Result<DeltaLoadTableResponse>
where
    T: ProvidesCommitCoordinator,
{
    // View-like table types (e.g. metric views) have no Delta log and no storage
    // of their own; the Delta API cannot serve them.
    if !matches!(
        TableType::try_from(table.table_type),
        Ok(TableType::Managed | TableType::External)
    ) {
        return Err(Error::invalid_argument(format!(
            "table '{}' is not a Delta table and cannot be loaded via the Delta API",
            table.full_name
        )));
    }

    let metadata = build_table_metadata(&table);

    let (commits, latest_table_version) = if table.table_type == TableType::Managed as i32
        && table.data_source_format == DataSourceFormat::Delta as i32
        && let Some(id) = table.table_id.as_deref()
    {
        let (commits, latest) = handler
            .commit_coordinator()
            .get_commits(id, 0, None)
            .await
            .map_err(|e| Error::generic(format!("commit coordinator: {e:?}")))?;
        (
            Some(commits.into_iter().map(to_delta_commit).collect()),
            Some(latest),
        )
    } else {
        (None, None)
    };

    Ok(DeltaLoadTableResponse {
        metadata,
        commits,
        uniform: None,
        latest_table_version,
    })
}

fn build_table_metadata(table: &Table) -> DeltaTableMetadata {
    let properties: BTreeMap<String, String> = table.properties.clone().into_iter().collect();
    let etag = match table.updated_at {
        Some(ts) => format!("etag-{ts}"),
        None => format!("etag-{}", table.table_id.clone().unwrap_or_default()),
    };
    let partition_columns: Vec<String> = {
        let mut p: Vec<(&i32, &Column)> = table
            .columns
            .iter()
            .filter_map(|c| c.partition_index.as_ref().map(|idx| (idx, c)))
            .collect();
        p.sort_by_key(|(idx, _)| **idx);
        p.into_iter().map(|(_, c)| c.name.clone()).collect()
    };

    DeltaTableMetadata {
        etag,
        table_type: if table.table_type == TableType::External as i32 {
            DeltaTableType::External
        } else {
            DeltaTableType::Managed
        },
        table_uuid: table.table_id.clone().unwrap_or_default(),
        location: table.storage_location.clone().unwrap_or_default(),
        created_time: table.created_at.unwrap_or_default(),
        updated_time: table.updated_at.or(table.created_at).unwrap_or_default(),
        columns: contract::uc_columns_to_delta(&table.columns),
        partition_columns: (!partition_columns.is_empty()).then_some(partition_columns),
        properties: properties.clone(),
        last_commit_version: properties
            .get(contract::PROP_LAST_UPDATE_VERSION)
            .and_then(|v| v.parse().ok()),
        last_commit_timestamp_ms: properties
            .get(contract::PROP_LAST_COMMIT_TIMESTAMP)
            .and_then(|v| v.parse().ok()),
    }
}

fn to_delta_commit(c: unitycatalog_delta_api::coordinator::CommitInfo) -> DeltaCommit {
    DeltaCommit {
        version: c.version,
        timestamp: c.timestamp,
        file_name: c.file_name,
        file_size: c.file_size,
        file_modification_timestamp: c.file_modification_timestamp,
    }
}

/// Map a vended [`TemporaryCredential`] onto the Delta API `DeltaStorageCredential`
/// (mirrors the reference `DeltaCredentialsMapper`).
fn to_storage_credential(
    prefix: &str,
    creds: &TemporaryCredential,
    operation: DeltaCredentialOperation,
) -> DeltaStorageCredential {
    let mut config = DeltaStorageCredentialConfig::default();
    match &creds.credentials {
        Some(Credentials::AwsTempCredentials(aws)) => {
            config.s3_access_key_id = Some(aws.access_key_id.clone());
            config.s3_secret_access_key = Some(aws.secret_access_key.clone());
            if !aws.session_token.is_empty() {
                config.s3_session_token = Some(aws.session_token.clone());
            }
        }
        Some(Credentials::AzureUserDelegationSas(az)) => {
            config.azure_sas_token = Some(az.sas_token.clone());
        }
        Some(Credentials::GcpOauthToken(gcp)) => {
            config.gcs_oauth_token = Some(gcp.oauth_token.clone());
        }
        // R2 reuses the S3-shaped fields; AAD is not surfaced in the Delta config.
        Some(Credentials::R2TempCredentials(r2)) => {
            config.s3_access_key_id = Some(r2.access_key_id.clone());
            config.s3_secret_access_key = Some(r2.secret_access_key.clone());
            if !r2.session_token.is_empty() {
                config.s3_session_token = Some(r2.session_token.clone());
            }
        }
        _ => {}
    }
    DeltaStorageCredential {
        prefix: prefix.to_string(),
        operation,
        config,
        expiration_time_ms: creds.expiration_time,
    }
}

// ===================================================================
// updateTable action dispatcher (DeltaUpdateTableMapper)
// ===================================================================

/// Apply an `updateTable` request: check requirements, apply the action list in the
/// reference's canonical order, route commit/backfill through the coordinator, and
/// return the refreshed table. Mirrors `DeltaUpdateTableMapper` +
/// `DeltaCommitRepository.applyCommitAndBackfillInSession`.
async fn update_table_impl<T>(
    handler: &T,
    path: TablePath,
    request: DeltaUpdateTableRequest,
    context: RequestContext,
) -> Result<DeltaLoadTableResponse>
where
    T: ResourceStore
        + Policy<RequestContext>
        + TableHandler<RequestContext>
        + ProvidesCommitCoordinator,
{
    // Resolve the table by name, then authorize WRITE.
    let mut table = TableHandler::get_table(
        handler,
        GetTableRequest {
            full_name: format!("{}.{}.{}", path.catalog, path.schema, path.table),
            include_delta_metadata: None,
            include_browse: None,
            include_manifest_capabilities: None,
        },
        context.clone(),
    )
    .await?;
    let table_uuid = table
        .table_id
        .clone()
        .ok_or_else(|| Error::invalid_argument("table has no id"))?;
    let uuid = uuid::Uuid::parse_str(&table_uuid)
        .map_err(|_| Error::invalid_argument("table id is not a valid UUID"))?;
    let table_ident = ResourceIdent::Table(ResourceRef::Uuid(uuid));
    handler
        .authorize_checked(&table_ident, &Permission::Write, &context)
        .await?;

    // --- Requirements (assert-table-uuid mandatory; assert-etag optional) ---
    let has_uuid_assert = request
        .requirements
        .iter()
        .any(|r| matches!(r, DeltaTableRequirement::AssertTableUuid { .. }));
    if !has_uuid_assert {
        return Err(Error::invalid_argument(
            "assert-table-uuid requirement is required.",
        ));
    }
    let current_etag = match table.updated_at {
        Some(ts) => format!("etag-{ts}"),
        None => format!("etag-{table_uuid}"),
    };
    for req in &request.requirements {
        match req {
            DeltaTableRequirement::AssertTableUuid { uuid } => {
                if uuid != &table_uuid {
                    return Err(Error::UpdateRequirementConflict(format!(
                        "assert-table-uuid failed: expected {uuid} but table has {table_uuid}"
                    )));
                }
            }
            DeltaTableRequirement::AssertEtag { etag } => {
                if etag != &current_etag {
                    return Err(Error::UpdateRequirementConflict(
                        "assert-etag failed: table has been modified".to_string(),
                    ));
                }
            }
        }
    }

    // --- Overlap checks (set/remove on the same key) ---
    let set_prop_keys: Vec<&String> = request
        .updates
        .iter()
        .filter_map(|u| match u {
            DeltaTableUpdate::SetProperties { updates } => Some(updates.keys()),
            _ => None,
        })
        .flatten()
        .collect();
    if let Some(removals) = request.updates.iter().find_map(|u| match u {
        DeltaTableUpdate::RemoveProperties { removals } => Some(removals),
        _ => None,
    }) {
        for r in removals {
            if set_prop_keys.contains(&r) {
                return Err(Error::invalid_argument(format!(
                    "set-properties and remove-properties overlap on key: {r}"
                )));
            }
        }
    }

    let mut properties: BTreeMap<String, String> = table.properties.clone().into_iter().collect();
    let is_managed = table.table_type == TableType::Managed as i32;
    let mut metadata_changed = false;

    // Apply in canonical order (not request order). We make multiple passes.
    // 1. set-columns / set-partition-columns
    apply_schema_and_partitions(&mut table, &request.updates)?;
    if request.updates.iter().any(|u| {
        matches!(
            u,
            DeltaTableUpdate::SetColumns { .. } | DeltaTableUpdate::SetPartitionColumns { .. }
        )
    }) {
        metadata_changed = true;
    }

    // 2. set-protocol (re-derive delta.feature.* and re-validate for MANAGED)
    if let Some(protocol) = request.updates.iter().find_map(|u| match u {
        DeltaTableUpdate::SetProtocol { protocol } => Some(protocol),
        _ => None,
    }) {
        // Drop existing feature props, then re-derive from the new protocol.
        properties.retain(|k, _| !k.starts_with("delta.feature."));
        contract::derive_from_protocol(&mut properties, protocol);
        if is_managed {
            contract::validate(protocol, None, &properties)?;
        }
        metadata_changed = true;
    }

    // 3. set-properties / 4. remove-properties
    for update in &request.updates {
        match update {
            DeltaTableUpdate::SetProperties { updates } => {
                properties.extend(updates.clone());
                metadata_changed = true;
            }
            DeltaTableUpdate::RemoveProperties { removals } => {
                for k in removals {
                    properties.remove(k);
                }
                metadata_changed = true;
            }
            _ => {}
        }
    }

    // 5. set-domain-metadata / 6. remove-domain-metadata
    for update in &request.updates {
        match update {
            DeltaTableUpdate::SetDomainMetadata { updates } => {
                contract::derive_from_domain_metadata(&mut properties, updates);
                metadata_changed = true;
            }
            DeltaTableUpdate::RemoveDomainMetadata { domains } => {
                for d in domains {
                    match d.as_str() {
                        "delta.clustering" => {
                            properties.remove("delta.clusteringColumns");
                        }
                        "delta.rowTracking" => {
                            properties.remove("delta.rowTracking.rowIdHighWaterMark");
                        }
                        other => {
                            return Err(Error::invalid_argument(format!(
                                "Unknown domain in remove-domain-metadata: {other}"
                            )));
                        }
                    }
                }
                metadata_changed = true;
            }
            _ => {}
        }
    }

    // 7. set-table-comment
    if let Some(comment) = request.updates.iter().find_map(|u| match u {
        DeltaTableUpdate::SetTableComment { comment } => Some(comment.clone()),
        _ => None,
    }) {
        table.comment = Some(comment);
        metadata_changed = true;
    }

    // 8. update-metadata-snapshot-version (EXTERNAL only)
    if let Some((v, ts)) = request.updates.iter().find_map(|u| match u {
        DeltaTableUpdate::UpdateMetadataSnapshotVersion {
            last_commit_version,
            last_commit_timestamp_ms,
        } => Some((*last_commit_version, *last_commit_timestamp_ms)),
        _ => None,
    }) {
        if is_managed {
            return Err(Error::invalid_argument(
                "update-metadata-snapshot-version is only valid for EXTERNAL tables",
            ));
        }
        properties.insert(
            contract::PROP_LAST_UPDATE_VERSION.to_string(),
            v.to_string(),
        );
        properties.insert(
            contract::PROP_LAST_COMMIT_TIMESTAMP.to_string(),
            ts.to_string(),
        );
        metadata_changed = true;
    }

    // 9. add-commit + set-latest-backfilled-version → commit coordinator
    let add_commit = request.updates.iter().find_map(|u| match u {
        DeltaTableUpdate::AddCommit { commit, .. } => Some(commit.clone()),
        _ => None,
    });
    let backfill = request.updates.iter().find_map(|u| match u {
        DeltaTableUpdate::SetLatestBackfilledVersion {
            latest_published_version,
        } => Some(*latest_published_version),
        _ => None,
    });
    if add_commit.is_some() || backfill.is_some() {
        if !is_managed {
            return Err(Error::invalid_argument(
                "add-commit / set-latest-backfilled-version require a MANAGED table",
            ));
        }
        let commit_info = add_commit.map(|c| unitycatalog_delta_api::coordinator::CommitInfo {
            version: c.version,
            timestamp: c.timestamp,
            file_name: c.file_name,
            file_size: c.file_size,
            file_modification_timestamp: c.file_modification_timestamp,
        });
        handler
            .commit_coordinator()
            .commit(&table_uuid, commit_info, backfill)
            .await?;
    }

    // Persist metadata changes (if any) before reloading.
    if metadata_changed {
        table.properties = properties.into_iter().collect();
        handler.update(&table_ident, table.clone().into()).await?;
    }

    build_load_table_response(handler, table).await
}

/// Apply `set-columns` and `set-partition-columns` in a combined pass, mirroring the
/// reference `applySchemaAndPartitionColumns`. Columns set replaces the schema;
/// partition columns re-derive partition indices.
fn apply_schema_and_partitions(table: &mut Table, updates: &[DeltaTableUpdate]) -> Result<()> {
    let new_columns = updates.iter().find_map(|u| match u {
        DeltaTableUpdate::SetColumns { columns } => Some(columns),
        _ => None,
    });
    let new_partitions = updates.iter().find_map(|u| match u {
        DeltaTableUpdate::SetPartitionColumns { partition_columns } => Some(partition_columns),
        _ => None,
    });
    if new_columns.is_none() && new_partitions.is_none() {
        return Ok(());
    }

    // Determine the schema to work from: the new one if set, else the existing.
    let columns: Vec<Column> = match new_columns {
        Some(struct_type) => {
            // Preserve existing partitioning unless new partitions are supplied.
            let existing_partitions: Vec<String> = {
                let mut p: Vec<(&i32, &Column)> = table
                    .columns
                    .iter()
                    .filter_map(|c| c.partition_index.as_ref().map(|idx| (idx, c)))
                    .collect();
                p.sort_by_key(|(idx, _)| **idx);
                p.into_iter().map(|(_, c)| c.name.clone()).collect()
            };
            let partitions = new_partitions.cloned().unwrap_or(existing_partitions);
            contract::delta_columns_to_uc(struct_type, Some(&partitions))?
        }
        None => {
            // Only partitions changed: re-apply indices to the existing columns.
            let partitions = new_partitions.expect("checked above");
            let mut cols = table.columns.clone();
            for col in &mut cols {
                col.partition_index = partitions
                    .iter()
                    .position(|p| p.eq_ignore_ascii_case(&col.name))
                    .map(|i| i as i32);
            }
            for p in partitions {
                if !cols.iter().any(|c| c.name.eq_ignore_ascii_case(p)) {
                    return Err(Error::invalid_argument(format!(
                        "partition column '{p}' is not present in the table schema"
                    )));
                }
            }
            cols
        }
    };
    table.columns = columns;
    Ok(())
}

#[cfg(all(test, feature = "memory"))]
mod tests {
    use std::sync::Arc;

    use unitycatalog_common::models::catalogs::v1::CreateCatalogRequest;
    use unitycatalog_common::models::credentials::v1::{
        AwsIamRoleConfig, CreateCredentialRequest, Purpose,
    };
    use unitycatalog_common::models::external_locations::v1::CreateExternalLocationRequest;
    use unitycatalog_common::models::schemas::v1::CreateSchemaRequest;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::api::{CatalogHandler, CredentialHandler, ExternalLocationHandler, SchemaHandler};
    use crate::memory::InMemoryResourceStore;
    use crate::policy::ConstantPolicy;
    use crate::services::ServerHandler;

    fn handler() -> ServerHandler<RequestContext> {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
        ServerHandler::try_new_tokio(policy, store.clone(), store).unwrap()
    }

    fn ctx() -> RequestContext {
        RequestContext {
            recipient: Principal::anonymous(),
        }
    }

    async fn setup(h: &ServerHandler<RequestContext>) {
        // The catalog's client-supplied root must be covered by a registered
        // external location.
        h.create_credential(
            CreateCredentialRequest {
                name: "cred".into(),
                purpose: Purpose::Storage as i32,
                aws_iam_role: Some(AwsIamRoleConfig {
                    role_arn: "arn:aws:iam::123456789012:role/test".into(),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_external_location(
            CreateExternalLocationRequest {
                name: "el".into(),
                url: "s3://bucket/cat".into(),
                credential_name: "cred".into(),
                ..Default::default()
            },
            ctx(),
        )
        .await
        .unwrap();
        h.create_catalog(
            CreateCatalogRequest {
                name: "cat".into(),
                storage_root: Some("s3://bucket/cat".into()),
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
    }

    /// Create a staging reservation directly (avoids the credential-vending path of
    /// the public createStagingTable) and return it.
    async fn stage(
        h: &ServerHandler<RequestContext>,
        name: &str,
    ) -> unitycatalog_common::models::staging_tables::v1::StagingTable {
        StagingTableHandler::create_staging_table(
            h,
            CreateStagingTableRequest {
                name: name.into(),
                catalog_name: "cat".into(),
                schema_name: "sch".into(),
            },
            ctx(),
        )
        .await
        .unwrap()
    }

    fn compliant_protocol() -> DeltaProtocol {
        DeltaProtocol {
            min_reader_version: 3,
            min_writer_version: 7,
            reader_features: Some(
                contract::REQUIRED_READER_FEATURES
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            writer_features: Some(
                contract::REQUIRED_WRITER_FEATURES
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
        }
    }

    fn compliant_properties(table_id: &str) -> BTreeMap<String, String> {
        let mut p: BTreeMap<String, String> = contract::REQUIRED_FIXED_PROPERTIES
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        p.insert(contract::PROP_UC_TABLE_ID.to_string(), table_id.to_string());
        p
    }

    fn create_req(name: &str, location: &str, table_id: &str) -> DeltaCreateTableRequest {
        DeltaCreateTableRequest {
            name: name.into(),
            location: location.into(),
            table_type: DeltaTableType::Managed,
            data_source_format: Some(DeltaDataSourceFormat::Delta),
            comment: None,
            columns: DeltaStructType {
                type_tag: Default::default(),
                fields: vec![DeltaStructField {
                    name: "id".into(),
                    data_type: DeltaDataType::Primitive("long".into()),
                    nullable: false,
                    metadata: Default::default(),
                }],
            },
            partition_columns: None,
            protocol: compliant_protocol(),
            properties: compliant_properties(table_id),
            domain_metadata: None,
            last_commit_timestamp_ms: 1700,
            uniform: None,
        }
    }

    fn schema_path() -> SchemaPath {
        SchemaPath {
            catalog: "cat".into(),
            schema: "sch".into(),
        }
    }

    fn table_path(table: &str) -> TablePath {
        TablePath {
            catalog: "cat".into(),
            schema: "sch".into(),
            table: table.into(),
        }
    }

    #[tokio::test]
    async fn get_config_returns_endpoints() {
        let h = handler();
        setup(&h).await;
        let cfg = h
            .get_config(
                GetConfigQuery {
                    catalog: "cat".into(),
                    protocol_versions: "1.0".into(),
                },
                ctx(),
            )
            .await
            .unwrap();
        assert_eq!(cfg.protocol_version, "1.0");
        assert_eq!(cfg.endpoints.len(), 12);
    }

    #[tokio::test]
    async fn create_managed_table_happy_path() {
        let h = handler();
        setup(&h).await;
        let st = stage(&h, "t").await;

        let resp = DeltaApiHandler::create_table(
            &h,
            schema_path(),
            create_req("t", &st.staging_location, &st.id),
            ctx(),
        )
        .await
        .unwrap();
        assert_eq!(resp.metadata.table_uuid, st.id);
        assert_eq!(resp.metadata.table_type, DeltaTableType::Managed);
        // Derived feature property present.
        assert_eq!(
            resp.metadata
                .properties
                .get("delta.feature.catalogManaged")
                .map(String::as_str),
            Some("supported")
        );
        // Newly created managed table has no commits yet.
        assert_eq!(resp.latest_table_version, Some(0));

        // The staging reservation is consumed on create: the table adopts its
        // id (objects are keyed by a single id PRIMARY KEY, so the staging
        // object must be removed to free that id), so it is no longer findable.
        let err = find_staging_table_by_location(&h, &st.staging_location)
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound), "{err:?}");
    }

    #[tokio::test]
    async fn create_managed_table_id_mismatch_rejected() {
        let h = handler();
        setup(&h).await;
        let st = stage(&h, "t").await;
        // tableId property points at a different uuid than the staging reservation.
        let req = create_req(
            "t",
            &st.staging_location,
            "00000000-0000-0000-0000-000000000000",
        );
        let err = DeltaApiHandler::create_table(&h, schema_path(), req, ctx())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
    }

    #[tokio::test]
    async fn create_managed_table_without_staging_rejected() {
        let h = handler();
        setup(&h).await;
        let req = create_req("t", "s3://bucket/cat/__unitystorage/tables/nope", "id");
        let err = DeltaApiHandler::create_table(&h, schema_path(), req, ctx())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::NotFound), "{err:?}");
    }

    #[tokio::test]
    async fn create_managed_table_non_compliant_rejected() {
        let h = handler();
        setup(&h).await;
        let st = stage(&h, "t").await;
        let mut req = create_req("t", &st.staging_location, &st.id);
        req.protocol.min_writer_version = 5; // below contract minimum
        let err = DeltaApiHandler::create_table(&h, schema_path(), req, ctx())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
    }

    #[tokio::test]
    async fn update_table_requires_assert_table_uuid() {
        let h = handler();
        setup(&h).await;
        let st = stage(&h, "t").await;
        DeltaApiHandler::create_table(
            &h,
            schema_path(),
            create_req("t", &st.staging_location, &st.id),
            ctx(),
        )
        .await
        .unwrap();

        let req = DeltaUpdateTableRequest {
            requirements: vec![],
            updates: vec![DeltaTableUpdate::SetProperties {
                updates: BTreeMap::from([("k".into(), "v".into())]),
            }],
        };
        let err = h
            .update_table(table_path("t"), req, ctx())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
    }

    #[tokio::test]
    async fn update_table_etag_mismatch_conflicts() {
        let h = handler();
        setup(&h).await;
        let st = stage(&h, "t").await;
        DeltaApiHandler::create_table(
            &h,
            schema_path(),
            create_req("t", &st.staging_location, &st.id),
            ctx(),
        )
        .await
        .unwrap();

        let req = DeltaUpdateTableRequest {
            requirements: vec![
                DeltaTableRequirement::AssertTableUuid {
                    uuid: st.id.clone(),
                },
                DeltaTableRequirement::AssertEtag {
                    etag: "etag-wrong".into(),
                },
            ],
            updates: vec![DeltaTableUpdate::SetProperties {
                updates: BTreeMap::from([("k".into(), "v".into())]),
            }],
        };
        let err = h
            .update_table(table_path("t"), req, ctx())
            .await
            .unwrap_err();
        assert!(
            matches!(err, Error::UpdateRequirementConflict(_)),
            "{err:?}"
        );
    }

    #[tokio::test]
    async fn update_table_add_commit_visible_in_load() {
        let h = handler();
        setup(&h).await;
        let st = stage(&h, "t").await;
        DeltaApiHandler::create_table(
            &h,
            schema_path(),
            create_req("t", &st.staging_location, &st.id),
            ctx(),
        )
        .await
        .unwrap();

        let req = DeltaUpdateTableRequest {
            requirements: vec![DeltaTableRequirement::AssertTableUuid {
                uuid: st.id.clone(),
            }],
            updates: vec![DeltaTableUpdate::AddCommit {
                commit: DeltaCommit {
                    version: 1,
                    timestamp: 1800,
                    file_name: "00000000-0000-0000-0000-00000000002a.json".into(),
                    file_size: 64,
                    file_modification_timestamp: 1800,
                },
                uniform: None,
            }],
        };
        h.update_table(table_path("t"), req, ctx()).await.unwrap();

        let loaded = h.load_table(table_path("t"), ctx()).await.unwrap();
        assert_eq!(loaded.latest_table_version, Some(1));
        let commits = loaded.commits.unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].version, 1);
    }

    #[tokio::test]
    async fn update_table_set_remove_property_overlap_rejected() {
        let h = handler();
        setup(&h).await;
        let st = stage(&h, "t").await;
        DeltaApiHandler::create_table(
            &h,
            schema_path(),
            create_req("t", &st.staging_location, &st.id),
            ctx(),
        )
        .await
        .unwrap();

        let req = DeltaUpdateTableRequest {
            requirements: vec![DeltaTableRequirement::AssertTableUuid {
                uuid: st.id.clone(),
            }],
            updates: vec![
                DeltaTableUpdate::SetProperties {
                    updates: BTreeMap::from([("k".into(), "v".into())]),
                },
                DeltaTableUpdate::RemoveProperties {
                    removals: vec!["k".into()],
                },
            ],
        };
        let err = h
            .update_table(table_path("t"), req, ctx())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
    }

    #[tokio::test]
    async fn report_metrics_table_id_mismatch_rejected() {
        let h = handler();
        setup(&h).await;
        let st = stage(&h, "t").await;
        DeltaApiHandler::create_table(
            &h,
            schema_path(),
            create_req("t", &st.staging_location, &st.id),
            ctx(),
        )
        .await
        .unwrap();

        let req = DeltaReportMetricsRequest {
            table_id: "00000000-0000-0000-0000-000000000000".into(),
            report: None,
        };
        let err = h
            .report_metrics(table_path("t"), req, ctx())
            .await
            .unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");

        // Matching id succeeds (accept-and-ack).
        let ok = DeltaReportMetricsRequest {
            table_id: st.id.clone(),
            report: None,
        };
        h.report_metrics(table_path("t"), ok, ctx()).await.unwrap();
    }
}
