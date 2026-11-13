//! updateTable atomicity, commit-retry and authorize-before-resolve semantics.
//!
//! A `Wrapped` backend delegates to [`InMemoryDeltaBackend`] and can (a) use the
//! port's default sequential `apply_update` instead of the in-memory atomic one,
//! (b) inject a concurrent metadata write right after the handler resolves its
//! snapshot, and (c) deny every authorization.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use async_trait::async_trait;
use rstest::rstest;
use unitycatalog_delta_api::DeltaApiHandler;
use unitycatalog_delta_api::authz::DeltaAction;
use unitycatalog_delta_api::backend::{
    BackendResult, CreateTableSpec, CredentialAccess, DeltaBackend, ResolvedTable, SchemaRef,
    StagingReservation, TableRef, TableUpdate, UpdateTableSpec, VendedCredential,
};
use unitycatalog_delta_api::contract;
use unitycatalog_delta_api::coordinator::CommitCoordinator;
use unitycatalog_delta_api::error::DeltaBackendError;
use unitycatalog_delta_api::models::*;
use unitycatalog_delta_api::testing::InMemoryDeltaBackend;

type Cx = ();

#[derive(Clone, Copy, Debug)]
enum Apply {
    /// The in-memory backend's atomic override.
    Atomic,
    /// The port's default `apply_update_sequential`.
    Sequential,
}

struct Wrapped {
    inner: InMemoryDeltaBackend,
    apply: Apply,
    /// When set, the next `resolve_table` lands a concurrent metadata write
    /// after taking the snapshot it returns (so the snapshot's etag is stale).
    race_next_resolve: AtomicBool,
    deny: bool,
    /// Every `resolve_table` call, to prove denial happens before resolution.
    resolves: Mutex<usize>,
}

impl Wrapped {
    fn new(apply: Apply) -> Self {
        Self {
            inner: InMemoryDeltaBackend::new(),
            apply,
            race_next_resolve: AtomicBool::new(false),
            deny: false,
            resolves: Mutex::new(0),
        }
    }

    fn denying() -> Self {
        Self {
            deny: true,
            ..Self::new(Apply::Atomic)
        }
    }
}

#[async_trait]
impl DeltaBackend<Cx> for Wrapped {
    async fn authorize(&self, action: DeltaAction<'_>, cx: &Cx) -> BackendResult<()> {
        if self.deny {
            return Err(DeltaBackendError::PermissionDenied("denied".into()));
        }
        self.inner.authorize(action, cx).await
    }
    async fn catalog_exists(&self, catalog: &str, cx: &Cx) -> BackendResult<()> {
        self.inner.catalog_exists(catalog, cx).await
    }
    async fn resolve_table(&self, table: &TableRef, cx: &Cx) -> BackendResult<ResolvedTable> {
        *self.resolves.lock().unwrap() += 1;
        let snapshot = self.inner.resolve_table(table, cx).await?;
        if self.race_next_resolve.swap(false, Ordering::SeqCst) {
            let mut properties = snapshot.properties.clone();
            properties.insert("concurrent.writer".into(), "1".into());
            let spec = UpdateTableSpec::new(
                snapshot.table_id.clone().unwrap(),
                snapshot.columns.clone(),
                properties,
            );
            DeltaBackend::<Cx>::update_table_row(&self.inner, spec, cx)
                .await
                .unwrap();
        }
        Ok(snapshot)
    }
    async fn validate_external_location(&self, location: &str, cx: &Cx) -> BackendResult<()> {
        self.inner.validate_external_location(location, cx).await
    }
    async fn create_table_row(
        &self,
        spec: CreateTableSpec,
        cx: &Cx,
    ) -> BackendResult<ResolvedTable> {
        self.inner.create_table_row(spec, cx).await
    }
    async fn apply_update(&self, update: TableUpdate, cx: &Cx) -> BackendResult<ResolvedTable> {
        match self.apply {
            Apply::Atomic => self.inner.apply_update(update, cx).await,
            Apply::Sequential => {
                unitycatalog_delta_api::backend::apply_update_sequential(self, update, cx).await
            }
        }
    }
    async fn update_table_row(
        &self,
        spec: UpdateTableSpec,
        cx: &Cx,
    ) -> BackendResult<ResolvedTable> {
        self.inner.update_table_row(spec, cx).await
    }
    async fn delete_table(&self, table: &TableRef, cx: &Cx) -> BackendResult<()> {
        DeltaBackend::<Cx>::delete_table(&self.inner, table, cx).await
    }
    async fn rename_table(&self, from: &TableRef, to_name: &str, cx: &Cx) -> BackendResult<()> {
        DeltaBackend::<Cx>::rename_table(&self.inner, from, to_name, cx).await
    }
    async fn allocate_staging(
        &self,
        at: &SchemaRef,
        name: &str,
        cx: &Cx,
    ) -> BackendResult<StagingReservation> {
        self.inner.allocate_staging(at, name, cx).await
    }
    async fn resolve_staging_by_location(
        &self,
        location: &str,
        cx: &Cx,
    ) -> BackendResult<StagingReservation> {
        self.inner.resolve_staging_by_location(location, cx).await
    }
    async fn resolve_staging_by_id(
        &self,
        table_id: &str,
        cx: &Cx,
    ) -> BackendResult<StagingReservation> {
        self.inner.resolve_staging_by_id(table_id, cx).await
    }
    async fn vend_table_credential(
        &self,
        table_id: &str,
        access: CredentialAccess,
        cx: &Cx,
    ) -> BackendResult<VendedCredential> {
        self.inner.vend_table_credential(table_id, access, cx).await
    }
    async fn vend_path_credential(
        &self,
        location: &str,
        access: CredentialAccess,
        cx: &Cx,
    ) -> BackendResult<VendedCredential> {
        self.inner.vend_path_credential(location, access, cx).await
    }
    fn commit_coordinator(&self) -> &dyn CommitCoordinator {
        DeltaBackend::<Cx>::commit_coordinator(&self.inner)
    }
}

