//! [`DeltaSsaTableProvider`]: the async-native, engine-free Delta [`TableProvider`] that
//! replaces the eager inline-executor scan path.
//!
//! This is the crate's **one public, table-level provider** — the type callers register for a
//! Delta table. The only other `TableProvider` in the crate, [`crate::exec::LoadTableProvider`], is
//! a `pub(crate)` internal leaf the SSA compiler emits *inside* the plan this provider produces;
//! see its module docs. There is no second table-level provider to choose between.
//!
//! Holds only a kernel [`SnapshotRef`] plus a small [`DeltaSsaScanConfig`] — **no engine**. At
//! `scan()` time it:
//!
//!   1. builds a kernel [`Scan`] from the snapshot (`scan_builder().build_replay()`),
//!   2. drives the scan's `sm_plans` coroutine state machine
//!      ([`Scan::scan_state_machine`]) to a [`ResultPlan`] through the engine-free
//!      [`DataFusionExecutor`] — a `!Send`, CPU-only planning step for commit-only tables
//!      (IO-free; see the `scan()` body for the checkpointed-table caveat),
//!   3. compiles the `ResultPlan` to a DataFusion `LogicalPlan`, and
//!   4. plans it against the **scan's own `Session`** (so the object store, runtime, and
//!      config are the caller's), applying projection + limit.
//!
//! The `!Send` SM drive happens entirely inside `scan()` (planning); the returned
//! `ExecutionPlan` is `Send` and streams lazily via DataFusion's own async parquet/object-store
//! stack. No `ExecutorHandle`/`InlineExecutor` is constructed anywhere on this path.

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::catalog::{Session, TableProvider};
use datafusion_common::Result as DfResult;
use datafusion_expr::{Expr, LogicalPlanBuilder, TableProviderFilterPushDown, TableType};
use datafusion_physical_plan::ExecutionPlan;
use delta_kernel::arrow::datatypes::SchemaRef as ArrowSchemaRef;
use delta_kernel::engine::arrow_conversion::TryIntoArrow;
use delta_kernel::scan::{Scan, ScanBuilder, StatsOptions};
use delta_kernel::snapshot::SnapshotRef;

use crate::DataFusionExecutor;
use crate::compile::stats::build_file_statistics;

/// Scan-time configuration for [`DeltaSsaTableProvider`]. Mirrors the subset of
/// `deltalake_core::delta_datafusion::DeltaScanConfig` the wasm preview path actually sets.
#[derive(Debug, Clone)]
pub struct DeltaSsaScanConfig {
    /// When `false`, the scan emits plain `Utf8`/`Binary` rather than `Utf8View`/`BinaryView`.
    ///
    /// The browser apache-arrow IPC reader cannot decode view types (mangrove issue #28), so the
    /// wasm preview path sets this `false`. It is honored via the DataFusion session config knob
    /// `datafusion.execution.parquet.schema_force_view_types`: the caller's `Session` (the one
    /// passed to [`TableProvider::scan`]) must have it set to match. This flag records the intent
    /// and is asserted against the session at scan time.
    pub schema_force_view_types: bool,
}

impl Default for DeltaSsaScanConfig {
    fn default() -> Self {
        // Match `DeltaScanConfig::default()` (DataFusion's own default is `true`); callers that
        // target the browser flip this to `false`.
        Self {
            schema_force_view_types: true,
        }
    }
}

/// Async-native Delta `TableProvider` driven by the kernel `sm_plans` scan state machine.
pub struct DeltaSsaTableProvider {
    snapshot: SnapshotRef,
    config: DeltaSsaScanConfig,
    /// Pre-materialized arrow logical schema so `schema()` is cheap and infallible.
    arrow_schema: ArrowSchemaRef,
}

impl std::fmt::Debug for DeltaSsaTableProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeltaSsaTableProvider")
            .field("version", &self.snapshot.version())
            .field(
                "schema_force_view_types",
                &self.config.schema_force_view_types,
            )
            .finish_non_exhaustive()
    }
}

