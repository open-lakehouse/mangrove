//! The catalog-managed Delta write+read flow, driven end-to-end through the UC
//! `/delta/v1` API for the conformance battery.
//!
//! This is the acceptance-test port of the proven spike in
//! `crates/datafusion/examples/managed_table_write.rs`. It creates a managed Delta
//! table through the staging flow, commits one data file, then reads it back with a
//! catalog-managed kernel snapshot — asserting `SELECT *` returns the written rows.
//!
//! Unlike the S3-backed example, this targets **local filesystem** storage: the OSS
//! fixtures vend `file://` managed locations (see `dev/uc-oss.compose.yaml` /
//! `dev/uc-rust-conformance.yaml`), so the write store is a plain
//! [`LocalFileSystem`](object_store::local::LocalFileSystem). That keeps the check
//! credential-free — no cloud identity, matching the local-managed recipe.
//!
//! **macOS local caveat:** the Java fixture vends `file:///tmp/uc-test/...`, but on
//! macOS `/tmp` is a symlink to `/private/tmp` and `LocalFileSystem` canonicalizes on
//! read, so the kernel read-back can 404 against a local Docker fixture. CI runs on
//! Linux (where `/tmp` is real), and the native Rust-server target is unaffected, so
//! this only bites a *local macOS* run against the Java Docker fixture.
//!
//! The lifecycle (all via `/delta/v1/...`):
//! 1. **createStagingTable** → table id + managed `location` + required protocol.
//! 2. Write `_delta_log/0.json` (protocol + metaData) to the vended location.
//! 3. **createTable** → the server validates the contract and registers v0.
//! 4. Write a parquet data file + a staged commit `_staged_commits/<v>.<uuid>.json`,
//!    then **updateTable** `add-commit` so the catalog ratifies v1.
//! 5. **loadTable** → assemble a catalog-managed snapshot from the ratified tail and
//!    `SELECT *`.

use std::sync::Arc;

