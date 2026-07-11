//! End-to-end tests for the Delta Log Explorer providers (Phase 1 wiring).
//!
//! Builds a synthetic Delta table on a local `file://` store — several commits
//! (adds, a metaData, a checkpoint, and a tombstoning overwrite) — then drives
//! both providers through a delta-kernel [`DefaultEngine`] and asserts:
//!
//! - **raw** materializes every action in the frozen log segment
//!   (checkpoint@v1 + commit v2): the checkpoint's live adds + protocol +
//!   metaData (`_commit = false`) and v2's overwrite — its removes + new add
//!   (`_commit = true`);
//! - **reconciled** materializes only the surviving add-files after replay;
//! - **raw ⊇ reconciled**: every reconciled add path also appears (as an add) in
//!   the raw log — the correctness oracle.

#![cfg(feature = "delta")]

use std::sync::Arc;

use datafusion::arrow::array::{Array, Int32Array, Int64Array, StringArray};
use datafusion::arrow::datatypes::{DataType, Field, Schema};
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::execution::context::SessionContext;
use datafusion_unitycatalog::log_explorer::{
    RawLogProvider, ReconciledLogProvider, build_default_engine,
};
use deltalake_core::DeltaTableBuilder;
use deltalake_core::protocol::checkpoints::create_checkpoint;
use object_store::local::LocalFileSystem;
use url::Url;

/// A two-column batch `(id, val)` with the given rows.
fn batch(ids: &[i32], vals: &[&str]) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int32, false),
        Field::new("val", DataType::Utf8, false),
    ]));
    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(Int32Array::from(ids.to_vec())),
            Arc::new(StringArray::from(vals.to_vec())),
        ],
    )
    .unwrap()
}

/// Build a synthetic Delta table under `dir` and return its `file://` root URL.
///
/// Log shape:
/// - v0: create + first append (2 adds, metaData, protocol)
/// - v1: second append (1 add)
/// - checkpoint @ v1
/// - v2: overwrite (removes the earlier adds — tombstones — and adds new files)
async fn build_table(dir: &std::path::Path) -> Url {
    let url = Url::from_directory_path(dir).unwrap();

    // v0: create with a first append.
    let table = DeltaTableBuilder::from_url(url.clone())
        .unwrap()
        .build()
        .unwrap();
    let table = table
        .write(vec![batch(&[1, 2], &["a", "b"])])
        .await
        .unwrap();

    // v1: a second append.
    let table = table.write(vec![batch(&[3], &["c"])]).await.unwrap();

    // Checkpoint at v1 so the raw reader crosses a checkpoint file.
    create_checkpoint(&table, None).await.unwrap();

    // v2: overwrite — tombstones the earlier adds, writes new files.
    table
        .write(vec![batch(&[4, 5], &["d", "e"])])
        .with_save_mode(deltalake_core::protocol::SaveMode::Overwrite)
        .await
        .unwrap();

    url
}

