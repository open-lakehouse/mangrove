//! Native end-to-end test of the preview pipeline: log discovery → facade open
//! (forced onto the inline executor, i.e. the browser execution model) →
//! qualified-name registration → contract-framed IPC chunks.
//!
//! The fixture Delta table is generated in memory at test time (parquet bytes
//! via `ArrowWriter`, a handwritten commit) so no binary fixtures are committed
//! and the add-file sizes are always correct.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use arrow_array::{ArrayRef, Int64Array, RecordBatch, StringArray};
use arrow_ipc::reader::StreamReader;
use arrow_schema::{DataType, Field, Schema};
use futures::TryStreamExt;
use object_store::memory::InMemory;
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt as _};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use url::Url;

use deltalake_core::kernel::InlineExecutor;
use query_wasm::engine::{execute_chunks, extract_table, open_table, register_table};
use query_wasm::resolve::discover_log;

const TABLE_PREFIX: &str = "tbl";

fn schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, true),
        Field::new("name", DataType::Utf8, true),
    ]))
}

fn batch(ids: &[i64], names: &[&str]) -> RecordBatch {
    RecordBatch::try_new(
        schema(),
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
async fn fixture_store() -> Arc<dyn ObjectStore> {
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

/// Decode one self-contained IPC chunk into its batches (must parse standalone).
fn decode_chunk(ipc: &[u8]) -> Vec<RecordBatch> {
    StreamReader::try_new(std::io::Cursor::new(ipc), None)
        .expect("each chunk carries its own schema message")
        .collect::<Result<Vec<_>, _>>()
        .expect("chunk decodes standalone")
}

#[tokio::test]
async fn preview_pipeline_end_to_end() {
    let store = fixture_store().await;

    // Discovery: probe the log with no catalog version hint (external table).
    let log = discover_log(&store, &Path::from(TABLE_PREFIX), None)
        .await
        .unwrap();
    assert_eq!(log.version, 0);

    // Open on the inline executor — the browser execution model.
    let opened = open_table(store, &table_url(), log, Some(InlineExecutor.into()))
        .await
        .unwrap();
    assert_eq!(opened.snapshot.version(), 0);

    // Register under exactly the name the preview SQL uses.
    let sql = "SELECT * FROM `uc`.`sales`.`orders` ORDER BY `id`";
    let (reference, address) = extract_table(sql, None, None).unwrap();
    assert_eq!(address.full_name(), "uc.sales.orders");
    register_table(&opened.ctx, &opened, &reference).unwrap();

    // Execute with the runner-contract row cap.
    let chunks: Vec<_> = execute_chunks(&opened.ctx, sql, Some(4))
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    assert!(!chunks.is_empty());
    let mut ids = Vec::new();
    for chunk in &chunks {
        let batches = decode_chunk(&chunk.ipc);
        let rows: usize = batches.iter().map(RecordBatch::num_rows).sum();
        assert_eq!(rows, chunk.num_rows, "num_rows must match the payload");
        for b in &batches {
            let col = b.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
            ids.extend(col.iter().flatten());
        }
    }
    assert_eq!(ids, vec![1, 2, 3, 4], "LIMIT 4 over ORDER BY id");
}

#[tokio::test]
async fn empty_result_yields_one_schema_only_chunk() {
    let store = fixture_store().await;
    let log = discover_log(&store, &Path::from(TABLE_PREFIX), None)
        .await
        .unwrap();
    let opened = open_table(store, &table_url(), log, Some(InlineExecutor.into()))
        .await
        .unwrap();

    let sql = "SELECT * FROM `uc`.`sales`.`orders` WHERE `id` < 0";
    let (reference, _) = extract_table(sql, None, None).unwrap();
    register_table(&opened.ctx, &opened, &reference).unwrap();

    let chunks: Vec<_> = execute_chunks(&opened.ctx, sql, Some(100))
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    // The contract: a query with no rows still yields exactly one schema-only
    // message so the grid can render the column headers.
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].num_rows, 0);
    let batches = decode_chunk(&chunks[0].ipc);
    let rows: usize = batches.iter().map(RecordBatch::num_rows).sum();
    assert_eq!(rows, 0);
    let reader = StreamReader::try_new(std::io::Cursor::new(&chunks[0].ipc), None).unwrap();
    assert_eq!(reader.schema().fields().len(), 2);
}

#[tokio::test]
async fn bare_reference_resolves_via_session_defaults() {
    let store = fixture_store().await;
    let log = discover_log(&store, &Path::from(TABLE_PREFIX), None)
        .await
        .unwrap();
    let opened = open_table(store, &table_url(), log, Some(InlineExecutor.into()))
        .await
        .unwrap();

    // Bare name in SQL + request-level defaults: the UC address comes from the
    // defaults while DataFusion resolution lands in the session's default
    // catalog/schema — registration must follow the latter.
    let sql = "SELECT count(*) AS n FROM orders";
    let (reference, address) = extract_table(sql, Some("uc"), Some("sales")).unwrap();
    assert_eq!(address.full_name(), "uc.sales.orders");
    register_table(&opened.ctx, &opened, &reference).unwrap();

    let chunks: Vec<_> = execute_chunks(&opened.ctx, sql, None)
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();
    let batches = decode_chunk(&chunks[0].ipc);
    let n = batches[0]
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    assert_eq!(n, 6);
}
