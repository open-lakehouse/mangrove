//! [`ReconciledLogProvider`] over an in-memory fixture whose second commit tombstones a file, so
//! the reconciled add-file count is provably fewer than the total adds ever written.
//!
//! Builds a kernel `SnapshotRef` directly over the fixture, registers the engine-free provider on a
//! single-partition DataFusion session, and runs SQL over the reconciled scan-file rows. Mirrors
//! `m1_ssa_scan_native.rs`'s fixture/session/register pattern; the provider drives the metadata
//! state machine engine-free and the physical plan reads the commit log lazily.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use datafusion::catalog::TableProvider;
use datafusion::execution::context::SessionContext;
use delta_kernel::arrow::array::{ArrayRef, AsArray, Int64Array, RecordBatch, StringArray};
use delta_kernel::arrow::datatypes::{DataType, Field, Int64Type, Schema};
use delta_kernel::parquet::arrow::ArrowWriter;
use delta_kernel::parquet::basic::Compression;
use delta_kernel::parquet::file::properties::WriterProperties;
use delta_kernel::snapshot::{Snapshot, SnapshotRef};
use delta_kernel_default_engine::DefaultEngineBuilder;
use object_store::ObjectStore;
use object_store::ObjectStoreExt;
use object_store::memory::InMemory;
use object_store::path::Path;
use olai_delta_df::{
    DeltaEngineSessionOptions, DeltaSsaScanConfig, ReconciledLogProvider, delta_engine_session,
};
use url::Url;

const TABLE_PREFIX: &str = "tbl";

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

const SCHEMA_STRING: &str = r#"{\"type\":\"struct\",\"fields\":[{\"name\":\"id\",\"type\":\"long\",\"nullable\":true,\"metadata\":{}},{\"name\":\"name\",\"type\":\"string\",\"nullable\":true,\"metadata\":{}}]}"#;

/// A two-commit Delta table in an in-memory store:
///   * v0 adds `f0` and `f1` (2 adds).
///   * v1 removes `f0` (tombstone) and adds `f2` (1 add, 1 remove).
///
/// Total adds ever written: 3. Reconciled (surviving) adds at v1: `f1`, `f2` = 2.
async fn fixture_store() -> Arc<InMemory> {
    let store = InMemory::new();
    let files = [
        ("f0.snappy.parquet", batch(&[1, 2, 3], &["a", "b", "c"])),
        ("f1.snappy.parquet", batch(&[4, 5, 6], &["d", "e", "f"])),
        ("f2.snappy.parquet", batch(&[7, 8, 9], &["g", "h", "i"])),
    ];
    let mut sizes = std::collections::HashMap::new();
    for (name, data) in &files {
        let bytes = parquet_bytes(data);
        sizes.insert(*name, bytes.len());
        store
            .put(&Path::from(format!("{TABLE_PREFIX}/{name}")), bytes.into())
            .await
            .unwrap();
    }
    let add = |name: &str| {
        format!(
            r#"{{"add":{{"path":"{name}","partitionValues":{{}},"size":{size},"modificationTime":0,"dataChange":true}}}}"#,
            size = sizes[name],
        )
    };

    // v0: protocol + metadata + two adds.
    let commit0 = format!(
        concat!(
            r#"{{"protocol":{{"minReaderVersion":1,"minWriterVersion":2}}}}"#,
            "\n",
            r#"{{"metaData":{{"id":"11111111-2222-3333-4444-555555555555","format":{{"provider":"parquet","options":{{}}}},"schemaString":"{schema}","partitionColumns":[],"configuration":{{}},"createdTime":0}}}}"#,
            "\n{add0}\n{add1}\n",
        ),
        schema = SCHEMA_STRING,
        add0 = add("f0.snappy.parquet"),
        add1 = add("f1.snappy.parquet"),
    );
    // v1: remove f0, add f2.
    let commit1 = format!(
        concat!(
            r#"{{"remove":{{"path":"f0.snappy.parquet","deletionTimestamp":1,"dataChange":true}}}}"#,
            "\n{add2}\n",
        ),
        add2 = add("f2.snappy.parquet"),
    );

    for (version, commit) in [(0u64, commit0), (1u64, commit1)] {
        store
            .put(
                &Path::from(format!("{TABLE_PREFIX}/_delta_log/{version:020}.json")),
                commit.into_bytes().into(),
            )
            .await
            .unwrap();
    }
    Arc::new(store)
}

fn table_url() -> Url {
    Url::parse(&format!("memory:///{TABLE_PREFIX}/")).unwrap()
}

fn build_kernel_snapshot(store: Arc<InMemory>) -> SnapshotRef {
    let engine = DefaultEngineBuilder::new(store).build();
    Snapshot::builder_for(table_url().as_str())
        .build(&engine)
        .expect("kernel snapshot build")
}

fn session_with_store(store: Arc<InMemory>) -> SessionContext {
    delta_engine_session(
        store as Arc<dyn ObjectStore>,
        &table_url(),
        &DeltaEngineSessionOptions::wasm(),
    )
}

