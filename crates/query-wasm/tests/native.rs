//! Native end-to-end test of the preview pipeline: log discovery → async-native
//! snapshot construction (no `PrimedStore` prefetch, no inline executor) →
//! qualified-name registration → contract-framed IPC chunks.
//!
//! The fixture Delta table is generated in memory at test time (parquet bytes
//! via `ArrowWriter`, a handwritten commit) so no binary fixtures are committed
//! and the add-file sizes are always correct.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use arrow_array::{ArrayRef, Int64Array, RecordBatch, StringArray};
use arrow_ipc::reader::StreamReader;
use arrow_schema::{DataType, Field, Fields, Schema};
use futures::TryStreamExt;
use object_store::memory::InMemory;
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt as _};
use parquet::arrow::ArrowWriter;
use parquet::basic::Compression;
use parquet::file::properties::WriterProperties;
use url::Url;

use query_wasm::engine::{
    LogKind, execute_chunks, extract_table, open_table, register_table, scan_log_chunks,
};
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

    // Async-native construction: no prime, no inline executor.
    let opened = open_table(store, &table_url(), log, None).await.unwrap();
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
    let opened = open_table(store, &table_url(), log, None).await.unwrap();

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
    let opened = open_table(store, &table_url(), log, None).await.unwrap();

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

