//! **Spike**: create and commit a Unity Catalog **catalog-managed** Delta table
//! end-to-end through the Rust client, then read it back.
//!
//! This is the write-path counterpart to `managed_table_snapshot.rs` (which only
//! reads). It exists to prove that the **Java Unity Catalog OSS server** correctly
//! handles the managed-table lifecycle and that the Rust `unitycatalog-client` +
//! `datafusion-unitycatalog` crates can drive its requests / parse its responses.
//! It is deliberately throwaway-quality: it hand-rolls the Delta log JSON so every
//! byte of the catalog-managed contract is explicit and inspectable. A production
//! integration would route commits through a delta-rs `LogStore` instead.
//!
//! ## The managed-table protocol, as exercised here
//!
//! A *catalog-managed* (coordinated-commit) table makes the catalog — not the
//! filesystem `_delta_log/` — the source of truth for the latest version. The
//! lifecycle (UC Delta API, `/delta/v1/...`):
//!
//! 1. **createStagingTable** → the server allocates a table id + a managed
//!    `location`, and tells us the required protocol/properties.
//! 2. The client writes **`_delta_log/0.json`** (protocol + metaData carrying the
//!    required features/properties, including `io.unitycatalog.tableId`).
//!    Credentials for this write are vended **by path** (`for_path` against the
//!    staging `location`), NOT by table name: the table is not a real table yet,
//!    so vending by name (`for_table` / getTable) 404s with `TABLE_NOT_FOUND`.
//! 3. **createTable** → the server validates the contract and registers v0.
//! 4. To commit v1: write the data file, write a **staged** commit
//!    `_delta_log/_staged_commits/<v>.<uuid>.json`, then **updateTable** with action
//!    `add-commit` to have the catalog ratify it.
//! 5. **Read back**: `loadTable` returns the ratified-but-unpublished commit tail +
//!    `latest_table_version`; `build_catalog_managed_snapshot` assembles the snapshot
//!    from that tail (no filesystem scan), and we `SELECT *`.
//! 6. **Publish + backfill** (optional, shown last): copy the staged file to
//!    `_delta_log/<v>.json` and `updateTable` with `set-latest-backfilled-version` so
//!    the catalog can stop tracking it.
//!
//! ## Prerequisite: the live Java UC OSS server
//!
//! Bring up the open-lakehouse live stack (Java UC OSS `ghcr.io/roeap/unitycatalog`
//! on `localhost:8081`, S3-compatible storage configured server-side):
//!
//! ```text
//! docker compose -f environments/live.compose.yaml up -d unity-catalog
//! ```
//!
//! ## Run
//!
//! ```text
//! UC_ENDPOINT=http://localhost:8081/api/2.1/unity-catalog/ \
//! UC_CATALOG=unity UC_SCHEMA=default UC_TABLE=spike_managed \
//! cargo run -p datafusion-unitycatalog --features delta --example managed_table_write
//! ```
//!
//! Set `UC_TOKEN` for an authenticated server (omit for a local unauthenticated OSS
//! server) and `AWS_REGION` if the storage backend needs it. The catalog/schema must
//! already exist (the live stack seeds `unity.default`); this example does not create
//! them.

use std::sync::Arc;

