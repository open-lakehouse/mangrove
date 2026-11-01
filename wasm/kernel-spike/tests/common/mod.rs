//! Shared in-memory Delta fixtures (hand-written commits + snappy parquet data files), usable on
//! native and wasm32.

use std::sync::Arc;

use buoyant_kernel::arrow::array::{ArrayRef, Int64Array, RecordBatch, StringArray};
use buoyant_kernel::arrow::datatypes::{DataType, Field, Schema};
use buoyant_kernel::parquet::arrow::ArrowWriter;
use buoyant_kernel::parquet::basic::Compression;
use buoyant_kernel::parquet::file::properties::WriterProperties;
use object_store::memory::InMemory;
use object_store::path::Path;
use object_store::{ObjectStore, ObjectStoreExt as _};
use url::Url;

pub const TABLE: &str = "tbl";

const SCHEMA_STRING: &str = r#"{\"type\":\"struct\",\"fields\":[{\"name\":\"id\",\"type\":\"long\",\"nullable\":true,\"metadata\":{}},{\"name\":\"name\",\"type\":\"string\",\"nullable\":true,\"metadata\":{}}]}"#;

pub fn table_url() -> Url {
    Url::parse(&format!("memory:///{TABLE}/")).unwrap()
}

fn parquet_bytes(ids: &[i64]) -> Vec<u8> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, true),
        Field::new("name", DataType::Utf8, true),
    ]));
    let names: Vec<String> = ids.iter().map(|i| format!("n{i}")).collect();
    let batch = RecordBatch::try_new(
        schema.clone(),
        vec![
            Arc::new(Int64Array::from(ids.to_vec())) as ArrayRef,
            Arc::new(StringArray::from(names)) as ArrayRef,
        ],
    )
    .unwrap();
    let props = WriterProperties::builder()
        .set_compression(Compression::SNAPPY)
        .build();
    let mut buf = Vec::new();
    let mut writer = ArrowWriter::try_new(&mut buf, schema, Some(props)).unwrap();
    writer.write(&batch).unwrap();
    writer.close().unwrap();
    buf
}

async fn put(store: &InMemory, path: &str, bytes: Vec<u8>) {
    store
        .put(&Path::from(format!("{TABLE}/{path}")), bytes.into())
        .await
        .unwrap();
}

async fn add(store: &InMemory, name: &str, ids: &[i64]) -> String {
    let bytes = parquet_bytes(ids);
    let line = format!(
        r#"{{"add":{{"path":"{name}","partitionValues":{{}},"size":{size},"modificationTime":0,"dataChange":true}}}}"#,
        size = bytes.len(),
    );
    put(store, name, bytes).await;
    line
}

/// Three commits:
/// - v0: protocol + metadata + `a` (ids 1..=3) + `b` (4..=6)
/// - v1: `c` (7..=8)
/// - v2: remove `a`, add `d` (9)
///
/// Live rows at v2: b + c + d = 3 + 2 + 1 = 6.
pub async fn three_commit_table() -> Arc<InMemory> {
    let store = InMemory::new();
    let mut v0 = format!(
        concat!(
            r#"{{"protocol":{{"minReaderVersion":1,"minWriterVersion":2}}}}"#,
            "\n",
            r#"{{"metaData":{{"id":"11111111-2222-3333-4444-555555555555","format":{{"provider":"parquet","options":{{}}}},"schemaString":"{schema}","partitionColumns":[],"configuration":{{}},"createdTime":0}}}}"#,
            "\n",
        ),
        schema = SCHEMA_STRING,
    );
    for (name, ids) in [("a.parquet", &[1, 2, 3][..]), ("b.parquet", &[4, 5, 6][..])] {
        v0.push_str(&add(&store, name, ids).await);
        v0.push('\n');
    }
    put(
        &store,
        "_delta_log/00000000000000000000.json",
        v0.into_bytes(),
    )
    .await;

    let v1 = add(&store, "c.parquet", &[7, 8]).await + "\n";
    put(
        &store,
        "_delta_log/00000000000000000001.json",
        v1.into_bytes(),
    )
    .await;

    let v2 = format!(
        "{}\n{}\n",
        r#"{"remove":{"path":"a.parquet","deletionTimestamp":0,"dataChange":true}}"#,
        add(&store, "d.parquet", &[9]).await
    );
    put(
        &store,
        "_delta_log/00000000000000000002.json",
        v2.into_bytes(),
    )
    .await;
    Arc::new(store)
}

/// Sorted paths of every object in `store` (diagnostics).
#[allow(dead_code)]
pub async fn object_paths(store: &dyn ObjectStore) -> Vec<String> {
    use futures::TryStreamExt as _;
    let mut paths: Vec<String> = store
        .list(None)
        .map_ok(|m| m.location.to_string())
        .try_collect()
        .await
        .unwrap();
    paths.sort();
    paths
}