fn schema_path() -> SchemaRef {
    SchemaRef {
        catalog: "catalog".into(),
        schema: "schema".into(),
    }
}

fn table_path(name: &str) -> TableRef {
    TableRef {
        catalog: "catalog".into(),
        schema: "schema".into(),
        table: name.into(),
    }
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

async fn create_managed(b: &Wrapped, name: &str) -> String {
    let staging = b
        .create_staging_table(
            schema_path(),
            DeltaCreateStagingTableRequest { name: name.into() },
            (),
        )
        .await
        .unwrap();
    let mut properties: BTreeMap<String, String> = contract::REQUIRED_FIXED_PROPERTIES
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    properties.insert(contract::PROP_UC_TABLE_ID.into(), staging.table_id.clone());
    b.create_table(
        schema_path(),
        DeltaCreateTableRequest {
            name: name.into(),
            location: staging.location,
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
            properties,
            domain_metadata: None,
            last_commit_timestamp_ms: 1700,
            uniform: None,
        },
        (),
    )
    .await
    .unwrap();
    staging.table_id
}

fn commit(version: i64, file_name: &str) -> DeltaTableUpdate {
    DeltaTableUpdate::AddCommit {
        commit: DeltaCommit {
            version,
            timestamp: 1_704_067_200_000 + version,
            file_name: file_name.into(),
            file_size: 2048,
            file_modification_timestamp: 1_704_067_200_000 + version,
        },
        uniform: None,
    }
}

fn set_prop(key: &str, value: &str) -> DeltaTableUpdate {
    DeltaTableUpdate::SetProperties {
        updates: BTreeMap::from([(key.to_string(), value.to_string())]),
    }
}

async fn load(b: &Wrapped, name: &str) -> DeltaLoadTableResponse {
    DeltaApiHandler::<Cx>::load_table(b, table_path(name), ())
        .await
        .unwrap()
}

/// The etag CAS loses to a concurrent writer *after* the handler's snapshot
/// check passed: the request is a 409 and the commit must not be ratified, under
/// both the atomic and the default sequential `apply_update`.
#[rstest]
#[case::atomic(Apply::Atomic)]
#[case::sequential(Apply::Sequential)]
#[tokio::test]
async fn lost_cas_leaves_no_ratified_commit(#[case] apply: Apply) {
    let b = Wrapped::new(apply);
    let table_id = create_managed(&b, "t").await;
    let etag = load(&b, "t").await.metadata.etag;

    b.race_next_resolve.store(true, Ordering::SeqCst);
    let err = b
        .update_table(
            table_path("t"),
            DeltaUpdateTableRequest {
                requirements: vec![
                    DeltaTableRequirement::AssertTableUuid {
                        uuid: table_id.clone(),
                    },
                    DeltaTableRequirement::AssertEtag { etag },
                ],
                updates: vec![commit(1, "v1.json"), set_prop("k", "v")],
            },
            (),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err.0, DeltaBackendError::UpdateRequirementConflict(_)),
        "{err:?}"
    );

    let loaded = load(&b, "t").await;
    assert_eq!(loaded.latest_table_version, Some(0), "no commit ratified");
    assert!(!loaded.metadata.properties.contains_key("k"));
}

