use std::pin::Pin;
use std::sync::{Arc, LazyLock};
use std::task::{Context, Poll};

use async_trait::async_trait;
use datafusion::arrow::array::{AsArray, BooleanArray};
use datafusion::arrow::compute::filter_record_batch;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::catalog::{Session, TableProvider};
use datafusion::common::DataFusionError;
use datafusion::common::error::Result;
use datafusion::execution::{RecordBatchStream, SendableRecordBatchStream, TaskContext};
use datafusion::logical_expr::utils::conjunction;
use datafusion::logical_expr::{ColumnarValue, Expr, TableProviderFilterPushDown, TableType};
use datafusion::physical_expr::EquivalenceProperties;
use datafusion::physical_plan::execution_plan::{Boundedness, EmissionType};
use datafusion::physical_plan::{
    DisplayAs, DisplayFormatType, ExecutionPlan, Partitioning, PhysicalExpr, PlanProperties,
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

/// A DataFusion [`TableProvider`] that replays a Delta table's log and exposes
/// the resulting scan-file rows (per [`scan_row_schema`]).
///
/// The provider carries the delta_kernel [`Engine`] used to read the log, so it
/// is fully self-contained and does not depend on any session extension.
pub(crate) struct DeltaLogReplayProvider {
    table: Url,
    engine: Arc<dyn Engine>,
}

impl std::fmt::Debug for DeltaLogReplayProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeltaLogReplayProvider")
            .field("table", &self.table)
            .finish_non_exhaustive()
    }
}

impl DeltaLogReplayProvider {
    pub(crate) fn new(mut table: Url, engine: Arc<dyn Engine>) -> Result<Self> {
        if !table.path().ends_with('/') {
            table.set_path(&format!("{}/", table.path()));
        }
        Ok(Self { table, engine })
    }

    pub(crate) fn scan_row_schema() -> ArrowSchemaRef {
        Arc::clone(&SCAN_ROW_SCHEMA)
    }
}

#[async_trait]
impl TableProvider for DeltaLogReplayProvider {
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
        state: &dyn Session,
        projection: Option<&Vec<usize>>,
        filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        // TODO: handle predicate - this needs to be applied in the stream where we produce the
        // record batches
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

        let predicate = if let Some(pred) = conjunction(filters.iter().cloned()) {
            let df_schema = projected_arrow.clone().try_into()?;
            state.create_physical_expr(pred, &df_schema).ok()
        } else {
            None
        };

        let exec = DeltaLogReplayExec::new(
            self.engine.clone(),
            scan.into(),
            PlanProperties::new(
                EquivalenceProperties::new(projected_arrow),
                Partitioning::UnknownPartitioning(1),
                EmissionType::Incremental,
                Boundedness::Bounded,
            ),
            projection.map(|p| p.to_vec()),
            predicate,
        );
        Ok(Arc::new(exec))
    }
}

struct DeltaLogReplayExec {
    engine: Arc<dyn Engine>,
    scan: Arc<Scan>,
    properties: Arc<PlanProperties>,
    projection: Option<Vec<usize>>,
    predicate: Option<Arc<dyn PhysicalExpr>>,
}

impl std::fmt::Debug for DeltaLogReplayExec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeltaLogReplayExec")
            .field("projection", &self.projection)
            .finish_non_exhaustive()
    }
}

impl DeltaLogReplayExec {
    pub fn new(
        engine: Arc<dyn Engine>,
        scan: Arc<Scan>,
        properties: PlanProperties,
        projection: Option<Vec<usize>>,
        predicate: Option<Arc<dyn PhysicalExpr>>,
    ) -> Self {
        Self {
            engine,
            scan,
            properties: Arc::new(properties),
            projection,
            predicate,
        }
    }
}

impl DisplayAs for DeltaLogReplayExec {
    fn fmt_as(&self, t: DisplayFormatType, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // TODO: actually implement formatting according to the type
        match t {
            DisplayFormatType::Default
            | DisplayFormatType::Verbose
            | DisplayFormatType::TreeRender => {
                write!(f, "DeltaLogReplayExec: ")
            }
        }
    }
}

impl ExecutionPlan for DeltaLogReplayExec {
    fn name(&self) -> &'static str {
        "DeltaLogReplayExec"
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
                "DeltaLogReplayExec only supports a single partition".into(),
            ));
        }

        let engine = self.engine.clone();
        // TODO: where should we do the work, also how does the work actually get executed?
        // Since internally we are using channels, we may just be blocking when we are polling ...
        let iter = self
            .scan
            .scan_metadata(engine.as_ref())
            .map_err(|e| DataFusionError::Execution(e.to_string()))?;
        let stream = DeltaLogReplayStream::new(
            self.schema().clone(),
            Box::new(iter),
            self.projection.clone(),
            self.predicate.clone(),
        );
        Ok(Box::pin(stream))
    }
}

// TODO: handle limits. to do this we likely need to also use ReceiverStreamBuilder
// since otherwise the read futures might not be dropped and we do the work anyways.
struct DeltaLogReplayStream {
    schema: SchemaRef,
    input: Box<dyn Iterator<Item = DeltaResult<ScanMetadata>> + Send>,
    projection: Option<Vec<usize>>,
    predicate: Option<Arc<dyn PhysicalExpr>>,
}

impl DeltaLogReplayStream {
    pub fn new(
        schema: SchemaRef,
        input: Box<dyn Iterator<Item = DeltaResult<ScanMetadata>> + Send>,
        projection: Option<Vec<usize>>,
        predicate: Option<Arc<dyn PhysicalExpr>>,
    ) -> Self {
        Self {
            schema,
            input,
            projection,
            predicate,
        }
    }
}

impl Stream for DeltaLogReplayStream {
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

                // Apply the selection vector to the record batch
                let predicate = BooleanArray::from(selection_vector);
                let mut record_batch = filter_record_batch(data.record_batch(), &predicate)
                    .map_err(|e| DataFusionError::ArrowError(Box::new(e), None));

                // Apply the predicate to the record batch
                if let (Some(predicate), Ok(batch)) = (&this.predicate, &record_batch) {
                    match predicate.evaluate(batch) {
                        Ok(result) => match result {
                            ColumnarValue::Array(array) => {
                                let bool_array = array.as_boolean();
                                record_batch = filter_record_batch(data.record_batch(), bool_array)
                                    .map_err(|e| DataFusionError::ArrowError(Box::new(e), None));
                            }
                            ColumnarValue::Scalar(_scalar) => {
                                todo!("handle scalar value");
                            }
                        },
                        Err(e) => {
                            tracing::error!("failed to evaluate predicate: {}", e);
                            return Poll::Ready(Some(Err(DataFusionError::Execution(
                                e.to_string(),
                            ))));
                        }
                    }
                }

                // Apply the projection to the record batch
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

impl RecordBatchStream for DeltaLogReplayStream {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
}