impl DeltaSsaTableProvider {
    /// Construct from a kernel [`SnapshotRef`] and scan config. The snapshot is built by the
    /// caller (natively, or via the `deltalake-wasm` facade on wasm); this provider derives
    /// everything else from the `Session` at scan time.
    pub fn new(snapshot: SnapshotRef, config: DeltaSsaScanConfig) -> DfResult<Self> {
        // The logical schema is fixed by the snapshot; convert once for `schema()`.
        let scan = build_scan(&snapshot)?;
        let arrow_schema: ArrowSchemaRef = Arc::new(
            scan.logical_schema()
                .as_ref()
                .try_into_arrow()
                .map_err(|e| {
                    crate::error::plan_compilation(format!(
                        "DeltaSsaTableProvider logical schema: {e}"
                    ))
                })?,
        );
        Ok(Self {
            snapshot,
            config,
            arrow_schema,
        })
    }

    /// The kernel snapshot this provider scans.
    pub fn snapshot(&self) -> &SnapshotRef {
        &self.snapshot
    }

    /// Drive the metadata-only stats SM and build the per-file `raw add.path -> Arc<Statistics>`
    /// map, or `None` if stats are unavailable.
    ///
    /// Statistics are a pruning **optimization**, never a correctness requirement (the kernel does
    /// its own file skipping), so every failure mode here degrades to `None` rather than failing the
    /// scan: the SM can't be built, the `!Send` drive errors, or the guard skips an IO-compiling
    /// plan. Runs in the same synchronous `block_on` confinement as the primary drive — nothing
    /// `!Send` escapes.
    fn build_file_stats(
        &self,
        session: &dyn Session,
        executor: &DataFusionExecutor,
        scan: &Scan,
    ) -> Option<Arc<crate::compile::stats::FileStatsMap>> {
        let sm = scan.scan_stats_metadata_state_machine().ok()?;
        let batches = futures::executor::block_on(executor.drive_ssa_to_batches(session, sm))
            .ok()
            .flatten()?;
        let map = build_file_statistics(scan, &batches);
        (!map.is_empty()).then(|| Arc::new(map))
    }
}

/// Build the kernel `Scan` used both for `schema()` and for driving the state machine.
/// `ScanBuilder::new` takes `impl Into<SnapshotRef>`, so we clone the `Arc` (cheap).
///
/// `with_stats(StatsOptions::all_struct())` requests per-file struct statistics on the reconciled
/// terminal (no JSON synthesis — the cheapest option that makes `physical_stats_schema()` non-`None`
/// so the stats SM's terminal carries a populated `stats` column). It has no effect on the primary
/// (data) scan drive, which never projects `stats`; only the metadata-stats SM reads it.
fn build_scan(snapshot: &SnapshotRef) -> DfResult<Scan> {
    ScanBuilder::new(snapshot.clone())
        .with_stats(StatsOptions::all_struct())
        .build_replay()
        .map_err(crate::error::wrap_delta_err)
}

