//! M2 verification: async-native, engine-free snapshot construction
//! ([`build_snapshot_from_manifest`]) resolves the same table state as the eager kernel
//! `DefaultEngine` build (`Snapshot::builder_for(..).build(engine)`) — for both a **commit-only**
//! table and a **classic-checkpointed** table (the latter exercises the async checkpoint-footer
//! `SchemaQuery` read, i.e. the #116 fix).
//!
//! The equivalence anchor: same `version()` and same logical `schema()` from both build paths
//! (schema identity proves Metadata's `schemaString` resolved identically; version proves the
//! P&M log replay reached the right commit). We additionally register a `DeltaSsaTableProvider`
//! on the manifest-built snapshot and run a `count(*)` to prove the whole pipeline
//! (construction + scan) works end to end over the async object store, with no `PrimedStore`
//! and no inline executor.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use datafusion::execution::context::SessionContext;
use delta_kernel::arrow::array::{ArrayRef, AsArray, Int64Array, RecordBatch, StringArray};
use delta_kernel::arrow::datatypes::{DataType, Field, Int64Type, Schema};
use delta_kernel::parquet::arrow::ArrowWriter;
use delta_kernel::parquet::basic::Compression;
use delta_kernel::parquet::file::properties::WriterProperties;
use delta_kernel::snapshot::{Snapshot, SnapshotRef};
use delta_kernel::{FileMeta, Version};
use delta_kernel_default_engine::DefaultEngineBuilder;
use object_store::local::LocalFileSystem;
use object_store::memory::InMemory;
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt};
use olai_delta_df::{
    DeltaEngineSessionOptions, DeltaSsaScanConfig, DeltaSsaTableProvider,
    build_snapshot_from_manifest, delta_engine_session,
};
use url::Url;

const TABLE_PREFIX: &str = "tbl";
const SCHEMA_STRING: &str = r#"{\"type\":\"struct\",\"fields\":[{\"name\":\"id\",\"type\":\"long\",\"nullable\":true,\"metadata\":{}},{\"name\":\"name\",\"type\":\"string\",\"nullable\":true,\"metadata\":{}}]}"#;

fn arrow_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, true),
        Field::new("name", DataType::Utf8, true),
    ]))
}

fn batch(ids: &[i64], names: &[&str]) -> RecordBatch {
    RecordBatch::try_new(
        arrow_schema(),
        vec![
            Arc::new(Int64Array::from(ids.to_vec())) as ArrayRef,
            Arc::new(StringArray::from(names.to_vec())) as ArrayRef,
        ],
    )
    .unwrap()
}

fn parquet_bytes(batch: &RecordBatch) -> Vec<u8> {
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();
    let mut buf = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buf, batch.schema(), Some(props)).unwrap();
    writer.write(batch).unwrap();
    writer.close().unwrap();
    buf
}

/// Commit-only in-memory fixture rooted at `prefix` (no trailing slash). A test can place the table
/// deep under a path — e.g. the `…/__unitystorage/catalogs/<id>/tables/<uuid>` shape a Unity Catalog
/// managed table has — to exercise table-root/log-root path joining.
async fn commit_only_store_at(prefix: &str) -> Arc<InMemory> {
    let store = InMemory::new();
    let files = [
        (
            "part-00000.snappy.parquet",
            batch(&[1, 2, 3], &["a", "b", "c"]),
        ),
        (
            "part-00001.snappy.parquet",
            batch(&[4, 5, 6], &["d", "e", "f"]),
        ),
    ];
    let mut commit = format!(
        concat!(
            r#"{{"protocol":{{"minReaderVersion":1,"minWriterVersion":2}}}}"#,
            "\n",
            r#"{{"metaData":{{"id":"11111111-2222-3333-4444-555555555555","format":{{"provider":"parquet","options":{{}}}},"schemaString":"{schema}","partitionColumns":[],"configuration":{{}},"createdTime":0}}}}"#,
            "\n",
        ),
        schema = SCHEMA_STRING,
    );
    for (name, data) in &files {
        let bytes = parquet_bytes(data);
        commit.push_str(&format!(
            r#"{{"add":{{"path":"{name}","partitionValues":{{}},"size":{size},"modificationTime":0,"dataChange":true}}}}"#,
            size = bytes.len(),
        ));
        commit.push('\n');
        store
            .put(&Path::from(format!("{prefix}/{name}")), bytes.into())
            .await
            .unwrap();
    }
    store
        .put(
            &Path::from(format!("{prefix}/_delta_log/00000000000000000000.json")),
            commit.into_bytes().into(),
        )
        .await
        .unwrap();
    Arc::new(store)
}

/// Commit-only in-memory fixture under [`TABLE_PREFIX`] (same shape as `native.rs::fixture_store`).
async fn commit_only_store() -> Arc<InMemory> {
    commit_only_store_at(TABLE_PREFIX).await
}

