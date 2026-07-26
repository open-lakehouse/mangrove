//! The ConnectRPC `FilesService` implementation, backed by a [`FileStore`].
//!
//! The generated service trait is implemented for [`AppState`]. Handlers read
//! request fields off the zero-copy view, copy out owned data before crossing an
//! `.await`, delegate to the store, and return owned response messages.

mod files;

use std::sync::Arc;

use connectrpc::Router;

use crate::store::FileStore;
use unitycatalog_files_proto::connect::portal::files::v1::FilesServiceExt;

/// Shared, cheaply-cloneable handler state. Holds the backing [`FileStore`]; the
/// value implements the generated `FilesService` trait. The store is behind an
/// `Arc<dyn FileStore>` so the backend (in-memory vs. Unity Catalog volumes) can
/// be chosen at startup without changing this type.
#[derive(Clone)]
pub struct AppState {
    pub(crate) files: Arc<dyn FileStore>,
}

impl AppState {
    pub fn new(files: Arc<dyn FileStore>) -> Self {
        Self { files }
    }

    /// Register the `FilesService` onto a ConnectRPC router.
    pub fn register_all(self, router: Router) -> Router {
        FilesServiceExt::register(Arc::new(self), router)
    }

    /// Build a self-contained axum router serving the `FilesService` as a fallback
    /// service, ready to `nest_service` under a base path. A convenience for
    /// embedders (e.g. the UC server) so they need not depend on `connectrpc`
    /// directly.
    pub fn into_axum_router(self) -> axum::Router {
        let connect = self.register_all(Router::new());
        axum::Router::new().fallback_service(connect.into_axum_service())
    }
}