/// Register both providers on a fresh session over the table at `url`.
async fn session_for(url: &Url) -> SessionContext {
    let store = Arc::new(LocalFileSystem::new());
    let engine = build_default_engine(store);

    let ctx = SessionContext::new();
    ctx.register_table(
        "raw_log",
        Arc::new(RawLogProvider::new(url.clone(), engine.clone())),
    )
    .unwrap();
    ctx.register_table(
        "reconciled_log",
        Arc::new(ReconciledLogProvider::new(url.clone(), engine)),
    )
    .unwrap();
    ctx
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_log_materializes_all_actions_across_commits_and_checkpoint() {
    let dir = tempfile::tempdir().unwrap();
    let url = build_table(dir.path()).await;
    let ctx = session_for(&url).await;

    // Count actions by kind. Every add/remove/metaData/commitInfo row has its
    // action struct non-null; `_commit` marks commit-file vs checkpoint origin.
    // camelCase action columns must be quoted, else DataFusion normalizes them
    // to lowercase and the column lookup fails.
    let df = ctx
        .sql(
            "SELECT \
               count(*) FILTER (WHERE add IS NOT NULL)          AS adds, \
               count(*) FILTER (WHERE remove IS NOT NULL)       AS removes, \
               count(*) FILTER (WHERE \"metaData\" IS NOT NULL) AS metas, \
               count(*) FILTER (WHERE protocol IS NOT NULL)     AS protocols, \
               count(*) FILTER (WHERE NOT \"_commit\")          AS from_checkpoint \
             FROM raw_log",
        )
        .await
        .unwrap();
    let rows = df.collect().await.unwrap();
    let b = &rows[0];
    let get = |name: &str| {
        let idx = b.schema().index_of(name).unwrap();
        b.column(idx)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0)
    };

    // `read_actions` reads the frozen log segment = checkpoint@v1 + commit v2.
    // The checkpoint holds the 2 files live at v1 (2 adds) + protocol + metaData;
    // commit v2's overwrite tombstones both (2 removes) and adds 1 new file — so
    // 3 adds total across the segment, 2 from the checkpoint.
    assert!(
        get("adds") >= 3,
        "expected >=3 add actions, got {}",
        get("adds")
    );
    assert!(
        get("removes") >= 2,
        "expected >=2 remove actions (the overwrite's tombstones), got {}",
        get("removes")
    );
    assert!(get("metas") >= 1, "expected a metaData action");
    assert!(get("protocols") >= 1, "expected a protocol action");
    assert!(
        get("from_checkpoint") >= 1,
        "expected some actions to come from the checkpoint file"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn reconciled_log_materializes_surviving_add_files() {
    let dir = tempfile::tempdir().unwrap();
    let url = build_table(dir.path()).await;
    let ctx = session_for(&url).await;

    // Reconciled (surviving add-files) vs every distinct add path the raw log
    // ever recorded. The overwrite at v2 tombstones the earlier files, so the
    // reconciled set is strictly smaller — proof that replay dropped tombstones.
    let rows = ctx
        .sql(
            "SELECT \
               (SELECT count(*) FROM reconciled_log)                                   AS reconciled, \
               (SELECT count(DISTINCT add.path) FROM raw_log WHERE add IS NOT NULL)     AS distinct_adds",
        )
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let col = |i: usize| {
        rows[0]
            .column(i)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0)
    };
    let (reconciled, distinct_adds) = (col(0), col(1));
    assert!(
        reconciled >= 1,
        "reconciled log should show the surviving add-file(s)"
    );
    assert!(
        reconciled < distinct_adds,
        "reconciliation must drop tombstoned adds: reconciled {reconciled} should be < distinct adds {distinct_adds}"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_is_a_superset_of_reconciled_add_paths() {
    let dir = tempfile::tempdir().unwrap();
    let url = build_table(dir.path()).await;
    let ctx = session_for(&url).await;

    // Every reconciled add path must appear as an add in the raw log. The
    // reconciled `scan_row_schema` exposes the file path column as `path`; the
    // raw add path is `add.path`.
    let rows = ctx
        .sql(
            "SELECT count(*) AS missing FROM reconciled_log r \
             WHERE r.path NOT IN (SELECT add.path FROM raw_log WHERE add IS NOT NULL)",
        )
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let missing = rows[0]
        .column(0)
        .as_any()
        .downcast_ref::<Int64Array>()
        .unwrap()
        .value(0);
    assert_eq!(
        missing, 0,
        "every reconciled add-file path must appear as an add in the raw log"
    );
}

/// Projection pushdown: reading only `add.path` (kernel reads only the `add`
/// column + skips checkpoint row groups with no adds) must return the exact same
/// set of add paths as an unprojected read — pruning must not change results.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_projection_pushdown_preserves_add_paths() {
    let dir = tempfile::tempdir().unwrap();
    let url = build_table(dir.path()).await;
    let ctx = session_for(&url).await;

    // Projected read: only `add` is touched → pushdown reads add-only.
    let projected = ctx
        .sql("SELECT add.path AS p FROM raw_log WHERE add IS NOT NULL ORDER BY p")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    // Full read of the same logical column (no pushdown pruning of other actions).
    let full = ctx
        .sql(
            "SELECT add.path AS p FROM raw_log \
             WHERE add IS NOT NULL AND (remove IS NULL AND protocol IS NULL) ORDER BY p",
        )
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    let paths = |batches: &[RecordBatch]| -> Vec<String> {
        batches
            .iter()
            .flat_map(|b| {
                let col = b.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                (0..col.len())
                    .map(|i| col.value(i).to_string())
                    .collect::<Vec<_>>()
            })
            .collect()
    };
    assert_eq!(
        paths(&projected),
        paths(&full),
        "add paths must be identical whether or not other action columns are projected"
    );
    assert!(
        !paths(&projected).is_empty(),
        "expected at least one add path"
    );
}

/// Cardinality preserved when only `_commit` is projected (no action column):
/// the provider still reads one narrow action column so every log row is emitted.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_commit_only_projection_keeps_all_rows() {
    let dir = tempfile::tempdir().unwrap();
    let url = build_table(dir.path()).await;
    let ctx = session_for(&url).await;

    async fn count(ctx: &SessionContext, sql: &str) -> i64 {
        let rows = ctx.sql(sql).await.unwrap().collect().await.unwrap();
        rows[0]
            .column(0)
            .as_any()
            .downcast_ref::<Int64Array>()
            .unwrap()
            .value(0)
    }
    let all = count(&ctx, "SELECT count(*) FROM raw_log").await;
    let commit_only = count(&ctx, "SELECT count(\"_commit\") FROM raw_log").await;
    assert!(all > 0);
    assert_eq!(
        all, commit_only,
        "projecting only _commit must not drop rows (cardinality preserved)"
    );
}

/// Predicate pushdown (meta_predicate): a filter on a value inside an action is
/// translated to a kernel predicate and pushed as a checkpoint row-group-skip
/// hint. Because it is only a hint (and reported `Inexact`), results must be
/// identical to the same query without the pushdown — pruning must never drop a
/// matching row. `add.size >= 0` holds for every real add file.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn raw_value_predicate_pushdown_preserves_results() {
    let dir = tempfile::tempdir().unwrap();
    let url = build_table(dir.path()).await;
    let ctx = session_for(&url).await;

    let paths = |batches: &[RecordBatch]| -> Vec<String> {
        batches
            .iter()
            .flat_map(|b| {
                let col = b.column(0).as_any().downcast_ref::<StringArray>().unwrap();
                (0..col.len())
                    .map(|i| col.value(i).to_string())
                    .collect::<Vec<_>>()
            })
            .collect()
    };

    // With a pushed value predicate on an action field.
    let filtered = ctx
        .sql(
            "SELECT add.path AS p FROM raw_log \
             WHERE add IS NOT NULL AND add.size >= 0 ORDER BY p",
        )
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    // Baseline: same logical result, no size predicate to push.
    let baseline = ctx
        .sql("SELECT add.path AS p FROM raw_log WHERE add IS NOT NULL ORDER BY p")
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();

    assert_eq!(
        paths(&filtered),
        paths(&baseline),
        "a pushed value predicate is an I/O hint and must not change results"
    );
    assert!(
        !paths(&filtered).is_empty(),
        "expected at least one add path to survive the predicate"
    );
}
