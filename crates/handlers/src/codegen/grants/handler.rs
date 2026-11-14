// @generated — do not edit by hand.
//! Handler trait for [`GrantHandler`].
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
//! Grants: the privileges principals hold on securables. The Unity Catalog OSS
//! permissions API, which Databricks shares.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::grants::v1::*;

#[async_trait]
pub trait GrantHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// Gets the privileges granted on a securable. Does not include inherited ones.
    async fn get_grants(&self, request: GetGrantsRequest, context: Cx)
    -> Result<GetGrantsResponse>;
    /// Grants and revokes privileges on a securable.
    async fn update_grants(
        &self,
        request: UpdateGrantsRequest,
        context: Cx,
    ) -> Result<UpdateGrantsResponse>;
}
