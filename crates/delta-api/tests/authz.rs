//! Authorization-contract tests for the Delta handler.
//!
//! These prove the property the uniform-authz refactor buys: every user-facing
//! operation routes through the single [`DeltaBackend::authorize`] hook with the
//! expected [`DeltaAction`] variant, and the staging creator-match is a backend
//! decision (not an in-crate string compare). A `RecordingBackend` wraps the
//! in-memory backend and appends each authorized action's label to a shared log.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use unitycatalog_delta_api::DeltaApiHandler;
use unitycatalog_delta_api::authz::DeltaAction;
use unitycatalog_delta_api::backend::{
    BackendResult, CreateTableSpec, CredentialAccess, DeltaBackend, ResolvedTable, SchemaRef,
    StagingReservation, TableRef, UpdateTableSpec, VendedCredential,
};
use unitycatalog_delta_api::contract;
use unitycatalog_delta_api::coordinator::CommitCoordinator;
use unitycatalog_delta_api::error::DeltaBackendError;
use unitycatalog_delta_api::models::*;
use unitycatalog_delta_api::testing::InMemoryDeltaBackend;

type Cx = ();

/// A `DeltaBackend` that records the label of every action it authorizes, then
/// delegates all work (including the authorization decision) to an inner
/// [`InMemoryDeltaBackend`].
struct RecordingBackend {
    inner: InMemoryDeltaBackend,
    log: Arc<Mutex<Vec<String>>>,
}