/// Build the `_delta_log` manifest (kernel `FileMeta`s) for `store` under `table_url` by listing
/// the log directory — standing in for mangrove's HEAD-probe discovery.
async fn manifest_for(store: &dyn ObjectStore, table_url: &Url) -> Vec<FileMeta> {
    // Join `_delta_log` onto the table path segment-wise (slash-safe) so this works whether or not
    // `table_url` carries a trailing slash — mirrors production discovery's `Path::join`.
    let log_prefix = Path::from(table_url.path().trim_start_matches('/')).join("_delta_log");
    let mut out = Vec::new();
    let mut listing = store.list(Some(&log_prefix));
    while let Some(meta) = futures::StreamExt::next(&mut listing).await {
        let meta = meta.unwrap();
        // Skip `_last_checkpoint` (not a versioned log file); the manifest names checkpoint parts
        // directly, as mangrove's discovery does.
        if meta.location.filename() == Some("_last_checkpoint") {
            continue;
        }
        let location = Url::parse(&format!(
            "{}://{}/{}",
            table_url.scheme(),
            table_url.host_str().unwrap_or(""),
            meta.location
        ))
        .unwrap();
        out.push(FileMeta {
            location,
            last_modified: 0,
            size: meta.size,
        });
    }
    out
}

fn session_with_store(table_url: &Url, store: Arc<dyn ObjectStore>) -> SessionContext {
    // The `wasm` preset mirrors the real query session (`query-wasm::engine::build_query_session`):
    // it configures the Delta engine, including disabling leaf-expression pushdown — the compiled
    // scan plan is optimized against this caller session and its FSR replay shape otherwise trips
    // `push_down_leaf_projections`. Explicit `::wasm()` (not `Default`) because this is a native
    // test binary emulating the browser view-types-off shape.
    delta_engine_session(store, table_url, &DeltaEngineSessionOptions::wasm())
}

fn eager_snapshot(store: Arc<dyn ObjectStore>, table_url: &Url, version: Version) -> SnapshotRef {
    let engine = DefaultEngineBuilder::new(store).build();
    Snapshot::builder_for(table_url.as_str())
        .at_version(version)
        .build(&engine)
        .expect("eager kernel snapshot build")
}

/// Commit-only: manifest-built snapshot matches the eager DefaultEngine build (version + schema),
/// and a scan over it counts all rows.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn snapshot_from_manifest_matches_eager_commit_only() {
    let store = commit_only_store().await;
    let table_url = Url::parse(&format!("memory:///{TABLE_PREFIX}/")).unwrap();

    let eager = eager_snapshot(Arc::clone(&store) as Arc<dyn ObjectStore>, &table_url, 0);

    let manifest = manifest_for(store.as_ref(), &table_url).await;
    let session = session_with_store(&table_url, Arc::clone(&store) as Arc<dyn ObjectStore>);
    let built = build_snapshot_from_manifest(&session, &table_url, manifest, 0)
        .await
        .expect("build_snapshot_from_manifest");

    assert_eq!(built.version(), eager.version(), "version must match");
    assert_eq!(built.schema(), eager.schema(), "logical schema must match");

    // End-to-end: register a scan on the manifest-built snapshot and count rows.
    let provider = DeltaSsaTableProvider::new(
        built,
        DeltaSsaScanConfig {
            schema_force_view_types: false,
        },
    )
    .unwrap();
    session
        .register_table("preview", Arc::new(provider))
        .unwrap();
    let batches = session
        .sql("SELECT count(*) AS n FROM preview")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let n = batches[0].column(0).as_primitive::<Int64Type>().value(0);
    assert_eq!(n, 6, "six rows across two data files");
}

/// Classic-checkpointed on-disk fixture (`app-txn-checkpoint`, checkpoint at v1): manifest-built
/// snapshot matches the eager build. This drives the checkpoint-footer `SchemaQuery` async read
/// (the #116 fix) — the P&M replay must read the checkpoint parquet.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn snapshot_from_manifest_matches_eager_classic_checkpoint() {
    // Vendored classic-checkpointed table (from delta-kernel-rs `kernel/tests/data`); see tests/data.
    let table_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/app-txn-checkpoint");
    let canonical = std::fs::canonicalize(table_dir)
        .unwrap_or_else(|e| panic!("app-txn-checkpoint fixture not found at {table_dir}: {e}"));
    let table_url = Url::from_directory_path(&canonical).unwrap();

    let store: Arc<dyn ObjectStore> = Arc::new(LocalFileSystem::new());

    let eager = eager_snapshot(Arc::clone(&store), &table_url, 1);
    assert_eq!(eager.version(), 1, "checkpoint fixture is at version 1");

    let manifest = manifest_for(store.as_ref(), &table_url).await;
    // Sanity: the manifest carries the checkpoint parquet (else this wouldn't exercise #116).
    assert!(
        manifest
            .iter()
            .any(|m| m.location.path().ends_with(".checkpoint.parquet")),
        "manifest must include the classic checkpoint parquet"
    );

    let session = session_with_store(&table_url, Arc::clone(&store));
    let built = build_snapshot_from_manifest(&session, &table_url, manifest, 1)
        .await
        .expect("build_snapshot_from_manifest (checkpointed)");

    assert_eq!(built.version(), eager.version(), "version must match");
    assert_eq!(built.schema(), eager.schema(), "logical schema must match");
}

