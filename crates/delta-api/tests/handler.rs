//! End-to-end tests for the Delta business logic, driven through the
//! `DeltaApiHandler` blanket impl over the in-memory `DeltaBackend`.
//!
//! These exercise the *semantics* — the managed-table contract, the `updateTable`
//! action dispatcher, and commit arbitration — without a real server, replacing
//! the server-coupled tests that previously lived in `crates/server/src/api/delta.rs`.

use std::collections::BTreeMap;

use unitycatalog_delta_api::DeltaApiHandler;
use unitycatalog_delta_api::backend::{SchemaRef, TableRef};
use unitycatalog_delta_api::contract;
use unitycatalog_delta_api::error::DeltaBackendError;
use unitycatalog_delta_api::handler::GetConfigQuery;
use unitycatalog_delta_api::models::*;
use unitycatalog_delta_api::testing::InMemoryDeltaBackend;

/// The context type: unit. `DeltaBackend<()>` is implemented for the in-memory
/// backend, and `DeltaApiHandler<()>` follows from the blanket impl.
type Cx = ();

fn backend() -> InMemoryDeltaBackend {
    InMemoryDeltaBackend::new()
}

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

/// Create a managed table end-to-end: stage → create → return its uuid.
async fn create_managed(b: &InMemoryDeltaBackend, name: &str) -> (String, String) {
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
    let location = staging.location.clone();

    let resp = b
        .create_table(
            schema_path(),
            DeltaCreateTableRequest {
                name: name.to_string(),
                location: location.clone(),
                table_type: DeltaTableType::Managed,
                data_source_format: Some(DeltaDataSourceFormat::Delta),
                comment: None,
                columns: simple_columns(),
                partition_columns: None,
                protocol: compliant_protocol(),
                properties: managed_properties(&table_id),
                domain_metadata: None,
                last_commit_timestamp_ms: 1700,
                uniform: None,
            },
            (),
        )
        .await
        .expect("create table");
    assert_eq!(resp.metadata.table_uuid, table_id);
    (table_id, location)
}

#[tokio::test]
async fn get_config_lists_endpoints() {
    let b = backend();
    let cfg = b
        .get_config(
            GetConfigQuery {
                catalog: "catalog".into(),
                protocol_versions: "1.0".into(),
            },
            (),
        )
        .await
        .unwrap();
    assert_eq!(cfg.protocol_version, "1.0");
    assert!(!cfg.endpoints.is_empty());
}

#[tokio::test]
async fn get_config_unknown_catalog_is_not_found() {
    let b = backend();
    let err = b
        .get_config(
            GetConfigQuery {
                catalog: "missing".into(),
                protocol_versions: "1.0".into(),
            },
            (),
        )
        .await
        .unwrap_err();
    assert!(matches!(err.0, DeltaBackendError::NotFound(_)), "{err:?}");
}

#[tokio::test]
async fn create_then_load_managed_table() {
    let b = backend();
    let (table_id, _) = create_managed(&b, "t").await;

    let loaded: DeltaLoadTableResponse = DeltaApiHandler::<Cx>::load_table(&b, table_path("t"), ())
        .await
        .unwrap();
    assert_eq!(loaded.metadata.table_uuid, table_id);
    assert_eq!(loaded.metadata.table_type, DeltaTableType::Managed);
    // A freshly-created managed table has no ratified commits, version 0.
    assert_eq!(loaded.latest_table_version, Some(0));
    assert_eq!(loaded.commits.as_deref(), Some(&[][..]));
}

#[tokio::test]
async fn create_table_rejects_noncompliant_protocol() {
    let b = backend();
    let staging = b
        .create_staging_table(
            schema_path(),
            DeltaCreateStagingTableRequest { name: "bad".into() },
            (),
        )
        .await
        .unwrap();
    let mut proto = compliant_protocol();
    proto.min_writer_version = 5; // below the required 7
    let err = b
        .create_table(
            schema_path(),
            DeltaCreateTableRequest {
                name: "bad".into(),
                location: staging.location,
                table_type: DeltaTableType::Managed,
                data_source_format: Some(DeltaDataSourceFormat::Delta),
                comment: None,
                columns: simple_columns(),
                partition_columns: None,
                protocol: proto,
                properties: managed_properties(&staging.table_id),
                domain_metadata: None,
                last_commit_timestamp_ms: 1700,
                uniform: None,
            },
            (),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err.0, DeltaBackendError::InvalidArgument(_)),
        "{err:?}"
    );
}

