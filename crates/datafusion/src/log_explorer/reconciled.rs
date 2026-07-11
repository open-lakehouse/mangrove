//! [`ReconciledLogProvider`]: the effective add-file state at a snapshot version.
//!
//! Runs kernel log replay via `Snapshot::scan_builder().scan_metadata(..)` and
//! materializes the surviving scan-file rows as Arrow (schema
//! [`scan_row_schema`]). This is the *reconciled* view — add/remove tombstoning
//! and protocol+metadata resolution already applied — the counterpart to
//! [`RawLogProvider`](super::RawLogProvider)'s as-written history.
//!
//! Generalized from the server's `DeltaLogReplayProvider`
//! (`crates/server/src/services/kernel/delta_log.rs`), which serves the same rows
//! for Delta Sharing `query_table`.
//!
//! **Checkpoint pushdown is already handled by the kernel.** `scan_metadata`
//! reads checkpoint parquet with an add/remove-*projected* schema
//! (`CHECKPOINT_READ_SCHEMA`) and applies its own row-group-skipping
//! `meta_predicate`, so the reconciled read never pulls the full action set or
//! irrelevant row groups — no extra column/row-group pushdown is available to add
//! here. DataFusion's column projection is applied to the emitted scan-file rows.
//! A caller predicate over *table data* columns (to prune the file list by data
//! stats via `scan_builder().with_predicate(..)`) is a Phase 2 feature: the
//! output here is scan-file metadata (`path`/`size`/`stats`/…), not table data,
//! so there is no data predicate to translate at the wiring stage.

use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use std::task::{Context, Poll};

use async_trait::async_trait;
use datafusion::arrow::array::BooleanArray;
use datafusion::arrow::compute::filter_record_batch;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
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
use delta_kernel::arrow::datatypes::SchemaRef as ArrowSchemaRef;
use delta_kernel::engine::arrow_conversion::TryIntoArrow as _;
use delta_kernel::engine::arrow_data::ArrowEngineData;
use delta_kernel::scan::{Scan, ScanMetadata, scan_row_schema};
use delta_kernel::snapshot::Snapshot;
use delta_kernel::{DeltaResult, Engine};
use futures::Stream;
use url::Url;

static SCAN_ROW_SCHEMA: LazyLock<ArrowSchemaRef> =
    LazyLock::new(|| Arc::new((scan_row_schema().as_ref()).try_into_arrow().unwrap()));

/// A DataFusion [`TableProvider`] over a Delta table's *reconciled* log: the
/// surviving scan-file rows (per [`scan_row_schema`]) after kernel log replay.
///
/// Carries the delta-kernel [`Engine`] used to read the log, so it is fully
/// self-contained and depends on no session extension.
pub struct ReconciledLogProvider {
    table: Url,
    engine: Arc<dyn Engine>,
}

impl std::fmt::Debug for ReconciledLogProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconciledLogProvider")
            .field("table", &self.table)
            .finish_non_exhaustive()
    }
}

impl ReconciledLogProvider {
    /// Build a provider for the Delta table rooted at `table`, reading its log
    /// with `engine`.
    pub fn new(mut table: Url, engine: Arc<dyn Engine>) -> Self {
        if !table.path().ends_with('/') {
            table.set_path(&format!("{}/", table.path()));
        }
        Self { table, engine }
    }

    /// The Arrow schema of the reconciled scan-file rows.
    pub fn scan_row_schema() -> ArrowSchemaRef {
        Arc::clone(&SCAN_ROW_SCHEMA)
    }
}

#[async_trait]
impl TableProvider for ReconciledLogProvider {
    fn schema(&self) -> ArrowSchemaRef {
        Self::scan_row_schema()
    }

    fn table_type(&self) -> TableType {
        TableType::Base
    }

