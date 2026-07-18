//! Lazy [`TableProvider`] for `NodeKind::Load`: defers all work to `scan()`, which lowers
//! the upstream `LogicalPlan` and wraps it in a [`super::LoadExec`]. Filter pushdown is
//! currently off; projection and limit flow through to [`super::LoadExec`].
//!
//! # Not a public entry point
//!
//! This is a `pub(crate)`, **internal** provider: the SSA compiler ([`lower_load`]) emits one per
//! `Load` IR node so a per-file read can appear as a table-scan leaf inside a compiled
//! `LogicalPlan`. It is NOT a table-level provider and is not interchangeable with the crate's one
//! public provider, [`crate::DeltaSsaTableProvider`] — which is what callers register for a Delta
//! table, and whose compiled scan plan is itself *built out of* these `LoadTableProvider` leaves.
//! Both `impl TableProvider` only because a DataFusion table-scan leaf is the idiomatic way to
//! splice a custom [`ExecutionPlan`] ([`LoadExec`]) into a plan.
//!
//! This DV-free port holds **no kernel `Engine`**: file decoding goes entirely through
//! DataFusion's parquet/json sources over the `Session`/`TaskContext` object store, and the
//! deletion-vector path (the POC's only engine consumer) is gated out in v1.
//!
//! [`lower_load`]: crate::compile
//! [`LoadExec`]: super::LoadExec

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::catalog::{Session, TableProvider};
use datafusion_common::Result as DfResult;
use datafusion_common::error::DataFusionError;
use datafusion_expr::logical_plan::LogicalPlan;
use datafusion_expr::{Expr, TableProviderFilterPushDown, TableType};
use datafusion_physical_plan::ExecutionPlan;
use delta_kernel::arrow::datatypes::SchemaRef as ArrowSchemaRef;
use delta_kernel::engine::arrow_conversion::TryIntoArrow;
use delta_kernel::schema::SchemaRef;
use delta_kernel::sm_plans::ir::nodes::LoadNode;

use datafusion_physical_expr_common::physical_expr::PhysicalExpr;

use crate::compile::stats::FileStatsMap;
use crate::exec::LoadExec;

pub struct LoadTableProvider {
    upstream_logical: LogicalPlan,
    node: Arc<LoadNode>,
    /// `file_schema_fields ++ passthrough_fields`, pre-materialized so `schema()` is cheap.
    output_schema: ArrowSchemaRef,
    /// Per-file statistics (keyed by raw `add.path`) to stamp onto each per-file `PartitionedFile`;
    /// `None` unless the provider drove a stats-enabled scan. Threaded to [`LoadExec`].
    file_stats: Option<Arc<FileStatsMap>>,
    /// Scan-global, logical-named parquet pruning predicate; `None` unless the provider lowered
    /// query filters. Threaded to [`LoadExec`], which applies it once onto the parquet source.
    predicate: Option<Arc<dyn PhysicalExpr>>,
}

impl LoadTableProvider {
    /// Construct from the SSA `NodeKind::Load` payload plus the precomputed kernel-typed
    /// output schema. The caller (SSA `lower_load`) computes `output_kernel_schema` by
    /// composing the load's `file_schema` with the per-passthrough-column types resolved
    /// against the upstream's kernel schema; this provider just converts it to arrow.
    /// `file_stats` (raw-`add.path`-keyed) is attached to each per-file plan at scan time;
    /// `predicate` (scan-global, logical-named) is applied once onto the parquet source.
    pub fn try_new(
        upstream_logical: LogicalPlan,
        node: Arc<LoadNode>,
        output_kernel_schema: SchemaRef,
        file_stats: Option<Arc<FileStatsMap>>,
        predicate: Option<Arc<dyn PhysicalExpr>>,
    ) -> Result<Self, DataFusionError> {
        let output_schema: ArrowSchemaRef = Arc::new(
            output_kernel_schema
                .as_ref()
                .try_into_arrow()
                .map_err(|e| {
                    crate::error::plan_compilation(format!("LoadTableProvider output schema: {e}"))
                })?,
        );
        Ok(Self {
            upstream_logical,
            node,
            output_schema,
            file_stats,
            predicate,
        })
    }
}

impl std::fmt::Debug for LoadTableProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadTableProvider")
            .field("file_type", &self.node.file_type)
            .field("output_field_count", &self.output_schema.fields().len())
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl TableProvider for LoadTableProvider {
    fn schema(&self) -> ArrowSchemaRef {
        Arc::clone(&self.output_schema)
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    async fn scan(
        &self,
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        limit: Option<usize>,
    ) -> DfResult<Arc<dyn ExecutionPlan>> {
        let upstream_physical = state.create_physical_plan(&self.upstream_logical).await?;
        let load_exec = LoadExec::new(
            upstream_physical,
            Arc::clone(&self.node),
            Arc::clone(&self.output_schema),
            projection.cloned(),
            limit,
            self.file_stats.clone(),
            self.predicate.clone(),
        )?;
        Ok(Arc::new(load_exec))
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> DfResult<Vec<TableProviderFilterPushDown>> {
        Ok(vec![
            TableProviderFilterPushDown::Unsupported;
            filters.len()
        ])
    }
}
