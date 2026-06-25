use std::sync::Arc;

use dashmap::DashMap;
use datafusion::catalog::{
    AsyncCatalogProvider, AsyncCatalogProviderList, AsyncSchemaProvider, TableProvider,
};
use datafusion::common::{DataFusionError, plan_datafusion_err};
use datafusion::error::Result;
use datafusion::execution::runtime_env::RuntimeEnv;
use object_store::path::Path;
use tracing::{debug, instrument};
use unitycatalog_common::models::tables::v1::DataSourceFormat;
use unitycatalog_object_store::{TableOperation, UnityObjectStoreFactory};
use url::Url;

use super::builder::TableProviderBuilder;
use crate::storage::RoutingObjectStore;

/// Shared state used while resolving Unity Catalog references for a query.
///
/// Holds the UC-backed object store factory (metadata + credential vending),
/// the session runtime on which per-table object stores are registered, the
/// per-bucket [`RoutingObjectStore`]s, and the embedder-supplied provider
/// builder. Cloning is cheap (everything is `Arc`-shared).
#[derive(Clone)]
struct UnityContext {
    factory: Arc<UnityObjectStoreFactory>,
    runtime: Arc<RuntimeEnv>,
    builder: Arc<dyn TableProviderBuilder>,
    /// `scheme://host` -> routing store registered on `runtime` for that host.
    routers: Arc<DashMap<String, RoutingObjectStore>>,
}

impl std::fmt::Debug for UnityContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnityContext").finish_non_exhaustive()
    }
}

impl UnityContext {
    /// Ensure a [`RoutingObjectStore`] is registered for the bucket of `url`,
    /// then route `url`'s prefix to `store` within it.
    ///
    /// The routing store sits behind DataFusion's coarse `scheme://host`
    /// registry key and dispatches to the correct per-table store at scan time.
    fn register_table_store(&self, url: &Url, store: Arc<dyn object_store::ObjectStore>) {
        let host_key = bucket_key(url);
        let router = self
            .routers
            .entry(host_key.clone())
            .or_insert_with(|| {
                let router = RoutingObjectStore::new();
                // Register once per host; the registry keys on scheme://host so
                // re-registering the same instance is harmless but unnecessary.
                if let Ok(bucket_url) = Url::parse(&format!("{host_key}/")) {
                    self.runtime
                        .register_object_store(&bucket_url, Arc::new(router.clone()));
                }
                router
            })
            .clone();
        router.register(Path::from_url_path(url.path()).unwrap_or_default(), store);
    }
}

/// `scheme://host` portion of a cloud URL, matching how DataFusion's
/// `ObjectStoreRegistry` keys stores.
fn bucket_key(url: &Url) -> String {
    format!(
        "{}://{}",
        url.scheme(),
        &url[url::Position::BeforeHost..url::Position::AfterPort]
    )
}

/// Catalog list backed by a live Unity Catalog instance.
///
/// Register with DataFusion via the async resolution flow: call
/// [`AsyncCatalogProviderList::resolve`] with the query's table references to
/// obtain a synchronous, query-scoped `CatalogProviderList`.
#[derive(Debug, Clone)]
pub struct UnityCatalogProviderList {
    ctx: UnityContext,
}

impl UnityCatalogProviderList {
    pub fn new(
        factory: Arc<UnityObjectStoreFactory>,
        runtime: Arc<RuntimeEnv>,
        builder: Arc<dyn TableProviderBuilder>,
    ) -> Self {
        Self {
            ctx: UnityContext {
                factory,
                runtime,
                builder,
                routers: Arc::new(DashMap::new()),
            },
        }
    }
}

#[async_trait::async_trait]
impl AsyncCatalogProviderList for UnityCatalogProviderList {
    #[instrument(skip(self), level = "debug")]
    async fn catalog(&self, name: &str) -> Result<Option<Arc<dyn AsyncCatalogProvider>>> {
        // Only fabricate a provider for catalogs Unity Catalog actually knows;
        // unknown names fall through to other registered catalogs.
        let exists = self
            .ctx
            .factory
            .unity_client()
            .catalog(name)
            .get()
            .await
            .is_ok();
        if !exists {
            return Ok(None);
        }
        Ok(Some(Arc::new(UnityCatalogProvider {
            ctx: self.ctx.clone(),
            catalog: name.to_string(),
        })))
    }
}

/// A single Unity Catalog catalog.
#[derive(Debug)]
pub struct UnityCatalogProvider {
    ctx: UnityContext,
    catalog: String,
}

#[async_trait::async_trait]
impl AsyncCatalogProvider for UnityCatalogProvider {
    #[instrument(skip(self), fields(catalog = %self.catalog), level = "debug")]
    async fn schema(&self, name: &str) -> Result<Option<Arc<dyn AsyncSchemaProvider>>> {
        let exists = self
            .ctx
            .factory
            .unity_client()
            .schema(&self.catalog, name)
            .get()
            .await
            .is_ok();
        if !exists {
            return Ok(None);
        }
        Ok(Some(Arc::new(UnityCatalogSchemaProvider {
            ctx: self.ctx.clone(),
            catalog: self.catalog.clone(),
            schema: name.to_string(),
        })))
    }
}