impl RecordingBackend {
    fn new(inner: InMemoryDeltaBackend) -> Self {
        Self {
            inner,
            log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn actions(&self) -> Vec<String> {
        self.log.lock().unwrap().clone()
    }
}

/// The stable label for a `DeltaAction` variant.
fn label(action: &DeltaAction<'_>) -> &'static str {
    match action {
        DeltaAction::CreateTable { .. } => "CreateTable",
        DeltaAction::ReadTable { .. } => "ReadTable",
        DeltaAction::WriteTable { .. } => "WriteTable",
        DeltaAction::DeleteTable { .. } => "DeleteTable",
        DeltaAction::RenameTable { .. } => "RenameTable",
        DeltaAction::VendTableCredential { .. } => "VendTableCredential",
        DeltaAction::VendPathCredential { .. } => "VendPathCredential",
        DeltaAction::CreateStaging { .. } => "CreateStaging",
        DeltaAction::AdoptStaging { .. } => "AdoptStaging",
        _ => "Unknown",
    }
}

#[async_trait]
impl DeltaBackend<Cx> for RecordingBackend {
    async fn authorize(&self, action: DeltaAction<'_>, cx: &Cx) -> BackendResult<()> {
        self.log.lock().unwrap().push(label(&action).to_string());
        self.inner.authorize(action, cx).await
    }

    async fn catalog_exists(&self, catalog: &str, cx: &Cx) -> BackendResult<()> {
        self.inner.catalog_exists(catalog, cx).await
    }

    async fn resolve_table(&self, table: &TableRef, cx: &Cx) -> BackendResult<ResolvedTable> {
        self.inner.resolve_table(table, cx).await
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

// ---------------------------------------------------------------------------
// Test helpers (self-contained; the sibling `handler` test binary has its own).
// ---------------------------------------------------------------------------

fn schema_path() -> SchemaRef {
    SchemaRef {
        catalog: "catalog".to_string(),
        schema: "schema".to_string(),
    }
}

fn table_path(name: &str) -> TableRef {
    TableRef {
        catalog: "catalog".to_string(),
        schema: "schema".to_string(),
        table: name.to_string(),
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

fn managed_properties(table_id: &str) -> BTreeMap<String, String> {
    let mut p: BTreeMap<String, String> = contract::REQUIRED_FIXED_PROPERTIES
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    p.insert(contract::PROP_UC_TABLE_ID.to_string(), table_id.to_string());
    p
}

fn simple_columns() -> DeltaStructType {
    DeltaStructType {
        type_tag: Default::default(),
        fields: vec![DeltaStructField {
            name: "id".into(),
            data_type: DeltaDataType::Primitive("long".into()),
            nullable: false,
            metadata: Default::default(),
        }],
    }
}

fn create_table_request(name: &str, location: &str, table_id: &str) -> DeltaCreateTableRequest {
    DeltaCreateTableRequest {
        name: name.to_string(),
        location: location.to_string(),
        table_type: DeltaTableType::Managed,
        data_source_format: Some(DeltaDataSourceFormat::Delta),
        comment: None,
        columns: simple_columns(),
        partition_columns: None,
        protocol: compliant_protocol(),
        properties: managed_properties(table_id),
        domain_metadata: None,
        last_commit_timestamp_ms: 1700,
        uniform: None,
    }
}

/// Drive the full managed create flow through a recording backend, returning it
/// plus the created table id.
async fn create_managed(b: &RecordingBackend, name: &str) -> String {
    let staging = b
        .create_staging_table(
            schema_path(),
            DeltaCreateStagingTableRequest {
                name: name.to_string(),
            },
            (),
        )
        .await
        .expect("create staging");
    let table_id = staging.table_id.clone();
    b.create_table(
        schema_path(),
        create_table_request(name, &staging.location, &table_id),
        (),
    )
    .await
    .expect("create table");
    table_id
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Every user-facing operation authorizes through the single hook with the
/// action variant it should. This is the type-enforced "no split-brain" property:
/// no operation reaches storage without a matching `authorize` call.
#[tokio::test]
async fn each_operation_authorizes_with_its_action() {
    let b = RecordingBackend::new(InMemoryDeltaBackend::new());

    // create staging + managed create → CreateStaging, CreateTable, AdoptStaging.
    let table_id = create_managed(&b, "t").await;
    assert_eq!(
        b.actions(),
        vec!["CreateStaging", "CreateTable", "AdoptStaging"]
    );

    b.load_table(table_path("t"), ()).await.expect("load");
    b.table_exists(table_path("t"), ()).await.expect("exists");
    b.get_table_credentials(table_path("t"), DeltaCredentialOperation::Read, ())
        .await
        .expect("table creds");
    b.get_temporary_path_credentials(
        "s3://bucket/some/path".to_string(),
        DeltaCredentialOperation::ReadWrite,
        (),
    )
    .await
    .expect("path creds");

    b.update_table(
        table_path("t"),
        DeltaUpdateTableRequest {
            requirements: vec![DeltaTableRequirement::AssertTableUuid {
                uuid: table_id.clone(),
            }],
            updates: vec![DeltaTableUpdate::SetProperties {
                updates: BTreeMap::from([("k".to_string(), "v".to_string())]),
            }],
        },
        (),
    )
    .await
    .expect("update");

    DeltaApiHandler::rename_table(
        &b,
        table_path("t"),
        DeltaRenameTableRequest {
            new_name: "t2".to_string(),
        },
        (),
    )
    .await
    .expect("rename");

    DeltaApiHandler::delete_table(&b, table_path("t2"), ())
        .await
        .expect("delete");

    assert_eq!(
        b.actions(),
        vec![
            "CreateStaging",
            "CreateTable",
            "AdoptStaging",
            "ReadTable",           // load_table
            "ReadTable",           // table_exists
            "VendTableCredential", // get_table_credentials
            "VendPathCredential",  // get_temporary_path_credentials
            "WriteTable",          // update_table
            "RenameTable",         // rename_table
            "DeleteTable",         // delete_table
        ]
    );
}

/// The creator-match is now a backend decision (`AdoptStaging`), not an in-crate
/// string compare. The creator adopts successfully; a backend reporting a
/// different principal denies adoption of the same reservation.
#[tokio::test]
async fn adopt_staging_is_a_backend_decision() {
    // The reservation's creator can adopt it end-to-end.
    let creator = RecordingBackend::new(InMemoryDeltaBackend::new().with_principal("alice"));
    let staging = creator
        .create_staging_table(
            schema_path(),
            DeltaCreateStagingTableRequest {
                name: "t".to_string(),
            },
            (),
        )
        .await
        .expect("create staging");
    let created = creator
        .create_table(
            schema_path(),
            create_table_request("t", &staging.location, &staging.table_id),
            (),
        )
        .await;
    assert!(created.is_ok(), "creator should be allowed to adopt");

    // A backend reporting a different principal denies adoption of an
    // "alice"-created reservation.
    let other = InMemoryDeltaBackend::new().with_principal("bob");
    let reservation = StagingReservation {
        table_id: "id".to_string(),
        name: "t".to_string(),
        location: "s3://bucket/staging/id".to_string(),
        created_by: Some("alice".to_string()),
        stage_committed: false,
    };
    let decision = DeltaBackend::<Cx>::authorize(
        &other,
        DeltaAction::AdoptStaging {
            reservation: &reservation,
        },
        &(),
    )
    .await;
    assert!(matches!(
        decision,
        Err(DeltaBackendError::PermissionDenied(_))
    ));
}
