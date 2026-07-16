// @generated — do not edit by hand.
//! Handler trait for [`SharingVolumeHandler`].
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
//! Open Sharing volume APIs: discovery and credential vending for shared
//! storage-backed volumes.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
#[async_trait]
pub trait SharingVolumeHandler<Cx = crate::DefaultContext>: Send + Sync + 'static {
    /// List the volumes in a given share's schema.
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        context: Cx,
    ) -> Result<ListVolumesResponse>;
    /// List all the volumes under a share, across all schemas.
    async fn list_all_volumes(
        &self,
        request: ListAllVolumesRequest,
        context: Cx,
    ) -> Result<ListAllVolumesResponse>;
    /// Get the metadata for a single shared volume.
    async fn get_volume(&self, request: GetVolumeRequest, context: Cx) -> Result<SharingVolume>;
    /// Generate temporary credentials scoped to a shared volume's storage
    /// location, for direct file access via the cloud storage API.
    async fn generate_temporary_volume_credentials(
        &self,
        request: GenerateTemporaryVolumeCredentialsRequest,
        context: Cx,
    ) -> Result<SharingTemporaryCredentials>;
}
