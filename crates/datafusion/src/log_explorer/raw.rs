//! [`RawLogProvider`]: every log action as written, across commits and
//! checkpoints.
//!
//! Reads the frozen [`LogSegment`](delta_kernel::log_segment::LogSegment) of the
//! snapshot via `Snapshot::log_segment().read_actions(engine, schema)` and
//! materializes each [`ActionsBatch`](delta_kernel::log_replay::ActionsBatch) as
//! a `RecordBatch`. Every row carries at most one non-null action (add / remove /
//! metaData / protocol / txn / commitInfo / cdc / sidecar / checkpointMetadata /
//! domainMetadata), plus a synthesized boolean [`COMMIT_MARKER_COLUMN`] marking
//! whether the batch came from a commit file (`true`) or a checkpoint/CRC
//! (`false`).
//!
//! **Projection pushdown.** The `schema` handed to `read_actions` is projected to
//! exactly the action types the query touches (derived from DataFusion's column
//! projection). This is the load-bearing optimization for well-checkpointed
//! tables: the kernel reads only those columns from the checkpoint **parquet**,
//! and auto-derives an `IS NOT NULL` predicate from the projected schema so
//! checkpoint parquet **row groups** with no relevant action are skipped
//! entirely. (Commit `.json` files are always read whole — there is no column or
//! row-group I/O to save there — but only the projected columns are decoded.)
//!
//! This is the *raw* history — no reconciliation — the counterpart to
//! [`ReconciledLogProvider`](super::ReconciledLogProvider). The kernel drives the
//! read (checkpoint schema, sidecars, and log compaction all handled), so we
//! never parse `_delta_log` files ourselves. Modeled on the kernel's
//! `inspect-table` example (`Commands::Actions`).

use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use std::task::{Context, Poll};

use async_trait::async_trait;
use datafusion::arrow::array::{ArrayRef, BooleanArray, RecordBatch, RecordBatchOptions};
use datafusion::arrow::datatypes::{DataType, Field, Schema, SchemaRef};
use datafusion::catalog::{Session, TableProvider};
use datafusion::common::DataFusionError;
use datafusion::common::error::Result;
use datafusion::execution::{RecordBatchStream, SendableRecordBatchStream, TaskContext};
use datafusion::logical_expr::{Expr, TableProviderFilterPushDown, TableType};
use datafusion::physical_expr::EquivalenceProperties;
use datafusion::physical_plan::execution_plan::{Boundedness, EmissionType};
use datafusion::physical_plan::{
    DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PlanProperties,
};
use delta_kernel::actions::get_all_actions_schema;
use delta_kernel::arrow::datatypes::SchemaRef as ArrowSchemaRef;
use delta_kernel::engine::arrow_conversion::TryIntoArrow as _;
use delta_kernel::engine::arrow_data::ArrowEngineData;
use delta_kernel::log_replay::ActionsBatch;
use delta_kernel::snapshot::Snapshot;
use delta_kernel::{DeltaResult, Engine, Predicate, PredicateRef};
use deltalake_core::delta_datafusion::to_delta_predicate;
use futures::Stream;
use url::Url;

/// Name of the synthesized column marking commit (`true`) vs checkpoint
/// (`false`) provenance for each action row.
pub const COMMIT_MARKER_COLUMN: &str = "_commit";

/// The raw-action Arrow schema: the kernel's full action schema
/// ([`get_all_actions_schema`]) plus the [`COMMIT_MARKER_COLUMN`].
static RAW_ACTION_SCHEMA: LazyLock<ArrowSchemaRef> = LazyLock::new(|| {
    let kernel_arrow: Schema = get_all_actions_schema()
        .as_ref()
        .try_into_arrow()
        .expect("kernel action schema converts to arrow");
    let mut fields: Vec<Field> = kernel_arrow
        .fields()
        .iter()
        .map(|f| f.as_ref().clone())
        .collect();
    fields.push(Field::new(COMMIT_MARKER_COLUMN, DataType::Boolean, false));
    Arc::new(Schema::new(fields))
});

/// A DataFusion [`TableProvider`] over a Delta table's *raw* log: every action
/// as written, across all commit and checkpoint files.
///
/// Carries the delta-kernel [`Engine`] used to read the log, so it is fully
/// self-contained and depends on no session extension.
pub struct RawLogProvider {
    table: Url,
    engine: Arc<dyn Engine>,
}

impl std::fmt::Debug for RawLogProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawLogProvider")
            .field("table", &self.table)
            .finish_non_exhaustive()
    }
}

impl RawLogProvider {
    /// Build a provider for the Delta table rooted at `table`, reading its log
    /// with `engine`.
    pub fn new(mut table: Url, engine: Arc<dyn Engine>) -> Self {
        if !table.path().ends_with('/') {
            table.set_path(&format!("{}/", table.path()));
        }
        Self { table, engine }
    }

    /// The Arrow schema of the raw action rows: the kernel action schema plus
    /// the [`COMMIT_MARKER_COLUMN`].
    pub fn raw_action_schema() -> ArrowSchemaRef {
        Arc::clone(&RAW_ACTION_SCHEMA)
    }
}

