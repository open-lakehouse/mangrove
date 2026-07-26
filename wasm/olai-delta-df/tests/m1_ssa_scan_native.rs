//! The `sm_plans` SSA scan runs engine-free over an in-memory two-file fixture table (ids
//! `[1..6]`, `id: long / name: string`).
//!
//! Builds a kernel `SnapshotRef` directly over the fixture, registers a
//! [`DeltaSsaTableProvider`] on a single-partition DataFusion session, and runs preview SQL. The
//! provider drives the scan state machine engine-free; the physical plan reads the parquet data
//! files lazily through DataFusion's own async object-store stack.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

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
    DeltaEngineSessionOptions, DeltaSsaScanConfig, DeltaSsaTableProvider, delta_engine_session,
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

/// Build a one-commit Delta table in an in-memory store under [`TABLE_PREFIX`].
async fn fixture_store() -> Arc<InMemory> {
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
            .put(&Path::from(format!("{TABLE_PREFIX}/{name}")), bytes.into())
            .await
            .unwrap();
    }
    store
        .put(
            &Path::from(format!(
                "{TABLE_PREFIX}/_delta_log/00000000000000000000.json"
            )),
            commit.into_bytes().into(),
        )
        .await
        .unwrap();
    Arc::new(store)
}

fn table_url() -> Url {
    Url::parse(&format!("memory:///{TABLE_PREFIX}/")).unwrap()
}

/// Build a kernel `SnapshotRef` over `store` at version 0, using a DefaultEngine purely for
/// snapshot construction (log replay). The provider itself is engine-free.
fn build_kernel_snapshot(store: Arc<InMemory>) -> SnapshotRef {
    let engine = DefaultEngineBuilder::new(store).build();
    Snapshot::builder_for(table_url().as_str())
        .at_version(0)
        .build(&engine)
        .expect("kernel snapshot build")
}

/// Delta-engine session with the fixture store registered. Uses the `wasm` preset (view types off
/// — the browser IPC reader cannot decode `Utf8View`) even though this is a native test binary.
fn session_with_store(store: Arc<InMemory>) -> SessionContext {
    delta_engine_session(
        store as Arc<dyn ObjectStore>,
        &table_url(),
        &DeltaEngineSessionOptions::wasm(),
    )
}

/// `LIMIT 4 ORDER BY id` yields `[1,2,3,4]` and the `name` column is plain `Utf8` (never
/// `Utf8View`).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ssa_scan_matches_inline_executor_oracle() {
    let store = fixture_store().await;
    let snapshot = build_kernel_snapshot(Arc::clone(&store));
    assert_eq!(snapshot.version(), 0);

    let provider = DeltaSsaTableProvider::new(
        snapshot,
        DeltaSsaScanConfig {
            schema_force_view_types: false,
        },
    )
    .expect("provider");

    let ctx = session_with_store(store);
    ctx.register_table("preview", Arc::new(provider)).unwrap();

    let df = ctx
        .sql("SELECT id, name FROM preview ORDER BY id LIMIT 4")
        .await
        .unwrap();
    let batches = df.collect().await.unwrap();

    // Row identity: [1, 2, 3, 4].
    let ids: Vec<i64> = batches
        .iter()
        .flat_map(|b| {
            b.column(b.schema().index_of("id").unwrap())
                .as_primitive::<Int64Type>()
                .values()
                .to_vec()
        })
        .collect();
    assert_eq!(
        ids,
        vec![1, 2, 3, 4],
        "LIMIT 4 ORDER BY id yields [1,2,3,4]"
    );

    // View-type override: name must be plain Utf8, never Utf8View/BinaryView.
    let name_field = batches[0].schema().field_with_name("name").unwrap().clone();
    assert_eq!(
        name_field.data_type(),
        &DataType::Utf8,
        "name column must be plain Utf8 (schema_force_view_types=false)"
    );
    for batch in &batches {
        for field in batch.schema().fields() {
            assert!(
                !matches!(field.data_type(), DataType::Utf8View | DataType::BinaryView),
                "no view types allowed in output; found {field:?}"
            );
        }
    }
}

/// `count(*)` over the whole table sees all six rows across both data files — proving the scan
/// reads every add-file, lazily, through DataFusion's async parquet source (no eager prefetch).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn ssa_scan_counts_all_rows() {
    let store = fixture_store().await;
    let snapshot = build_kernel_snapshot(Arc::clone(&store));
    let provider = DeltaSsaTableProvider::new(
        snapshot,
        DeltaSsaScanConfig {
            schema_force_view_types: false,
        },
    )
    .expect("provider");

    let ctx = session_with_store(store);
    ctx.register_table("preview", Arc::new(provider)).unwrap();

    let batches = ctx
        .sql("SELECT count(*) AS n FROM preview")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let n = batches[0].column(0).as_primitive::<Int64Type>().value(0);
    assert_eq!(n, 6, "table has six rows across two data files");
}