use datafusion::arrow::array::{Int64Array, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::util::pretty::print_batches;
use datafusion::prelude::SessionContext;
use datafusion_unitycatalog::RoutingObjectStore;
use datafusion_unitycatalog::catalog::{
    ManagedReadState, build_catalog_managed_snapshot, ensure_trailing_slash,
    resolve_managed_read_state,
};
use deltalake_core::delta_datafusion::DeltaScanNext;
use deltalake_core::delta_datafusion::engine::DataFusionEngine;
use deltalake_core::logstore::{StorageConfig, default_logstore};
use deltalake_core::parquet::arrow::ArrowWriter;
use object_store::aws::AmazonS3Builder;
use object_store::{ObjectStore, ObjectStoreExt, PutPayload, path::Path};
use serde_json::json;
use unitycatalog_delta_api::models::{
    DeltaCommit, DeltaCreateStagingTableRequest, DeltaCreateTableRequest, DeltaDataSourceFormat,
    DeltaDataType, DeltaProtocol, DeltaStagingTableResponse, DeltaStructField, DeltaStructType,
    DeltaTableRequirement, DeltaTableType, DeltaTableUpdate, DeltaUpdateTableRequest,
};
use unitycatalog_object_store::UnityObjectStoreFactory;
use url::Url;

type BoxError = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), BoxError> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let endpoint = std::env::var("UC_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8081/api/2.1/unity-catalog/".to_string());
    let catalog = std::env::var("UC_CATALOG").unwrap_or_else(|_| "demo".to_string());
    let schema = std::env::var("UC_SCHEMA").unwrap_or_else(|_| "managed_demo".to_string());
    let table = std::env::var("UC_TABLE").unwrap_or_else(|_| "spike_managed_2".to_string());
    let full_name = format!("{catalog}.{schema}.{table}");

    // UC object store factory: drives both the `/delta/v1` metadata calls and
    // credential vending. Auth mirrors the other examples — bearer token if set,
    // otherwise allow an unauthenticated local server.
    let mut builder = UnityObjectStoreFactory::builder().with_uri(endpoint);
    match std::env::var("UC_TOKEN") {
        Ok(token) => builder = builder.with_token(token),
        Err(_) => builder = builder.with_allow_unauthenticated(true),
    }
    if let Ok(region) = std::env::var("AWS_REGION") {
        builder = builder.with_aws_region(region);
    }
    let factory = builder.build().await?;
    let delta = factory.unity_client().delta_v1();

    // ===================================================================
    // Stage 1 — createStagingTable: allocate table id + managed location.
    // ===================================================================
    println!("== createStagingTable {full_name} ==");
    let staging = delta
        .create_staging_table(
            &catalog,
            &schema,
            &DeltaCreateStagingTableRequest {
                name: table.clone(),
            },
        )
        .await?;
    let table_id = staging.table_id.clone();
    let location = Url::parse(&ensure_trailing_slash(&staging.location))?;
    println!("  table_id      = {table_id}");
    println!("  location      = {location}");
    println!("  required_protocol   = {:?}", staging.required_protocol);
    println!("  required_properties = {:?}", staging.required_properties);

    // Build a credentialed object store from the credentials the staging response
    // *already* handed us, rather than making a second vending round-trip.
    //
    // Why not vend again? During staging the table is not yet a real table, so
    // vending by name (`for_table` / getTable) 404s with TABLE_NOT_FOUND, and
    // vending by path (`for_path` / temporary-path-credentials) on this server
    // returns a credential without a usable `url`, which fails store construction
    // with `InvalidUrl("relative URL without a base")`. The `createStagingTable`
    // response carries `storage_credentials` scoped to the staging location — the
    // authoritative creds for the staging-write phase — so we use those directly.
    // They cover the whole table directory, so this one store serves the data
    // file, the `_delta_log` writes, and the later read-back.
    let ctx = SessionContext::new();
    let root = build_staging_store(&staging, &location)?;
    register_routing_store(&ctx, &location, root.clone())?;

    // The table schema we'll create + write: (id BIGINT, name STRING).
    let arrow_schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, true),
        Field::new("name", DataType::Utf8, true),
    ]));

    // -------------------------------------------------------------------
    // Write _delta_log/0.json — protocol + metaData carrying the required
    // catalog-managed features/properties. The server's UcManagedDeltaContract
    // validates these on createTable.
    // -------------------------------------------------------------------
    let created_ts = 1_704_067_200_000i64; // fixed epoch-ms; spike determinism.
    let zero_json = build_zero_commit(&table_id, &table, created_ts);
    let zero_path = log_path(&location, "00000000000000000000.json");
    println!("== put {zero_path} ==");
    root.put(&zero_path, PutPayload::from(zero_json.into_bytes()))
        .await?;

    // -------------------------------------------------------------------
    // createTable — finalize the staging reservation at v0.
    // -------------------------------------------------------------------
    println!("== createTable {full_name} ==");
    let create_req = DeltaCreateTableRequest {
        name: table.clone(),
        location: location.to_string(),
        table_type: DeltaTableType::Managed,
        data_source_format: Some(DeltaDataSourceFormat::Delta),
        comment: None,
        columns: managed_columns(),
        partition_columns: None,
        protocol: managed_protocol(),
        properties: managed_properties(&table_id, created_ts),
        domain_metadata: None,
        last_commit_timestamp_ms: created_ts,
        uniform: None,
    };
    let created = delta.create_table(&catalog, &schema, &create_req).await?;
    println!(
        "  created: table_uuid={} version={:?}",
        created.metadata.table_uuid, created.latest_table_version
    );

    // ===================================================================
    // Stage 2 — commit v1: data file + staged commit + add-commit.
    // ===================================================================

    // Write a parquet data file under the table root.
    let batch = RecordBatch::try_new(
        arrow_schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3])),
            Arc::new(StringArray::from(vec!["alice", "bob", "carol"])),
        ],
    )?;
    let data_file_name = "part-00000-spike.parquet";
    let data_bytes = write_parquet(&batch)?;
    let data_size = data_bytes.len() as i64;
    let data_path = child_path(&location, data_file_name);
    println!("== put {data_path} ({data_size} bytes) ==");
    root.put(&data_path, PutPayload::from(data_bytes)).await?;

    // Build the staged commit (commitInfo with in-commit timestamp + add action),
    // and write it under _delta_log/_staged_commits/<file_name>. The file name is
    // <version>.<uuid>.json; the catalog tracks it by this name.
    let commit_ts = created_ts + 1000;
    let commit_uuid = "00000000-0000-0000-0000-0000000000a1"; // fixed for spike determinism.
    let commit_file_name = format!("00000000000000000001.{commit_uuid}.json");
    let commit_json = build_data_commit(1, commit_ts, data_file_name, data_size, batch.num_rows());
    let commit_size = commit_json.len() as i64;
    let staged_path = staged_commit_path(&location, &commit_file_name);
    println!("== put {staged_path} ==");
    root.put(&staged_path, PutPayload::from(commit_json.into_bytes()))
        .await?;

    // Propose the staged commit to the catalog (add-commit). The server ratifies it
    // iff version == last_ratified + 1, returning 409 on conflict.
    println!("== updateTable add-commit v1 ==");
    let add_commit = DeltaUpdateTableRequest {
        requirements: vec![DeltaTableRequirement::AssertTableUuid {
            uuid: table_id.clone(),
        }],
        updates: vec![DeltaTableUpdate::AddCommit {
            commit: DeltaCommit {
                version: 1,
                timestamp: commit_ts,
                file_name: commit_file_name.clone(),
                file_size: commit_size,
                file_modification_timestamp: commit_ts,
            },
            uniform: None,
        }],
    };
    let after_commit = delta
        .update_table(&catalog, &schema, &table, &add_commit)
        .await?;
    println!(
        "  ratified: latest_table_version={:?} commit_tail={}",
        after_commit.latest_table_version,
        after_commit.commits.as_deref().unwrap_or(&[]).len()
    );

    // ===================================================================
    // Stage 3 — read back from the catalog's ratified (unpublished) tail.
    // ===================================================================
    println!("== loadTable + build_catalog_managed_snapshot ==");
    let loaded = delta.load_table(&catalog, &schema, &table).await?;
    let (commits, latest) = match resolve_managed_read_state(&loaded)? {
        ManagedReadState::Managed { commits, latest } => (commits, latest),
        ManagedReadState::NotManaged => {
            return Err("expected a catalog-managed table after createTable".into());
        }
    };
    println!(
        "  loadTable: latest_table_version={latest} commit_tail={}",
        commits.len()
    );

    let config = StorageConfig::default();
    let prefixed = config.decorate_store(root.clone(), &location)?;
    let log_store = default_logstore(Arc::from(prefixed), root.clone(), &location, &config);
    let engine = DataFusionEngine::new_from_context(ctx.task_ctx());
    let snapshot =
        build_catalog_managed_snapshot(engine.as_ref(), &location, &commits, latest as i64, None)?;
    println!("  snapshot version = {}", snapshot.version());

    let provider = DeltaScanNext::builder()
        .with_snapshot(Arc::new(snapshot))
        .with_log_store(log_store)
        .await?;
    ctx.register_table(table.as_str(), provider)?;
    let df = ctx
        .sql(&format!("SELECT * FROM {table} ORDER BY id"))
        .await?;
    let batches = df.collect().await?;
    let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    println!("  scanned {rows} rows (expected 3)");
    print_batches(&batches)?;
    assert_eq!(
        rows, 3,
        "expected the 3 rows written via the managed commit"
    );
    assert_eq!(latest, 1, "catalog should track latest_table_version == 1");

    // ===================================================================
    // Stage 4 — publish + backfill: copy staged -> _delta_log/<v>.json and
    // notify the catalog so it can stop tracking the ratified commit.
    // ===================================================================
    let published_path = log_path(&location, "00000000000000000001.json");
    println!("== publish: copy staged -> {published_path} ==");
    root.copy(&staged_path, &published_path).await?;
    println!("== updateTable set-latest-backfilled-version 1 ==");
    let backfill = DeltaUpdateTableRequest {
        requirements: vec![DeltaTableRequirement::AssertTableUuid { uuid: table_id }],
        updates: vec![DeltaTableUpdate::SetLatestBackfilledVersion {
            latest_published_version: 1,
        }],
    };
    delta
        .update_table(&catalog, &schema, &table, &backfill)
        .await?;
    let after_backfill = delta.load_table(&catalog, &schema, &table).await?;
    println!(
        "  after backfill: latest_table_version={:?} commit_tail={} (tail typically empties post-publish)",
        after_backfill.latest_table_version,
        after_backfill.commits.as_deref().unwrap_or(&[]).len()
    );

    println!("\nSPIKE OK: created a managed table and round-tripped one client-driven commit.");
    Ok(())
}