use datafusion::arrow::array::{Int64Array, RecordBatch, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
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
use object_store::local::LocalFileSystem;
use object_store::{ObjectStore, ObjectStoreExt, PutPayload, path::Path};
use serde_json::json;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_delta_api::models::{
    DeltaCommit, DeltaCreateStagingTableRequest, DeltaCreateTableRequest, DeltaDataType,
    DeltaProtocol, DeltaStructField, DeltaStructType, DeltaTableRequirement, DeltaTableType,
    DeltaTableUpdate, DeltaUpdateTableRequest,
};
use url::Url;

use crate::{AcceptanceError, AcceptanceResult};

/// Fixed epoch-ms so the hand-rolled commits are deterministic across runs.
const CREATED_TS: i64 = 1_704_067_200_000;

fn err(msg: impl std::fmt::Display) -> AcceptanceError {
    AcceptanceError::JourneyExecution(msg.to_string())
}

/// Drive the full managed-table create+commit+read flow against `client`, writing
/// the Delta files to the vended `file://` location. Returns the number of rows
/// read back through the catalog-managed snapshot (expected: 3).
///
/// The catalog/schema must already exist; the table must not.
pub async fn create_commit_read(
    client: &UnityCatalogClient,
    catalog: &str,
    schema: &str,
    table: &str,
) -> AcceptanceResult<usize> {
    let delta = client.delta_v1();

    // --- Stage 1: createStagingTable → table id + managed location. ---
    let staging = delta
        .create_staging_table(
            catalog,
            schema,
            &DeltaCreateStagingTableRequest {
                name: table.to_string(),
            },
        )
        .await
        .map_err(err)?;
    let table_id = staging.table_id.clone();
    let location = Url::parse(&ensure_trailing_slash(&staging.location)).map_err(|e| {
        err(format!(
            "staging location {:?} is not a URL: {e}",
            staging.location
        ))
    })?;
    if location.scheme() != "file" {
        return Err(crate::conformance::skip(format!(
            "managed-table staging vended a {} location; this check needs file:// storage",
            location.scheme()
        )));
    }
    // Local filesystem store rooted at the OS root: the vended `file://` path is
    // absolute and host-visible (bind-mounted 1:1 for the Java fixture; native for
    // the Rust server), so we address files by their absolute path under `/`. All
    // paths use the vended `location` verbatim — server-facing fields (createTable
    // `location`) must match what the staging table was registered under, and the
    // kernel resolves its log/data reads against the same URL, so the physical
    // writes must line up with it too.
    let store: Arc<dyn ObjectStore> =
        Arc::new(LocalFileSystem::new().with_automatic_cleanup(false));
    let ctx = SessionContext::new();
    register_routing_store(&ctx, &location, store.clone())?;

    let arrow_schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, true),
        Field::new("name", DataType::Utf8, true),
    ]));

    // Drive the protocol + properties from the staging response's `required_*`
    // rather than hardcoding: which reader/writer features a managed table must
    // carry (e.g. `deletionVectors`) varies by server, and the server rejects a
    // create that does not honor exactly what it advertised.
    let protocol = staging.required_protocol.clone();
    let writer_features: Vec<String> = protocol.writer_features.clone().unwrap_or_default();
    let properties = required_properties(&staging.required_properties, &table_id, &writer_features);

    // --- Write _delta_log/0.json (protocol + metaData). ---
    let zero_json = build_zero_commit(&table_id, table, &protocol, &properties);
    store
        .put(
            &log_path(&location, "00000000000000000000.json"),
            PutPayload::from(zero_json.into_bytes()),
        )
        .await
        .map_err(|e| err(format!("write _delta_log/0.json: {e}")))?;

    // --- Stage 1b: createTable → finalize the staging reservation at v0. ---
    let create_req = DeltaCreateTableRequest {
        name: table.to_string(),
        location: location.to_string(),
        table_type: DeltaTableType::Managed,
        // The stock Java UC OSS server (unitycatalog/unitycatalog:v0.5.0) hardcodes
        // DELTA and rejects an unrecognized `data-source-format` field on
        // createTable, so omit it. (An older roeap dev build required it — see the
        // field's doc comment in the delta-api models.)
        data_source_format: None,
        comment: None,
        columns: managed_columns(),
        partition_columns: None,
        protocol: protocol.clone(),
        properties: properties.clone(),
        domain_metadata: None,
        last_commit_timestamp_ms: CREATED_TS,
        uniform: None,
    };
    delta
        .create_table(catalog, schema, &create_req)
        .await
        .map_err(err)?;

    // --- Stage 2: commit v1 — data file + staged commit + add-commit. ---
    let batch = RecordBatch::try_new(
        arrow_schema.clone(),
        vec![
            Arc::new(Int64Array::from(vec![1, 2, 3])),
            Arc::new(StringArray::from(vec!["alice", "bob", "carol"])),
        ],
    )
    .map_err(|e| err(format!("build record batch: {e}")))?;
    let data_file_name = "part-00000-conf.parquet";
    let data_bytes = write_parquet(&batch)?;
    let data_size = data_bytes.len() as i64;
    store
        .put(
            &child_path(&location, data_file_name),
            PutPayload::from(data_bytes),
        )
        .await
        .map_err(|e| err(format!("write data file: {e}")))?;

    let commit_ts = CREATED_TS + 1000;
    let commit_uuid = "00000000-0000-0000-0000-0000000000a1";
    let commit_file_name = format!("00000000000000000001.{commit_uuid}.json");
    let commit_json = build_data_commit(1, commit_ts, data_file_name, data_size, batch.num_rows());
    let commit_size = commit_json.len() as i64;
    store
        .put(
            &staged_commit_path(&location, &commit_file_name),
            PutPayload::from(commit_json.into_bytes()),
        )
        .await
        .map_err(|e| err(format!("write staged commit: {e}")))?;

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
    delta
        .update_table(catalog, schema, table, &add_commit)
        .await
        .map_err(err)?;

    // --- Stage 3: read back from the catalog's ratified (unpublished) tail. ---
    let loaded = delta
        .load_table(catalog, schema, table)
        .await
        .map_err(err)?;
    let (commits, latest) = match resolve_managed_read_state(&loaded).map_err(err)? {
        ManagedReadState::Managed { commits, latest } => (commits, latest),
        ManagedReadState::NotManaged => {
            return Err(err("expected a catalog-managed table after createTable"));
        }
    };

    let config = StorageConfig::default();
    let prefixed = config
        .decorate_store(store.clone(), &location)
        .map_err(|e| err(format!("decorate store: {e}")))?;
    let log_store = default_logstore(Arc::from(prefixed), store.clone(), &location, &config);
    let engine = DataFusionEngine::new_from_context(ctx.task_ctx());
    // The kernel snapshot builder reads the `_delta_log` synchronously via an
    // internal `block_on`. Against a real-IO store (LocalFileSystem here) that
    // would panic when called directly on a runtime worker ("Cannot start a
    // runtime from within a runtime"); `block_in_place` hands the worker's other
    // tasks off first so the nested block_on is allowed. Requires the
    // multi-thread runtime flavor (see the conformance test attributes).
    let snapshot = tokio::task::block_in_place(|| {
        build_catalog_managed_snapshot(engine.as_ref(), &location, &commits, latest as i64, None)
    })
    .map_err(err)?;

    let provider = DeltaScanNext::builder()
        .with_snapshot(Arc::new(snapshot))
        .with_log_store(log_store)
        .await
        .map_err(|e| err(format!("build DeltaScanNext: {e}")))?;
    ctx.register_table(table, provider)
        .map_err(|e| err(format!("register table provider: {e}")))?;
    let df = ctx
        .sql(&format!("SELECT * FROM {table} ORDER BY id"))
        .await
        .map_err(|e| err(format!("plan SELECT: {e}")))?;
    let batches = df
        .collect()
        .await
        .map_err(|e| err(format!("collect SELECT: {e}")))?;
    Ok(batches.iter().map(|b| b.num_rows()).sum())
}