#[async_trait]
impl TableProvider for DeltaSsaTableProvider {
    fn schema(&self) -> ArrowSchemaRef {
        Arc::clone(&self.arrow_schema)
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        session: &dyn Session,
        projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        limit: Option<usize>,
    ) -> DfResult<Arc<dyn ExecutionPlan>> {
        // Assert the caller's session carries the full Delta engine config, not just the
        // view-type knob: the SM drive below and the final `create_physical_plan` both run against
        // this session, so it must have leaf-pushdown off / single partition / no stats, and its
        // `schema_force_view_types` must match this provider's config (a mismatch would emit
        // `Utf8View` the browser IPC reader can't decode — mangrove #28). Hard error, not
        // auto-repair: the fix is to build the session via `delta_engine_session` /
        // `with_delta_engine`, not to silently rewrite it here.
        crate::validate_delta_engine_session(session, self.config.schema_force_view_types)?;

        // Build the kernel scan and drive its `sm_plans` coroutine state machine to a ResultPlan.
        // This is the engine-free, no-InlineExecutor planning step.
        //
        // The SM future is `!Send` (genawaiter2 `rc` — an `Rc<Cell<..>>` airlock), but DataFusion's
        // `TableProvider::scan` requires the returned future to be `Send`. To keep this `scan`
        // future `Send`, we confine the `!Send` drive to a single synchronous `block_on`: nothing
        // `!Send` is held across any `.await` in `scan`, so the returned `ExecutionPlan` future
        // stays `Send`.
        //
        // Blocking here is currently safe: this drive performs no object-store IO for the tables
        // that reach it. Commit-only tables short-circuit shape resolution and defer add-file
        // enumeration into the returned `ResultPlan` (commit `.json`s become `Values -> Load` nodes
        // the *executor* reads lazily, outside this drive); a classic checkpoint's shape resolution
        // stays entirely CPU-side for this fixture too. If a future kernel shape ever makes the scan
        // drive `.await` a real store read (a checkpoint-footer `SchemaQuery` / sidecar `Consume`),
        // this `block_on` would become a browser-hang risk — a `fetch` settles only when the JS
        // event loop runs, which a blocked worker thread starves — and the fix would be to pre-drive
        // the scan SM at snapshot-open time (an async, `!Send`-tolerant context; see `snapshot_build`)
        // and hand this provider a resolved `ResultPlan`. Not required today.
        //
        // The drive runs against the *caller's* `session` (passed per call, not a throwaway) — so
        // the drive's object store, scalar functions, and `execution_props` (the `now()` anchor)
        // match the final scan plan the same session runs at `create_physical_plan` below. The
        // `!Send` SM future is confined to this synchronous `block_on` and dropped before the
        // `.await` at the end of `scan`, so nothing `!Send` is held across an `.await` and the
        // returned future stays `Send` (`session` itself, a `&dyn Session`, is `Send`). The session
        // must disable `enable_leaf_expression_pushdown` (validated above) — the FSR replay shape
        // otherwise trips `push_down_leaf_projections` with a `scan.add`/`add` ambiguity on
        // checkpointed tables (see `session::configure_delta_engine_config`, apache/datafusion#20432).
        let scan = build_scan(&self.snapshot)?;
        let sm = scan
            .scan_state_machine()
            .map_err(crate::error::wrap_delta_err)?;
        let executor = DataFusionExecutor::new();
        let result_plan = futures::executor::block_on(executor.drive_to_completion(session, sm))
            .map_err(crate::error::wrap_delta_err)?;

        // Second drive: the metadata-only *stats* SM. Its small terminal (one row per live file,
        // with a physical-named `stats` struct) is materialized and remapped to per-file DataFusion
        // `Statistics`, keyed by raw `add.path`, then threaded onto the compiled `Load` leaf's
        // per-file `PartitionedFile`s. It reuses the SAME `block_on` confinement as the primary
        // drive. Unlike the primary (planning-only) drive, this one *executes* the plan, which reads
        // the commit log; that IO under `block_on` is safe on native but not on a browser worker, so
        // `drive_ssa_to_batches` skips (returns `None`, no stats) on wasm32 — see its docs. Stats are
        // a pruning optimization, so skipping is a clean degrade.
        let file_stats = self.build_file_stats(session, &executor, &scan);

        // Compile the SSA result plan to a bare LogicalPlan, then plan it against the *caller's*
        // session so file reads go through the caller's object store + runtime.
        let logical = DataFusionExecutor::compile_result_plan_with_stats(&result_plan, file_stats)
            .map_err(crate::error::wrap_delta_err)?;

        // Apply projection + limit at the logical level so DataFusion pushes them into the
        // per-file parquet sources.
        let mut builder = LogicalPlanBuilder::from(logical);
        if let Some(proj) = projection {
            let exprs = proj
                .iter()
                .map(|&i| {
                    let field = self.arrow_schema.field(i);
                    datafusion_expr::col(field.name())
                })
                .collect::<Vec<_>>();
            builder = builder.project(exprs)?;
        }
        if let Some(n) = limit {
            builder = builder.limit(0, Some(n))?;
        }
        let logical = builder.build()?;

        session.create_physical_plan(&logical).await
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> DfResult<Vec<TableProviderFilterPushDown>> {
        // Filter pushdown is off for v1 (as it is for the internal per-file `LoadTableProvider`
        // leaves this scan compiles to); projection and limit flow through. DataFusion re-applies
        // filters above the scan.
        Ok(vec![
            TableProviderFilterPushDown::Unsupported;
            filters.len()
        ])
    }
}

/// End-to-end statistics tests over a real column-mapped fixture: `with_stats` populates the
/// metadata-stats terminal, the second SM drive materializes it, and the raw-`add.path` key matches
/// the data-file rows — so `build_file_stats` produces correct per-logical-column `Statistics` under
/// **both** id and name mode (the exact scenario `collect_statistics=false` avoided). The pure
/// physical->logical remap / collapse / precision logic is unit-tested in `crate::compile::stats`.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod stats_e2e_tests {
    use std::collections::HashMap;

