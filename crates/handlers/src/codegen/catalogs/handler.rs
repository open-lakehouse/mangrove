// @generated — do not edit by hand.
//! Handler trait for [`CatalogHandler`].
//!
//! Implement this trait to provide a custom backend for this service, then mount the
//! generated handler functions (in the sibling `server` module) onto an `axum::Router`
//! with your implementation as state.
//!
//! # Composability
//!
//! A single struct can implement multiple handler traits to serve multiple
//! services. Use [`axum::Router::merge`] to compose per-service routers together.
//!
//! Manage catalogs and schemas in the service.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::catalogs::v1::*;
#[async_trait]
pub trait CatalogHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// List catalogs
    ///
    /// Gets an array of catalogs in the metastore. If the caller is the metastore admin,
    /// all catalogs will be retrieved. Otherwise, only catalogs owned by the caller
    /// (or for which the caller has the USE_CATALOG privilege) will be retrieved.
    /// There is no guarantee of a specific ordering of the elements in the array.
    async fn list_catalogs(
        &self,
        request: ListCatalogsRequest,
        context: Cx,
    ) -> Result<ListCatalogsResponse>;
    /// Create a new catalog
    ///
    /// Creates a new catalog instance in the parent metastore if the caller
    /// is a metastore admin or has the CREATE_CATALOG privilege.
    async fn create_catalog(&self, request: CreateCatalogRequest, context: Cx) -> Result<Catalog>;
    /// Get a catalog
    ///
    /// Gets the specified catalog in a metastore. The caller must be a metastore admin,
    /// the owner of the catalog, or a user that has the USE_CATALOG privilege set for their account.
    async fn get_catalog(&self, request: GetCatalogRequest, context: Cx) -> Result<Catalog>;
    /// Update a catalog
    ///
    /// Updates the catalog that matches the supplied name. The caller must be either
    /// the owner of the catalog, or a metastore admin (when changing the owner field of the catalog).
    async fn update_catalog(&self, request: UpdateCatalogRequest, context: Cx) -> Result<Catalog>;
    /// Delete a catalog
    ///
    /// Deletes the catalog that matches the supplied name. The caller must
    /// be a metastore admin or the owner of the catalog.
    async fn delete_catalog(&self, request: DeleteCatalogRequest, context: Cx) -> Result<()>;
}