// =======================================================================
// Helpers — Delta log JSON (hand-rolled so the contract bytes are explicit).
// =======================================================================

/// The catalog-managed table features, aligned to exactly what the live server
/// advertised in `createStagingTable`'s `required_protocol` — nothing extra
/// (e.g. no `deletionVectors`/`rowTracking`, which the server did not require).
fn reader_features() -> Vec<&'static str> {
    vec!["catalogManaged", "v2Checkpoint", "vacuumProtocolCheck"]
}
fn writer_features() -> Vec<&'static str> {
    vec![
        "catalogManaged",
        "v2Checkpoint",
        "vacuumProtocolCheck",
        "inCommitTimestamp",
    ]
}

fn managed_protocol() -> DeltaProtocol {
    DeltaProtocol {
        min_reader_version: 3,
        min_writer_version: 7,
        reader_features: Some(reader_features().into_iter().map(String::from).collect()),
        writer_features: Some(writer_features().into_iter().map(String::from).collect()),
    }
}

/// The createTable `columns` payload: (id BIGINT, name STRING).
fn managed_columns() -> DeltaStructType {
    DeltaStructType {
        type_tag: Default::default(),
        fields: vec![
            DeltaStructField {
                name: "id".into(),
                data_type: DeltaDataType::Primitive("long".into()),
                nullable: true,
                metadata: Default::default(),
            },
            DeltaStructField {
                name: "name".into(),
                data_type: DeltaDataType::Primitive("string".into()),
                nullable: true,
                metadata: Default::default(),
            },
        ],
    }
}

