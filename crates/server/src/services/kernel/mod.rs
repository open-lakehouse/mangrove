//! Minimal delta_kernel integration for the Unity Catalog server.
//!
//! This module provides the small slice of `deltalake-datafusion` that the server
//! actually relies on: an [`ObjectStoreFactory`] abstraction (implemented by the
//! server to resolve credentialed object stores per storage location) and a
//! kernel [`Engine`] builder. The Delta-log `query_table` response is served by
//! the shared [`ReconciledLogProvider`](unitycatalog_datafusion::log_explorer::ReconciledLogProvider)
//! from the `olai-uc-datafusion` crate.
//!
//! Rather than registering a custom DataFusion-backed kernel engine as a session
//! extension, we construct delta_kernel's built-in [`DefaultEngine`] directly from
//! the object store resolved for a given table root and hand it to the provider.

use std::sync::Arc;

use datafusion::common::Result as DFResult;
use delta_kernel::Engine;
use delta_kernel_default_engine::DefaultEngine;
use object_store::DynObjectStore;
use url::Url;

/// Resolves an [`object_store`] for a given storage location.
///
/// The server implements this to map a storage URL to a credentialed object
/// store (see `services::object_store`).
#[async_trait::async_trait]
pub trait ObjectStoreFactory: Send + Sync + 'static {
    async fn create_object_store(&self, url: &Url) -> DFResult<Arc<DynObjectStore>>;
}

/// Build a delta_kernel [`Engine`] for the given table root.
///
/// Resolves the object store for `table_root` via `factory` and wraps it in
/// delta_kernel's [`DefaultEngine`], which manages its own background executor.
pub(crate) async fn build_engine(
    factory: &dyn ObjectStoreFactory,
    table_root: &Url,
) -> DFResult<Arc<dyn Engine>> {
    let store = factory.create_object_store(table_root).await?;
    Ok(Arc::new(DefaultEngine::builder(store).build()))
}
