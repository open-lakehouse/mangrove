//! M3 verification: the async-native `sm_plans` scan reads **column-mapped** Delta tables
//! correctly through the live [`FieldIdPhysicalExprAdapterFactory`] path (the deferred "M3"
//! verification from `handover-wasm-async-native-table-provider.md`).
//!
//! This is the first end-to-end exercise of column mapping against a *real* column-mapped table
//! (the crate's field-id adapter unit tests only use hand-built schemas). Each fixture writes
//! parquet files whose **physical** column names differ from the table's **logical** names, stamps
//! `PARQUET:field_id` on the parquet fields, and declares the logical↔physical relation in the
//! Delta `schemaString` via `delta.columnMapping.id` + `delta.columnMapping.physicalName`.
//!
//! Two modes are covered:
//!   * **name mode** (`delta.columnMapping.mode = "name"`) — physical name = the annotated
//!     `physicalName`; the kernel stamps field ids too.
//!   * **id mode** (`delta.columnMapping.mode = "id"`) — the physical schema is matched by
//!     `PARQUET:field_id`. The fixture deliberately gives the physical columns names that do **not**
//!     match the logical names, so a silent regression to name-fallback matching (instead of id
//!     matching) would surface as wrong/absent data.
//!
//! A third fixture adds a **partition column** to exercise the partition-passthrough path
//! (`load_helpers.rs` metadata strip) under column mapping.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::HashMap;
use std::sync::Arc;

use datafusion::catalog::TableProvider;
use datafusion::execution::context::SessionContext;
use delta_kernel::arrow::array::{ArrayRef, AsArray, Int64Array, RecordBatch, StringArray};
use delta_kernel::arrow::datatypes::{DataType, Field, Int64Type, Schema};
use delta_kernel::parquet::arrow::ArrowWriter;
use delta_kernel::parquet::arrow::PARQUET_FIELD_ID_META_KEY;
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

/// Field ids assigned to the two logical columns, shared across every fixture so the parquet
/// `PARQUET:field_id` and the Delta `delta.columnMapping.id` agree.
const ID_FIELD_ID: i64 = 1;
const NAME_FIELD_ID: i64 = 2;

/// Physical (on-disk) column names, deliberately different from the logical `id` / `name` so that
/// id-mode matching cannot silently fall back to name matching.
const ID_PHYSICAL: &str = "col-1a2b3c4d";
const NAME_PHYSICAL: &str = "col-5e6f7a8b";
const PART_PHYSICAL: &str = "col-9c0d1e2f";

/// Arrow schema for the data written to parquet: **physical** column names, each stamped with its
/// `PARQUET:field_id`. This is what the on-disk file actually contains under column mapping.
fn physical_arrow_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        field_with_id(ID_PHYSICAL, DataType::Int64, ID_FIELD_ID),
        field_with_id(NAME_PHYSICAL, DataType::Utf8, NAME_FIELD_ID),
    ]))
}

/// A nullable arrow [`Field`] carrying a `PARQUET:field_id` metadata annotation, so the parquet
/// writer emits the field id into the file schema (the join key the field-id adapter matches on).
fn field_with_id(name: &str, dt: DataType, id: i64) -> Field {
    let mut md = HashMap::new();
    md.insert(PARQUET_FIELD_ID_META_KEY.to_string(), id.to_string());
    Field::new(name, dt, true).with_metadata(md)
}