/// A resolved read/output plan for one scan, derived from the DataFusion
/// projection. Holds the pushdown decision so `scan()` and `execute()` agree.
#[derive(Debug, Clone)]
struct RawProjection {
    /// Kernel action field names to read (a subset of `get_all_actions_schema`),
    /// in the order they should appear before `_commit` is appended. Drives the
    /// pushed-down `read_actions` schema.
    read_actions: Vec<String>,
    /// The output Arrow schema (subset of [`RawLogProvider::raw_action_schema`],
    /// in the requested order).
    output_schema: ArrowSchemaRef,
    /// Index remap: for each output column, its position in the assembled
    /// `[read_actions.., _commit?]` intermediate batch. Reorders the kernel's
    /// projected columns back into the exact order DataFusion asked for.
    output_indices: Vec<usize>,
}

impl RawProjection {
    /// Resolve the projection against the full raw schema.
    ///
    /// `projection` is `None` (all columns) or column indices into
    /// [`RawLogProvider::raw_action_schema`] (the 10 action structs then
    /// `_commit`). When no action column is selected (e.g. `SELECT _commit` or
    /// `count(*)`), one cheap action column is still read to preserve row
    /// cardinality, then dropped from the output.
    fn resolve(projection: Option<&Vec<usize>>) -> Result<Self> {
        let full = RawLogProvider::raw_action_schema();
        let commit_idx = full.fields().len() - 1; // `_commit` is last.

        let indices: Vec<usize> = match projection {
            Some(p) => p.clone(),
            None => (0..full.fields().len()).collect(),
        };

        // Action field names requested, in projection order (excluding _commit).
        let mut read_actions: Vec<String> = indices
            .iter()
            .filter(|&&i| i != commit_idx)
            .map(|&i| full.field(i).name().to_string())
            .collect();

        // Preserve cardinality when only `_commit` (or nothing) is requested:
        // read one narrow action column purely so the kernel yields rows. `txn`
        // (SetTransaction) is a small action struct — cheap to read.
        if read_actions.is_empty() {
            read_actions.push("txn".to_string());
        }

        // Output schema in the requested order.
        let output_fields: Vec<Field> = indices
            .iter()
            .map(|&i| full.field(i).as_ref().clone())
            .collect();
        let output_schema = Arc::new(Schema::new(output_fields));

        // Map each output column to its slot in the assembled intermediate batch
        // `[read_actions.., _commit?]`.
        let commit_slot = read_actions.len(); // _commit appended after actions.
        let output_indices: Vec<usize> = indices
            .iter()
            .map(|&i| {
                if i == commit_idx {
                    commit_slot
                } else {
                    let name = full.field(i).name();
                    // Position within `read_actions` (stable: same order).
                    read_actions
                        .iter()
                        .position(|n| n == name)
                        .expect("requested action is in read set")
                }
            })
            .collect();

        Ok(Self {
            read_actions,
            output_schema,
            output_indices,
        })
    }

    /// The projected kernel action schema to hand to `read_actions` — the
    /// pushdown that prunes checkpoint parquet columns and row groups.
    fn read_schema(&self) -> DeltaResult<delta_kernel::schema::SchemaRef> {
        get_all_actions_schema().project(&self.read_actions)
    }
}

#[async_trait]
impl TableProvider for RawLogProvider {
    fn schema(&self) -> ArrowSchemaRef {
        Self::raw_action_schema()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        // A pushed filter only skips checkpoint parquet row groups (an I/O hint);
        // surviving rows still include non-matching ones, so every filter is
        // `Inexact` — DataFusion must re-apply it. Filters we cannot translate
        // are simply not pushed (they still get re-applied), so `Inexact` is safe
        // for all of them.
        Ok(vec![TableProviderFilterPushDown::Inexact; filters.len()])
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let plan = RawProjection::resolve(projection)?;
        let meta_predicate = meta_predicate_from_filters(filters);
        let exec = RawLogExec {
            engine: self.engine.clone(),
            table: self.table.clone(),
            meta_predicate,
            properties: Arc::new(PlanProperties::new(
                EquivalenceProperties::new(plan.output_schema.clone()),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Incremental,
                Boundedness::Bounded,
            )),
            plan,
        };
        Ok(Arc::new(exec))
    }
}

/// Translate the DataFusion `filters` into one kernel [`Predicate`] for
/// checkpoint-parquet row-group skipping (a *hint*: it never removes matching
/// rows, only lets the kernel skip row groups that cannot match).
///
/// Filters that don't translate — [`to_delta_predicate`] returns an error, e.g.
/// UDFs or shapes the kernel predicate model can't represent — are dropped; they
/// are still re-applied by DataFusion (we report every filter `Inexact`), so
/// dropping them only forgoes an I/O optimization, never correctness. Returns
/// `None` when nothing translated.
fn meta_predicate_from_filters(filters: &[Expr]) -> Option<PredicateRef> {
    let translated: Vec<Predicate> = filters
        .iter()
        .filter_map(|f| to_delta_predicate(f).ok())
        .collect();
    let mut it = translated.into_iter();
    let first = it.next()?;
    Some(Arc::new(it.fold(first, Predicate::and)))
}

