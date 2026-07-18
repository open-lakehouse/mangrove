//! [`DeltaSsaTableProvider`]: the async-native, engine-free Delta [`TableProvider`].
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
//! stack.

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::catalog::{Session, TableProvider};
use datafusion_common::{DFSchema, Result as DfResult};
use datafusion_expr::utils::conjunction;
use datafusion_expr::{Expr, LogicalPlanBuilder, TableProviderFilterPushDown, TableType};
use datafusion_physical_expr_common::physical_expr::PhysicalExpr;
use datafusion_physical_plan::ExecutionPlan;
use delta_kernel::arrow::datatypes::SchemaRef as ArrowSchemaRef;
use delta_kernel::engine::arrow_conversion::TryIntoArrow;
use delta_kernel::expressions::Predicate;
use delta_kernel::scan::{Scan, ScanBuilder, StatsOptions};
use delta_kernel::snapshot::SnapshotRef;

use crate::DataFusionExecutor;
use crate::compile::expr_translator::df_expr_to_kernel_pred;
use crate::compile::stats::build_file_statistics;

/// Scan-time configuration for [`DeltaSsaTableProvider`]. Mirrors the subset of
/// `deltalake_core::delta_datafusion::DeltaScanConfig` the wasm preview path actually sets.
#[derive(Debug, Clone)]
pub struct DeltaSsaScanConfig {
    /// When `false`, the scan emits plain `Utf8`/`Binary` rather than `Utf8View`/`BinaryView`.
    ///
    /// The browser apache-arrow IPC reader cannot decode view types, so the wasm preview path sets
    /// this `false`. It is honored via the DataFusion session config knob
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
        // The logical schema is fixed by the snapshot; convert once for `schema()`. No predicate:
        // this scan is only used to derive the logical schema, never driven.
        let scan = build_scan(&snapshot, None)?;
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

    /// Lower the scan's query `filters` to a single scan-global, **logical-named**
    /// [`PhysicalExpr`] for parquet row-group / page pruning (threaded onto the compiled `Load`
    /// leaf's parquet source). `None` when there are no filters or none lower cleanly.
    ///
    /// Like statistics, the pushdown predicate is a pruning **optimization**, never a correctness
    /// requirement: `supports_filters_pushdown` reports `Inexact`, so DataFusion re-applies a
    /// `FilterExec` above the scan regardless. Any lowering failure therefore degrades to `None`
    /// rather than failing the scan. The predicate is built against this provider's *logical* arrow
    /// schema; the per-file `FieldIdPhysicalExprAdapterFactory` reconciles it to each file's physical
    /// schema at decode time, after pruning — so no column-mapping rewrite happens here.
    fn build_pushdown_predicate(
        &self,
        session: &dyn Session,
        filters: &[Expr],
    ) -> Option<Arc<dyn PhysicalExpr>> {
        let conjoined = conjunction(filters.iter().cloned())?;
        let df_schema = DFSchema::try_from(self.arrow_schema.as_ref().clone()).ok()?;
        session.create_physical_expr(conjoined, &df_schema).ok()
    }
}

/// Build the kernel `Scan` used both for `schema()` and for driving the state machine.
/// `ScanBuilder::new` takes `impl Into<SnapshotRef>`, so we clone the `Arc` (cheap).
///
/// `with_stats(StatsOptions::all_struct())` requests per-file struct statistics on the reconciled
/// terminal (no JSON synthesis — the cheapest option that makes `physical_stats_schema()` non-`None`
/// so the stats SM's terminal carries a populated `stats` column). It has no effect on the primary
/// (data) scan drive, which never projects `stats`; only the metadata-stats SM reads it.
///
/// `predicate` is the kernel file-list-skipping predicate (logical-named): when `Some`, the
/// `sm_plans` SSA scan path inserts a `FilterNode` over the reconciled `add.stats_parsed` rows so
/// whole files can be pruned from the live-file list *before* they enter the plan. The kernel
/// rewrites the logical column refs to physical itself (via `Scan::physical_predicate()`). `new()`
/// passes `None` (schema only); `scan()` passes the lowered query filters. A `None` predicate
/// produces a byte-identical plan to before.
fn build_scan(snapshot: &SnapshotRef, predicate: Option<Predicate>) -> DfResult<Scan> {
    ScanBuilder::new(snapshot.clone())
        .with_stats(StatsOptions::all_struct())
        .with_predicate(predicate.map(Arc::new))
        .build_replay()
        .map_err(crate::error::wrap_delta_err)
}

