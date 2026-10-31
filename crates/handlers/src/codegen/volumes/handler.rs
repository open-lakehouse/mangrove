// @generated — do not edit by hand.
//! Handler trait for [`VolumeHandler`].
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
//! Service for managing volumes in Unity Catalog.
//! Volumes represent logical storage locations (managed or external) within a schema.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::volumes::v1::*;
#[async_trait]
pub trait VolumeHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// Lists volumes.
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        context: Cx,
    ) -> Result<ListVolumesResponse>;
    async fn create_volume(&self, request: CreateVolumeRequest, context: Cx) -> Result<Volume>;
    async fn get_volume(&self, request: GetVolumeRequest, context: Cx) -> Result<Volume>;
    async fn update_volume(&self, request: UpdateVolumeRequest, context: Cx) -> Result<Volume>;
    async fn delete_volume(&self, request: DeleteVolumeRequest, context: Cx) -> Result<()>;
}
