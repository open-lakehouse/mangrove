//! Streaming physical plan behind [`super::LoadTableProvider`].
//!
//! For each upstream metadata row, [`build_load_stream`] runs `buffer_unordered` over open
//! futures: each builds a per-file plan via [`build_per_file_plan`] (a bare `DataSourceExec`
//! over DataFusion's async parquet/json source) and drains it. Output ordering across files is
//! unspecified; intra-file order is preserved.
//!
//! Deletion vectors are gated to unsupported upstream, so this exec carries no kernel `Engine`
//! and no `_row_number` virtual column. The scan SSA always attaches a `dv_ref` column pointer,
//! so the guard is per-row: a row bearing a non-null DV descriptor errors in `extract_row_inputs`.

use std::fmt;
use std::sync::Arc;

use datafusion_common::Result as DfResult;
use datafusion_common::error::DataFusionError;
use datafusion_execution::TaskContext;
use datafusion_physical_expr::equivalence::EquivalenceProperties;
use datafusion_physical_expr_common::physical_expr::PhysicalExpr;
use datafusion_physical_plan::execution_plan::EmissionType;
use datafusion_physical_plan::stream::RecordBatchStreamAdapter;
use datafusion_physical_plan::{
    DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
    SendableRecordBatchStream,
};
use delta_kernel::arrow::array::RecordBatch;
use delta_kernel::arrow::datatypes::SchemaRef as ArrowSchemaRef;
use delta_kernel::sm_plans::ir::nodes::LoadNode;
use futures::stream::{Stream, StreamExt, TryStreamExt};

use crate::compile::stats::FileStatsMap;
use crate::exec::load_helpers::{
    RowInputs, build_file_source, build_per_file_plan, extract_row_inputs,
};

/// Per-partition file-open concurrency when `target_partitions` is zero.
const DEFAULT_LOAD_CONCURRENCY: usize = 8;

/// Caps task pressure for very-wide scans.
const MAX_LOAD_CONCURRENCY: usize = 64;

pub struct LoadExec {
    node: Arc<LoadNode>,
    upstream: Arc<dyn ExecutionPlan>,
    /// Pre-projection schema (= file_schema fields ++ passthrough fields). Kept so
    /// `with_new_children` can rebuild against the same shape.
    full_schema: ArrowSchemaRef,
    projection: Option<Vec<usize>>,
    output_schema: ArrowSchemaRef,
    limit: Option<usize>,
    /// Indices into `node.passthrough_columns` to materialize, in projected order. `Arc` so
    /// per-row open futures can clone cheaply.
    projected_passthrough: Arc<Vec<usize>>,
    /// File source for the (DV-free) load path. Carries the scan-global pushdown predicate (parquet
    /// only) when one was threaded in — see [`build_file_source`].
    file_source: Arc<dyn datafusion_datasource::file::FileSource>,
    /// Per-file statistics (keyed by raw `add.path`) to stamp onto each per-file `PartitionedFile`;
    /// `None` unless the provider drove a stats-enabled scan. Cloned across `with_new_children`.
    file_stats: Option<Arc<FileStatsMap>>,
    /// Scan-global, logical-named parquet pruning predicate; `None` unless the provider lowered
    /// query filters. Kept as a field so `with_new_children` can rebuild `file_source` with it.
    predicate: Option<Arc<dyn PhysicalExpr>>,
    properties: Arc<PlanProperties>,
}

impl LoadExec {
    pub fn new(
        upstream: Arc<dyn ExecutionPlan>,
        node: Arc<LoadNode>,
        full_schema: ArrowSchemaRef,
        projection: Option<Vec<usize>>,
        limit: Option<usize>,
        file_stats: Option<Arc<FileStatsMap>>,
        predicate: Option<Arc<dyn PhysicalExpr>>,
    ) -> DfResult<Self> {
        // The scan SSA always attaches a `dv_ref` column pointer (see `extract_row_inputs`), so
        // rejecting on `node.dv_ref.is_some()` here would reject every commit-only table. The
        // guard is per-row instead: a non-null DV descriptor errors during the stream.
        let file_count = node.file_schema.fields().len();
        let passthrough_count = node.passthrough_columns.len();
        debug_assert_eq!(full_schema.fields().len(), file_count + passthrough_count);

        let file_source = build_file_source(
            node.file_type,
            &full_schema,
            file_count,
            projection.as_deref(),
            predicate.clone(),
        )?;

        let output_schema = match projection.as_ref() {
            Some(proj) => Arc::new(full_schema.project(proj)?),
            None => Arc::clone(&full_schema),
        };

        // Filter projection indices to passthrough range (>= file_count) and translate to
        // passthrough-local indices.
        let projected_passthrough: Vec<usize> = match projection.as_ref() {
            Some(proj) => proj
                .iter()
                .copied()
                .filter(|&i| i >= file_count)
                .map(|i| i - file_count)
                .collect(),
            None => (0..passthrough_count).collect(),
        };

        let properties = Arc::new(PlanProperties::new(
            EquivalenceProperties::new(Arc::clone(&output_schema)),
            // Single partition: the merger interleaves files within one stream.
            Partitioning::UnknownPartitioning(1),
            EmissionType::Incremental,
            upstream.properties().boundedness,
        ));
        Ok(Self {
            node,
            upstream,
            full_schema,
            projection,
            output_schema,
            limit,
            projected_passthrough: Arc::new(projected_passthrough),
            file_source,
            file_stats,
            predicate,
            properties,
        })
    }
}

