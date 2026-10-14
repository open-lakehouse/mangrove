//! End-to-end catalog-managed Delta table lifecycle against **any** storage
//! backend the UC server can vend for — exercised here against the Azurite
//! (Azure Blob emulator) stack from open-lakehouse.
//!
//! Unlike `managed_table_write.rs` (which hand-builds an S3 store and writes the
//! Delta log byte-by-byte), this example is fully cloud-agnostic: it drives the
//! public `create_managed_table` / `append_to_managed_table` APIs and reads back
//! through the same `UnityObjectStoreFactory` the query server uses. Every store
//! is built from a vended credential, so it works for S3, real Azure, or Azurite
//! with no per-cloud code here.
//!
//! It proves the credential-vending data path end-to-end: create (writes
//! `_delta_log/0.json` via a vended SAS), append (a ratified commit + data file),
//! and read (a snapshot assembled from the catalog's commit tail, scanned through
//! the vended store).
//!
//! ## Running against the Azurite stack
//!
//! ```bash
//! # 1. Azurite (open-lakehouse):  just env-up azurite
//! # 2. Rust UC server (this repo), pointed at the azurite config:
//! #      cargo run -p unitycatalog-cli -- server --rest --port 8081 \
//! #        --config ../open-lakehouse/environments/config/azurite/uc-config.yaml
//! # 3. Seed the credential + external location:
//! #      ../open-lakehouse/scripts/azurite-seed.sh
//! # 4. Create the catalog/schema (managed root inherited from the server config):
//! #      curl -XPOST .../catalogs           -d '{"name":"azc"}'
//! #      curl -XPOST .../schemas            -d '{"name":"s","catalog_name":"azc"}'
//! # 5. Run this example:
//! UC_ENDPOINT=http://localhost:8081/api/2.1/unity-catalog/ \
//! UC_CATALOG=azc UC_SCHEMA=s UC_TABLE=orders \
//! cargo run -p datafusion-unitycatalog --features delta --example managed_table_azurite
//! ```

use std::sync::Arc;

use datafusion::arrow::array::{Int64Array, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::util::pretty::pretty_format_batches;
use datafusion::prelude::SessionContext;
use datafusion_unitycatalog::RoutingObjectStore;
use datafusion_unitycatalog::catalog::build_catalog_managed_snapshot;
use datafusion_unitycatalog::catalog::{ManagedReadState, resolve_managed_read_state};
use datafusion_unitycatalog::managed::{append_to_managed_table, create_managed_table};
use deltalake_core::delta_datafusion::DeltaScanNext;
use deltalake_core::delta_datafusion::engine::DataFusionEngine;
use deltalake_core::logstore::{StorageConfig, default_logstore};
use object_store::path::Path;
use unitycatalog_client::TableOperation;
use unitycatalog_object_store::UnityObjectStoreFactory;
use url::Url;

type BoxError = Box<dyn std::error::Error>;

const ENGINE_INFO: &str = "managed_table_azurite-example/0.1";

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let endpoint = std::env::var("UC_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8081/api/2.1/unity-catalog/".to_string());
    let catalog = std::env::var("UC_CATALOG").unwrap_or_else(|_| "azc".to_string());
    let schema = std::env::var("UC_SCHEMA").unwrap_or_else(|_| "s".to_string());
    let table = std::env::var("UC_TABLE").unwrap_or_else(|_| "orders".to_string());
    let full_name = format!("{catalog}.{schema}.{table}");

    let mut builder = UnityObjectStoreFactory::builder().with_uri(endpoint);
    match std::env::var("UC_TOKEN") {
        Ok(token) => builder = builder.with_token(token),
        Err(_) => builder = builder.with_allow_unauthenticated(true),
    }
    if let Ok(region) = std::env::var("AWS_REGION") {
        builder = builder.with_aws_region(region);
    }
    let factory = Arc::new(builder.build().await?);
    let client = Arc::new(factory.unity_client().delta_v1());

    let arrow_schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("item", DataType::Utf8, true),
    ]));

    // ── Stage 1: create the managed table (writes _delta_log/0.json via a vended credential) ──
    println!("creating managed table {full_name} …");
    let created = create_managed_table(
        client.clone(),
        &catalog,
        &schema,
        &table,
        arrow_schema.clone(),
        vec![],
        ENGINE_INFO,
    )
    .await?;
    println!(
        "  created at {} (table_id {})",
        created.location, created.table_id
    );

    // ── Stage 2: append a batch of rows (a ratified commit + data file) ──
    let batch = RecordBatch::try_new(
        arrow_schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3])),
            Arc::new(StringArray::from(vec!["alpha", "beta", "gamma"])),
        ],
    )?;
    let version = append_to_managed_table(
        factory.clone(),
        &catalog,
        &schema,
        &table,
        batch,
        ENGINE_INFO,
    )
    .await?;
    println!("  appended 3 rows, committed version {version}");

    // ── Stage 3: read it back through the vended store + catalog commit tail ──
    let rows = read_managed_table(&factory, &catalog, &schema, &table, &full_name).await?;
    println!(
        "read back {} rows:\n{}",
        rows.iter().map(|b| b.num_rows()).sum::<usize>(),
        pretty_format_batches(&rows)?
    );

    let total: usize = rows.iter().map(|b| b.num_rows()).sum();
    if total != 3 {
        return Err(format!("expected 3 rows, read {total}").into());
    }
    println!("\nSUCCESS: managed Delta table create + append + read works end-to-end.");
    Ok(())
}

/// Read a catalog-managed table through the vended object store, assembling the
/// snapshot from the catalog's ratified commit tail (mirrors the query server's
/// read path and `managed_table_snapshot.rs`).
async fn read_managed_table(
    factory: &Arc<UnityObjectStoreFactory>,
    catalog: &str,
    schema: &str,
    table: &str,
    full_name: &str,
) -> Result<Vec<RecordBatch>, BoxError> {
    let loaded = factory
        .unity_client()
        .delta_v1()
        .load_table(catalog, schema, table)
        .await?;
    let (commits, latest) = match resolve_managed_read_state(&loaded)? {
        ManagedReadState::Managed { commits, latest } => (commits, latest),
        ManagedReadState::NotManaged => {
            return Err(format!("{full_name} is not catalog-managed").into());
        }
    };

    let ctx = SessionContext::new();
    let uc_store = factory
        .for_table(full_name.to_string(), TableOperation::Read)
        .await?;
    let location = uc_store.url().clone();

    // Route by sub-path under the bucket/container key (see managed_table_snapshot.rs).
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
    ctx.runtime_env()
        .register_object_store(&Url::parse(&format!("{bucket_key}/"))?, Arc::new(router));

    let root_store = ctx
        .runtime_env()
        .object_store_registry
        .get_store(&location)?;
    let config = StorageConfig::default();
    let prefixed_store = config.decorate_store(root_store.clone(), &location)?;
    let log_store = default_logstore(Arc::from(prefixed_store), root_store, &location, &config);
    let engine = DataFusionEngine::new_from_context(ctx.task_ctx());

    let snapshot =
        build_catalog_managed_snapshot(engine.clone(), &location, &commits, latest as i64, None)
            .await?;
    let provider = DeltaScanNext::builder()
        .with_snapshot(Arc::new(snapshot))
        .with_log_store(log_store)
        .await?;
    ctx.register_table(table, provider)?;
    let df = ctx
        .sql(&format!("SELECT * FROM {table} ORDER BY id"))
        .await?;
    Ok(df.collect().await?)
}