/// The commit is rejected after the metadata write: nothing of the request may
/// persist. The atomic path never writes; the sequential path compensates.
#[rstest]
#[case::atomic(Apply::Atomic)]
#[case::sequential(Apply::Sequential)]
#[tokio::test]
async fn rejected_commit_rolls_back_metadata(#[case] apply: Apply) {
    let b = Wrapped::new(apply);
    let table_id = create_managed(&b, "t").await;
    let before = load(&b, "t").await;

    let err = b
        .update_table(
            table_path("t"),
            DeltaUpdateTableRequest {
                requirements: vec![DeltaTableRequirement::AssertTableUuid {
                    uuid: table_id.clone(),
                }],
                // Version 1 is skipped: the coordinator rejects the gap.
                updates: vec![commit(1, "v1.json")],
            },
            (),
        )
        .await
        .map(|_| ());
    assert!(err.is_ok(), "{err:?}");
    let err = b
        .update_table(
            table_path("t"),
            DeltaUpdateTableRequest {
                requirements: vec![DeltaTableRequirement::AssertTableUuid { uuid: table_id }],
                updates: vec![commit(3, "v3.json"), set_prop("k", "v")],
            },
            (),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err.0, DeltaBackendError::InvalidArgument(_)),
        "{err:?}"
    );

    let after = load(&b, "t").await;
    assert_eq!(after.latest_table_version, Some(1));
    assert!(
        !after.metadata.properties.contains_key("k"),
        "metadata write undone"
    );
    assert_eq!(after.metadata.properties, before.metadata.properties);
}

/// A byte-identical retry of a landed commit — whose assert-etag is now stale,
/// because the original attempt moved the etag — returns 200 with the current
/// table and applies nothing a second time.
#[rstest]
#[case::atomic(Apply::Atomic)]
#[case::sequential(Apply::Sequential)]
#[tokio::test]
async fn identical_retry_of_landed_commit_succeeds(#[case] apply: Apply) {
    let b = Wrapped::new(apply);
    let table_id = create_managed(&b, "t").await;
    let etag = load(&b, "t").await.metadata.etag;
    let request = DeltaUpdateTableRequest {
        requirements: vec![
            DeltaTableRequirement::AssertTableUuid { uuid: table_id },
            DeltaTableRequirement::AssertEtag { etag },
        ],
        updates: vec![commit(1, "v1.json"), set_prop("k", "v")],
    };
    let first = b
        .update_table(table_path("t"), request.clone(), ())
        .await
        .unwrap();
    let retry = b.update_table(table_path("t"), request, ()).await.unwrap();
    assert_eq!(
        retry.metadata.etag, first.metadata.etag,
        "nothing re-applied"
    );
    assert_eq!(retry.latest_table_version, Some(1));
    assert_eq!(
        retry.metadata.properties.get("k").map(String::as_str),
        Some("v")
    );
}

#[tokio::test]
async fn different_commit_at_landed_version_conflicts() {
    let b = Wrapped::new(Apply::Atomic);
    let table_id = create_managed(&b, "t").await;
    let req = |file: &str| DeltaUpdateTableRequest {
        requirements: vec![DeltaTableRequirement::AssertTableUuid {
            uuid: table_id.clone(),
        }],
        updates: vec![commit(1, file)],
    };
    b.update_table(table_path("t"), req("a.json"), ())
        .await
        .unwrap();
    let err = b
        .update_table(table_path("t"), req("b.json"), ())
        .await
        .unwrap_err();
    assert!(
        matches!(err.0, DeltaBackendError::CommitVersionConflict(_)),
        "{err:?}"
    );
}

/// A caller without access is denied before the table is looked up, so a missing
/// table and a forbidden one are indistinguishable.
#[tokio::test]
async fn unauthorized_table_operations_never_resolve() {
    let b = Wrapped::denying();
    let missing = table_path("does-not-exist");
    let errors = [
        b.get_table_credentials(missing.clone(), DeltaCredentialOperation::Read, ())
            .await
            .unwrap_err(),
        b.update_table(
            missing.clone(),
            DeltaUpdateTableRequest {
                requirements: vec![DeltaTableRequirement::AssertTableUuid { uuid: "x".into() }],
                updates: vec![set_prop("k", "v")],
            },
            (),
        )
        .await
        .unwrap_err(),
        b.report_metrics(
            missing.clone(),
            DeltaReportMetricsRequest {
                table_id: "x".into(),
                report: None,
            },
            (),
        )
        .await
        .unwrap_err(),
        DeltaApiHandler::<Cx>::load_table(&b, missing.clone(), ())
            .await
            .unwrap_err(),
    ];
    for err in errors {
        assert!(
            matches!(err.0, DeltaBackendError::PermissionDenied(_)),
            "{err:?}"
        );
    }
    assert_eq!(*b.resolves.lock().unwrap(), 0, "denied before any resolve");
}

#[tokio::test]
async fn get_config_is_authorized() {
    let b = Wrapped::denying();
    let err = b
        .get_config(
            unitycatalog_delta_api::handler::GetConfigQuery {
                catalog: "catalog".into(),
                protocol_versions: "1.0".into(),
            },
            (),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err.0, DeltaBackendError::PermissionDenied(_)),
        "{err:?}"
    );
}
