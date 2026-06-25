use std::sync::{Arc, LazyLock};

use bytes::Bytes;
use datafusion::arrow::array::{AsArray, RecordBatch};
use datafusion::arrow::json::LineDelimitedWriter;
use datafusion::catalog::{CatalogProvider, MemoryCatalogProvider, MemorySchemaProvider};
use datafusion::common::TableReference as DfTableReference;
use datafusion::functions::core::expr_ext::FieldAccessor;
use datafusion::logical_expr::ColumnarValue;
use datafusion::physical_plan::PhysicalExpr;
use datafusion::prelude::SessionContext;
use datafusion::prelude::{Expr, col, lit, named_struct};
use delta_kernel::{Snapshot, Version};
use itertools::Itertools;
use unitycatalog_common::models::tables::v1::DataSourceFormat;

use super::kernel::{DeltaLogReplayProvider, ObjectStoreFactory, build_engine};
use super::location::StorageLocationUrl;
use super::sharing::SharingTableReference;
use crate::api::tables::TableManager;
use crate::{Error, Result};

const UC_RS_SYSTEM_CATALOG_NAME: &str = "uc_rs_system";
const UC_RS_LOG_REPLAY_SCHEMA_NAME: &str = "uc_rs_log_replay";

static PQ_FILE_EXTRACT: LazyLock<Expr> = LazyLock::new(|| {
    named_struct(vec![
        lit("file"),
        named_struct(vec![
            lit("path"),
            col("path"),
            lit("partitionValues"),
            col("\"fileConstantValues\"").field("partitionValues"),
            lit("size"),
            col("size"),
        ]),
    ])
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableReference {
    Sharing(SharingTableReference),
    Datafusion(DfTableReference),
}

struct Extractors {
    sharing_pq_files: Arc<dyn PhysicalExpr>,
}

impl Extractors {
    pub fn new(ctx: &SessionContext) -> Result<Self> {
        let df_schema = DeltaLogReplayProvider::scan_row_schema()
            .try_into()
            .map_err(|_| Error::Generic("failed to convert schema".to_string()))?;
        let sharing_pq_files = ctx
            .create_physical_expr(PQ_FILE_EXTRACT.clone(), &df_schema)
            .map_err(|e| Error::Generic(e.to_string()))?;
        Ok(Self { sharing_pq_files })
    }
}

pub struct KernelSession {
    ctx: SessionContext,
    extractors: Extractors,
    factory: Arc<dyn ObjectStoreFactory>,
}

impl KernelSession {
    pub fn new(object_store_factory: Arc<dyn ObjectStoreFactory>) -> Result<Self> {
        let ctx = SessionContext::new();
        let catalog = Arc::new(MemoryCatalogProvider::new());
        catalog
            .register_schema(
                UC_RS_LOG_REPLAY_SCHEMA_NAME,
                Arc::new(MemorySchemaProvider::new()),
            )
            .map_err(|e| Error::Generic(e.to_string()))?;
        ctx.register_catalog(UC_RS_SYSTEM_CATALOG_NAME, catalog);

        Ok(Self {
            extractors: Extractors::new(&ctx)?,
            ctx,
            factory: object_store_factory,
        })
    }

    pub fn ctx(&self) -> &SessionContext {
        &self.ctx
    }

    pub fn system_catalog(&self) -> Arc<dyn CatalogProvider> {
        self.ctx
            .catalog(UC_RS_SYSTEM_CATALOG_NAME)
            .expect("system catalog should be registered in kernel session")
    }

    pub(super) async fn extract_sharing_query_response(
        &self,
        table_ref: &SharingTableReference,
        location: &StorageLocationUrl,
    ) -> Result<Bytes> {
        let log_replay_table_name = table_ref.system_table_name();
        let inner_ref = DfTableReference::full(
            UC_RS_SYSTEM_CATALOG_NAME,
            UC_RS_LOG_REPLAY_SCHEMA_NAME,
            log_replay_table_name,
        );
        if !self
            .ctx
            .table_exist(inner_ref.clone())
            .map_err(|e| Error::Generic(e.to_string()))?
        {
            let engine = build_engine(self.factory.as_ref(), location.location())
                .await
                .map_err(|e| Error::Generic(e.to_string()))?;
            self.ctx
                .register_table(
                    inner_ref.clone(),
                    Arc::new(
                        DeltaLogReplayProvider::new(location.location().clone(), engine)
                            .map_err(|e| Error::Generic(e.to_string()))?,
                    ),
                )
                .map_err(|e| Error::Generic(e.to_string()))?;
        }
        let table = self
            .ctx
            .table(inner_ref)
            .await
            .map_err(|e| Error::Generic(e.to_string()))?
            .collect()
            .await
            .map_err(|e| Error::Generic(e.to_string()))?;
        let results: Vec<_> = table
            .iter()
            .map(|batch| {
                let res = match self
                    .extractors
                    .sharing_pq_files
                    .evaluate(batch)
                    .map_err(|e| Error::Generic(e.to_string()))?
                {
                    ColumnarValue::Array(arr) => arr,
                    ColumnarValue::Scalar(scalar) => scalar
                        .to_array_of_size(batch.num_rows())
                        .map_err(|e| Error::Generic(e.to_string()))?,
                };
                Ok::<_, Error>(RecordBatch::from(res.as_struct()))
            })
            .try_collect()?;
        encode_nd_json(&results) // spellchecker:disable-line
    }
}

#[async_trait::async_trait]
impl TableManager for KernelSession {
    async fn read_snapshot(
        &self,
        location: &StorageLocationUrl,
        format: &DataSourceFormat,
        version: Option<Version>,
    ) -> Result<Arc<Snapshot>> {
        if *format != DataSourceFormat::Delta {
            return Err(Error::InvalidArgument(format!(
                "unsupported data source format in kernel session: {format:?}"
            )));
        }
        let engine = build_engine(self.factory.as_ref(), location.location())
            .await
            .map_err(|e| Error::Generic(e.to_string()))?;
        let table_root = location.location().clone();
        let snapshot = tokio::task::spawn_blocking(move || {
            let mut builder = Snapshot::builder_for(table_root.as_str());
            if let Some(version) = version {
                builder = builder.at_version(version);
            }
            builder.build(engine.as_ref())
        })
        .await
        .map_err(|e| Error::Generic(e.to_string()))?
        .map_err(|e| Error::Generic(e.to_string()))?;
        Ok(snapshot)
    }
}

// spellchecker:ignore-next-line
pub fn encode_nd_json(data: &[RecordBatch]) -> Result<Bytes> {
    let mut writer = LineDelimitedWriter::new(Vec::new());
    for batch in data {
        writer
            .write(batch)
            .map_err(|e| Error::Generic(e.to_string()))?;
    }
    Ok(Bytes::from(writer.into_inner()))
}