struct RawLogExec {
    engine: Arc<dyn Engine>,
    table: Url,
    plan: RawProjection,
    /// Checkpoint-parquet row-group-skipping hint from the query's filters.
    meta_predicate: Option<PredicateRef>,
    properties: Arc<PlanProperties>,
}

impl std::fmt::Debug for RawLogExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawLogExec")
            .field("read_actions", &self.plan.read_actions)
            .finish_non_exhaustive()
    }
}

impl DisplayAs for RawLogExec {
    fn fmt_as(&self, t: DisplayFormatType, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match t {
            DisplayFormatType::Default
            | DisplayFormatType::Verbose
            | DisplayFormatType::TreeRender => {
                write!(f, "RawLogExec: actions={:?}", self.plan.read_actions)
            }
        }
    }
}

impl ExecutionPlan for RawLogExec {
    fn name(&self) -> &'static str {
        "RawLogExec"
    }

    fn properties(&self) -> &Arc<PlanProperties> {
        &self.properties
    }

    fn children(&self) -> Vec<&Arc<dyn ExecutionPlan>> {
        vec![]
    }

    fn with_new_children(
        self: Arc<Self>,
        _: Vec<Arc<dyn ExecutionPlan>>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        Ok(self)
    }

    fn execute(
        &self,
        partition: usize,
        _context: Arc<TaskContext>,
    ) -> Result<SendableRecordBatchStream> {
        if partition != 0 {
            return Err(DataFusionError::Execution(
                "RawLogExec only supports a single partition".into(),
            ));
        }

        // Reading the log segment drives synchronous kernel replay; do it up
        // front (mirroring the reconciled provider's `scan_metadata` call) and
        // materialize the batches into memory. A log is small relative to table
        // data, so a plain in-memory iterator keeps the stream simple.
        let batches = read_raw_actions(
            self.engine.as_ref(),
            &self.table,
            &self.plan,
            self.meta_predicate.clone(),
        )
        .map_err(|e| DataFusionError::Execution(e.to_string()))?;

        Ok(Box::pin(RawLogStream {
            schema: self.plan.output_schema.clone(),
            batches: batches.into_iter(),
        }))
    }
}

/// Read the raw actions the projection selected and assemble the output batches.
///
/// The kernel reads only the projected action columns from the checkpoint
/// parquet (column pruning) and, when `meta_predicate` is set, skips checkpoint
/// parquet row groups that cannot match it (combined AND with the schema-derived
/// `IS NOT NULL` skip). We append `_commit` and reorder to the exact projection.
fn read_raw_actions(
    engine: &dyn Engine,
    table: &Url,
    plan: &RawProjection,
    meta_predicate: Option<PredicateRef>,
) -> DeltaResult<Vec<RecordBatch>> {
    let snapshot = Snapshot::builder_for(table.as_str()).build(engine)?;
    let read_schema = plan.read_schema()?;
    // Same schema for commits and checkpoints; the kernel derives the
    // `IS NOT NULL` action-presence skip from it and ANDs our `meta_predicate`.
    let actions = snapshot
        .log_segment()
        .read_actions_with_projected_checkpoint_actions(
            engine,
            read_schema.clone(),
            read_schema,
            meta_predicate,
            None,
            None,
        )?
        .actions;

    let mut out = Vec::new();
    for batch in actions {
        let ActionsBatch {
            actions,
            is_log_batch,
        } = batch?;
        let data = ArrowEngineData::try_from_engine_data(actions)?;
        let kernel_batch = data.record_batch();
        let num_rows = kernel_batch.num_rows();

        // Assemble the intermediate `[read_actions.. , _commit]` (the marker is
        // always appended so the `_commit` slot in `output_indices` is valid),
        // then reorder/drop to the exact requested projection.
        let mut columns: Vec<ArrayRef> = kernel_batch.columns().to_vec();
        let marker: ArrayRef = Arc::new(BooleanArray::from(vec![is_log_batch; num_rows]));
        columns.push(marker);

        let projected: Vec<ArrayRef> = plan
            .output_indices
            .iter()
            .map(|&i| columns[i].clone())
            .collect();
        // A zero-column projection (`count(*)`) still needs the row count, which
        // an empty column list cannot convey — set it explicitly.
        let options = RecordBatchOptions::new().with_row_count(Some(num_rows));
        let batch =
            RecordBatch::try_new_with_options(plan.output_schema.clone(), projected, &options)
                .map_err(|e| delta_kernel::Error::generic(e.to_string()))?;
        out.push(batch);
    }
    Ok(out)
}

struct RawLogStream {
    schema: SchemaRef,
    batches: std::vec::IntoIter<RecordBatch>,
}

impl Stream for RawLogStream {
    type Item = Result<RecordBatch>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(self.get_mut().batches.next().map(Ok))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.batches.size_hint()
    }
}

impl RecordBatchStream for RawLogStream {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
}
