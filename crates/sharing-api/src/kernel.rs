//! Minimal delta_kernel integration for the sharing query path.
//!
//! This module provides the small slice of `deltalake-datafusion` the sharing
//! query path relies on: an [`ObjectStoreFactory`] abstraction (implemented by a
//! server to resolve credentialed object stores per storage location) and a
//! kernel [`Engine`] builder. The Delta-log `query_table` response is served by
//! the shared [`ReconciledLogProvider`](unitycatalog_datafusion::log_explorer::ReconciledLogProvider)
//! from the `olai-uc-datafusion` crate.
//!
//! Rather than registering a custom DataFusion-backed kernel engine as a session
//! extension, we construct delta_kernel's built-in [`DefaultEngine`] directly from
//! the object store resolved for a given table root and hand it to the provider.

use std::sync::Arc;

use delta_kernel::Engine;
use delta_kernel_default_engine::DefaultEngine;
use object_store::DynObjectStore;
use url::Url;

use crate::error::{Error, Result};

/// Resolves an [`object_store`] for a given storage location.
///
/// A server implements this to map a storage URL to a credentialed object store.
/// It returns the crate's [`Error`] so a server implementation names no
/// DataFusion type — the DataFusion coupling stays entirely inside this crate.
#[async_trait::async_trait]
pub trait ObjectStoreFactory: Send + Sync + 'static {
    async fn create_object_store(&self, url: &Url) -> Result<Arc<DynObjectStore>>;
}

/// Build a delta_kernel [`Engine`] for the given table root.
///
/// Resolves the object store for `table_root` via `factory` and wraps it in
/// delta_kernel's [`DefaultEngine`], which manages its own background executor.
pub(crate) async fn build_engine(
    factory: &dyn ObjectStoreFactory,
    table_root: &Url,
) -> Result<Arc<dyn Engine>> {
    let store = factory
        .create_object_store(table_root)
        .await
        .map_err(|e| Error::generic(e.to_string()))?;
    Ok(Arc::new(DefaultEngine::builder(store).build()))
}