fn provider(store: &Arc<InMemory>) -> ReconciledLogProvider {
    let snapshot = build_kernel_snapshot(Arc::clone(store));
    assert_eq!(snapshot.version(), 1, "fixture is at v1");
    ReconciledLogProvider::new(
        snapshot,
        DeltaSsaScanConfig {
            schema_force_view_types: false,
        },
    )
    .expect("provider")
}

/// `count(*)` over the reconciled log yields the surviving add count (2), not the total adds (3):
/// `f0` was tombstoned in v1, `f1` and `f2` survive.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconciled_log_counts_surviving_adds() {
    let store = fixture_store().await;
    let ctx = session_with_store(Arc::clone(&store));
    ctx.register_table("reconciled_log", Arc::new(provider(&store)))
        .unwrap();

    let batches = ctx
        .sql("SELECT count(*) AS n FROM reconciled_log")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let n = batches[0].column(0).as_primitive::<Int64Type>().value(0);
    assert_eq!(
        n, 2,
        "2 surviving adds (f1, f2); f0 tombstoned, so < 3 total adds"
    );
}

/// `SELECT path` returns exactly the surviving file paths, in some order.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconciled_log_lists_surviving_paths() {
    let store = fixture_store().await;
    let ctx = session_with_store(Arc::clone(&store));
    ctx.register_table("reconciled_log", Arc::new(provider(&store)))
        .unwrap();

    let batches = ctx
        .sql("SELECT path FROM reconciled_log ORDER BY path")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let paths: Vec<String> = batches
        .iter()
        .flat_map(|b| {
            b.column(b.schema().index_of("path").unwrap())
                .as_string::<i32>()
                .iter()
                .map(|s| s.unwrap().to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(
        paths,
        vec![
            "f1.snappy.parquet".to_string(),
            "f2.snappy.parquet".to_string()
        ],
        "surviving add paths are f1 and f2",
    );
}

/// The advertised schema is the flat scan-file-row shape (`path`/`size`/`deletionVector`/
/// `fileConstantValues`/`stats`), and every field carries a logical (table-facing) name — never a
/// physical `col-…` / field-id leaf name.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconciled_log_schema_is_flat_scan_file_row_with_logical_names() {
    let store = fixture_store().await;
    let provider = provider(&store);
    let schema = TableProvider::schema(&provider);

    let top: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    // path/size/deletionVector/fileConstantValues always present; stats present because the
    // fixture has an indexed data column (`id`).
    assert!(top.contains(&"path"), "has path; got {top:?}");
    assert!(top.contains(&"size"), "has size; got {top:?}");
    assert!(
        top.contains(&"fileConstantValues"),
        "has fileConstantValues; got {top:?}"
    );

    // No `modificationTime` / string `stats` (that's the eager `scan_row_schema()` shape).
    assert!(
        !top.contains(&"modificationTime"),
        "sm_plans shape drops modificationTime; got {top:?}"
    );

    // Recursively assert no field name looks like a physical column-mapping name.
    fn assert_no_physical_names(fields: &delta_kernel::arrow::datatypes::Fields) {
        for f in fields.iter() {
            assert!(
                !f.name().starts_with("col-"),
                "field {:?} must carry a logical name, not a physical col-… name",
                f.name()
            );
            if let DataType::Struct(inner) = f.data_type() {
                assert_no_physical_names(inner);
            }
        }
    }
    assert_no_physical_names(schema.fields());
}

/// A predicate over an emitted column (`path`) is applied exactly — the provider reports `Exact`
/// and splices a `Filter` over the scan-file rows, so only the matching row survives. Proves the
/// no-translation filter pushdown works end-to-end.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconciled_log_filters_on_emitted_path_column() {
    let store = fixture_store().await;
    let ctx = session_with_store(Arc::clone(&store));
    ctx.register_table("reconciled_log", Arc::new(provider(&store)))
        .unwrap();

    // Equality on `path` selects exactly f2 (a surviving add).
    let batches = ctx
        .sql("SELECT path FROM reconciled_log WHERE path = 'f2.snappy.parquet'")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let paths: Vec<String> = batches
        .iter()
        .flat_map(|b| {
            b.column(b.schema().index_of("path").unwrap())
                .as_string::<i32>()
                .iter()
                .map(|s| s.unwrap().to_string())
                .collect::<Vec<_>>()
        })
        .collect();
    assert_eq!(paths, vec!["f2.snappy.parquet".to_string()]);

    // A predicate that also excludes the tombstoned file's path is a no-op (f0 already gone):
    // count stays 2. `size > 0` holds for both surviving files.
    let batches = ctx
        .sql("SELECT count(*) AS n FROM reconciled_log WHERE size > 0")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let n = batches[0].column(0).as_primitive::<Int64Type>().value(0);
    assert_eq!(n, 2, "both surviving files have size > 0");

    // A predicate that matches nothing yields zero rows (the filter really runs).
    let batches = ctx
        .sql("SELECT count(*) AS n FROM reconciled_log WHERE path = 'does-not-exist'")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let n = batches[0].column(0).as_primitive::<Int64Type>().value(0);
    assert_eq!(n, 0, "no file matches; filter is applied");
}
