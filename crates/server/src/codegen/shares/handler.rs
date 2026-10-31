// @generated — do not edit by hand.
//! Handler trait for [`ShareHandler`].
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
//! Service for managing shares
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::shares::v1::*;
#[async_trait]
pub trait ShareHandler<Cx = crate::api::RequestContext>: Send + Sync + 'static {
    /// List shares.
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: Cx,
    ) -> Result<ListSharesResponse>;
    /// Create a new share.
    async fn create_share(&self, request: CreateShareRequest, context: Cx) -> Result<Share>;
    /// Get a share by name.
    async fn get_share(&self, request: GetShareRequest, context: Cx) -> Result<Share>;
    /// Update a share.
    async fn update_share(&self, request: UpdateShareRequest, context: Cx) -> Result<Share>;
    /// Deletes a share.
    async fn delete_share(&self, request: DeleteShareRequest, context: Cx) -> Result<()>;
    /// Gets the permissions for a data share from the metastore.
    async fn get_permissions(
        &self,
        request: GetPermissionsRequest,
        context: Cx,
    ) -> Result<GetPermissionsResponse>;
    /// Updates the permissions for a data share in the metastore.
    async fn update_permissions(
        &self,
        request: UpdatePermissionsRequest,
        context: Cx,
    ) -> Result<UpdatePermissionsResponse>;
}
