//! The `DeltaApiHandler` trait and its generic implementation over a
//! [`DeltaBackend`].
//!
//! One trait method per `delta.yaml` operation. The blanket
//! `impl<B: DeltaBackend<Cx>, Cx> DeltaApiHandler<Cx> for B` contains all the
//! Delta business logic — the managed-table contract, the `updateTable` action
//! dispatcher (applied in the reference's canonical order), `loadTable` commit-list
//! construction, and the credential→config mapping — expressed purely in terms of
//! the [`DeltaBackend`] port. A server that implements the port gets the handler,
//! and thus the router, without writing any of that logic itself.

use std::collections::BTreeMap;

use async_trait::async_trait;

use crate::authz::DeltaAction;
use crate::backend::{
    CreateTableSpec, CredentialAccess, DeltaBackend, ResolvedTable, SchemaRef, TableRef,
    TableUpdate, UpdateTableSpec, VendedCredential, etag_of,
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

#[async_trait]
impl<B, Cx> DeltaApiHandler<Cx> for B
where
    B: DeltaBackend<Cx>,
    Cx: Send + Sync + 'static,
{
    async fn get_config(&self, query: GetConfigQuery, context: Cx) -> Result<DeltaCatalogConfig> {
        if query.catalog.is_empty() {
            return Err(DeltaApiError::invalid_argument(
                "catalog query parameter is required",
            ));
        }
        // Negotiate before the catalog lookup so an unsupported client fails fast
        // with 400 without incurring a catalog resolve.
        let protocol_version = crate::config::negotiate_version(&query.protocol_versions)?;
        self.authorize(
            DeltaAction::ReadCatalog {
                catalog: &query.catalog,
            },
            &context,
        )
        .await?;
        self.catalog_exists(&query.catalog, &context)
            .await
            .map_err(|e| match e {
                // The only thing getConfig looks up is the catalog, so any
                // not-found is the catalog's.
                DeltaBackendError::NotFound(m) | DeltaBackendError::NotFoundGeneric(m) => {
                    DeltaBackendError::NoSuchCatalog(m)
                }
                other => other,
            })?;
        Ok(DeltaCatalogConfig {
            endpoints: crate::config::endpoints_for(self.capabilities()),
            protocol_version,
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
            storage_credentials: vec![to_storage_credential(&staging.location, &creds)?],
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
        if !matches!(
            request.table_type,
            DeltaTableType::Managed | DeltaTableType::External
        ) {
            return Err(DeltaApiError::invalid_argument(
                "table-type must be MANAGED or EXTERNAL.",
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

        // For MANAGED, resolve + authorize the staging reservation. It is not
        // consumed here: the reservation is handed to `create_table_row`, which
        // consumes it and creates the table atomically (see `CreateTableSpec`).
        let (table_id, adopt_staging) = if request.table_type == DeltaTableType::Managed {
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
            (Some(staging.table_id.clone()), Some(staging))
        } else {
            // EXTERNAL: the location must live inside a registered external location.
            self.validate_external_location(&request.location, &context)
                .await?;
            (None, None)
        };

        // A managed table lives where the server reserved it. The client may spell
        // that location differently (a trailing slash, say), and the reservation
        // lookup tolerates that, but the stored location is the reservation's own,
        // so readers that join paths onto it (`<location>/_delta_log/...`) get the
        // canonical spelling back from loadTable.
        let location = match &adopt_staging {
            Some(staging) => staging.location.clone(),
            None => request.location,
        };
        let full_name = format!("{}.{}.{}", path.catalog, path.schema, request.name);
        let stored = self
            .create_table_row(
                CreateTableSpec {
                    at: path,
                    name: request.name,
                    table_type: request.table_type,
                    location,
                    comment: request.comment,
                    columns,
                    properties: stored_properties,
                    table_id,
                    adopt_staging,
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
        let access = to_access(operation);
        self.authorize(
            DeltaAction::VendTableCredential {
                table: &path,
                access,
            },
            &context,
        )
        .await?;
        let table = self.resolve_table(&path, &context).await?;
        let table_id = table_id_of(&table)?;
        let creds = self
            .vend_table_credential(&table_id, access, &context)
            .await?;
        Ok(DeltaCredentialsResponse {
            storage_credentials: vec![to_storage_credential(&creds.url, &creds)?],
        })
    }

    async fn report_metrics(
        &self,
        path: TableRef,
        request: DeltaReportMetricsRequest,
        context: Cx,
    ) -> Result<()> {
        // Metrics reports feed table maintenance (compaction/checkpointing) and are
        // an assertion by a writer about commits it has made, so we gate them on
        // WRITE. This is deliberately stricter than the pre-crate behavior, which
        // required no permission on this endpoint at all.
        self.authorize(DeltaAction::WriteTable { table: &path }, &context)
            .await?;
        let table = self.resolve_table(&path, &context).await?;
        let table_id = table_id_of(&table)?;
        require_delta(&table, &path.full_name())?;
        if table_id != request.table_id {
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
        // Staging credentials are READ_WRITE on the table's future location. They
        // are authorized by the reservation itself: they belong to its creator
        // (who was authorized to create it, `CreateStaging`) and only until the
        // table is created, after which the table's own credential endpoint (with
        // its table permissions) applies. They are deliberately *not* a
        // `VendPathCredential`: a managed location need not lie in any external
        // location the caller holds grants on.
        self.authorize(
            DeltaAction::AdoptStaging {
                reservation: &staging,
            },
            &context,
        )
        .await?;
        if staging.stage_committed {
            return Err(DeltaApiError::invalid_argument(format!(
                "staging table '{table_id}' has already been committed; use the table's \
                 credentials endpoint"
            )));
        }
        let creds = self
            .vend_path_credential(&staging.location, CredentialAccess::ReadWrite, &context)
            .await?;
        Ok(DeltaCredentialsResponse {
            storage_credentials: vec![to_storage_credential(&staging.location, &creds)?],
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
            storage_credentials: vec![to_storage_credential(&creds.url, &creds)?],
        })
    }
}

// ===================================================================
// updateTable action dispatcher (DeltaUpdateTableMapper)
// ===================================================================

/// The requirements of an `updateTable` request, at most one of each type.
struct CollectedRequirements<'a> {
    table_uuid: &'a str,
    etag: Option<&'a str>,
}

/// Shape-check the requirements: at most one of each type, and `assert-table-uuid`
/// is mandatory (without it a client holding a cached three-part name could
/// silently commit to a recreated table).
fn collect_requirements(
    requirements: &[DeltaTableRequirement],
) -> Result<CollectedRequirements<'_>> {
    let mut table_uuid = None;
    let mut etag = None;
    for req in requirements {
        match req {
            DeltaTableRequirement::AssertTableUuid { uuid } => put_once(
                &mut table_uuid,
                uuid.as_str(),
                "assert-table-uuid requirement",
            )?,
            DeltaTableRequirement::AssertEtag { etag: e } => {
                put_once(&mut etag, e.as_str(), "assert-etag requirement")?
            }
        }
    }
    let table_uuid = table_uuid.ok_or_else(|| {
        DeltaApiError::invalid_argument("assert-table-uuid requirement is required.")
    })?;
    Ok(CollectedRequirements { table_uuid, etag })
}

/// The update actions of an `updateTable` request, at most one of each type.
///
/// The spec makes request order meaningless (actions apply in a fixed canonical
/// order) and allows one action per type, so collecting into named slots is
/// lossless.
#[derive(Default)]
struct CollectedUpdates<'a> {
    set_properties: Option<&'a BTreeMap<String, String>>,
    remove_properties: Option<&'a [String]>,
    set_columns: Option<&'a DeltaStructType>,
    set_table_comment: Option<&'a str>,
    add_commit: Option<&'a DeltaCommit>,
    set_latest_backfilled_version: Option<i64>,
    set_protocol: Option<&'a DeltaProtocol>,
    set_domain_metadata: Option<&'a DeltaDomainMetadataUpdates>,
    remove_domain_metadata: Option<&'a [String]>,
    set_partition_columns: Option<&'a [String]>,
    update_snapshot_version: Option<(i64, i64)>,
}

/// Shape-check the update actions (`ManagedDeltaTablesSpec` "Request shape
/// rules"): at least one action, at most one per type, and no set/remove overlap on
/// the same property key or domain. Violations are 400s.
fn collect_updates(updates: &[DeltaTableUpdate]) -> Result<CollectedUpdates<'_>> {
    if updates.is_empty() {
        return Err(DeltaApiError::invalid_argument(
            "At least one update is required.",
        ));
    }
    let mut c = CollectedUpdates::default();
    for update in updates {
        match update {
            DeltaTableUpdate::SetProperties { updates } => {
                put_once(&mut c.set_properties, updates, "set-properties action")?
            }
            DeltaTableUpdate::RemoveProperties { removals } => put_once(
                &mut c.remove_properties,
                removals,
                "remove-properties action",
            )?,
            DeltaTableUpdate::SetColumns { columns } => {
                put_once(&mut c.set_columns, columns, "set-columns action")?
            }
            DeltaTableUpdate::SetTableComment { comment } => put_once(
                &mut c.set_table_comment,
                comment,
                "set-table-comment action",
            )?,
            DeltaTableUpdate::AddCommit { commit, .. } => {
                put_once(&mut c.add_commit, commit, "add-commit action")?
            }
            DeltaTableUpdate::SetLatestBackfilledVersion {
                latest_published_version,
            } => put_once(
                &mut c.set_latest_backfilled_version,
                *latest_published_version,
                "set-latest-backfilled-version action",
            )?,
            DeltaTableUpdate::SetProtocol { protocol } => {
                put_once(&mut c.set_protocol, protocol, "set-protocol action")?
            }
            DeltaTableUpdate::SetDomainMetadata { updates } => put_once(
                &mut c.set_domain_metadata,
                updates,
                "set-domain-metadata action",
            )?,
            DeltaTableUpdate::RemoveDomainMetadata { domains } => put_once(
                &mut c.remove_domain_metadata,
                domains,
                "remove-domain-metadata action",
            )?,
            DeltaTableUpdate::SetPartitionColumns { partition_columns } => put_once(
                &mut c.set_partition_columns,
                partition_columns,
                "set-partition-columns action",
            )?,
            DeltaTableUpdate::UpdateMetadataSnapshotVersion {
                last_commit_version,
                last_commit_timestamp_ms,
            } => put_once(
                &mut c.update_snapshot_version,
                (*last_commit_version, *last_commit_timestamp_ms),
                "update-metadata-snapshot-version action",
            )?,
        }
    }

    if let (Some(set), Some(removals)) = (c.set_properties, c.remove_properties) {
        let overlap: Vec<&str> = removals
            .iter()
            .filter(|k| set.contains_key(*k))
            .map(String::as_str)
            .collect();
        if !overlap.is_empty() {
            return Err(DeltaApiError::invalid_argument(format!(
                "set-properties and remove-properties overlap on keys: {overlap:?}"
            )));
        }
    }
    if let (Some(set), Some(removals)) = (c.set_domain_metadata, c.remove_domain_metadata) {
        let set_domains = contract::domains_set_in(set);
        let overlap: Vec<&str> = removals
            .iter()
            .map(String::as_str)
            .filter(|d| set_domains.contains(d))
            .collect();
        if !overlap.is_empty() {
            return Err(DeltaApiError::invalid_argument(format!(
                "set-domain-metadata and remove-domain-metadata overlap on domains: {overlap:?}"
            )));
        }
    }
    Ok(c)
}

/// Fill a once-only slot, rejecting a second occurrence with a 400.
fn put_once<T>(slot: &mut Option<T>, value: T, what: &str) -> Result<()> {
    if slot.replace(value).is_some() {
        return Err(DeltaApiError::invalid_argument(format!(
            "a request may carry at most one {what}"
        )));
    }
    Ok(())
}

/// Apply an `updateTable` request: shape-check it, check requirements, apply the
/// actions in the reference's canonical order, re-validate the managed contract
/// against the post-apply state, route commit/backfill through the coordinator,
/// persist metadata, and return the refreshed table.
async fn update_table_impl<B, Cx>(
    backend: &B,
    path: TableRef,
    request: DeltaUpdateTableRequest,
    context: Cx,
) -> Result<DeltaLoadTableResponse>
where
    B: DeltaBackend<Cx> + ?Sized,
    Cx: Send + Sync + 'static,
{
    // Pure shape checks first: a malformed request is a 400 regardless of the
    // table's state.
    let requirements = collect_requirements(&request.requirements)?;
    let updates = collect_updates(&request.updates)?;

    backend
        .authorize(DeltaAction::WriteTable { table: &path }, &context)
        .await?;
    let table = backend.resolve_table(&path, &context).await?;
    let table_uuid = table_id_of(&table)?;
    require_delta(&table, &path.full_name())?;

    // --- Requirements ---
    //
    // `assert-table-uuid` is validated against the resolved uuid (identity never
    // changes under an update, so no race). `assert-etag` is fast-failed here
    // against the resolved snapshot so a stale etag rejects the whole request
    // before the add-commit coordinator call or any write lands. The asserted etag
    // is *also* threaded into `UpdateTableSpec::expected_etag`, where the backend
    // re-checks it as a compare-and-swap at write time.
    if requirements.table_uuid != table_uuid {
        return Err(DeltaApiError(DeltaBackendError::UpdateRequirementConflict(
            format!(
                "assert-table-uuid failed: expected {} but table has {table_uuid}",
                requirements.table_uuid
            ),
        )));
    }
    // A retried `add-commit` that already landed is answered with the current
    // table state before anything else is checked: the original attempt moved
    // the etag, so the retry's `assert-etag` no longer matches, and the spec
    // requires a recognized retry to return 200 (never a 409 that would make the
    // client rebase and duplicate its write).
    if let Some(commit) = updates.add_commit
        && backend
            .commit_coordinator()
            .find_commit(&table_uuid, commit.version)
            .await?
            .is_some_and(|stored| stored.file_name == commit.file_name)
    {
        return build_load_table_response(backend, &path.full_name(), table).await;
    }

    if let Some(etag) = requirements.etag
        && etag != etag_of(&table)
    {
        return Err(DeltaApiError(DeltaBackendError::UpdateRequirementConflict(
            "assert-etag failed: table has been modified".to_string(),
        )));
    }
    let expected_etag = requirements.etag.map(str::to_string);

    // Apply onto copies: `table` stays the pre-update snapshot (`TableUpdate::base`).
    let mut properties: BTreeMap<String, String> = table.properties.clone();
    let mut columns: Vec<Column> = table.columns.clone();
    let mut comment: Option<String> = None;
    let is_managed = table.table_type == Some(DeltaTableType::Managed);
    let mut metadata_changed = false;

    // Apply in canonical order (not request order).
    // 1. set-columns / set-partition-columns
    if apply_schema_and_partitions(
        &mut columns,
        updates.set_columns,
        updates.set_partition_columns,
    )? {
        metadata_changed = true;
    }

    // 2. set-protocol (a full replacement: re-derive delta.feature.*)
    if let Some(protocol) = updates.set_protocol {
        properties.retain(|k, _| !k.starts_with("delta.feature."));
        contract::derive_from_protocol(&mut properties, protocol);
        metadata_changed = true;
    }

    // 3. set-properties / 4. remove-properties
    if let Some(set) = updates.set_properties {
        if is_managed {
            contract::check_managed_property_changes(set.keys(), &table_uuid, |k| {
                set.get(k).map(String::as_str)
            })?;
        }
        properties.extend(set.clone());
        metadata_changed = true;
    }
    if let Some(removals) = updates.remove_properties {
        if is_managed {
            contract::check_managed_property_changes(removals.iter(), &table_uuid, |_| None)?;
        }
        for k in removals {
            properties.remove(k);
        }
        metadata_changed = true;
    }

    // 5. set-domain-metadata / 6. remove-domain-metadata
    if let Some(dm) = updates.set_domain_metadata {
        contract::derive_from_domain_metadata(&mut properties, dm);
        metadata_changed = true;
    }
    if let Some(domains) = updates.remove_domain_metadata {
        for d in domains {
            contract::remove_domain_metadata(&mut properties, d)?;
        }
        metadata_changed = true;
    }

    // 7. set-table-comment
    if let Some(c) = updates.set_table_comment {
        comment = Some(c.to_string());
        metadata_changed = true;
    }

    // 8. update-metadata-snapshot-version (EXTERNAL only)
    if let Some((v, ts)) = updates.update_snapshot_version {
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

    // Re-validate the MANAGED contract against the final post-apply state.
    // set-protocol runs full validation; set-domain-metadata alone only needs the
    // domain-metadata-vs-writer-features check.
    if is_managed {
        if let Some(protocol) = updates.set_protocol {
            contract::validate(protocol, None, &properties)?;
            contract::validate_domain_metadata_against_properties(&properties)?;
        } else if updates.set_domain_metadata.is_some() {
            contract::validate_domain_metadata_against_properties(&properties)?;
        }
    }

    // 9. add-commit + set-latest-backfilled-version → commit coordinator
    let latest_backfilled_version = updates.set_latest_backfilled_version;
    if (updates.add_commit.is_some() || latest_backfilled_version.is_some()) && !is_managed {
        return Err(DeltaApiError::invalid_argument(
            "add-commit / set-latest-backfilled-version require a MANAGED table",
        ));
    }
    let commit = updates.add_commit.map(|c| CommitInfo {
        version: c.version,
        timestamp: c.timestamp,
        file_name: c.file_name.clone(),
        file_size: c.file_size,
        file_modification_timestamp: c.file_modification_timestamp,
    });
    if commit.is_none() && latest_backfilled_version.is_none() && !metadata_changed {
        // Unreachable for a well-formed request (every action either changes
        // metadata or the commit log), kept as a guard.
        return build_load_table_response(backend, &path.full_name(), table).await;
    }

    // Hand the whole request to the backend to persist atomically: the metadata
    // write (with its assert-etag CAS) and the commit ratification succeed or
    // fail together (see `DeltaBackend::apply_update`).
    let metadata = metadata_changed.then(|| {
        let mut spec = UpdateTableSpec::new(&table_uuid, columns, properties);
        spec.comment = comment;
        spec.expected_etag = expected_etag;
        spec
    });
    let update = TableUpdate {
        base: table,
        metadata,
        commit,
        latest_backfilled_version,
    };
    let refreshed = match backend.apply_update(update, &context).await {
        Ok(t) => t,
        // A concurrent retry of the same commit won the race after our
        // pre-check: the work is done, answer with the current table.
        Err(DeltaBackendError::CommitAlreadyAccepted) => {
            backend.resolve_table(&path, &context).await?
        }
        Err(e) => return Err(e.into()),
    };
    build_load_table_response(backend, &path.full_name(), refreshed).await
}

/// Apply `set-columns` / `set-partition-columns` to `columns` in place, returning
/// whether any change was made. Mirrors the reference `applySchemaAndPartitionColumns`.
fn apply_schema_and_partitions(
    columns: &mut Vec<Column>,
    new_columns: Option<&DeltaStructType>,
    new_partitions: Option<&[String]>,
) -> Result<bool> {
    if new_columns.is_none() && new_partitions.is_none() {
        return Ok(false);
    }

    let result: Vec<Column> = match new_columns {
        Some(struct_type) => {
            // Preserve existing partitioning unless new partitions are supplied.
            let existing_partitions = partition_names(columns);
            let partitions = new_partitions.unwrap_or(&existing_partitions);
            contract::delta_columns_to_uc(struct_type, Some(partitions))?
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

/// The resolved table's id. A table the backend hands out without an id is a
/// server-side data problem, not a client error, so this is a 500.
fn table_id_of(table: &ResolvedTable) -> Result<String> {
    table
        .table_id
        .clone()
        .ok_or_else(|| DeltaApiError::internal("table has no id"))
}

/// Reject tables the Delta API does not serve: non-table securables (views,
/// metric views, …; `table_type == None`) and tables whose data source format is
/// known and not Delta. Clients fall back to the UC REST API on
/// `UnsupportedTableFormatException`. A backend that does not report a format
/// (`None`) is trusted to serve only Delta tables.
fn require_delta(table: &ResolvedTable, full_name: &str) -> Result<DeltaTableType> {
    let Some(table_type) = table
        .table_type
        .filter(|t| matches!(t, DeltaTableType::Managed | DeltaTableType::External))
    else {
        return Err(DeltaApiError(DeltaBackendError::UnsupportedTableFormat(
            format!(
                "table '{full_name}' is not a Delta table and cannot be served via the Delta API"
            ),
        )));
    };
    if let Some(format) = table.data_source_format
        && format != DeltaDataSourceFormat::Delta
    {
        return Err(DeltaApiError(DeltaBackendError::UnsupportedTableFormat(
            format!("table '{full_name}' has data source format {format:?}, not DELTA"),
        )));
    }
    Ok(table_type)
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
    Cx: Send + Sync,
{
    let table_type = require_delta(&table, full_name)?;

    // Commit-coordinator state only exists for catalog-managed tables.
    let (commits, latest_table_version) = if table_type == DeltaTableType::Managed
        && let Some(id) = table.table_id.as_deref()
    {
        let (commits, latest) = backend
            .commit_coordinator()
            .get_commits(id, 0, None)
            .await
            .map_err(DeltaApiError::from)?;
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
    op.into()
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

/// Map a vended [`VendedCredential`] onto the wire `DeltaStorageCredential`.
///
/// The `operation` is the access the credential was actually granted. The
/// `prefix` is overridden with `prefix`, the location as the catalog spells it
/// (no trailing slash), which is what the UC spec examples and the Java reference
/// (`DeltaCredentialsMapper`: `prefix = credentials.getUrl()`) return; the trestle
/// conversion would emit the scope's canonical directory form (`…/`).
fn to_storage_credential(prefix: &str, creds: &VendedCredential) -> Result<DeltaStorageCredential> {
    let mut wire = DeltaStorageCredential::try_from(&creds.credential)
        .map_err(|e| DeltaApiError(DeltaBackendError::backend(e)))?;
    wire.prefix = prefix.to_string();
    Ok(wire)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolved(
        table_type: Option<DeltaTableType>,
        format: Option<DeltaDataSourceFormat>,
    ) -> ResolvedTable {
        ResolvedTable {
            table_id: Some("id".into()),
            location: "s3://b/t".into(),
            table_type,
            data_source_format: format,
            columns: vec![],
            properties: BTreeMap::new(),
            comment: None,
            created_at_ms: None,
            updated_at_ms: None,
            version: 0,
        }
    }

    #[test]
    fn require_delta_accepts_delta_and_unreported_formats() {
        for format in [Some(DeltaDataSourceFormat::Delta), None] {
            let t = resolved(Some(DeltaTableType::Managed), format);
            assert_eq!(require_delta(&t, "c.s.t").unwrap(), DeltaTableType::Managed);
        }
    }

    #[test]
    fn require_delta_rejects_views_and_other_formats() {
        for t in [
            resolved(None, Some(DeltaDataSourceFormat::Delta)),
            resolved(
                Some(DeltaTableType::External),
                Some(DeltaDataSourceFormat::Parquet),
            ),
        ] {
            let err = require_delta(&t, "c.s.t").unwrap_err();
            assert!(
                matches!(err.0, DeltaBackendError::UnsupportedTableFormat(_)),
                "{err:?}"
            );
        }
    }

    #[test]
    fn missing_table_id_is_internal() {
        let mut t = resolved(Some(DeltaTableType::Managed), None);
        t.table_id = None;
        let err = table_id_of(&t).unwrap_err();
        assert!(matches!(err.0, DeltaBackendError::Internal(_)), "{err:?}");
    }
}
