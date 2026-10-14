//! A Delta-backed [`TableProviderBuilder`].
//!
//! Enabled by the `delta` feature. This builds a DataFusion [`TableProvider`]
//! for a Unity Catalog Delta table by reading its log through whatever object
//! store the resolver has already registered on the session runtime for the
//! table's storage location.
//!
//! For **catalog-managed** (coordinated-commit) tables the filesystem `_delta_log/`
//! is not authoritative: the catalog tracks the latest ratified version and the
//! newest commits may be unbackfilled. So the builder calls the `/delta/v1`
//! `loadTable` endpoint to fetch the commit tail + latest version and builds a
//! catalog-managed kernel snapshot from them (see [`super::kernel`]). External
//! Delta tables keep the plain filesystem snapshot path.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use datafusion::catalog::TableProvider;
use datafusion::common::DataFusionError;
use datafusion::execution::context::SessionContext;
use deltalake_core::delta_datafusion::DeltaScanNext;
use deltalake_core::delta_datafusion::engine::DataFusionEngine;
use deltalake_core::kernel::Snapshot;
use deltalake_core::logstore::{LogStoreRef, StorageConfig, default_logstore};
use tracing::debug;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::tables::v1::Table;
use url::Url;

use super::builder::{TableProviderBuilder, TableProviderError};
use super::kernel::{ManagedReadState, build_catalog_managed_snapshot, resolve_managed_read_state};

/// Builds Delta [`TableProvider`]s for Unity Catalog tables.
///
/// Holds a [`SessionContext`] so it can read through the runtime's object store
/// registry — by the time [`build_delta`](TableProviderBuilder::build_delta) is
/// called, the resolver has already registered a credential-vended store for the
/// table's storage location, so the log scan succeeds against that store. Also
/// holds a [`UnityCatalogClient`] to call the `/delta/v1` `loadTable` endpoint for
/// catalog-managed tables.
#[derive(Clone)]
pub struct DeltaTableProviderBuilder {
    ctx: SessionContext,
    client: UnityCatalogClient,
    /// Set once a `/delta/v1` loadTable call shows the endpoint is unavailable on
    /// this deployment (unsupported table format, not implemented, or route
    /// missing — see [`unitycatalog_client::Error::should_fall_back_to_legacy`]).
    /// Subsequent Delta tables then skip the loadTable round-trip and go straight
    /// to the filesystem snapshot. A deferred `getConfig` probe could set this up
    /// front instead; the reactive path is the proven mechanism (review A6).
    delta_v1_unsupported: Arc<AtomicBool>,
}

impl DeltaTableProviderBuilder {
    /// Create a builder that resolves Delta tables through `ctx`'s runtime and
    /// loads catalog-managed metadata through `client`.
    pub fn new(ctx: SessionContext, client: UnityCatalogClient) -> Self {
        Self {
            ctx,
            client,
            delta_v1_unsupported: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Build the log store for `location` from the resolver-registered object store.
    ///
    /// Built directly rather than via `logstore_with` (which dispatches on the URL
    /// scheme to a registered logstore factory) because we depend only on
    /// `deltalake-core`, so no cloud-scheme factories (`s3`/`gs`/`az`) are
    /// registered. The prefixed store roots paths at the table location; the root
    /// store stays bucket-rooted.
    fn log_store_for(&self, location: &Url) -> Result<LogStoreRef, TableProviderError> {
        let root_store = self
            .ctx
            .runtime_env()
            .object_store_registry
            .get_store(location)
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        let config = StorageConfig::default();
        let prefixed_store = config
            .decorate_store(root_store.clone(), location)
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        Ok(default_logstore(
            Arc::from(prefixed_store),
            root_store,
            location,
            &config,
        ))
    }

    /// Build the plain filesystem snapshot for a non-catalog-managed table (external
    /// tables, and the A6 fallback path when `/delta/v1` is unavailable). The
    /// filesystem `_delta_log/` is authoritative here, so no catalog version is set.
    async fn filesystem_snapshot(&self, location: &Url) -> Result<Snapshot, TableProviderError> {
        let engine = DataFusionEngine::new_from_context(self.ctx.task_ctx());
        Snapshot::try_new_with_engine(engine, location.clone(), None)
            .await
            .map_err(|e| DataFusionError::External(Box::new(e)))
    }

    /// Assemble a provider from a built snapshot and the table's log store.
    async fn provider_from_snapshot(
        &self,
        snapshot: Snapshot,
        log_store: LogStoreRef,
    ) -> Result<Arc<dyn TableProvider>, TableProviderError> {
        DeltaScanNext::builder()
            .with_snapshot(Arc::new(snapshot))
            .with_log_store(log_store)
            .await
    }
}

impl std::fmt::Debug for DeltaTableProviderBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeltaTableProviderBuilder")
            .finish_non_exhaustive()
    }
}

