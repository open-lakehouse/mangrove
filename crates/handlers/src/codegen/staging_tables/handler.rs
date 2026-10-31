// @generated — do not edit by hand.
//! Handler trait for [`StagingTableHandler`].
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
//! A staging table reserves an id and storage location for a Unity Catalog
//! managed table. After creating one, the client writes the initial Delta commit
//! at the returned staging_location and finalizes the table via CreateTable.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::staging_tables::v1::*;

#[async_trait]
pub trait StagingTableHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// Creates a new staging table, allocating an immutable table id and a storage
    /// location under the parent schema/catalog managed storage root. The caller
    /// must have the CREATE privilege on the parent schema.
    async fn create_staging_table(
        &self,
        request: CreateStagingTableRequest,
        context: Cx,
    ) -> Result<StagingTable>;
}