/// Lower a slice of DataFusion query `filters` to a single conjoined kernel data-skipping
/// [`Predicate`] (Layer 1), or `None` if none of them translate. Each filter is lowered
/// independently via [`df_expr_to_kernel_pred`] and the translatable ones are `AND`-ed — a filter
/// that can't be represented in the data-skipping subset is simply omitted, which is safe because
/// the provider reports `Inexact` (DataFusion re-applies the full filter above the scan).
fn lower_skipping_predicate(filters: &[Expr]) -> Option<Predicate> {
    let preds: Vec<Predicate> = filters.iter().filter_map(df_expr_to_kernel_pred).collect();
    match preds.len() {
        0 => None,
        1 => preds.into_iter().next(),
        _ => Some(Predicate::and_from(preds)),
    }
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
        filters: &[Expr],
        limit: Option<usize>,
    ) -> DfResult<Arc<dyn ExecutionPlan>> {
        // Assert the caller's session carries the full Delta engine config, not just the
        // view-type knob: the SM drive below and the final `create_physical_plan` both run against
        // this session, so it must have leaf-pushdown off / single partition / no stats, and its
        // `schema_force_view_types` must match this provider's config (a mismatch would emit
        // `Utf8View` the browser IPC reader can't decode). Hard error, not
        // auto-repair: the fix is to build the session via `delta_engine_session` /
        // `with_delta_engine`, not to silently rewrite it here.
        crate::validate_delta_engine_session(session, self.config.schema_force_view_types)?;

        // Build the kernel scan and drive its `sm_plans` coroutine state machine to a ResultPlan
        // (engine-free planning).
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
        // stays entirely CPU-side too. A kernel shape that made the drive `.await` a real store read
        // would turn this `block_on` into a browser-hang risk (a `fetch` settles only when the JS
        // event loop runs, which a blocked worker thread starves); the fix would be to pre-drive the
        // scan SM at snapshot-open time (see `snapshot_build`) and hand this provider a resolved plan.
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
        // Layer 1 (kernel file-list skipping): lower the query filters to a logical-named kernel
        // data-skipping predicate. Best-effort — untranslatable filters drop to a coarser (or
        // absent) predicate, which only forgoes a skip, never changes results (`Inexact` above +
        // the kernel re-checks stats conservatively). The kernel's SSA scan path applies this as a
        // `FilterNode` over `add.stats_parsed`, pruning whole files before they enter the plan.
        let skipping_predicate = lower_skipping_predicate(filters);
        let scan = build_scan(&self.snapshot, skipping_predicate)?;
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
        // Lower the query filters to a logical-named pushdown predicate for parquet pruning. A
        // pruning optimization only (reported `Inexact` below), so a `None` here just skips pruning.
        let predicate = self.build_pushdown_predicate(session, filters);
        let channels = crate::compile::SideChannels {
            file_stats,
            predicate,
        };
        let logical =
            DataFusionExecutor::compile_result_plan_with_side_channels(&result_plan, channels)
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
        // `Inexact`: `scan()` lowers these filters to a logical pushdown predicate handed to the
        // per-file parquet source for row-group / page pruning (against the per-file
        // statistics), but pruning is conservative and some filters may not lower — so DataFusion
        // must still re-apply a `FilterExec` above the scan for correctness. Only this top-level
        // provider's report matters: the internal `LoadTableProvider` leaves receive no `Expr`
        // filters (the predicate reaches them via the explicit side channel), so they stay
        // `Unsupported`. Projection and limit continue to flow through separately.
        Ok(vec![TableProviderFilterPushDown::Inexact; filters.len()])
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
    use delta_kernel::arrow::array::types::Int64Type;
    use delta_kernel::arrow::array::{ArrayRef, AsArray, Int64Array, RecordBatch, StringArray};
    use delta_kernel::arrow::datatypes::{DataType, Field, Schema};
    use delta_kernel::parquet::arrow::ArrowWriter;
    use delta_kernel::parquet::arrow::PARQUET_FIELD_ID_META_KEY;
    use delta_kernel::snapshot::Snapshot;
    use delta_kernel_default_engine::DefaultEngineBuilder;
    use futures::StreamExt;
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

        let scan = build_scan(provider.snapshot(), None).expect("scan");
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

    // === Filter pushdown / file skipping =================================================

    /// End-to-end: register the provider and run `SELECT ... WHERE id >= 4` over the two-file CM
    /// fixture (file A `id∈[1,3]`, file B `id∈[4,6]`). With filter pushdown live the provider lowers
    /// the predicate onto the per-file parquet source; DataFusion re-applies the `Inexact` filter
    /// above the scan, so the result is correct regardless of how aggressively pruning fired. Proves
    /// the whole pushdown path runs correctly under **both** id and name mode.
    async fn assert_pushdown_result_correct(mode: &str) {
        use datafusion::execution::context::SessionContext;

        let store = fixture(mode).await;
        let ctx: SessionContext = delta_engine_session(
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
        ctx.register_table("preview", Arc::new(provider)).unwrap();

        let batches = ctx
            .sql("SELECT id, name FROM preview WHERE id >= 4 ORDER BY id")
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();

        let mut rows: Vec<(i64, String)> = Vec::new();
        for b in &batches {
            let ids = b.column(0).as_primitive::<Int64Type>();
            let names = b.column(1).as_string::<i32>();
            for i in 0..b.num_rows() {
                rows.push((ids.value(i), names.value(i).to_string()));
            }
        }
        assert_eq!(
            rows,
            vec![
                (4, "d".to_string()),
                (5, "e".to_string()),
                (6, "f".to_string())
            ],
            "[{mode}] `WHERE id >= 4` must return only file B's rows"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn filter_pushdown_result_correct_name_mode() {
        assert_pushdown_result_correct("name").await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn filter_pushdown_result_correct_id_mode() {
        assert_pushdown_result_correct("id").await;
    }

    /// Direct proof that the wired predicate + attached per-file `Statistics` actually *prune*: build
    /// a single-file parquet `DataSourceExec` through the exact seam `scan()` uses
    /// (`build_file_source(..., predicate)` → `build_per_file_plan(..., statistics)`), execute it,
    /// and read the source's own `files_ranges_pruned_statistics` metric (unreachable through the
    /// top-level plan, since `LoadExec` builds per-file execs lazily). An out-of-range predicate
    /// prunes the file (0 rows, 1 pruned); an in-range predicate keeps it (rows, 0 pruned).
    ///
    /// The predicate is logical-named (`id`), matching the logical file schema the pruner sees; the
    /// parquet is written with logical names here (no CM rename) since this test isolates the
    /// pruning mechanism, not column mapping (covered by the e2e tests above).
    async fn assert_per_file_pruning(predicate_lo: i64, expect_pruned: bool, expect_rows: usize) {
        use datafusion::prelude::{col, lit};
        use datafusion_datasource::file::FileSource;
        use datafusion_physical_plan::metrics::MetricValue;
        use delta_kernel::sm_plans::ir::nodes::FileType;

        // Logical single-column file schema `id: Int64`, one data file `id ∈ [10, 20]`.
        let file_schema = Arc::new(Schema::new(vec![Field::new("id", DataType::Int64, true)]));
        let batch = RecordBatch::try_new(
            file_schema.clone(),
            vec![Arc::new(Int64Array::from(vec![10i64, 15, 20])) as ArrayRef],
        )
        .unwrap();
        let mut buf = Vec::new();
        let mut w = ArrowWriter::try_new(&mut buf, file_schema.clone(), None).unwrap();
        w.write(&batch).unwrap();
        w.close().unwrap();
        let size = buf.len();

        let store = InMemory::new();
        store
            .put(&Path::from("data.parquet"), PutPayload::from(buf))
            .await
            .unwrap();
        let store = Arc::new(store);
        let base = Url::parse("memory:///").unwrap();

        // Build the file source with the logical pushdown predicate `id >= predicate_lo`.
        let df_schema =
            datafusion_common::DFSchema::try_from(file_schema.as_ref().clone()).unwrap();
        let ctx = datafusion::execution::context::SessionContext::new();
        ctx.runtime_env()
            .register_object_store(&base, Arc::clone(&store) as Arc<dyn ObjectStore>);
        let predicate = ctx
            .create_physical_expr(col("id").gt_eq(lit(predicate_lo)), &df_schema)
            .unwrap();
        let file_source = crate::exec::load_helpers::build_file_source(
            FileType::Parquet,
            &file_schema,
            1,
            None,
            Some(predicate),
        )
        .unwrap();

        // Attach per-file stats matching the data (id ∈ [10,20], tight, no nulls).
        let stats = Arc::new(Statistics {
            num_rows: Precision::Exact(3),
            total_byte_size: Precision::Absent,
            column_statistics: vec![datafusion_common::ColumnStatistics {
                null_count: Precision::Exact(0),
                min_value: Precision::Exact(ScalarValue::Int64(Some(10))),
                max_value: Precision::Exact(ScalarValue::Int64(Some(20))),
                sum_value: Precision::Absent,
                distinct_count: Precision::Absent,
                byte_size: Precision::Absent,
            }],
        });

        let inputs = crate::exec::load_helpers::RowInputs {
            url: base.join("data.parquet").unwrap(),
            size: size as i64,
            partition_values: vec![],
            raw_path: "data.parquet".to_string(),
        };
        let task_ctx = ctx.task_ctx();
        let plan = crate::exec::load_helpers::build_per_file_plan(
            inputs,
            Arc::<dyn FileSource>::clone(&file_source),
            FileType::Parquet,
            &file_schema,
            task_ctx.as_ref(),
            Some(stats),
        )
        .await
        .unwrap();

        // Execute and count rows.
        let mut stream = plan.execute(0, task_ctx).unwrap();
        let mut rows = 0usize;
        while let Some(b) = stream.next().await {
            rows += b.unwrap().num_rows();
        }
        assert_eq!(
            rows, expect_rows,
            "row count (predicate id >= {predicate_lo})"
        );

        // Read `files_ranges_pruned_statistics` off the exec's own metrics.
        let pruned = plan
            .metrics()
            .map(|m| {
                m.iter()
                    .filter_map(|mv| match mv.value() {
                        MetricValue::PruningMetrics {
                            name,
                            pruning_metrics,
                        } if name.as_ref() == "files_ranges_pruned_statistics" => {
                            Some(pruning_metrics.pruned())
                        }
                        _ => None,
                    })
                    .sum::<usize>()
            })
            .unwrap_or(0);
        if expect_pruned {
            assert_eq!(pruned, 1, "out-of-range predicate must prune the file");
        } else {
            assert_eq!(pruned, 0, "in-range predicate must not prune the file");
        }
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn per_file_predicate_prunes_out_of_range_file() {
        // `id >= 100` vs a file with id ∈ [10,20] → pruned, 0 rows.
        assert_per_file_pruning(100, true, 0).await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn per_file_predicate_keeps_in_range_file() {
        // `id >= 5` vs a file with id ∈ [10,20] → kept, 3 rows, 0 pruned.
        assert_per_file_pruning(5, false, 3).await;
    }

    /// **Layer 1** (kernel file-list skipping): with a lowered `id >= 4` predicate on the scan, the
    /// `sm_plans` SSA path inserts a data-skipping `FilterNode` over `add.stats_parsed`, so file A
    /// (`id ∈ [1,3]`) is dropped from the **kernel's live-file terminal** before it enters the plan.
    ///
    /// The metadata-stats SM emits one terminal row per *surviving* live file, so the per-file stats
    /// map is the direct observation point: with the predicate the map holds only file B; without it
    /// (`None`, the control) both files survive. This proves the kernel — not just DataFusion's
    /// per-file pruner — skips the file. Asserted under both column-mapping modes since the kernel
    /// rewrites the logical `id` ref to the physical name itself.
    async fn assert_layer1_file_skipping(mode: &str) {
        use datafusion::prelude::{col, lit};

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
        let executor = DataFusionExecutor::new();
        let state = session.state();

        // Control: no predicate → the kernel enumerates both live files.
        let scan_all = build_scan(provider.snapshot(), None).expect("scan (no predicate)");
        let all = provider
            .build_file_stats(&state, &executor, &scan_all)
            .unwrap_or_else(|| panic!("[{mode}] expected stats for both files"));
        assert_eq!(all.len(), 2, "[{mode}] control: both files live");

        // With `id >= 4`: the kernel's data-skipping FilterNode drops file A (id ∈ [1,3]).
        let filters = [col("id").gt_eq(lit(4i64))];
        let predicate = lower_skipping_predicate(&filters);
        assert!(
            predicate.is_some(),
            "[{mode}] `id >= 4` must lower to a skipping predicate"
        );
        let scan_pruned = build_scan(provider.snapshot(), predicate).expect("scan (predicate)");
        let pruned = provider
            .build_file_stats(&state, &executor, &scan_pruned)
            .unwrap_or_else(|| panic!("[{mode}] file B still lives"));
        assert_eq!(
            pruned.len(),
            1,
            "[{mode}] kernel skipping must drop file A from the live-file list"
        );
        assert!(
            pruned.contains_key("part-b.parquet"),
            "[{mode}] the surviving file must be file B"
        );
        assert!(
            !pruned.contains_key("part-a.parquet"),
            "[{mode}] file A must be pruned by the kernel"
        );
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn layer1_file_skipping_name_mode() {
        assert_layer1_file_skipping("name").await;
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn layer1_file_skipping_id_mode() {
        assert_layer1_file_skipping("id").await;
    }
}