/// Regression: a deep table root passed WITHOUT a trailing slash still resolves. This is the exact
/// shape `creds::resolve_storage` produces for a Unity Catalog managed table
/// (`…/__unitystorage/catalogs/<id>/tables/<uuid>`, no trailing `/`). `build_snapshot_from_manifest`
/// must anchor the trailing slash before `join("_delta_log/")`; otherwise `Url::join` drops the
/// `<uuid>` segment, the kernel resolves commits against `…/tables/_delta_log/`, and every read
/// 404s with a doubled `…/tables/_delta_log/…/tables/<uuid>/_delta_log/…json` path.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn deep_table_root_without_trailing_slash_resolves() {
    let prefix = "lakehouse/__unitystorage/catalogs/019f6d3e/tables/9e3a1584";
    let store = commit_only_store_at(prefix).await;

    // NO trailing slash — the production URL shape. `manifest_for` lists the real object paths and
    // builds full-from-root `memory:///lakehouse/.../tables/9e3a1584/_delta_log/…json` FileMeta URLs.
    let table_url = Url::parse(&format!("memory:///{prefix}")).unwrap();
    assert!(
        !table_url.path().ends_with('/'),
        "root must lack a trailing slash for the regression"
    );

    let manifest = manifest_for(store.as_ref(), &table_url).await;
    let session = session_with_store(&table_url, Arc::clone(&store) as Arc<dyn ObjectStore>);
    let built = build_snapshot_from_manifest(&session, &table_url, manifest, 0)
        .await
        .expect("build_snapshot_from_manifest must resolve a slash-less deep table root");
    assert_eq!(built.version(), 0);

    // End-to-end scan: proves data-file paths (`add.path`) also resolve against the anchored root.
    let provider = DeltaSsaTableProvider::new(
        built,
        DeltaSsaScanConfig {
            schema_force_view_types: false,
        },
    )
    .unwrap();
    session
        .register_table("preview", Arc::new(provider))
        .unwrap();
    let batches = session
        .sql("SELECT count(*) AS n FROM preview")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let n = batches[0].column(0).as_primitive::<Int64Type>().value(0);
    assert_eq!(
        n, 6,
        "six rows read back through a slash-less deep table root"
    );
}

/// Regression: driving `DeltaSsaTableProvider::scan()` on a **classic-checkpointed** table
/// succeeds end to end.
///
/// The scan's compiled `LogicalPlan` is optimized against the caller session (via
/// `session.create_physical_plan`). For a checkpointed table the FSR replay shape produces a
/// `named_struct` build that DataFusion's `push_down_leaf_projections` pass inlines into every
/// Filter leaf, yielding an ambiguous `scan.add`/`add` schema and failing the optimizer — UNLESS
/// the caller session disables `enable_leaf_expression_pushdown`, the same load-bearing override
/// the scan executor's own session sets (`replay_session_config`). `session_with_store` here — like
/// the real `query-wasm::engine::build_query_session` — sets it, so the checkpointed preview works.
/// (Commit-only tables never hit the ambiguous shape; regression-guarded here for the checkpoint
/// case, which `query-wasm`'s `resolve.rs` discovers rather than gates. apache/datafusion#20432.)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn scan_on_classic_checkpoint_succeeds() {
    let table_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/app-txn-checkpoint");
    let canonical = std::fs::canonicalize(table_dir)
        .unwrap_or_else(|e| panic!("app-txn-checkpoint fixture not found at {table_dir}: {e}"));
    let table_url = Url::from_directory_path(&canonical).unwrap();
    let store: Arc<dyn ObjectStore> = Arc::new(LocalFileSystem::new());

    let manifest = manifest_for(store.as_ref(), &table_url).await;
    let session = session_with_store(&table_url, Arc::clone(&store));
    let snapshot = build_snapshot_from_manifest(&session, &table_url, manifest, 1)
        .await
        .expect("snapshot construction (store-backed) resolves the checkpointed table");

    let provider = DeltaSsaTableProvider::new(
        snapshot,
        DeltaSsaScanConfig {
            schema_force_view_types: false,
        },
    )
    .expect("provider constructs");
    session
        .register_table("preview", Arc::new(provider))
        .unwrap();

    // The scan runs at physical planning; with leaf-pushdown disabled on the caller session the
    // checkpoint reconciliation plan no longer trips `push_down_leaf_projections`.
    let batches = session
        .sql("SELECT count(*) AS n FROM preview")
        .await
        .expect("SQL parses/plans")
        .collect()
        .await
        .expect("scanning a classic-checkpointed table succeeds");
    let n = batches[0].column(0).as_primitive::<Int64Type>().value(0);
    assert!(
        n >= 1,
        "checkpointed fixture must return a non-empty row count, got {n}"
    );
}