impl fmt::Debug for LoadExec {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoadExec")
            .field("file_type", &self.node.file_type)
            .field("projection", &self.projection)
            .field("limit", &self.limit)
            .field("output_fields", &self.output_schema.fields().len())
            .finish_non_exhaustive()
    }
}

impl DisplayAs for LoadExec {
    fn fmt_as(&self, _: DisplayFormatType, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "LoadExec(file_type={:?}, projection={:?}, limit={:?}, output_fields={})",
            self.node.file_type,
            self.projection,
            self.limit,
            self.output_schema.fields().len(),
        )
    }
}

impl ExecutionPlan for LoadExec {
    fn name(&self) -> &str {
        "LoadExec"
    }

    fn schema(&self) -> ArrowSchemaRef {
        Arc::clone(&self.output_schema)
    }

    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }

    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![&self.upstream]
    }

    fn with_new_children(
        self: Arc<Self>,
        children: Vec<Arc<dyn ExecutionPlan>>,
    ) -> DfResult<Arc<dyn ExecutionPlan>> {
        let [upstream] = children.try_into().map_err(|c: Vec<_>| {
            DataFusionError::Plan(format!(
                "LoadExec requires exactly one child, got {}",
                c.len()
            ))
        })?;
        Ok(Arc::new(LoadExec::new(
            upstream,
            Arc::clone(&self.node),
            Arc::clone(&self.full_schema),
            self.projection.clone(),
            self.limit,
            self.file_stats.clone(),
            self.predicate.clone(),
        )?))
    }

    fn execute(
        &self,
        partition: usize,
        context: Arc<TaskContext>,
    ) -> DfResult<SendableRecordBatchStream> {
        if partition != 0 {
            return Err(DataFusionError::Plan(format!(
                "LoadExec only supports partition 0, got {partition}"
            )));
        }
        // Coalesce the upstream's partitions into one stream so the row expander sees one
        // batch at a time.
        let upstream = datafusion::physical_plan::execute_stream(
            Arc::clone(&self.upstream),
            Arc::clone(&context),
        )?;
        let concurrency = context
            .session_config()
            .target_partitions()
            .clamp(1, MAX_LOAD_CONCURRENCY);
        let concurrency = if concurrency == 0 {
            DEFAULT_LOAD_CONCURRENCY
        } else {
            concurrency
        };
        let stream = build_load_stream(
            upstream,
            Arc::clone(&self.node),
            Arc::clone(&self.file_source),
            Arc::clone(&self.projected_passthrough),
            Arc::clone(&self.output_schema),
            context,
            self.limit,
            concurrency,
            self.file_stats.clone(),
        );
        Ok(Box::pin(RecordBatchStreamAdapter::new(
            Arc::clone(&self.output_schema),
            stream,
        )))
    }
}

/// Up to `concurrency` per-row open futures run at once via `buffer_unordered`; the outer
/// `try_flatten` interleaves files freely while preserving intra-file batch order. `limit`
/// is enforced by slicing the final batch + early-terminating.
#[allow(clippy::too_many_arguments)]
fn build_load_stream(
    upstream: SendableRecordBatchStream,
    node: Arc<LoadNode>,
    file_source: Arc<dyn datafusion_datasource::file::FileSource>,
    projected_passthrough: Arc<Vec<usize>>,
    output_schema: ArrowSchemaRef,
    task_context: Arc<TaskContext>,
    limit: Option<usize>,
    concurrency: usize,
    file_stats: Option<Arc<FileStatsMap>>,
) -> impl Stream<Item = DfResult<RecordBatch>> + Send + 'static {
    // Explode upstream batches into one item per row.
    let row_stream = upstream
        .map_ok(|batch| {
            let n = batch.num_rows();
            let batch = Arc::new(batch);
            futures::stream::iter((0..n).map(move |row| DfResult::Ok((Arc::clone(&batch), row))))
        })
        .try_flatten();

    // Per row, an open future producing the per-file `RecordBatch` stream.
    let per_file_streams = row_stream.map(move |row_result: DfResult<_>| {
        let node = Arc::clone(&node);
        let task_ctx = Arc::clone(&task_context);
        let pt = Arc::clone(&projected_passthrough);
        let file_source = Arc::clone(&file_source);
        let output_schema = Arc::clone(&output_schema);
        let file_stats = file_stats.clone();

        async move {
            let (batch, row) = row_result?;
            let inputs: RowInputs = extract_row_inputs(&batch, row, &node, &pt)?;

            // Look up this file's per-file statistics by its raw `add.path` (the map is keyed the
            // same way). `None` when stats are disabled or the file has no recorded stats.
            let statistics = file_stats
                .as_ref()
                .and_then(|m| m.get(&inputs.raw_path).cloned());

            let plan = build_per_file_plan(
                inputs,
                file_source,
                node.file_type,
                &output_schema,
                task_ctx.as_ref(),
                statistics,
            )
            .await?;
            let stream = plan.execute(0, task_ctx)?;
            Ok::<_, DataFusionError>(stream)
        }
    });

    // Concurrent flatten + limit slicing.
    let flattened = per_file_streams.buffer_unordered(concurrency).try_flatten();
    async_stream::try_stream! {
        let mut remaining = limit;
        let mut s = std::pin::pin!(flattened);
        while let Some(batch) = s.try_next().await? {
            let mut out = batch;
            if let Some(rem) = remaining.as_mut() {
                if out.num_rows() > *rem {
                    out = out.slice(0, *rem);
                }
                *rem -= out.num_rows();
            }
            if out.num_rows() > 0 {
                yield out;
            }
            if matches!(remaining, Some(0)) {
                return;
            }
        }
    }
}
