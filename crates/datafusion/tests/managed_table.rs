//! Integration test for creating a Unity Catalog catalog-managed Delta table via the
//! kernel-committer framework (`datafusion_unitycatalog::managed::create_managed_table`).
//!
//! Hits a live Java Unity Catalog OSS server + its backing object store, so it's
//! `#[ignore]`d by default and requires the `delta` feature. Bring up the open-lakehouse
//! live stack first, then run:
//!
//! ```text
//! UC_ENDPOINT=http://localhost:8081/api/2.1/unity-catalog/ \
//! UC_CATALOG=demo UC_SCHEMA=managed_demo UC_TABLE=mt_itest \
//! AWS_REGION=eu-central-1 \
//! cargo test -p datafusion-unitycatalog --features delta --test managed_table -- --ignored --nocapture
//! ```
//!
//! Set `UC_TOKEN` for an authenticated server; omit for a local unauthenticated OSS server.
//! The catalog/schema must already exist (the live stack seeds `demo.managed_demo`); pick a
//! `UC_TABLE` name that does not yet exist.
#![cfg(feature = "delta")]

use std::sync::Arc;

use datafusion::arrow::array::{Int64Array, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::prelude::SessionContext;
use datafusion_unitycatalog::RoutingObjectStore;
use datafusion_unitycatalog::catalog::build_catalog_managed_snapshot;
use datafusion_unitycatalog::managed::{append_to_managed_table, create_managed_table};
use deltalake_core::delta_datafusion::DeltaScanNext;
use deltalake_core::delta_datafusion::engine::DataFusionEngine;
use deltalake_core::logstore::{StorageConfig, default_logstore};
use object_store::path::Path;
use unitycatalog_object_store::{TableOperation, UnityObjectStoreFactory};
use url::Url;

/// Build the UC object-store factory from env (mirrors `unity_resolve.rs`).
async fn factory_from_env() -> Option<UnityObjectStoreFactory> {
    let uri = std::env::var("UC_ENDPOINT").ok()?;
    let mut builder = UnityObjectStoreFactory::builder().with_uri(uri);
    match std::env::var("UC_TOKEN") {
        Ok(token) => builder = builder.with_token(token),
        Err(_) => builder = builder.with_allow_unauthenticated(true),
    }
    if let Ok(region) = std::env::var("AWS_REGION") {
        builder = builder.with_aws_region(region);
    }
    Some(builder.build().await.expect("failed to build UC factory"))
}

/// Create a managed table via the framework, then read it back through the catalog-managed
/// snapshot path and assert the (empty, version-0) table resolves.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "requires a live Unity Catalog server (set UC_ENDPOINT)"]
async fn create_managed_table_round_trip() {
    let Some(factory) = factory_from_env().await else {
        eprintln!("UC_ENDPOINT not set; skipping");
        return;
    };
    let factory = Arc::new(factory);
    let catalog = std::env::var("UC_CATALOG").unwrap_or_else(|_| "demo".into());
    let schema = std::env::var("UC_SCHEMA").unwrap_or_else(|_| "managed_demo".into());
    let table = std::env::var("UC_TABLE").unwrap_or_else(|_| "mt_itest".into());

    let arrow_schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, true),
        Field::new("name", DataType::Utf8, true),
    ]));

    // === Create the managed table (staging -> 0.json via committer -> createTable) ===
    let created = create_managed_table(
        Arc::new(factory.unity_client().delta_v1()),
        &catalog,
        &schema,
        &table,
        arrow_schema.clone(),
        vec![],
        "unitycatalog-rs-itest/0.1",
    )
    .await
    .expect("create_managed_table failed");
    println!(
        "created managed table {catalog}.{schema}.{table}: id={} location={}",
        created.table_id, created.location
    );

    // === Append a batch of 3 rows (commit v1 through the catalog committer) ===
    let batch = RecordBatch::try_new(
        arrow_schema,
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3])),
            Arc::new(StringArray::from(vec!["alice", "bob", "carol"])),
        ],
    )
    .unwrap();
    let version = append_to_managed_table(
        factory.clone(),
        &catalog,
        &schema,
        &table,
        batch,
        "unitycatalog-rs-itest/0.1",
    )
    .await
    .expect("append_to_managed_table failed");
    println!("appended commit version {version}");
    assert_eq!(version, 1, "first append should be version 1");

    // === Read it back via loadTable + the catalog-managed snapshot ===
    let loaded = factory
        .unity_client()
        .delta_v1()
        .load_table(&catalog, &schema, &table)
        .await
        .expect("loadTable failed");
    assert_eq!(
        loaded.metadata.table_type,
        unitycatalog_delta_api::models::DeltaTableType::Managed
    );
    let (commits, latest) =
        match datafusion_unitycatalog::catalog::resolve_managed_read_state(&loaded)
            .expect("resolve_managed_read_state failed")
        {
            datafusion_unitycatalog::catalog::ManagedReadState::Managed { commits, latest } => {
                (commits, latest)
            }
            other => panic!("expected a managed table, got {other:?}"),
        };
    println!(
        "loadTable: latest_table_version={latest} commit_tail={}",
        commits.len()
    );

    // Build a credentialed store + snapshot and scan (empty table -> 0 rows, but it must resolve).
    let ctx = SessionContext::new();
    let uc_store = factory
        .for_table(format!("{catalog}.{schema}.{table}"), TableOperation::Read)
        .await
        .expect("vend read creds");
    let location = uc_store.url().clone();
    let bucket_key = format!(
        "{}://{}",
        location.scheme(),
        &location[url::Position::BeforeHost..url::Position::AfterPort]
    );
    let router = RoutingObjectStore::new();
    router.register(
        Path::from_url_path(location.path()).unwrap_or_default(),
        uc_store.root(),
    );
    ctx.runtime_env().register_object_store(
        &Url::parse(&format!("{bucket_key}/")).unwrap(),
        Arc::new(router),
    );

    let root = ctx
        .runtime_env()
        .object_store_registry
        .get_store(&location)
        .unwrap();
    let config = StorageConfig::default();
    let prefixed = config.decorate_store(root.clone(), &location).unwrap();
    let log_store = default_logstore(Arc::from(prefixed), root, &location, &config);
    let engine = DataFusionEngine::new_from_context(ctx.task_ctx());
    let snapshot =
        build_catalog_managed_snapshot(engine.as_ref(), &location, &commits, latest as i64, None)
            .expect("build_catalog_managed_snapshot failed");
    assert_eq!(snapshot.version(), latest, "snapshot at catalog version");

    let provider = DeltaScanNext::builder()
        .with_snapshot(Arc::new(snapshot))
        .with_log_store(log_store)
        .await
        .expect("build DeltaScanNext");
    ctx.register_table(table.as_str(), provider).unwrap();
    let batches = ctx
        .sql(&format!("SELECT * FROM {table}"))
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    println!("scanned {rows} rows from managed table (expected 3)");
    assert_eq!(rows, 3, "should read back the 3 appended rows");
}
