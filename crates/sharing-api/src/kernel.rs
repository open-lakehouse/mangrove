//! The `delta_kernel` engine the Delta table format reads with.

use std::sync::Arc;

use delta_kernel::Engine;
use delta_kernel_default_engine::DefaultEngine;
use object_store::DynObjectStore;

/// Build a kernel [`Engine`] over `store`.
///
/// Wraps delta_kernel's [`DefaultEngine`], which manages its own background
/// executor for parquet / JSON reads. `store` must address objects by their path
/// from the bucket / container root (see
/// [`TableStorage::store`](crate::access::TableStorage::store)).
pub(crate) fn build_engine(store: Arc<DynObjectStore>) -> Arc<dyn Engine> {
    Arc::new(DefaultEngine::builder(store).build())
}
