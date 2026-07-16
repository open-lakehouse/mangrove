//! The DataFusion + delta-kernel NDJSON query path.
//!
//! [`KernelSession`] holds a DataFusion [`SessionContext`] and serves the Open
//! Sharing `query_table` / `metadata` / `version` responses by registering the
//! shared [`ReconciledLogProvider`] over a shared table's storage location,
//! collecting the log-replay batches, projecting them into the Delta Sharing
//! `file` action shape, and encoding NDJSON.
//!
//! This is the only DataFusion-coupled part of the sharing surface, deliberately
//! isolated in this crate so downstream servers stay free of the DataFusion +
//! git-pinned `delta_kernel` dependencies.

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

use unitycatalog_datafusion::log_explorer::ReconciledLogProvider;

use crate::backend::{ResolvedLocation, SharingTableReference};
use crate::error::{Error, Result};
use crate::kernel::{ObjectStoreFactory, build_engine};

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

struct Extractors {
    sharing_pq_files: Arc<dyn PhysicalExpr>,
}

impl Extractors {
    fn new(ctx: &SessionContext) -> Result<Self> {
        let df_schema = ReconciledLogProvider::scan_row_schema()
            .try_into()
            .map_err(|_| Error::Generic("failed to convert schema".to_string()))?;
        let sharing_pq_files = ctx
            .create_physical_expr(PQ_FILE_EXTRACT.clone(), &df_schema)
            .map_err(|e| Error::Generic(e.to_string()))?;
        Ok(Self { sharing_pq_files })
    }
}

/// A DataFusion session that serves the sharing query path over kernel log replay.
pub struct KernelSession {
    ctx: SessionContext,
    extractors: Extractors,
    factory: Arc<dyn ObjectStoreFactory>,
}

impl KernelSession {
    /// Build a session backed by the given object-store factory.
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

    #[allow(dead_code)] // accessor retained for the kernel session API
    pub fn ctx(&self) -> &SessionContext {
        &self.ctx
    }

    #[allow(dead_code)] // accessor retained for the kernel session API
    pub fn system_catalog(&self) -> Arc<dyn CatalogProvider> {
        self.ctx
            .catalog(UC_RS_SYSTEM_CATALOG_NAME)
            .expect("system catalog should be registered in kernel session")
    }

    /// Read a Delta snapshot for a shared table's location, at an optional version.
    pub async fn read_snapshot(
        &self,
        location: &ResolvedLocation,
        version: Option<Version>,
    ) -> Result<Arc<Snapshot>> {
        let engine = build_engine(self.factory.as_ref(), &location.url).await?;
        let table_root = location.url.clone();
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

    /// Serve the Open Sharing `query_table` response for a shared table as NDJSON
    /// `file` actions.
    pub async fn extract_sharing_query_response(
        &self,
        table_ref: &SharingTableReference,
        location: &ResolvedLocation,
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
            let engine = build_engine(self.factory.as_ref(), &location.url).await?;
            self.ctx
                .register_table(
                    inner_ref.clone(),
                    Arc::new(ReconciledLogProvider::new(location.url.clone(), engine)),
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

/// Encode a slice of record batches as newline-delimited JSON.
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
