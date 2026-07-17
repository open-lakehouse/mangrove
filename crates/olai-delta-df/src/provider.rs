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
use delta_kernel::scan::{Scan, ScanBuilder};
use delta_kernel::snapshot::SnapshotRef;

use crate::DataFusionExecutor;

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
}

/// Build the kernel `Scan` used both for `schema()` and for driving the state machine.
/// `ScanBuilder::new` takes `impl Into<SnapshotRef>`, so we clone the `Arc` (cheap).
fn build_scan(snapshot: &SnapshotRef) -> DfResult<Scan> {
    ScanBuilder::new(snapshot.clone())
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

        // Compile the SSA result plan to a bare LogicalPlan, then plan it against the *caller's*
        // session so file reads go through the caller's object store + runtime.
        let logical = DataFusionExecutor::compile_result_plan(&result_plan)
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