/// Table properties, aligned to exactly the `required_properties` the live server
/// advertised in `createStagingTable`: `delta.checkpointPolicy=v2`,
/// `delta.enableInCommitTimestamps=true`, and `io.unitycatalog.tableId`. Plus the
/// `delta.feature.*` entries for the required protocol features (delta enables a
/// feature by both listing it in the protocol and setting its `feature.*` prop).
/// Nothing beyond the required set — if the server rejects/needs more, its
/// `required_properties` response is the source of truth to reconcile against.
fn managed_properties(
    table_id: &str,
    _created_ts: i64,
) -> std::collections::BTreeMap<String, String> {
    let mut p = std::collections::BTreeMap::new();
    // Required properties (verbatim from the staging response).
    p.insert("delta.checkpointPolicy".into(), "v2".into());
    p.insert("delta.enableInCommitTimestamps".into(), "true".into());
    p.insert("io.unitycatalog.tableId".into(), table_id.into());
    // Feature-enable flags for the required protocol features.
    p.insert("delta.feature.catalogManaged".into(), "supported".into());
    p.insert("delta.feature.v2Checkpoint".into(), "supported".into());
    p.insert(
        "delta.feature.vacuumProtocolCheck".into(),
        "supported".into(),
    );
    p.insert("delta.feature.inCommitTimestamp".into(), "supported".into());
    p
}