    use datafusion_common::stats::Precision;
    use datafusion_common::{ScalarValue, Statistics};
    use delta_kernel::arrow::array::{ArrayRef, Int64Array, RecordBatch, StringArray};
    use delta_kernel::arrow::datatypes::{DataType, Field, Schema};
    use delta_kernel::parquet::arrow::ArrowWriter;
    use delta_kernel::parquet::arrow::PARQUET_FIELD_ID_META_KEY;
    use delta_kernel::snapshot::Snapshot;
    use delta_kernel_default_engine::DefaultEngineBuilder;
    use object_store::memory::InMemory;
    use object_store::path::Path;
    use object_store::{ObjectStore, ObjectStoreExt, PutPayload};
    use url::Url;

    use super::*;
    use crate::{DeltaEngineSessionOptions, DeltaSsaScanConfig, delta_engine_session};

    const PREFIX: &str = "stats_tbl";
    const ID_PHYS: &str = "col-1a2b3c4d";
    const NAME_PHYS: &str = "col-5e6f7a8b";

    fn field_with_id(name: &str, dt: DataType, id: i64) -> Field {
        let mut md = HashMap::new();
        md.insert(PARQUET_FIELD_ID_META_KEY.to_string(), id.to_string());
        Field::new(name, dt, true).with_metadata(md)
    }

    /// Two data files under **physical** column names, each with a distinct id/name range so per-file
    /// min/max are distinguishable.
    fn parquet_for(ids: &[i64], names: &[&str]) -> Vec<u8> {
        let schema = Arc::new(Schema::new(vec![
            field_with_id(ID_PHYS, DataType::Int64, 1),
            field_with_id(NAME_PHYS, DataType::Utf8, 2),
        ]));
        let batch = RecordBatch::try_new(
            schema.clone(),
            vec![
                Arc::new(Int64Array::from(ids.to_vec())) as ArrayRef,
                Arc::new(StringArray::from(names.to_vec())) as ArrayRef,
            ],
        )
        .unwrap();
        let mut buf = Vec::new();
        let mut w = ArrowWriter::try_new(&mut buf, schema, None).unwrap();
        w.write(&batch).unwrap();
        w.close().unwrap();
        buf
    }

    const CM_SCHEMA: &str = concat!(
        r#"{\"type\":\"struct\",\"fields\":["#,
        r#"{\"name\":\"id\",\"type\":\"long\",\"nullable\":true,"#,
        r#"\"metadata\":{\"delta.columnMapping.id\":1,\"delta.columnMapping.physicalName\":\"col-1a2b3c4d\"}},"#,
        r#"{\"name\":\"name\",\"type\":\"string\",\"nullable\":true,"#,
        r#"\"metadata\":{\"delta.columnMapping.id\":2,\"delta.columnMapping.physicalName\":\"col-5e6f7a8b\"}}"#,
        r#"]}"#,
    );

    /// A one-commit CM table whose two `add` actions carry per-file `stats` JSON keyed by the
    /// **physical** column names (how Delta records stats under column mapping). `tightBounds` true.
    async fn fixture(mode: &str) -> Arc<InMemory> {
        let store = InMemory::new();
        // file A: id 1..=3 / name a..=c ; file B: id 4..=6 / name d..=f
        let files = [
            (
                "part-a.parquet",
                &[1i64, 2, 3][..],
                ["a", "b", "c"],
                1i64,
                3i64,
                "a",
                "c",
            ),
            (
                "part-b.parquet",
                &[4i64, 5, 6][..],
                ["d", "e", "f"],
                4i64,
                6i64,
                "d",
                "f",
            ),
        ];
        let mut commit = format!(
            concat!(
                r#"{{"protocol":{{"minReaderVersion":2,"minWriterVersion":5}}}}"#,
                "\n",
                r#"{{"metaData":{{"id":"11111111-2222-3333-4444-555555555555","format":{{"provider":"parquet","options":{{}}}},"schemaString":"{schema}","partitionColumns":[],"configuration":{{"delta.columnMapping.mode":"{mode}"}},"createdTime":0}}}}"#,
                "\n",
            ),
            schema = CM_SCHEMA,
            mode = mode,
        );
        for (name, ids, names, id_min, id_max, nm_min, nm_max) in files {
            let bytes = parquet_for(ids, &names);
            // stats keyed by physical names; nullCount 0 for both leaves.
            let stats = format!(
                r#"{{\"numRecords\":3,\"minValues\":{{\"{ID_PHYS}\":{id_min},\"{NAME_PHYS}\":\"{nm_min}\"}},\"maxValues\":{{\"{ID_PHYS}\":{id_max},\"{NAME_PHYS}\":\"{nm_max}\"}},\"nullCount\":{{\"{ID_PHYS}\":0,\"{NAME_PHYS}\":0}},\"tightBounds\":true}}"#,
            );
            commit.push_str(&format!(
                r#"{{"add":{{"path":"{name}","partitionValues":{{}},"size":{size},"modificationTime":0,"dataChange":true,"stats":"{stats}"}}}}"#,
                size = bytes.len(),
            ));
            commit.push('\n');
            store
                .put(
                    &Path::from(format!("{PREFIX}/{name}")),
                    PutPayload::from(bytes),
                )
                .await
                .unwrap();
        }
        store
            .put(
                &Path::from(format!("{PREFIX}/_delta_log/00000000000000000000.json")),
                PutPayload::from(commit.into_bytes()),
            )
            .await
            .unwrap();
        Arc::new(store)
    }