// ======================================================================
// Delta log JSON — hand-rolled so the catalog-managed contract is explicit.
// (Ported verbatim in intent from managed_table_write.rs.)
// ======================================================================

/// Build the table `properties` for createTable from the staging response's
/// `required_properties`, filling any `null` (server-chooses) value with a
/// sensible default, then adding `io.unitycatalog.tableId` and a
/// `delta.feature.<name>=supported` flag for every required writer feature (delta
/// enables a feature by both listing it in the protocol and setting its prop).
fn required_properties(
    required: &std::collections::BTreeMap<String, Option<String>>,
    table_id: &str,
    writer_features: &[String],
) -> std::collections::BTreeMap<String, String> {
    let mut p = std::collections::BTreeMap::new();
    for (k, v) in required {
        // A null required value means "any valid value"; supply a benign default.
        p.insert(k.clone(), v.clone().unwrap_or_else(|| "true".to_string()));
    }
    p.insert("io.unitycatalog.tableId".into(), table_id.into());
    for feat in writer_features {
        p.entry(format!("delta.feature.{feat}"))
            .or_insert_with(|| "supported".to_string());
    }
    p
}

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

/// `_delta_log/0.json`: protocol + metaData. Both the protocol features and the
/// metaData `configuration` are taken from the values negotiated with the server
/// (the staging response's `required_protocol` / `required_properties`), so the
/// on-disk log matches exactly what createTable was told to expect.
fn build_zero_commit(
    table_id: &str,
    table_name: &str,
    protocol: &DeltaProtocol,
    properties: &std::collections::BTreeMap<String, String>,
) -> String {
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
            "minReaderVersion": protocol.min_reader_version,
            "minWriterVersion": protocol.min_writer_version,
            "readerFeatures": protocol.reader_features.clone().unwrap_or_default(),
            "writerFeatures": protocol.writer_features.clone().unwrap_or_default()
        }
    });
    let configuration: serde_json::Map<String, serde_json::Value> = properties
        .iter()
        .map(|(k, v)| (k.clone(), json!(v)))
        .collect();
    let metadata = json!({
        "metaData": {
            "id": table_id,
            "name": table_name,
            "format": {"provider": "parquet", "options": {}},
            "schemaString": schema_string,
            "partitionColumns": [],
            "configuration": serde_json::Value::Object(configuration),
            "createdTime": CREATED_TS
        }
    });
    let commit_info = json!({
        "commitInfo": {
            "timestamp": CREATED_TS,
            "inCommitTimestamp": CREATED_TS,
            "operation": "CREATE TABLE",
            "operationParameters": {}
        }
    });
    format!("{commit_info}\n{protocol}\n{metadata}\n")
}

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

// ======================================================================
// Paths, parquet, store registration.
// ======================================================================

/// `object_store::Path` for `<table-path>/_delta_log/<name>`. The LocalFileSystem
/// store is rooted at `/`, so paths are the location's absolute path (sans scheme)
/// with the relative suffix appended.
fn log_path(location: &Url, name: &str) -> Path {
    child_path(location, &format!("_delta_log/{name}"))
}

fn staged_commit_path(location: &Url, file_name: &str) -> Path {
    child_path(location, &format!("_delta_log/_staged_commits/{file_name}"))
}

fn child_path(location: &Url, rel: &str) -> Path {
    let base = location
        .path()
        .trim_start_matches('/')
        .trim_end_matches('/');
    Path::from(format!("{base}/{rel}"))
}

fn write_parquet(batch: &RecordBatch) -> AcceptanceResult<Vec<u8>> {
    let mut buf = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buf, batch.schema(), None)
        .map_err(|e| err(format!("parquet writer: {e}")))?;
    writer
        .write(batch)
        .map_err(|e| err(format!("parquet write: {e}")))?;
    writer
        .close()
        .map_err(|e| err(format!("parquet close: {e}")))?;
    Ok(buf)
}

/// Register a path-dispatching routing store under the `file://` bucket key, the
/// way the UC resolver does (mirrors `managed_table_write.rs`).
fn register_routing_store(
    ctx: &SessionContext,
    location: &Url,
    store: Arc<dyn ObjectStore>,
) -> AcceptanceResult<()> {
    let router = RoutingObjectStore::new();
    router.register(
        Path::from_url_path(location.path()).unwrap_or_default(),
        store,
    );
    // For file:// there is no host/port; register under the bare scheme root.
    let bucket_key =
        Url::parse("file:///").map_err(|e| err(format!("build file:// bucket key: {e}")))?;
    ctx.runtime_env()
        .register_object_store(&bucket_key, Arc::new(router));
    Ok(())
}