/// `_delta_log/0.json`: protocol + metaData. The metaData `schemaString` is the
/// Delta JSON schema for (id, name); `configuration` carries the table properties.
fn build_zero_commit(table_id: &str, table_name: &str, created_ts: i64) -> String {
    let schema_string = json!({
        "type": "struct",
        "fields": [
            {"name": "id", "type": "long", "nullable": true, "metadata": {}},
            {"name": "name", "type": "string", "nullable": true, "metadata": {}}
        ]
    })
    .to_string();

    let protocol = json!({
        "protocol": {
            "minReaderVersion": 3,
            "minWriterVersion": 7,
            "readerFeatures": reader_features(),
            "writerFeatures": writer_features()
        }
    });
    let metadata = json!({
        "metaData": {
            "id": table_id,
            "name": table_name,
            "format": {"provider": "parquet", "options": {}},
            "schemaString": schema_string,
            "partitionColumns": [],
            "configuration": props_as_json(table_id, created_ts),
            "createdTime": created_ts
        }
    });
    let commit_info = json!({
        "commitInfo": {
            "timestamp": created_ts,
            "inCommitTimestamp": created_ts,
            "operation": "CREATE TABLE",
            "operationParameters": {}
        }
    });
    // One action per line (newline-delimited JSON).
    format!("{commit_info}\n{protocol}\n{metadata}\n")
}

/// `_staged_commits/...1.json`: commitInfo (with in-commit timestamp) + add.
fn build_data_commit(
    version: i64,
    commit_ts: i64,
    data_file_name: &str,
    data_size: i64,
    num_rows: usize,
) -> String {
    let commit_info = json!({
        "commitInfo": {
            "timestamp": commit_ts,
            "inCommitTimestamp": commit_ts,
            "operation": "WRITE",
            "operationParameters": {"mode": "Append"},
            "operationMetrics": {"numFiles": "1", "numOutputRows": num_rows.to_string()},
            "version": version
        }
    });
    let add = json!({
        "add": {
            "path": data_file_name,
            "partitionValues": {},
            "size": data_size,
            "modificationTime": commit_ts,
            "dataChange": true
        }
    });
    format!("{commit_info}\n{add}\n")
}