/// A fixture identical to [`fixture_store`] but whose protocol advertises the
/// `catalogManaged` table feature — the kernel then refuses to build a snapshot
/// unless `max_catalog_version` is supplied. This is the shape a Unity Catalog
/// managed table has, and the regression the wasm engine must handle.
async fn catalog_managed_fixture_store() -> Arc<dyn ObjectStore> {
    let store = InMemory::new();
    let data = batch(&[1, 2, 3], &["a", "b", "c"]);
    let bytes = parquet_bytes(&data);
    let name = "part-00000.snappy.parquet";
    let commit = format!(
        concat!(
            r#"{{"commitInfo":{{"timestamp":0,"inCommitTimestamp":0}}}}"#,
            "\n",
            r#"{{"protocol":{{"minReaderVersion":3,"minWriterVersion":7,"readerFeatures":["catalogManaged","v2Checkpoint"],"writerFeatures":["catalogManaged","v2Checkpoint","inCommitTimestamp"]}}}}"#,
            "\n",
            r#"{{"metaData":{{"id":"11111111-2222-3333-4444-555555555555","format":{{"provider":"parquet","options":{{}}}},"schemaString":"{schema}","partitionColumns":[],"configuration":{{"delta.enableInCommitTimestamps":"true"}},"createdTime":0}}}}"#,
            "\n",
            r#"{{"add":{{"path":"{name}","partitionValues":{{}},"size":{size},"modificationTime":0,"dataChange":true}}}}"#,
            "\n",
        ),
        schema = SCHEMA_STRING,
        name = name,
        size = bytes.len(),
    );
    store
        .put(&Path::from(format!("{TABLE_PREFIX}/{name}")), bytes.into())
        .await
        .unwrap();
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

/// Regression: a catalog-managed table opens and queries when the catalog's
/// latest version is threaded through as `max_catalog_version`. Without it the
/// kernel errors "Catalog-managed table requires max_catalog_version to be set".
#[tokio::test]
async fn catalog_managed_table_opens_with_max_catalog_version() {
    let store = catalog_managed_fixture_store().await;

    // Managed tables: the catalog reports the latest ratified version, which
    // bounds discovery and must be passed to the kernel as max_catalog_version.
    let latest = Some(0u64);
    let log = discover_log(&store, &Path::from(TABLE_PREFIX), latest)
        .await
        .unwrap();
    assert_eq!(log.version, 0);

    let opened = open_table(store, &table_url(), log, latest)
        .await
        .expect("catalog-managed table must open when max_catalog_version is supplied");
    assert_eq!(opened.snapshot.version(), 0);

    let sql = "SELECT * FROM `uc`.`sales`.`orders` ORDER BY `id`";
    let (reference, _) = extract_table(sql, None, None).unwrap();
    register_table(&opened.ctx, &opened, &reference).unwrap();

    let chunks: Vec<_> = execute_chunks(&opened.ctx, sql, Some(100))
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    // The emitted IPC schema must not use Arrow "view" types: the browser-side
    // apache-arrow (v21) reader cannot decode Utf8View/BinaryView and fails with
    // "Unrecognized type: undefined (24)". The `name` (string) column is the one
    // arrow-rs would otherwise materialize as Utf8View.
    let ipc_schema = StreamReader::try_new(std::io::Cursor::new(&chunks[0].ipc), None)
        .unwrap()
        .schema();
    for f in ipc_schema.fields() {
        assert!(
            !matches!(f.data_type(), DataType::Utf8View | DataType::BinaryView),
            "column {} emitted a view type ({:?}); the JS IPC reader cannot decode it",
            f.name(),
            f.data_type(),
        );
    }
    let name_type = ipc_schema.field_with_name("name").unwrap().data_type();
    assert_eq!(
        name_type,
        &DataType::Utf8,
        "string column must be plain Utf8"
    );

    let mut ids = Vec::new();
    for chunk in &chunks {
        for b in decode_chunk(&chunk.ipc) {
            let col = b.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
            ids.extend(col.iter().flatten());
        }
    }
    assert_eq!(ids, vec![1, 2, 3], "catalog-managed table rows read back");
}

/// The whole pipeline is inline-executor-free.
///
/// Both snapshot construction and the scan are async-native now: `open_table` drives the kernel
/// P&M state machine (no `PrimedStore` prefetch), and the `DeltaSsaTableProvider` scan needs no
/// executor. This test asserts the preview still produces the `[1, 2, 3, 4]` oracle output with no
/// inline executor anywhere on the path.
#[tokio::test]
async fn scan_runs_with_no_inline_executor() {
    let store = fixture_store().await;
    let log = discover_log(&store, &Path::from(TABLE_PREFIX), None)
        .await
        .unwrap();

    let opened = open_table(store, &table_url(), log, None).await.unwrap();
    assert_eq!(opened.snapshot.version(), 0);

    let sql = "SELECT * FROM `uc`.`sales`.`orders` ORDER BY `id`";
    let (reference, _) = extract_table(sql, None, None).unwrap();
    register_table(&opened.ctx, &opened, &reference).unwrap();

    let chunks: Vec<_> = execute_chunks(&opened.ctx, sql, Some(4))
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    let mut ids = Vec::new();
    for chunk in &chunks {
        for b in decode_chunk(&chunk.ipc) {
            let col = b.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
            ids.extend(col.iter().flatten());
        }
    }
    assert_eq!(
        ids,
        vec![1, 2, 3, 4],
        "scan produces the oracle output with no inline executor"
    );
}

/// Recursively assert no field (including nested struct/list children) uses an
/// Arrow "view" type. The log schemas carry deeply nested structs
/// (`stats`/`fileConstantValues` on the reconciled surface; the six action
/// slots on the action surface), so a view type could hide below the top level
/// and break the browser arrow-js IPC reader (mangrove #28).
fn assert_no_view_types(fields: &Fields) {
    for f in fields {
        assert!(
            !matches!(f.data_type(), DataType::Utf8View | DataType::BinaryView),
            "field {} emitted a view type ({:?}); the JS IPC reader cannot decode it",
            f.name(),
            f.data_type(),
        );
        match f.data_type() {
            DataType::Struct(children) => assert_no_view_types(children),
            DataType::List(child)
            | DataType::LargeList(child)
            | DataType::FixedSizeList(child, _)
            | DataType::Map(child, _) => assert_no_view_types(&Fields::from(vec![child.clone()])),
            _ => {}
        }
    }
}

/// The reconciled-log provider registers and scans over the browser-shaped
/// session: `SELECT * FROM reconciled_log` yields one row per surviving add file
/// with flat `path`/`size` columns and no view types anywhere in the schema.
#[tokio::test]
async fn reconciled_log_registers_and_scans() {
    let store = fixture_store().await;
    let log = discover_log(&store, &Path::from(TABLE_PREFIX), None)
        .await
        .unwrap();
    let opened = open_table(store, &table_url(), log, None).await.unwrap();

    let chunks: Vec<_> = scan_log_chunks(&opened.ctx, &opened, LogKind::Reconciled, Some(100))
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    let ipc_schema = StreamReader::try_new(std::io::Cursor::new(&chunks[0].ipc), None)
        .unwrap()
        .schema();
    assert_no_view_types(ipc_schema.fields());
    // Flat scan-file-row shape: `path` (non-null string) and `size` (long).
    assert_eq!(
        ipc_schema.field_with_name("path").unwrap().data_type(),
        &DataType::Utf8,
        "reconciled `path` must be plain Utf8"
    );
    assert!(matches!(
        ipc_schema.field_with_name("size").unwrap().data_type(),
        DataType::Int64
    ));

    let rows: usize = chunks.iter().map(|c| c.num_rows).sum();
    assert_eq!(
        rows, 2,
        "one row per surviving add file (fixture has two adds, no removes)"
    );
}

/// The action-log provider registers and scans: `SELECT * FROM action_log`
/// emits the reconciled six-slot action stream (`add`, `remove`, `metaData`,
/// `protocol`, `domainMetadata`, `txn`), all nullable structs, no view types.
#[tokio::test]
async fn action_log_registers_and_scans() {
    let store = fixture_store().await;
    let log = discover_log(&store, &Path::from(TABLE_PREFIX), None)
        .await
        .unwrap();
    let opened = open_table(store, &table_url(), log, None).await.unwrap();

    let chunks: Vec<_> = scan_log_chunks(&opened.ctx, &opened, LogKind::Actions, Some(100))
        .await
        .unwrap()
        .try_collect()
        .await
        .unwrap();

    let ipc_schema = StreamReader::try_new(std::io::Cursor::new(&chunks[0].ipc), None)
        .unwrap()
        .schema();
    assert_no_view_types(ipc_schema.fields());

    // The FSR six-slot top-level shape: each slot is a struct. (Reconciliation
    // materializes every slot as a *present* struct on every row and keys "is
    // this an add row" on an inner leaf being non-null, not on slot-level
    // nullability — so we assert structure, not nullability.)
    for slot in [
        "add",
        "remove",
        "metaData",
        "protocol",
        "domainMetadata",
        "txn",
    ] {
        let field = ipc_schema
            .field_with_name(slot)
            .unwrap_or_else(|_| panic!("action_log missing slot `{slot}`"));
        assert!(
            matches!(field.data_type(), DataType::Struct(_)),
            "slot `{slot}` must be a struct, got {:?}",
            field.data_type()
        );
    }

    // The fixture's single commit reconciles to at least an add + metaData +
    // protocol row.
    let rows: usize = chunks.iter().map(|c| c.num_rows).sum();
    assert!(rows >= 1, "reconciled action stream must carry rows");
}
