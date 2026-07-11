//! The `DeltaApiHandler` trait and its generic implementation over a
//! [`DeltaBackend`].
//!
//! One trait method per `delta.yaml` operation. The blanket
//! `impl<B: DeltaBackend<Cx>, Cx> DeltaApiHandler<Cx> for B` contains all the
//! Delta business logic — the managed-table contract, the `updateTable` action
//! dispatcher (applied in the reference's canonical order), `loadTable` commit-list
//! construction, and the credential→config mapping — expressed purely in terms of
//! the [`DeltaBackend`] port. Any server that implements the port gets the handler
//! (and thus the router) for free.

use std::collections::BTreeMap;

use async_trait::async_trait;

use crate::authz::DeltaAction;
use crate::backend::{
    CreateTableSpec, CredentialAccess, DeltaBackend, ResolvedTable, SchemaRef, TableRef,
    UpdateTableSpec, VendedCredential, VendedCredentialKind,
};
use crate::column::Column;
use crate::contract;
use crate::coordinator::CommitInfo;
use crate::error::{DeltaApiError, DeltaApiResult as Result, DeltaBackendError};
use crate::models::*;

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
/// as typed structs; request bodies use the wire model types.
#[async_trait]
pub trait DeltaApiHandler<Cx>: Send + Sync + 'static {
    /// `GET /delta/v1/config`
    async fn get_config(&self, query: GetConfigQuery, context: Cx) -> Result<DeltaCatalogConfig>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/staging-tables`
    async fn create_staging_table(
        &self,
        path: SchemaRef,
        request: DeltaCreateStagingTableRequest,
        context: Cx,
    ) -> Result<DeltaStagingTableResponse>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables`
    async fn create_table(
        &self,
        path: SchemaRef,
        request: DeltaCreateTableRequest,
        context: Cx,
    ) -> Result<DeltaLoadTableResponse>;

    /// `GET /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn load_table(&self, path: TableRef, context: Cx) -> Result<DeltaLoadTableResponse>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn update_table(
        &self,
        path: TableRef,
        request: DeltaUpdateTableRequest,
        context: Cx,
    ) -> Result<DeltaLoadTableResponse>;

    /// `DELETE /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn delete_table(&self, path: TableRef, context: Cx) -> Result<()>;

    /// `HEAD /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}`
    async fn table_exists(&self, path: TableRef, context: Cx) -> Result<()>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/rename`
    async fn rename_table(
        &self,
        path: TableRef,
        request: DeltaRenameTableRequest,
        context: Cx,
    ) -> Result<()>;

    /// `GET /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/credentials`
    async fn get_table_credentials(
        &self,
        path: TableRef,
        operation: DeltaCredentialOperation,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse>;

    /// `POST /delta/v1/catalogs/{catalog}/schemas/{schema}/tables/{table}/metrics`
    async fn report_metrics(
        &self,
        path: TableRef,
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

/// The static endpoint list `getConfig` advertises.
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

#[async_trait]
impl<B, Cx> DeltaApiHandler<Cx> for B
where
    B: DeltaBackend<Cx>,
    Cx: Send + 'static,
{
    async fn get_config(&self, query: GetConfigQuery, context: Cx) -> Result<DeltaCatalogConfig> {
        if query.catalog.is_empty() {
            return Err(DeltaApiError::invalid_argument(
                "catalog query parameter is required",
            ));
        }
        self.catalog_exists(&query.catalog, &context).await?;
        Ok(DeltaCatalogConfig {
            endpoints: ENDPOINTS.iter().map(|s| s.to_string()).collect(),
            protocol_version: "1.0".to_string(),
        })
    }

    async fn create_staging_table(
        &self,
        path: SchemaRef,
        request: DeltaCreateStagingTableRequest,
        context: Cx,
    ) -> Result<DeltaStagingTableResponse> {
        self.authorize(
            DeltaAction::CreateStaging {
                at: &path,
                name: &request.name,
            },
            &context,
        )
        .await?;

        let staging = self
            .allocate_staging(&path, &request.name, &context)
            .await?;

        let creds = self
            .vend_path_credential(&staging.location, CredentialAccess::ReadWrite, &context)
            .await?;

        Ok(DeltaStagingTableResponse {
            table_id: staging.table_id.clone(),
            table_type: DeltaTableType::Managed,
            location: staging.location.clone(),
            storage_credentials: vec![to_storage_credential(
                &staging.location,
                &creds,
                DeltaCredentialOperation::ReadWrite,
            )],
            required_protocol: DeltaProtocol {
                min_reader_version: contract::REQUIRED_MIN_READER_VERSION,
                min_writer_version: contract::REQUIRED_MIN_WRITER_VERSION,
                reader_features: Some(feature_vec(contract::REQUIRED_READER_FEATURES)),
                writer_features: Some(feature_vec(contract::REQUIRED_WRITER_FEATURES)),
            },
            suggested_protocol: Some(DeltaSuggestedProtocol {
                reader_features: Some(feature_vec(contract::SUGGESTED_READER_FEATURES)),
                writer_features: Some(feature_vec(contract::SUGGESTED_WRITER_FEATURES)),
            }),
            required_properties: contract::required_properties(&staging.table_id),
            suggested_properties: Some(contract::suggested_properties()),
        })
    }

    async fn create_table(
        &self,
        path: SchemaRef,
        request: DeltaCreateTableRequest,
        context: Cx,
    ) -> Result<DeltaLoadTableResponse> {
        if request.name.is_empty() {
            return Err(DeltaApiError::invalid_argument("Table name is required."));
        }
        if request.location.is_empty() {
            return Err(DeltaApiError::invalid_argument(
                "Table location is required.",
            ));
        }

        self.authorize(
            DeltaAction::CreateTable {
                at: &path,
                name: &request.name,
                table_type: request.table_type,
            },
            &context,
        )
        .await?;

        // MANAGED-only: validate the full catalog-managed contract.
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

        // For MANAGED, finalize the staging reservation (creator-match, tableId
        // identity, adopt the staging uuid).
        let table_id = if request.table_type == DeltaTableType::Managed {
            let staging = self
                .resolve_staging_by_location(&request.location, &context)
                .await?;
            // The creator-match: the backend decides, in its own identity terms,
            // whether this caller may adopt the reservation.
            self.authorize(
                DeltaAction::AdoptStaging {
                    reservation: &staging,
                },
                &context,
            )
            .await?;
            if staging.stage_committed {
                return Err(DeltaApiError::invalid_argument(format!(
                    "staging table at '{}' has already been committed",
                    request.location
                )));
            }
            contract::validate_table_id_property(&request.properties, &staging.table_id)?;
            self.finalize_staging(&staging.table_id, &context).await?;
            Some(staging.table_id)
        } else {
            // EXTERNAL: the location must live inside a registered external location.
            self.validate_external_location(&request.location, &context)
                .await?;
            None
        };

        let full_name = format!("{}.{}.{}", path.catalog, path.schema, request.name);
        let stored = self
            .create_table_row(
                CreateTableSpec {
                    at: path,
                    name: request.name,
                    table_type: request.table_type,
                    location: request.location,
                    comment: request.comment,
                    columns,
                    properties: stored_properties,
                    table_id,
                },
                &context,
            )
            .await?;

        build_load_table_response(self, &full_name, stored).await
    }

    async fn load_table(&self, path: TableRef, context: Cx) -> Result<DeltaLoadTableResponse> {
        self.authorize(DeltaAction::ReadTable { table: &path }, &context)
            .await?;
        let table = self.resolve_table(&path, &context).await?;
        build_load_table_response(self, &path.full_name(), table).await
    }

    async fn update_table(
        &self,
        path: TableRef,
        request: DeltaUpdateTableRequest,
        context: Cx,
    ) -> Result<DeltaLoadTableResponse> {
        update_table_impl(self, path, request, context).await
    }

    async fn delete_table(&self, path: TableRef, context: Cx) -> Result<()> {
        self.authorize(DeltaAction::DeleteTable { table: &path }, &context)
            .await?;
        self.delete_table(&path, &context).await?;
        Ok(())
    }

    async fn table_exists(&self, path: TableRef, context: Cx) -> Result<()> {
        self.authorize(DeltaAction::ReadTable { table: &path }, &context)
            .await?;
        self.resolve_table(&path, &context).await?;
        Ok(())
    }

    async fn rename_table(
        &self,
        path: TableRef,
        request: DeltaRenameTableRequest,
        context: Cx,
    ) -> Result<()> {
        self.authorize(
            DeltaAction::RenameTable {
                from: &path,
                to: &request.new_name,
            },
            &context,
        )
        .await?;
        self.rename_table(&path, &request.new_name, &context)
            .await?;
        Ok(())
    }

    async fn get_table_credentials(
        &self,
        path: TableRef,
        operation: DeltaCredentialOperation,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse> {
        let table = self.resolve_table(&path, &context).await?;
        let table_id = table
            .table_id
            .ok_or_else(|| DeltaApiError::invalid_argument("table has no id"))?;
        let access = to_access(operation);
        self.authorize(
            DeltaAction::VendTableCredential {
                table_id: &table_id,
                access,
            },
            &context,
        )
        .await?;
        let creds = self
            .vend_table_credential(&table_id, access, &context)
            .await?;
        Ok(DeltaCredentialsResponse {
            storage_credentials: vec![to_storage_credential(&creds.url, &creds, operation)],
        })
    }

    async fn report_metrics(
        &self,
        path: TableRef,
        request: DeltaReportMetricsRequest,
        context: Cx,
    ) -> Result<()> {
        let table = self.resolve_table(&path, &context).await?;
        let table_id = table
            .table_id
            .clone()
            .ok_or_else(|| DeltaApiError::invalid_argument("table has no id"))?;
        self.authorize(
            DeltaAction::WriteTable {
                table: &path,
                table_id: &table_id,
            },
            &context,
        )
        .await?;
        if table.table_id.as_deref() != Some(request.table_id.as_str()) {
            return Err(DeltaApiError::invalid_argument(
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
            return Err(DeltaApiError::invalid_argument(
                "commit-version must be non-negative",
            ));
        }
        // Accept-and-ack: no maintenance scheduler yet.
        Ok(())
    }

    async fn get_staging_table_credentials(
        &self,
        table_id: String,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse> {
        let staging = self.resolve_staging_by_id(&table_id, &context).await?;
        self.authorize(
            DeltaAction::VendPathCredential {
                location: &staging.location,
                access: CredentialAccess::ReadWrite,
            },
            &context,
        )
        .await?;
        let creds = self
            .vend_path_credential(&staging.location, CredentialAccess::ReadWrite, &context)
            .await?;
        Ok(DeltaCredentialsResponse {
            storage_credentials: vec![to_storage_credential(
                &staging.location,
                &creds,
                DeltaCredentialOperation::ReadWrite,
            )],
        })
    }

    async fn get_temporary_path_credentials(
        &self,
        location: String,
        operation: DeltaCredentialOperation,
        context: Cx,
    ) -> Result<DeltaCredentialsResponse> {
        let access = to_access(operation);
        self.authorize(
            DeltaAction::VendPathCredential {
                location: &location,
                access,
            },
            &context,
        )
        .await?;
        let creds = self
            .vend_path_credential(&location, access, &context)
            .await?;
        Ok(DeltaCredentialsResponse {
            storage_credentials: vec![to_storage_credential(&creds.url, &creds, operation)],
        })
    }
}

// ===================================================================
// updateTable action dispatcher (DeltaUpdateTableMapper)
// ===================================================================

/// Apply an `updateTable` request: check requirements, apply the action list in
/// the reference's canonical order, route commit/backfill through the coordinator,
/// persist metadata, and return the refreshed table.
async fn update_table_impl<B, Cx>(
    backend: &B,
    path: TableRef,
    request: DeltaUpdateTableRequest,
    context: Cx,
) -> Result<DeltaLoadTableResponse>
where
    B: DeltaBackend<Cx> + ?Sized,
    Cx: Send + 'static,
{
    let mut table = backend.resolve_table(&path, &context).await?;
    let table_uuid = table
        .table_id
        .clone()
        .ok_or_else(|| DeltaApiError::invalid_argument("table has no id"))?;
    backend
        .authorize(
            DeltaAction::WriteTable {
                table: &path,
                table_id: &table_uuid,
            },
            &context,
        )
        .await?;

    // --- Requirements (assert-table-uuid mandatory; assert-etag optional) ---
    let has_uuid_assert = request
        .requirements
        .iter()
        .any(|r| matches!(r, DeltaTableRequirement::AssertTableUuid { .. }));
    if !has_uuid_assert {
        return Err(DeltaApiError::invalid_argument(
            "assert-table-uuid requirement is required.",
        ));
    }
    let current_etag = etag_of(&table);
    for req in &request.requirements {
        match req {
            DeltaTableRequirement::AssertTableUuid { uuid } => {
                if uuid != &table_uuid {
                    return Err(DeltaApiError(DeltaBackendError::UpdateRequirementConflict(
                        format!(
                            "assert-table-uuid failed: expected {uuid} but table has {table_uuid}"
                        ),
                    )));
                }
            }
            DeltaTableRequirement::AssertEtag { etag } => {
                if etag != &current_etag {
                    return Err(DeltaApiError(DeltaBackendError::UpdateRequirementConflict(
                        "assert-etag failed: table has been modified".to_string(),
                    )));
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
                return Err(DeltaApiError::invalid_argument(format!(
                    "set-properties and remove-properties overlap on key: {r}"
                )));
            }
        }
    }

    // Work on the fields directly (taken, not cloned); the untouched `table` is
    // reassembled below when no metadata change needs persisting.
    let mut properties: BTreeMap<String, String> = std::mem::take(&mut table.properties);
    let mut columns: Vec<Column> = std::mem::take(&mut table.columns);
    let mut comment: Option<String> = None;
    let is_managed = table.table_type == Some(DeltaTableType::Managed);
    let mut metadata_changed = false;

    // Apply in canonical order (not request order).
    // 1. set-columns / set-partition-columns
    if apply_schema_and_partitions(&mut columns, &request.updates)? {
        metadata_changed = true;
    }

    // 2. set-protocol (re-derive delta.feature.* and re-validate for MANAGED)
    if let Some(protocol) = request.updates.iter().find_map(|u| match u {
        DeltaTableUpdate::SetProtocol { protocol } => Some(protocol),
        _ => None,
    }) {
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
                            return Err(DeltaApiError::invalid_argument(format!(
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
    if let Some(c) = request.updates.iter().find_map(|u| match u {
        DeltaTableUpdate::SetTableComment { comment } => Some(comment.clone()),
        _ => None,
    }) {
        comment = Some(c);
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
            return Err(DeltaApiError::invalid_argument(
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
            return Err(DeltaApiError::invalid_argument(
                "add-commit / set-latest-backfilled-version require a MANAGED table",
            ));
        }
        let commit_info = add_commit.map(|c| CommitInfo {
            version: c.version,
            timestamp: c.timestamp,
            file_name: c.file_name,
            file_size: c.file_size,
            file_modification_timestamp: c.file_modification_timestamp,
        });
        backend
            .commit_coordinator()
            .commit(&table_uuid, commit_info, backfill)
            .await
            .map_err(commit_error)?;
    }

    // Persist metadata changes and reload; the pure add-commit path never
    // touches the table row, so the in-hand table is still current there.
    let refreshed = if metadata_changed {
        backend
            .update_table_row(
                UpdateTableSpec {
                    table_id: table_uuid,
                    columns,
                    properties,
                    comment,
                },
                &context,
            )
            .await?;
        backend.resolve_table(&path, &context).await?
    } else {
        table.properties = properties;
        table.columns = columns;
        table
    };
    build_load_table_response(backend, &path.full_name(), refreshed).await
}

/// Apply `set-columns` / `set-partition-columns` to `columns` in place, returning
/// whether any change was made. Mirrors the reference `applySchemaAndPartitionColumns`.
fn apply_schema_and_partitions(
    columns: &mut Vec<Column>,
    updates: &[DeltaTableUpdate],
) -> Result<bool> {
    let new_columns = updates.iter().find_map(|u| match u {
        DeltaTableUpdate::SetColumns { columns } => Some(columns),
        _ => None,
    });
    let new_partitions = updates.iter().find_map(|u| match u {
        DeltaTableUpdate::SetPartitionColumns { partition_columns } => Some(partition_columns),
        _ => None,
    });
    if new_columns.is_none() && new_partitions.is_none() {
        return Ok(false);
    }

    let result: Vec<Column> = match new_columns {
        Some(struct_type) => {
            // Preserve existing partitioning unless new partitions are supplied.
            let existing_partitions = partition_names(columns);
            let partitions = new_partitions.cloned().unwrap_or(existing_partitions);
            contract::delta_columns_to_uc(struct_type, Some(&partitions))?
        }
        None => {
            let partitions = new_partitions.expect("checked above");
            let mut cols = columns.clone();
            for col in &mut cols {
                col.partition_index = partitions
                    .iter()
                    .position(|p| p.eq_ignore_ascii_case(&col.name))
                    .map(|i| i as i32);
            }
            for p in partitions {
                if !cols.iter().any(|c| c.name.eq_ignore_ascii_case(p)) {
                    return Err(DeltaApiError::invalid_argument(format!(
                        "partition column '{p}' is not present in the table schema"
                    )));
                }
            }
            cols
        }
    };
    *columns = result;
    Ok(true)
}

// ===================================================================
// loadTable response construction
// ===================================================================

/// Build a `DeltaLoadTableResponse` from a resolved table, appending unbackfilled
/// commits + `latest_table_version` for catalog-managed Delta tables.
///
/// Rejects table types the Delta API cannot serve (views, metric views, …) with
/// 400; `full_name` is only used for that error message.
async fn build_load_table_response<B, Cx>(
    backend: &B,
    full_name: &str,
    table: ResolvedTable,
) -> Result<DeltaLoadTableResponse>
where
    B: DeltaBackend<Cx> + ?Sized,
{
    let Some(table_type) = table.table_type else {
        return Err(DeltaApiError::invalid_argument(format!(
            "table '{full_name}' is not a Delta table and cannot be loaded via the Delta API"
        )));
    };

    // Commit-coordinator state only exists for catalog-managed *Delta* tables;
    // a MANAGED table of another format gets no commits/version fields.
    let (commits, latest_table_version) = if table_type == DeltaTableType::Managed
        && table.data_source_format == Some(DeltaDataSourceFormat::Delta)
        && let Some(id) = table.table_id.as_deref()
    {
        let (commits, latest) = backend
            .commit_coordinator()
            .get_commits(id, 0, None)
            .await
            .map_err(commit_error)?;
        (
            Some(commits.into_iter().map(to_delta_commit).collect()),
            Some(latest),
        )
    } else {
        (None, None)
    };

    Ok(DeltaLoadTableResponse {
        metadata: build_table_metadata(table, table_type),
        commits,
        uniform: None,
        latest_table_version,
    })
}

fn build_table_metadata(table: ResolvedTable, table_type: DeltaTableType) -> DeltaTableMetadata {
    let etag = etag_of(&table);
    let partition_columns = partition_names(&table.columns);
    let columns = contract::uc_columns_to_delta(&table.columns);
    let last_commit_version = table
        .properties
        .get(contract::PROP_LAST_UPDATE_VERSION)
        .and_then(|v| v.parse().ok());
    let last_commit_timestamp_ms = table
        .properties
        .get(contract::PROP_LAST_COMMIT_TIMESTAMP)
        .and_then(|v| v.parse().ok());

    DeltaTableMetadata {
        etag,
        table_type,
        table_uuid: table.table_id.unwrap_or_default(),
        location: table.location,
        created_time: table.created_at_ms.unwrap_or_default(),
        updated_time: table
            .updated_at_ms
            .or(table.created_at_ms)
            .unwrap_or_default(),
        columns,
        partition_columns: (!partition_columns.is_empty()).then_some(partition_columns),
        properties: table.properties,
        last_commit_version,
        last_commit_timestamp_ms,
    }
}

// ===================================================================
// Helpers
// ===================================================================

fn feature_vec(features: &[&str]) -> Vec<String> {
    features.iter().map(|s| s.to_string()).collect()
}

fn to_access(op: DeltaCredentialOperation) -> CredentialAccess {
    match op {
        DeltaCredentialOperation::Read => CredentialAccess::Read,
        DeltaCredentialOperation::ReadWrite => CredentialAccess::ReadWrite,
    }
}

/// The etag for a resolved table: `etag-<updated_ms>`, else `etag-<uuid>`.
fn etag_of(table: &ResolvedTable) -> String {
    match table.updated_at_ms {
        Some(ts) => format!("etag-{ts}"),
        None => format!("etag-{}", table.table_id.clone().unwrap_or_default()),
    }
}

/// The partition-column names of a column set, ordered by partition index.
fn partition_names(columns: &[Column]) -> Vec<String> {
    let mut p: Vec<(&i32, &Column)> = columns
        .iter()
        .filter_map(|c| c.partition_index.as_ref().map(|idx| (idx, c)))
        .collect();
    p.sort_by_key(|(idx, _)| **idx);
    p.into_iter().map(|(_, c)| c.name.clone()).collect()
}

fn to_delta_commit(c: CommitInfo) -> DeltaCommit {
    DeltaCommit {
        version: c.version,
        timestamp: c.timestamp,
        file_name: c.file_name,
        file_size: c.file_size,
        file_modification_timestamp: c.file_modification_timestamp,
    }
}

/// Map a coordinator [`CommitError`](crate::coordinator::CommitError) into a
/// [`DeltaApiError`].
fn commit_error(err: crate::coordinator::CommitError) -> DeltaApiError {
    use crate::coordinator::CommitError;
    DeltaApiError(match err {
        CommitError::VersionConflict(m) => DeltaBackendError::CommitVersionConflict(m),
        CommitError::InvalidArgument(m) => DeltaBackendError::InvalidArgument(m),
        CommitError::ResourceExhausted(m) => DeltaBackendError::ResourceExhausted(m),
        CommitError::Backend(m) => DeltaBackendError::Internal(m),
    })
}

/// Map a vended [`VendedCredential`] onto the wire `DeltaStorageCredential`.
fn to_storage_credential(
    prefix: &str,
    creds: &VendedCredential,
    operation: DeltaCredentialOperation,
) -> DeltaStorageCredential {
    let mut config = DeltaStorageCredentialConfig::default();
    match &creds.kind {
        VendedCredentialKind::S3 {
            access_key_id,
            secret_access_key,
            session_token,
        } => {
            config.s3_access_key_id = Some(access_key_id.clone());
            config.s3_secret_access_key = Some(secret_access_key.clone());
            config.s3_session_token = session_token.clone();
        }
        VendedCredentialKind::AzureSas { sas_token } => {
            config.azure_sas_token = Some(sas_token.clone());
        }
        VendedCredentialKind::GcsOauth { oauth_token } => {
            config.gcs_oauth_token = Some(oauth_token.clone());
        }
        VendedCredentialKind::None => {}
    }
    DeltaStorageCredential {
        prefix: prefix.to_string(),
        operation,
        config,
        expiration_time_ms: creds.expiration_time_ms,
    }
}
