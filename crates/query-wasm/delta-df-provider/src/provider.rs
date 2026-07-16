//! [`DeltaSsaTableProvider`]: the async-native, engine-free Delta [`TableProvider`] that
//! replaces the eager inline-executor scan path.
//!
//! Holds only a kernel [`SnapshotRef`] plus a small [`DeltaSsaScanConfig`] — **no engine**. At
//! `scan()` time it:
//!
//!   1. builds a kernel [`Scan`] from the snapshot (`scan_builder().build_replay()`),
//!   2. drives the scan's `sm_plans` coroutine state machine
//!      ([`Scan::scan_state_machine`]) to a [`ResultPlan`] through the engine-free
//!      [`DataFusionExecutor`] — a `!Send`, IO-free, CPU-only planning step,
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
        // Belt-and-suspenders: the view-type override is applied through the session config knob
        // (`datafusion.execution.parquet.schema_force_view_types`). When the provider is
        // configured for plain types (browser), the caller's session must agree, otherwise the
        // physical parquet reader would emit `Utf8View` and the browser IPC reader would choke
        // (mangrove #28). We only warn-by-error on a genuine mismatch to keep the seam explicit.
        let session_force_view = session
            .config_options()
            .execution
            .parquet
            .schema_force_view_types;
        if session_force_view != self.config.schema_force_view_types {
            return Err(crate::error::plan_compilation(format!(
                "DeltaSsaTableProvider: session parquet.schema_force_view_types={session_force_view} \
                 disagrees with provider config schema_force_view_types={} — set them to match \
                 (the wasm preview path uses false so the browser arrow IPC reader can decode)",
                self.config.schema_force_view_types
            )));
        }

        // Build the kernel scan and drive its `sm_plans` coroutine state machine to a ResultPlan.
        // This is the `!Send`, IO-free planning step — no engine, no InlineExecutor.
        //
        // The SM future is `!Send` (genawaiter2 `rc`), but the kernel guarantees it never awaits
        // real IO — every yield is a synchronous trampoline hop. So we drive it to completion
        // with `futures::executor::block_on`, which needs no `Send` bound and completes without
        // cooperating with the outer runtime. This confines the entire `!Send` region to a single
        // synchronous call: nothing `!Send` is ever held across the `#[async_trait]` `scan`
        // future's `.await` points, so the returned `ExecutionPlan` future stays `Send`.
        let scan = build_scan(&self.snapshot)?;
        let sm = scan
            .scan_state_machine()
            .map_err(crate::error::wrap_delta_err)?;
        let executor = DataFusionExecutor::new();
        let result_plan = futures::executor::block_on(executor.drive_to_completion(sm))
            .map_err(crate::error::wrap_delta_err)?;

        // Compile the SSA result plan to a bare LogicalPlan, then plan it against the *caller's*
        // session so file reads go through the caller's object store + runtime.
        let logical = executor
            .compile_result_plan(&result_plan)
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
        // Filter pushdown is off for v1 (mirrors the POC's LoadTableProvider); projection and
        // limit flow through. DataFusion re-applies filters above the scan.
        Ok(vec![
            TableProviderFilterPushDown::Unsupported;
            filters.len()
        ])
    }
}