    fn supports_filters_pushdown(
        &self,
        filters: &[&Expr],
    ) -> Result<Vec<TableProviderFilterPushDown>> {
        Ok(vec![
            TableProviderFilterPushDown::Unsupported;
            filters.len()
        ])
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        let engine = self.engine.clone();
        let table_root = self.table.clone();

        let snapshot = tokio::task::spawn_blocking(move || {
            Snapshot::builder_for(table_root.as_str())
                .build(engine.as_ref())
                .map_err(|e| DataFusionError::Execution(e.to_string()))
        })
        .await
        .map_err(|e| DataFusionError::Execution(e.to_string()))??;

        let projected_arrow = projection
            .map(|p| {
                Self::scan_row_schema()
                    .project(p)
                    .map_err(|e| DataFusionError::Execution(e.to_string()))
                    .map(Arc::new)
            })
            .transpose()?
            .unwrap_or_else(Self::scan_row_schema);

        let scan = snapshot
            .scan_builder()
            .build()
            .map_err(|e| DataFusionError::Execution(e.to_string()))?;

        let exec = ReconciledLogExec::new(
            self.engine.clone(),
            scan.into(),
            PlanProperties::new(
                EquivalenceProperties::new(projected_arrow),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Incremental,
                Boundedness::Bounded,
            ),
            projection.map(|p| p.to_vec()),
        );
        Ok(Arc::new(exec))
    }
}

struct ReconciledLogExec {
    engine: Arc<dyn Engine>,
    scan: Arc<Scan>,
    properties: Arc<PlanProperties>,
    projection: Option<Vec<usize>>,
}

impl std::fmt::Debug for ReconciledLogExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReconciledLogExec")
            .field("projection", &self.projection)
            .finish_non_exhaustive()
    }
}

impl ReconciledLogExec {
    fn new(
        engine: Arc<dyn Engine>,
        scan: Arc<Scan>,
        properties: PlanProperties,
        projection: Option<Vec<usize>>,
    ) -> Self {
        Self {
            engine,
            scan,
            properties: Arc::new(properties),
            projection,
        }
    }
}

impl DisplayAs for ReconciledLogExec {
    fn fmt_as(&self, t: DisplayFormatType, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match t {
            DisplayFormatType::Default
            | DisplayFormatType::Verbose
            | DisplayFormatType::TreeRender => {
                write!(f, "ReconciledLogExec: ")
            }
        }
    }
}

impl ExecutionPlan for ReconciledLogExec {
    fn name(&self) -> &'static str {
        "ReconciledLogExec"
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
                "ReconciledLogExec only supports a single partition".into(),
            ));
        }

        let engine = self.engine.clone();
        let iter = self
            .scan
            .scan_metadata(engine.as_ref())
            .map_err(|e| DataFusionError::Execution(e.to_string()))?;
        let stream = ReconciledLogStream::new(
            self.schema().clone(),
            Box::new(iter),
            self.projection.clone(),
        );
        Ok(Box::pin(stream))
    }
}

struct ReconciledLogStream {
    schema: SchemaRef,
    input: Box<dyn Iterator<Item = DeltaResult<ScanMetadata>> + Send>,
    projection: Option<Vec<usize>>,
}

impl ReconciledLogStream {
    fn new(
        schema: SchemaRef,
        input: Box<dyn Iterator<Item = DeltaResult<ScanMetadata>> + Send>,
        projection: Option<Vec<usize>>,
    ) -> Self {
        Self {
            schema,
            input,
            projection,
        }
    }
}

impl Stream for ReconciledLogStream {
    type Item = Result<RecordBatch>;

    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        match this.input.next() {
            Some(Ok(metadata)) => {
                let (scan_files, selection_vector) = metadata.scan_files.into_parts();
                let data = match ArrowEngineData::try_from_engine_data(scan_files) {
                    Ok(data) => data,
                    Err(e) => {
                        tracing::error!("failed to convert scan metadata to record batch: {}", e);
                        return Poll::Ready(Some(Err(DataFusionError::Execution(e.to_string()))));
                    }
                };

                // Apply the kernel selection vector to the record batch.
                let predicate = BooleanArray::from(selection_vector);
                let mut record_batch = filter_record_batch(data.record_batch(), &predicate)
                    .map_err(|e| DataFusionError::ArrowError(Box::new(e), None));

                // Apply the projection.
                if let Some(projection) = &this.projection {
                    record_batch = record_batch.and_then(|batch| {
                        batch
                            .project(projection)
                            .map_err(|e| DataFusionError::ArrowError(Box::new(e), None))
                    });
                }

                Poll::Ready(Some(record_batch))
            }
            Some(Err(e)) => {
                tracing::error!("failed to get scan metadata: {}", e);
                Poll::Ready(Some(Err(DataFusionError::Execution(e.to_string()))))
            }
            None => Poll::Ready(None),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.input.size_hint()
    }
}

impl RecordBatchStream for ReconciledLogStream {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
}