#[tokio::test]
async fn update_table_add_commit_then_load_shows_commit() {
    let b = backend();
    let (table_id, _) = create_managed(&b, "t").await;

    let resp = b
        .update_table(
            table_path("t"),
            DeltaUpdateTableRequest {
                requirements: vec![DeltaTableRequirement::AssertTableUuid {
                    uuid: table_id.clone(),
                }],
                updates: vec![DeltaTableUpdate::AddCommit {
                    commit: DeltaCommit {
                        version: 1,
                        timestamp: 1704067200000,
                        file_name: "00000000-0000-0000-0000-00000000002a.json".into(),
                        file_size: 2048,
                        file_modification_timestamp: 1704067200000,
                    },
                    uniform: None,
                }],
            },
            (),
        )
        .await
        .unwrap();
    assert_eq!(resp.latest_table_version, Some(1));
    let commits = resp.commits.expect("commits present");
    assert_eq!(commits.len(), 1);
    assert_eq!(commits[0].version, 1);
}

#[tokio::test]
async fn update_table_requires_assert_table_uuid() {
    let b = backend();
    let (_id, _) = create_managed(&b, "t").await;
    let err = b
        .update_table(
            table_path("t"),
            DeltaUpdateTableRequest {
                requirements: vec![],
                updates: vec![DeltaTableUpdate::SetProperties {
                    updates: BTreeMap::from([("k".to_string(), "v".to_string())]),
                }],
            },
            (),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err.0, DeltaBackendError::InvalidArgument(_)),
        "{err:?}"
    );
}

#[tokio::test]
async fn update_table_wrong_uuid_conflicts() {
    let b = backend();
    let (_id, _) = create_managed(&b, "t").await;
    let err = b
        .update_table(
            table_path("t"),
            DeltaUpdateTableRequest {
                requirements: vec![DeltaTableRequirement::AssertTableUuid {
                    uuid: "not-the-real-uuid".into(),
                }],
                updates: vec![],
            },
            (),
        )
        .await
        .unwrap_err();
    assert!(
        matches!(err.0, DeltaBackendError::UpdateRequirementConflict(_)),
        "{err:?}"
    );
}

#[tokio::test]
async fn update_table_set_properties_persists() {
    let b = backend();
    let (table_id, _) = create_managed(&b, "t").await;
    b.update_table(
        table_path("t"),
        DeltaUpdateTableRequest {
            requirements: vec![DeltaTableRequirement::AssertTableUuid {
                uuid: table_id.clone(),
            }],
            updates: vec![DeltaTableUpdate::SetProperties {
                updates: BTreeMap::from([("custom.key".to_string(), "value".to_string())]),
            }],
        },
        (),
    )
    .await
    .unwrap();

    let loaded = DeltaApiHandler::<Cx>::load_table(&b, table_path("t"), ())
        .await
        .unwrap();
    assert_eq!(
        loaded
            .metadata
            .properties
            .get("custom.key")
            .map(String::as_str),
        Some("value")
    );
}

#[tokio::test]
async fn table_credentials_and_delete() {
    let b = backend();
    create_managed(&b, "t").await;

    let creds = b
        .get_table_credentials(table_path("t"), DeltaCredentialOperation::ReadWrite, ())
        .await
        .unwrap();
    assert_eq!(creds.storage_credentials.len(), 1);
    assert_eq!(
        creds.storage_credentials[0].operation,
        DeltaCredentialOperation::ReadWrite
    );

    DeltaApiHandler::<Cx>::delete_table(&b, table_path("t"), ())
        .await
        .unwrap();
    let err = DeltaApiHandler::<Cx>::table_exists(&b, table_path("t"), ())
        .await
        .unwrap_err();
    assert!(matches!(err.0, DeltaBackendError::NotFound(_)), "{err:?}");
}

#[tokio::test]
async fn rename_is_supported_by_the_in_memory_backend() {
    let b = backend();
    create_managed(&b, "t").await;
    b.rename_table(
        table_path("t"),
        DeltaRenameTableRequest {
            new_name: "renamed".into(),
        },
        (),
    )
    .await
    .unwrap();
    DeltaApiHandler::<Cx>::table_exists(&b, table_path("renamed"), ())
        .await
        .unwrap();
}

#[tokio::test]
async fn temporary_path_credentials() {
    let b = backend();
    let creds = b
        .get_temporary_path_credentials(
            "s3://bucket/some/path".into(),
            DeltaCredentialOperation::Read,
            (),
        )
        .await
        .unwrap();
    assert_eq!(creds.storage_credentials.len(), 1);
    assert_eq!(creds.storage_credentials[0].prefix, "s3://bucket/some/path");
}