fn physical_batch(schema: &Arc<Schema>, ids: &[i64], names: &[&str]) -> RecordBatch {
    RecordBatch::try_new(
        Arc::clone(schema),
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

/// A column-mapped `schemaString` for a two-column (`id: long`, `name: string`) table, with each
/// field carrying `delta.columnMapping.id` + `delta.columnMapping.physicalName`. Logical names are
/// `id` / `name`; the physical names are the `col-*` uuids the parquet files use.
const CM_SCHEMA_STRING: &str = concat!(
    r#"{\"type\":\"struct\",\"fields\":["#,
    r#"{\"name\":\"id\",\"type\":\"long\",\"nullable\":true,"#,
    r#"\"metadata\":{\"delta.columnMapping.id\":1,\"delta.columnMapping.physicalName\":\"col-1a2b3c4d\"}},"#,
    r#"{\"name\":\"name\",\"type\":\"string\",\"nullable\":true,"#,
    r#"\"metadata\":{\"delta.columnMapping.id\":2,\"delta.columnMapping.physicalName\":\"col-5e6f7a8b\"}}"#,
    r#"]}"#,
);

/// Column-mapped schema with a trailing partition column `region: string`
/// (id=3, physicalName=`col-9c0d1e2f`).
const CM_SCHEMA_STRING_PARTITIONED: &str = concat!(
    r#"{\"type\":\"struct\",\"fields\":["#,
    r#"{\"name\":\"id\",\"type\":\"long\",\"nullable\":true,"#,
    r#"\"metadata\":{\"delta.columnMapping.id\":1,\"delta.columnMapping.physicalName\":\"col-1a2b3c4d\"}},"#,
    r#"{\"name\":\"name\",\"type\":\"string\",\"nullable\":true,"#,
    r#"\"metadata\":{\"delta.columnMapping.id\":2,\"delta.columnMapping.physicalName\":\"col-5e6f7a8b\"}},"#,
    r#"{\"name\":\"region\",\"type\":\"string\",\"nullable\":true,"#,
    r#"\"metadata\":{\"delta.columnMapping.id\":3,\"delta.columnMapping.physicalName\":\"col-9c0d1e2f\"}}"#,
    r#"]}"#,
);

/// Build a one-commit column-mapped Delta table in an in-memory store under [`TABLE_PREFIX`].
///
/// `mode` is `"name"` or `"id"`. Protocol is `minReaderVersion=2 / minWriterVersion=5` with the
/// `delta.columnMapping.mode` table property set — the version-2 form the kernel accepts without
/// explicit reader features (`column_mapping_mode`: `(Some(mode), 2) => mode`).
///
/// The two data files carry ids `[1,2,3]` and `[4,5,6]` under the **physical** column names.
async fn cm_fixture_store(mode: &str) -> Arc<InMemory> {
    let store = InMemory::new();
    let schema = physical_arrow_schema();
    let files = [
        (
            "part-00000.snappy.parquet",
            physical_batch(&schema, &[1, 2, 3], &["a", "b", "c"]),
        ),
        (
            "part-00001.snappy.parquet",
            physical_batch(&schema, &[4, 5, 6], &["d", "e", "f"]),
        ),
    ];
    let mut commit = format!(
        concat!(
            r#"{{"protocol":{{"minReaderVersion":2,"minWriterVersion":5}}}}"#,
            "\n",
            r#"{{"metaData":{{"id":"11111111-2222-3333-4444-555555555555","format":{{"provider":"parquet","options":{{}}}},"schemaString":"{schema}","partitionColumns":[],"configuration":{{"delta.columnMapping.mode":"{mode}"}},"createdTime":0}}}}"#,
            "\n",
        ),
        schema = CM_SCHEMA_STRING,
        mode = mode,
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
    put_commit(&store, commit).await;
    Arc::new(store)
}

/// Build a one-commit **partitioned** column-mapped Delta table: logical columns `id`, `name`, plus
/// a `region` partition column. Data files are laid out per Delta convention under the
/// **physical** partition path (`<physicalName>=<value>/…`) and only carry the non-partition
/// (`id`, `name`) columns; the partition value comes from the add action's `partitionValues`
/// (keyed by the **physical** partition-column name).
async fn cm_partitioned_fixture_store(mode: &str) -> Arc<InMemory> {
    let store = InMemory::new();
    let schema = physical_arrow_schema();
    // Two partitions: region=west (ids 1,2,3) and region=east (ids 4,5,6).
    let files = [
        (
            "west",
            "part-00000.snappy.parquet",
            physical_batch(&schema, &[1, 2, 3], &["a", "b", "c"]),
        ),
        (
            "east",
            "part-00001.snappy.parquet",
            physical_batch(&schema, &[4, 5, 6], &["d", "e", "f"]),
        ),
    ];
    let mut commit = format!(
        concat!(
            r#"{{"protocol":{{"minReaderVersion":2,"minWriterVersion":5}}}}"#,
            "\n",
            r#"{{"metaData":{{"id":"11111111-2222-3333-4444-555555555555","format":{{"provider":"parquet","options":{{}}}},"schemaString":"{schema}","partitionColumns":["region"],"configuration":{{"delta.columnMapping.mode":"{mode}"}},"createdTime":0}}}}"#,
            "\n",
        ),
        schema = CM_SCHEMA_STRING_PARTITIONED,
        mode = mode,
    );
    for (region, name, data) in &files {
        let bytes = parquet_bytes(data);
        // Physical partition directory layout: `<physical partition name>=<value>/<file>`.
        let rel = format!("{PART_PHYSICAL}={region}/{name}");
        commit.push_str(&format!(
            r#"{{"add":{{"path":"{rel}","partitionValues":{{"{PART_PHYSICAL}":"{region}"}},"size":{size},"modificationTime":0,"dataChange":true}}}}"#,
            size = bytes.len(),
        ));
        commit.push('\n');
        store
            .put(&Path::from(format!("{TABLE_PREFIX}/{rel}")), bytes.into())
            .await
            .unwrap();
    }
    put_commit(&store, commit).await;
    Arc::new(store)
}

async fn put_commit(store: &InMemory, commit: String) {
    store
        .put(
            &Path::from(format!(
                "{TABLE_PREFIX}/_delta_log/00000000000000000000.json"
            )),
            commit.into_bytes().into(),
        )
        .await
        .unwrap();
}

fn table_url() -> Url {
    Url::parse(&format!("memory:///{TABLE_PREFIX}/")).unwrap()
}

/// Build a kernel `SnapshotRef` at version 0 (DefaultEngine used only for log replay; the provider
/// itself is engine-free).
fn build_kernel_snapshot(store: Arc<InMemory>) -> SnapshotRef {
    let engine = DefaultEngineBuilder::new(store).build();
    Snapshot::builder_for(table_url().as_str())
        .at_version(0)
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

fn provider_for(store: &Arc<InMemory>) -> DeltaSsaTableProvider {
    let snapshot = build_kernel_snapshot(Arc::clone(store));
    assert_eq!(snapshot.version(), 0);
    DeltaSsaTableProvider::new(
        snapshot,
        DeltaSsaScanConfig {
            schema_force_view_types: false,
        },
    )
    .expect("provider")
}

/// Collect `id` and `name` columns from a `SELECT id, name … ORDER BY id` result, as parallel
/// vectors of `(i64, String)`.
fn collect_id_name(batches: &[RecordBatch]) -> Vec<(i64, String)> {
    let mut out = Vec::new();
    for b in batches {
        let ids = b
            .column(b.schema().index_of("id").unwrap())
            .as_primitive::<Int64Type>();
        let names = b
            .column(b.schema().index_of("name").unwrap())
            .as_string::<i32>();
        for i in 0..b.num_rows() {
            out.push((ids.value(i), names.value(i).to_string()));
        }
    }
    out
}

/// The provider's `schema()` exposes the **logical** column names (`id`, `name`) — never the
/// physical `col-*` names — for both column-mapping modes.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn schema_exposes_logical_names_both_modes() {
    for mode in ["name", "id"] {
        let store = cm_fixture_store(mode).await;
        let provider = provider_for(&store);
        let schema = provider.schema();
        let names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert_eq!(
            names,
            vec!["id", "name"],
            "schema() must expose logical names in {mode} mode, got {names:?}"
        );
        // Guard against leaking a physical name.
        for f in schema.fields() {
            assert!(
                !f.name().starts_with("col-"),
                "physical name {} leaked into schema() in {mode} mode",
                f.name()
            );
        }
    }
}

/// A `SELECT id, name` over the column-mapped table is byte/row-identical to the non-column-mapped
/// twin (`m1`'s `[1..6]` / `id: long, name: string` shape), under **both** name and id mode.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn select_matches_non_cm_twin_both_modes() {
    let expected = vec![
        (1, "a".to_string()),
        (2, "b".to_string()),
        (3, "c".to_string()),
        (4, "d".to_string()),
        (5, "e".to_string()),
        (6, "f".to_string()),
    ];
    for mode in ["name", "id"] {
        let store = cm_fixture_store(mode).await;
        let provider = provider_for(&store);
        let ctx = session_with_store(store);
        ctx.register_table("preview", Arc::new(provider)).unwrap();

        let batches = ctx
            .sql("SELECT id, name FROM preview ORDER BY id")
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();

        // Output columns are plain Utf8 (never Utf8View — mangrove #28), through the rename chain.
        let name_field = batches[0].schema().field_with_name("name").unwrap().clone();
        assert_eq!(
            name_field.data_type(),
            &DataType::Utf8,
            "name must be plain Utf8 in {mode} mode"
        );

        let rows = collect_id_name(&batches);
        assert_eq!(
            rows, expected,
            "column-mapped {mode}-mode read must match the non-CM twin"
        );
    }
}

/// **id-mode resolves by field id under renamed physical columns.** The physical parquet columns
/// are named `col-1a2b3c4d` / `col-5e6f7a8b` — deliberately *not* `id` / `name` — so the only way a
/// `SELECT id, name` can return correct data in id mode is by matching `PARQUET:field_id`
/// (1 and 2). If a regression made the adapter fall back to name matching, no physical column would
/// match the logical `id`/`name` and the read would be all-null or error. This guards that silent
/// name-fallback path.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn id_mode_resolves_by_field_id_not_name() {
    let store = cm_fixture_store("id").await;
    let provider = provider_for(&store);
    let ctx = session_with_store(store);
    ctx.register_table("preview", Arc::new(provider)).unwrap();

    let batches = ctx
        .sql("SELECT id, name FROM preview ORDER BY id LIMIT 3")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    let rows = collect_id_name(&batches);
    assert_eq!(
        rows,
        vec![
            (1, "a".to_string()),
            (2, "b".to_string()),
            (3, "c".to_string()),
        ],
        "id-mode must resolve physical columns by field id even though physical names differ"
    );
    // Non-empty proves the field-id match actually bound to on-disk data (a name-fallback miss
    // would NULL-fill or error, never yield 'a'/'b'/'c').
    assert!(
        rows.iter().all(|(_, n)| !n.is_empty()),
        "id-mode read produced empty name values — field-id matching likely regressed to name-fallback"
    );
}

/// `count(*)` sees all six rows across both data files, both modes.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn counts_all_rows_both_modes() {
    for mode in ["name", "id"] {
        let store = cm_fixture_store(mode).await;
        let provider = provider_for(&store);
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
        assert_eq!(n, 6, "{mode}-mode table has six rows across two data files");
    }
}

