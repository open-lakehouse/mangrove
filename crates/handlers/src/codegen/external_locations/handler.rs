// @generated — do not edit by hand.
//! Handler trait for [`ExternalLocationHandler`].
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
//! Service for managing external locations in Unity Catalog.
//! External locations define cloud storage paths accessible via storage credentials.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::external_locations::v1::*;
#[async_trait]
pub trait ExternalLocationHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// List external locations
    async fn list_external_locations(
        &self,
        request: ListExternalLocationsRequest,
        context: Cx,
    ) -> Result<ListExternalLocationsResponse>;
    /// Create a new external location
    async fn create_external_location(
        &self,
        request: CreateExternalLocationRequest,
        context: Cx,
    ) -> Result<ExternalLocation>;
    /// Get an external location
    async fn get_external_location(
        &self,
        request: GetExternalLocationRequest,
        context: Cx,
    ) -> Result<ExternalLocation>;
    /// Update an external location
    async fn update_external_location(
        &self,
        request: UpdateExternalLocationRequest,
        context: Cx,
    ) -> Result<ExternalLocation>;
    /// Delete an external location
    async fn delete_external_location(
        &self,
        request: DeleteExternalLocationRequest,
        context: Cx,
    ) -> Result<()>;
}
