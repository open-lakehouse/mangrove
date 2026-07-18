//! [`ReconciledLogProvider`]: a DataFusion [`TableProvider`] over a Delta table's *reconciled* log
//! — the surviving scan-file rows after log replay (add/remove tombstoning + protocol/metadata
//! resolution).
//!
//! This is the async-native, **engine-free** counterpart to the eager
//! `ReconciledLogProvider` in the `olai-uc-datafusion` crate's `log_explorer` module. Where that
//! one holds a kernel `Engine` and wraps `scan.scan_metadata(engine)` in a custom `ExecutionPlan`,
//! this one holds only a [`SnapshotRef`] and drives the kernel `sm_plans` metadata coroutine state
//! machine through the stateless [`DataFusionExecutor`] — the same planning path as
//! [`crate::DeltaSsaTableProvider`], so it works natively and on wasm.
//!
//! # Schema
//!
//! The emitted rows are the `sm_plans` **flat scan-file-row** shape (kernel
//! `project_scan_file_row`), **not** the eager kernel `scan_row_schema()` the old crate exposes:
//!
//! ```text
//! {
//!   path: STRING NOT NULL,
//!   size: LONG NOT NULL,
//!   deletionVector: STRUCT<…>?,
//!   fileConstantValues: STRUCT<
//!     baseRowId, defaultRowCommitVersion, tags, clusteringProvider,
//!     partitionValues_parsed?: STRUCT<…>,   // present iff the table is partitioned
//!   >?,
//!   stats?: STRUCT<numRecords, nullCount{…}, minValues{…}, maxValues{…}, tightBounds>,
//! }
//! ```
//!
//! Compared to `scan_row_schema()` this drops `modificationTime`, the string `stats`, and the raw
//! `partitionValues` map, and reorders `fileConstantValues`. The `stats` and `partitionValues_parsed`
//! sub-structs are named with **logical** (table-facing) column names — the intended, user-facing
//! improvement for log inspection over the raw physical (`col-…` / field-id) leaf names. So SQL
//! written against the old provider does not necessarily port verbatim.
//!
//! The `stats` column is present exactly when the snapshot's table carries indexed data columns
//! (the scan is always built with `StatsOptions::all_struct()`); it is a pruning-free, purely
//! informational view of the per-file `add.stats_parsed`.

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::catalog::{Session, TableProvider};
use datafusion_common::Result as DfResult;
use datafusion_expr::utils::conjunction;
use datafusion_expr::{Expr, LogicalPlanBuilder, TableProviderFilterPushDown, TableType};
use datafusion_physical_plan::ExecutionPlan;
use delta_kernel::arrow::datatypes::SchemaRef as ArrowSchemaRef;
use delta_kernel::engine::arrow_conversion::TryIntoArrow;
use delta_kernel::snapshot::SnapshotRef;

use crate::DataFusionExecutor;
use crate::provider::{DeltaSsaScanConfig, build_scan};

/// A DataFusion [`TableProvider`] over a Delta table's reconciled log: the surviving scan-file rows
/// after kernel `sm_plans` log replay.
///
/// Engine-free and async-native — holds only a [`SnapshotRef`] plus a [`DeltaSsaScanConfig`], and
/// derives everything else from the `Session` at scan time (like [`crate::DeltaSsaTableProvider`]).
/// The caller builds the snapshot (via [`crate::build_snapshot_from_manifest`], or a native
/// `Snapshot::builder_for(..).build(&engine)` in tests) and the `Session` via
/// [`crate::delta_engine_session`].
///
/// See the [module docs](self) for the emitted schema, which differs from the eager crate's
/// `scan_row_schema()`.
pub struct ReconciledLogProvider {
    snapshot: SnapshotRef,
    config: DeltaSsaScanConfig,
    /// Pre-materialized arrow schema so `schema()` is cheap and infallible. Computed once in
    /// [`Self::new`] from the kernel `Scan::scan_file_row_schema()` accessor — the exact schema the
    /// metadata terminal emits, without driving the state machine.
    arrow_schema: ArrowSchemaRef,
}

impl std::fmt::Debug for ReconciledLogProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconciledLogProvider")
            .field("version", &self.snapshot.version())
            .field(
                "schema_force_view_types",
                &self.config.schema_force_view_types,
            )
            .finish_non_exhaustive()
    }
}