fn props_as_json(table_id: &str, created_ts: i64) -> serde_json::Value {
    let mut map = serde_json::Map::new();
    for (k, v) in managed_properties(table_id, created_ts) {
        map.insert(k, json!(v));
    }
    serde_json::Value::Object(map)
}

// =======================================================================
// Helpers — paths, parquet, store registration.
// =======================================================================

/// Build a bucket-rooted S3 [`ObjectStore`] from the credentials the
/// `createStagingTable` response handed us. Picks a `READ_WRITE` credential whose
/// `prefix` covers the staging `location` and feeds its static S3 keys
/// (access-key / secret / session-token) into an [`AmazonS3Builder`]. Region comes
/// from `AWS_REGION` if set (the live SeaweedFS/MinIO-style backend tolerates any).
fn build_staging_store(
    staging: &DeltaStagingTableResponse,
    location: &Url,
) -> Result<Arc<dyn ObjectStore>, BoxError> {
    let cred = staging
        .storage_credentials
        .iter()
        .find(|c| {
            location
                .as_str()
                .starts_with(c.prefix.trim_end_matches('/'))
        })
        .or_else(|| staging.storage_credentials.first())
        .ok_or("staging response carried no storage_credentials")?;

    let cfg = &cred.config;
    let bucket = location
        .host_str()
        .ok_or("staging location has no bucket/host")?;

    let mut builder = AmazonS3Builder::new()
        .with_bucket_name(bucket)
        .with_access_key_id(
            cfg.s3_access_key_id
                .clone()
                .ok_or("staging credential missing s3.access-key-id")?,
        )
        .with_secret_access_key(
            cfg.s3_secret_access_key
                .clone()
                .ok_or("staging credential missing s3.secret-access-key")?,
        );
    if let Some(token) = cfg.s3_session_token.clone() {
        builder = builder.with_token(token);
    }
    // Honor an explicit endpoint/region for non-AWS S3 backends (SeaweedFS/MinIO).
    if let Ok(endpoint) = std::env::var("AWS_ENDPOINT_URL") {
        builder = builder.with_endpoint(endpoint).with_allow_http(true);
    }
    builder =
        builder.with_region(std::env::var("AWS_REGION").unwrap_or_else(|_| "us-east-1".into()));

    Ok(Arc::new(builder.build()?))
}

/// `object_store::Path` for `<table>/_delta_log/<name>` (root store is bucket-rooted).
fn log_path(location: &Url, name: &str) -> Path {
    child_path(location, &format!("_delta_log/{name}"))
}

fn staged_commit_path(location: &Url, file_name: &str) -> Path {
    child_path(location, &format!("_delta_log/_staged_commits/{file_name}"))
}

/// Join a path under the table location, expressed against the bucket-rooted store.
fn child_path(location: &Url, rel: &str) -> Path {
    let base = location
        .path()
        .trim_start_matches('/')
        .trim_end_matches('/');
    Path::from(format!("{base}/{rel}"))
}

fn write_parquet(batch: &RecordBatch) -> Result<Vec<u8>, BoxError> {
    let mut buf = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buf, batch.schema(), None)?;
    writer.write(batch)?;
    writer.close()?;
    Ok(buf)
}

/// Register a path-dispatching routing store under the bucket key, the way the UC
/// resolver does (mirrors `managed_table_snapshot.rs`).
fn register_routing_store(
    ctx: &SessionContext,
    location: &Url,
    store: Arc<dyn ObjectStore>,
) -> Result<(), BoxError> {
    let bucket_key = format!(
        "{}://{}",
        location.scheme(),
        &location[url::Position::BeforeHost..url::Position::AfterPort]
    );
    let router = RoutingObjectStore::new();
    router.register(
        Path::from_url_path(location.path()).unwrap_or_default(),
        store,
    );
    ctx.runtime_env()
        .register_object_store(&Url::parse(&format!("{bucket_key}/"))?, Arc::new(router));
    Ok(())
}