/// A single Unity Catalog schema. Resolves tables to DataFusion providers.
#[derive(Debug)]
pub struct UnityCatalogSchemaProvider {
    ctx: UnityContext,
    catalog: String,
    schema: String,
}

#[async_trait::async_trait]
impl AsyncSchemaProvider for UnityCatalogSchemaProvider {
    #[instrument(
        skip(self),
        fields(catalog = %self.catalog, schema = %self.schema, table = name),
        level = "info",
    )]
    async fn table(&self, name: &str) -> Result<Option<Arc<dyn TableProvider>>> {
        let full_name = format!("{}.{}.{}", self.catalog, self.schema, name);

        // 1. Fetch table metadata from Unity Catalog.
        let table = match self
            .ctx
            .factory
            .unity_client()
            .table(&self.catalog, &self.schema, name)
            .get()
            .await
        {
            Ok(table) => table,
            // A missing table is not an error here: another provider may serve
            // it, or planning will surface a clear "table not found" later.
            Err(e) => {
                debug!("table '{full_name}' not found in Unity Catalog: {e}");
                return Ok(None);
            }
        };

        // 2a. Metric views are resolved before the Delta-only check: they have
        //     no storage location of their own, only a definition referencing a
        //     source relation.
        #[cfg(feature = "metric-view")]
        if let Some(provider) = self.try_resolve_metric_view(&table).await? {
            return Ok(Some(provider));
        }

        // 2. Otherwise resolve as a (Delta) base table.
        let provider = self.build_base_table(&full_name, &table).await?;
        Ok(Some(provider))
    }
}

impl UnityCatalogSchemaProvider {
    /// Resolve a base (non-view) Unity Catalog table to a [`TableProvider`].
    ///
    /// Vends credentials, registers the per-table object store, and delegates
    /// provider construction to the host session's builder. Only Delta is
    /// supported for now.
    async fn build_base_table(
        &self,
        full_name: &str,
        table: &unitycatalog_common::models::tables::v1::Table,
    ) -> Result<Arc<dyn TableProvider>> {
        let format = table
            .data_source_format
            .as_known()
            .unwrap_or(DataSourceFormat::DATA_SOURCE_FORMAT_UNSPECIFIED);
        if format != DataSourceFormat::Delta {
            return Err(DataFusionError::NotImplemented(format!(
                "Unity Catalog table '{full_name}' has unsupported data source format {format:?}; \
                 only Delta is supported"
            )));
        }

        let location = table
            .storage_location
            .as_deref()
            .ok_or_else(|| plan_datafusion_err!("table '{full_name}' has no storage location"))?;
        let location = Url::parse(location).map_err(|e| {
            plan_datafusion_err!("invalid storage location '{location}' for '{full_name}': {e}")
        })?;

        // Vend credentials and build the per-table object store, then route it
        // under the bucket's routing store so the engine can read it.
        let uc_store = self
            .ctx
            .factory
            .for_table(full_name.to_string(), TableOperation::Read)
            .await
            .map_err(|e| {
                plan_datafusion_err!("failed to vend credentials for '{full_name}': {e}")
            })?;
        // Register the bucket-rooted store; the routing store forwards the full
        // request path unchanged, and the credential is scoped to `prefix()`.
        self.ctx
            .register_table_store(uc_store.url(), uc_store.root());

        // Delegate provider construction to the host session.
        self.ctx.builder.build_delta(&location, table).await
    }

    /// If `table` is a metric view, resolve it: parse the definition, resolve
    /// the source relation through the same Unity Catalog path, and build a
    /// metric-view provider. Returns `Ok(None)` for non-metric-view tables.
    #[cfg(feature = "metric-view")]
    async fn try_resolve_metric_view(
        &self,
        table: &unitycatalog_common::models::tables::v1::Table,
    ) -> Result<Option<Arc<dyn TableProvider>>> {
        let view = match crate::metric_view::detect::metric_view_of(table) {
            Ok(Some(view)) => view,
            Ok(None) => return Ok(None),
            Err(e) => {
                return Err(plan_datafusion_err!(
                    "metric view '{}': {e}",
                    table.full_name
                ));
            }
        };

        // Resolve the source relation (a three-part `catalog.schema.table` name).
        let source_name = view.source.trim();
        let parts: Vec<&str> = source_name.split('.').collect();
        let [catalog, schema, table_name] = parts.as_slice() else {
            return Err(DataFusionError::NotImplemented(format!(
                "metric-view source '{source_name}' must be a three-part \
                 catalog.schema.table name (inline SQL sources are not yet supported)"
            )));
        };

        let source_table = self
            .ctx
            .factory
            .unity_client()
            .table(*catalog, *schema, *table_name)
            .get()
            .await
            .map_err(|e| {
                plan_datafusion_err!("metric-view source '{source_name}' not found: {e}")
            })?;
        let source_provider = self.build_base_table(source_name, &source_table).await?;

        let provider = self
            .ctx
            .builder
            .build_metric_view(&view, source_provider, source_name)
            .await?;
        Ok(Some(provider))
    }
}
