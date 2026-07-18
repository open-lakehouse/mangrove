//! [`ActionsLogProvider`] over an in-memory fixture carrying every reconciled action slot: two
//! adds + a tombstone (add/remove reconciliation), plus a `metaData`, a `protocol`, a `txn`
//! (transaction id), and a `domainMetadata`. Asserts the provider surfaces the reconciled action
//! stream — not just surviving files — engine-free.
//!
//! Mirrors `m4_reconciled_log_native.rs`'s fixture / session / register pattern: builds a kernel
//! `SnapshotRef` over the fixture, registers the engine-free provider on a single-partition
//! DataFusion session, and runs SQL over the reconciled action rows. The provider drives the
//! kernel Full State coroutine engine-free and the physical plan reads the commit log lazily.

#![cfg(not(target_arch = "wasm32"))]

use std::sync::Arc;

use datafusion::catalog::TableProvider;
use datafusion::execution::context::SessionContext;
use delta_kernel::arrow::array::{
    Array, ArrayRef, AsArray, Int64Array, RecordBatch, StringArray, StructArray,
};
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
    ActionsLogProvider, DeltaEngineSessionOptions, DeltaSsaScanConfig, delta_engine_session,
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

/// A two-commit Delta table in an in-memory store carrying all reconciled action slots:
///   * v0: `protocol` + `metaData` + adds `f0`, `f1`.
///   * v1: removes `f0` (tombstone), adds `f2`, and writes a `txn` (appId `app-1`, version 7) and a
///     `domainMetadata` (`my.domain`).
///
/// Reconciled add-files at v1: `f1`, `f2`. Reconciled non-file actions: the latest `protocol`,
/// `metaData`, `txn` per appId, and `domainMetadata` per domain.
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
    // v1: remove f0, add f2, plus a txn and a domainMetadata.
    let commit1 = format!(
        concat!(
            r#"{{"remove":{{"path":"f0.snappy.parquet","deletionTimestamp":1,"dataChange":true}}}}"#,
            "\n",
            r#"{{"txn":{{"appId":"app-1","version":7,"lastUpdated":1}}}}"#,
            "\n",
            r#"{{"domainMetadata":{{"domain":"my.domain","configuration":"{{}}","removed":false}}}}"#,
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

fn provider(store: &Arc<InMemory>) -> ActionsLogProvider {
    let snapshot = build_kernel_snapshot(Arc::clone(store));
    assert_eq!(snapshot.version(), 1, "fixture is at v1");
    ActionsLogProvider::new(
        snapshot,
        DeltaSsaScanConfig {
            schema_force_view_types: false,
        },
    )
    .expect("provider")
}

/// Collect all reconciled action rows by driving the provider's scan directly.
///
/// Deliberately *not* via `ctx.sql("SELECT …")`: DataFusion's SQL identifier normalization
/// lower-cases unquoted names and its projection optimizer chokes on the camelCase action-slot
/// names (`metaData`, `domainMetadata`) — a SQL-surface quirk, orthogonal to the provider, which
/// emits the correct rows. Driving `scan()` + `collect()` exercises the real engine-free path (the
/// same physical plan any consumer executes) and preserves the schema verbatim.
async fn all_rows(ctx: &SessionContext, provider: &ActionsLogProvider) -> Vec<RecordBatch> {
    let state = ctx.state();
    let plan = TableProvider::scan(provider, &state, None, &[], None)
        .await
        .unwrap();
    datafusion::physical_plan::collect(plan, state.task_ctx())
        .await
        .unwrap()
}

/// Count rows belonging to an action slot, discriminated by an inner leaf field being non-null.
///
/// The reconciled stream materializes every top-level action slot as a *present* (non-null) struct
/// on every row, null-filling the inactive slots' inner fields (kernel reconciliation keys rows on
/// `add.path IS NOT NULL` / `remove.path IS NOT NULL`, not slot-level nullability). So "is this an
/// `add` row" is "`add`'s inner `path` leaf is non-null", not "`add` is non-null" — the latter is
/// true on every row. Evaluated at the Arrow level (the Delta engine session's `ExprPlanner` does
/// not register nested struct-field access, so `slot['leaf']` SQL is unavailable here).
fn count_by_leaf(batches: &[RecordBatch], slot: &str, leaf: &str) -> usize {
    batches
        .iter()
        .map(|b| {
            let col = b.column(b.schema().index_of(slot).unwrap());
            let st: &StructArray = col.as_struct();
            let leaf_col = st.column_by_name(leaf).expect("leaf field present");
            (0..leaf_col.len())
                .filter(|&i| leaf_col.is_valid(i))
                .count()
        })
        .sum()
}

/// The single non-null string value of `slot.leaf` across all rows (asserts exactly one).
fn single_string_leaf(batches: &[RecordBatch], slot: &str, leaf: &str) -> String {
    let mut found: Vec<String> = Vec::new();
    for b in batches {
        let col = b.column(b.schema().index_of(slot).unwrap());
        let st: &StructArray = col.as_struct();
        let leaf_col = st.column_by_name(leaf).unwrap().as_string::<i32>();
        for i in 0..leaf_col.len() {
            if leaf_col.is_valid(i) {
                found.push(leaf_col.value(i).to_string());
            }
        }
    }
    assert_eq!(found.len(), 1, "expected exactly one {slot}.{leaf}");
    found.pop().unwrap()
}

/// The single non-null i64 value of `slot.leaf` across all rows (asserts exactly one).
fn single_i64_leaf(batches: &[RecordBatch], slot: &str, leaf: &str) -> i64 {
    let mut found: Vec<i64> = Vec::new();
    for b in batches {
        let col = b.column(b.schema().index_of(slot).unwrap());
        let st: &StructArray = col.as_struct();
        let leaf_col = st.column_by_name(leaf).unwrap().as_primitive::<Int64Type>();
        for i in 0..leaf_col.len() {
            if leaf_col.is_valid(i) {
                found.push(leaf_col.value(i));
            }
        }
    }
    assert_eq!(found.len(), 1, "expected exactly one {slot}.{leaf}");
    found.pop().unwrap()
}

/// The advertised schema is the six reconciled action slots (FSR_BASE) — add/remove/metaData/
/// protocol/domainMetadata/txn — with `add.stats` surfaced as the parsed `add.stats_parsed` struct
/// (the provider always requests per-file struct stats).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actions_log_schema_is_the_six_reconciled_action_slots() {
    let store = fixture_store().await;
    let provider = provider(&store);
    let schema = TableProvider::schema(&provider);

    let top: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    for slot in [
        "add",
        "remove",
        "metaData",
        "protocol",
        "domainMetadata",
        "txn",
    ] {
        assert!(
            top.contains(&slot),
            "missing action slot {slot}; got {top:?}"
        );
    }

    // `add` carries the parsed stats struct, not the raw JSON `stats` string.
    let add = schema.field_with_name("add").unwrap();
    let DataType::Struct(add_fields) = add.data_type() else {
        panic!("add is a struct");
    };
    let add_names: Vec<&str> = add_fields.iter().map(|f| f.name().as_str()).collect();
    assert!(
        add_names.contains(&"stats_parsed"),
        "add.stats_parsed present (with_stats); got {add_names:?}"
    );
    assert!(
        !add_names.contains(&"stats"),
        "raw JSON add.stats replaced by add.stats_parsed; got {add_names:?}"
    );
}

/// The reconciled add-file count is the surviving adds (2: `f1`, `f2`), not the total adds (3) —
/// `f0` was tombstoned. Proves add/remove reconciliation is applied (this is the actions view, but
/// the file slots are still reconciled).
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actions_log_reconciles_add_files() {
    let store = fixture_store().await;
    let ctx = session_with_store(Arc::clone(&store));
    let provider = provider(&store);

    let rows = all_rows(&ctx, &provider).await;
    let n_add = count_by_leaf(&rows, "add", "path");
    assert_eq!(n_add, 2, "surviving adds f1, f2 (f0 tombstoned)");
}

/// The reconciled non-file actions surface: exactly one `protocol`, one `metaData`, one `txn`, and
/// one `domainMetadata` survive reconciliation, and their key fields carry the written values.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actions_log_surfaces_metadata_protocol_txn_domain() {
    let store = fixture_store().await;
    let ctx = session_with_store(Arc::clone(&store));
    let provider = provider(&store);

    let rows = all_rows(&ctx, &provider).await;
    assert_eq!(
        count_by_leaf(&rows, "protocol", "minReaderVersion"),
        1,
        "one protocol"
    );
    assert_eq!(count_by_leaf(&rows, "metaData", "id"), 1, "one metaData");
    assert_eq!(count_by_leaf(&rows, "txn", "appId"), 1, "one txn");
    assert_eq!(
        count_by_leaf(&rows, "domainMetadata", "domain"),
        1,
        "one domainMetadata"
    );

    // The txn carries the written appId + version (transaction id inspection — the headline use).
    assert_eq!(
        single_string_leaf(&rows, "txn", "appId"),
        "app-1",
        "txn.appId"
    );
    assert_eq!(single_i64_leaf(&rows, "txn", "version"), 7, "txn.version");

    // The metaData carries the written table id.
    assert_eq!(
        single_string_leaf(&rows, "metaData", "id"),
        "11111111-2222-3333-4444-555555555555",
        "metaData.id",
    );

    // The domainMetadata carries the written domain.
    assert_eq!(
        single_string_leaf(&rows, "domainMetadata", "domain"),
        "my.domain",
        "domainMetadata.domain",
    );
}