impl ReconciledLogProvider {
    /// Construct from a kernel [`SnapshotRef`] and scan config.
    ///
    /// Session-free: the flat scan-file-row schema is a pure function of the table schema + scan
    /// options, so it is materialized once here from the kernel `Scan::scan_file_row_schema()`
    /// accessor (no planning drive, no session needed). Mirrors [`crate::DeltaSsaTableProvider::new`].
    pub fn new(snapshot: SnapshotRef, config: DeltaSsaScanConfig) -> DfResult<Self> {
        // Build the scan the same way `scan()` does (with struct stats requested), so the schema we
        // declare here matches what the driven metadata terminal emits, including the `stats` column.
        let scan = build_scan(&snapshot, None)?;
        let kernel_schema = scan.scan_file_row_schema();
        let arrow_schema: ArrowSchemaRef =
            Arc::new(kernel_schema.as_ref().try_into_arrow().map_err(|e| {
                crate::error::plan_compilation(format!("scan-file-row schema: {e}"))
            })?);
        Ok(Self {
            snapshot,
            config,
            arrow_schema,
        })
    }

    /// The kernel snapshot this provider reflects.
    pub fn snapshot(&self) -> &SnapshotRef {
        &self.snapshot
    }
}

#[async_trait]
impl TableProvider for ReconciledLogProvider {
    fn schema(&self) -> ArrowSchemaRef {
        Arc::clone(&self.arrow_schema)
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> DfResult<Vec<TableProviderFilterPushDown>> {
        // `Exact`: unlike the data provider (whose filters are over *table* columns and must be
        // translated into a stats/parquet pruning predicate — hence `Inexact`), this provider
        // exposes the scan-file-row schema *directly as its query schema*. So a query predicate is
        // already expressed in the exact columns we emit (`path`, `size`, `stats.minValues.…`, …);
        // no translation is needed. `scan()` splices it as a `Filter` node over the emitted rows,
        // applying it completely — so DataFusion can drop its own redundant `FilterExec`.
        Ok(vec![TableProviderFilterPushDown::Exact; filters.len()])
    }

    async fn scan(
        &self,
        session: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        limit: Option<usize>,
    ) -> DfResult<Arc<dyn ExecutionPlan>> {
        // Same session contract as the data provider: the SM drive and the final
        // `create_physical_plan` both run against this session, so it must carry the Delta engine
        // config (leaf-pushdown off, single partition, no stats) and a matching
        // `schema_force_view_types`. Build it via `delta_engine_session` / `with_delta_engine`.
        crate::validate_delta_engine_session(session, self.config.schema_force_view_types)?;

        // Drive the metadata-with-stats coroutine state machine to a bare `LogicalPlan`. This is
        // planning only — `drive_ssa_to_plan` runs `drive_to_completion` + `compile_result_plan`,
        // never `create_physical_plan` / `collect` — so the `!Send` drive is confined to this
        // synchronous `block_on` (nothing `!Send` is held across the `.await` below, keeping the
        // returned future `Send`) and performs no object-store IO: commit-only tables defer the
        // commit-`.json` `Load` into the plan, and a checkpoint's footer `SchemaQuery` resolves
        // CPU-side. Real reads happen later, on the returned `ExecutionPlan`, via DataFusion's
        // async stack — so this works on native and wasm alike, like `DeltaSsaTableProvider::scan`.
        //
        // The *stats* metadata SM (not the plain one) is driven so the terminal carries the `stats`
        // column that `schema()` advertises.
        let scan = build_scan(&self.snapshot, None)?;
        let sm = scan
            .scan_stats_metadata_state_machine()
            .map_err(crate::error::wrap_delta_err)?;
        let executor = DataFusionExecutor::new();
        let logical = futures::executor::block_on(executor.drive_ssa_to_plan(session, sm))
            .map_err(crate::error::wrap_delta_err)?;

        // Splice filter -> projection -> limit at the logical level.
        //
        // Filter first, and over the FULL emitted schema (before projection prunes columns): a
        // predicate may reference a column the query doesn't select. Because our query schema *is*
        // the scan-file-row schema, each `Expr` already references the emitted columns verbatim —
        // no translation — so a plain conjoined `Filter` node applies it exactly. We reported
        // `Exact`, so DataFusion adds no `FilterExec` of its own; this is the sole application.
        let mut builder = LogicalPlanBuilder::from(logical);
        if let Some(predicate) = conjunction(filters.iter().cloned()) {
            builder = builder.filter(predicate)?;
        }
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
}