#[async_trait::async_trait]
impl TableProviderBuilder for DeltaTableProviderBuilder {
    async fn build_delta(
        &self,
        location: &Url,
        table: &Table,
    ) -> Result<Arc<dyn TableProvider>, TableProviderError> {
        let log_store = self.log_store_for(location)?;

        // A prior table already showed this deployment doesn't serve `/delta/v1`:
        // skip the loadTable round-trip and read the filesystem log directly.
        if self.delta_v1_unsupported.load(Ordering::Relaxed) {
            let snapshot = self.filesystem_snapshot(location).await?;
            return self.provider_from_snapshot(snapshot, log_store).await;
        }

        // Ask the catalog whether this is a managed (coordinated-commit) table and,
        // if so, for its ratified commit tail. The `/delta/v1` loadTable response
        // carries the table type, the unbackfilled commits, and the latest ratified
        // version a reader needs to materialize the catalog's snapshot.
        let loaded = match self
            .client
            .delta_v1()
            .load_table(&table.catalog_name, &table.schema_name, &table.name)
            .await
        {
            Ok(loaded) => loaded,
            // A6: the `/delta/v1` endpoint is unavailable on this deployment (older
            // OSS / production Databricks). The legacy `tables` API already gave us
            // the storage location, so fall back to the filesystem snapshot rather
            // than failing every Delta read. A genuine NoSuchTable / auth / other
            // error propagates (we must not mask a missing table).
            Err(e) if e.should_fall_back_to_legacy() => {
                debug!(
                    table = %table.full_name, error = %e,
                    "/delta/v1 loadTable unavailable; falling back to filesystem snapshot"
                );
                self.delta_v1_unsupported.store(true, Ordering::Relaxed);
                let snapshot = self.filesystem_snapshot(location).await?;
                return self.provider_from_snapshot(snapshot, log_store).await;
            }
            Err(e) => return Err(DataFusionError::External(Box::new(e))),
        };

        let snapshot = match resolve_managed_read_state(&loaded)? {
            // The catalog is the source of truth: build from the ratified commit
            // tail + latest version rather than scanning `_delta_log/`.
            ManagedReadState::Managed { commits, latest } => {
                let engine = DataFusionEngine::new_from_context(self.ctx.task_ctx());
                build_catalog_managed_snapshot(
                    engine.as_ref(),
                    location,
                    &commits,
                    latest as i64,
                    None,
                )?
            }
            // External / not-catalog-managed: the filesystem `_delta_log/` is
            // authoritative and the read must not set a catalog version.
            ManagedReadState::NotManaged => self.filesystem_snapshot(location).await?,
        };

        self.provider_from_snapshot(snapshot, log_store).await
    }

    #[cfg(feature = "metric-view")]
    async fn build_metric_view(
        &self,
        view: &crate::metric_view::MetricView,
        source: Arc<dyn TableProvider>,
        source_name: &str,
    ) -> Result<Arc<dyn TableProvider>, TableProviderError> {
        use datafusion::datasource::provider_as_source;
        use datafusion::logical_expr::LogicalPlanBuilder;

        use crate::metric_view::MetricViewTableProvider;

        // Build a scan over the resolved source provider as the view's input
        // plan. The metric view's dimension/measure expressions are written
        // against this relation's columns.
        let source_plan =
            LogicalPlanBuilder::scan(source_name, provider_as_source(source), None)?.build()?;

        let provider = MetricViewTableProvider::try_new(&self.ctx.state(), view, source_plan)?;
        Ok(Arc::new(provider))
    }
}