/// The driven Full State terminal's `RecordBatch` schema matches the provider's advertised schema
/// (`FullState::output_schema()`), so the pure accessor cannot drift from what the SM emits.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actions_log_driven_schema_matches_declared() {
    let store = fixture_store().await;
    let ctx = session_with_store(Arc::clone(&store));
    let provider = provider(&store);
    let declared = TableProvider::schema(&provider);

    // Drive the full scan (no projection/filter) and assert the physical plan's output schema is
    // exactly the declared schema — the kernel FSR terminal projects to `output_schema()`, dropping
    // the pipeline's internal bookkeeping columns.
    let state = ctx.state();
    let plan = TableProvider::scan(&provider, &state, None, &[], None)
        .await
        .unwrap();
    let got = plan.schema();
    let declared_names: Vec<&str> = declared
        .fields()
        .iter()
        .map(|f| f.name().as_str())
        .collect();
    let got_names: Vec<&str> = got.fields().iter().map(|f| f.name().as_str()).collect();
    assert_eq!(got_names, declared_names, "driven schema matches declared");

    // And the plan actually executes and yields the reconciled rows.
    let batches = datafusion::physical_plan::collect(plan, state.task_ctx())
        .await
        .unwrap();
    let rows: usize = batches.iter().map(|b| b.num_rows()).sum();
    assert_eq!(
        rows, 6,
        "2 surviving adds + protocol + metaData + txn + domainMetadata"
    );
}
