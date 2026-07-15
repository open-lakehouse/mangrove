// @generated — do not edit by hand.
//! Handler trait for [`ModelVersionHandler`].
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
//! Manage model versions in the service.
//!
//! A model version belongs to a registered model and is identified by the model's
//! three-level name plus a server-assigned integer version. It carries its own
//! artifact storage location and READY/PENDING lifecycle.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::model_versions::v1::*;
#[async_trait]
pub trait ModelVersionHandler<Cx = crate::api::RequestContext>: Send + Sync + 'static {
    /// List model versions
    ///
    /// List the model versions of the specified registered model. If the caller is
    /// the metastore admin, all model versions are returned. Otherwise, the caller
    /// must have the appropriate privileges on the parent model.
    async fn list_model_versions(
        &self,
        request: ListModelVersionsRequest,
        context: Cx,
    ) -> Result<ListModelVersionsResponse>;
    /// Create a model version
    ///
    /// Creates a new model version in PENDING_REGISTRATION status. The server
    /// assigns the version number and a storage location for the artifacts. The
    /// caller must be a metastore admin or the owner of the parent registered model.
    async fn create_model_version(
        &self,
        request: CreateModelVersionRequest,
        context: Cx,
    ) -> Result<ModelVersion>;
    /// Get a model version
    ///
    /// Gets a model version by its parent model name and version number.
    async fn get_model_version(
        &self,
        request: GetModelVersionRequest,
        context: Cx,
    ) -> Result<ModelVersion>;
    /// Update a model version
    ///
    /// Updates the model version that matches the supplied name and version.
    async fn update_model_version(
        &self,
        request: UpdateModelVersionRequest,
        context: Cx,
    ) -> Result<ModelVersion>;
    /// Delete a model version
    ///
    /// Deletes the model version that matches the supplied name and version. For the
    /// deletion to succeed, the caller must be the owner of the parent registered
    /// model.
    async fn delete_model_version(
        &self,
        request: DeleteModelVersionRequest,
        context: Cx,
    ) -> Result<()>;
    /// Finalize a model version
    ///
    /// Transitions a model version to READY once all artifacts have been written to
    /// its storage location.
    async fn finalize_model_version(
        &self,
        request: FinalizeModelVersionRequest,
        context: Cx,
    ) -> Result<ModelVersion>;
}