    fn table_url() -> Url {
        Url::parse(&format!("memory:///{PREFIX}/")).unwrap()
    }

    fn snapshot(store: Arc<InMemory>) -> SnapshotRef {
        let engine = DefaultEngineBuilder::new(store).build();
        Snapshot::builder_for(table_url().as_str())
            .at_version(0)
            .build(&engine)
            .expect("snapshot")
    }

    async fn assert_stats_correct(mode: &str) {
        let store = fixture(mode).await;
        let session = delta_engine_session(
            Arc::clone(&store) as Arc<dyn ObjectStore>,
            &table_url(),
            &DeltaEngineSessionOptions::wasm(),
        );
        let provider = DeltaSsaTableProvider::new(
            snapshot(store),
            DeltaSsaScanConfig {
                schema_force_view_types: false,
            },
        )
        .expect("provider");

        let scan = build_scan(provider.snapshot()).expect("scan");
        let executor = DataFusionExecutor::new();
        let state = session.state();
        let map = provider
            .build_file_stats(&state, &executor, &scan)
            .unwrap_or_else(|| panic!("[{mode}] expected non-empty per-file stats map"));

        // Both data files present, keyed by their raw `add.path`.
        assert_eq!(map.len(), 2, "[{mode}] one Statistics per file");
        let a = map.get("part-a.parquet").expect("[mode] file A key hits");
        let b = map.get("part-b.parquet").expect("[mode] file B key hits");

        // Column order is logical: [id, name]. File A: id in [1,3]; File B: id in [4,6].
        check_file(mode, "A", a, 1, 3, "a", "c");
        check_file(mode, "B", b, 4, 6, "d", "f");
    }

    fn check_file(
        mode: &str,
        label: &str,
        stats: &Statistics,
        id_lo: i64,
        id_hi: i64,
        nm_lo: &str,
        nm_hi: &str,
    ) {
        assert_eq!(
            stats.num_rows,
            Precision::Exact(3),
            "[{mode}/{label}] num_rows"
        );
        assert_eq!(
            stats.column_statistics.len(),
            2,
            "[{mode}/{label}] logical col count"
        );
        let id = &stats.column_statistics[0];
        let name = &stats.column_statistics[1];
        // tightBounds=true => Exact bounds; nullCount 0 => Exact(0). Remapped to LOGICAL positions.
        assert_eq!(
            id.null_count,
            Precision::Exact(0),
            "[{mode}/{label}] id null_count"
        );
        assert_eq!(
            id.min_value,
            Precision::Exact(ScalarValue::Int64(Some(id_lo))),
            "[{mode}/{label}] id min"
        );
        assert_eq!(
            id.max_value,
            Precision::Exact(ScalarValue::Int64(Some(id_hi))),
            "[{mode}/{label}] id max"
        );
        assert_eq!(
            name.min_value,
            Precision::Exact(ScalarValue::from(nm_lo)),
            "[{mode}/{label}] name min"
        );
        assert_eq!(
            name.max_value,
            Precision::Exact(ScalarValue::from(nm_hi)),
            "[{mode}/{label}] name max"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn per_file_stats_correct_name_mode() {
        assert_stats_correct("name").await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn per_file_stats_correct_id_mode() {
        assert_stats_correct("id").await;
    }
}