/// A **partitioned** column-mapped table returns correct partition-column values alongside the
/// mapped data columns, under both modes. This exercises the partition-passthrough path
/// (`load_helpers.rs` metadata strip) for column-mapped tables — the value flows from the add
/// action's `partitionValues` (keyed by physical name) up to the logical `region` column.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn partition_column_values_correct_both_modes() {
    for mode in ["name", "id"] {
        let store = cm_partitioned_fixture_store(mode).await;
        let provider = provider_for(&store);

        // schema() exposes the logical partition name.
        let schema = provider.schema();
        let names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        assert!(
            names.contains(&"region"),
            "{mode}-mode partitioned schema must expose logical `region`, got {names:?}"
        );

        let ctx = session_with_store(store);
        ctx.register_table("preview", Arc::new(provider)).unwrap();

        let batches = ctx
            .sql("SELECT id, region FROM preview ORDER BY id")
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();

        let mut rows: Vec<(i64, String)> = Vec::new();
        for b in &batches {
            let ids = b
                .column(b.schema().index_of("id").unwrap())
                .as_primitive::<Int64Type>();
            let regions = b
                .column(b.schema().index_of("region").unwrap())
                .as_string::<i32>();
            for i in 0..b.num_rows() {
                rows.push((ids.value(i), regions.value(i).to_string()));
            }
        }
        assert_eq!(
            rows,
            vec![
                (1, "west".to_string()),
                (2, "west".to_string()),
                (3, "west".to_string()),
                (4, "east".to_string()),
                (5, "east".to_string()),
                (6, "east".to_string()),
            ],
            "{mode}-mode partition values must map to the correct rows"
        );
    }
}
