// @generated — do not edit by hand.
//! Handler trait for [`RegisteredModelHandler`].
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
//! Manage registered models in the service.
//!
//! A registered model is a securable in the three-level namespace
//! (catalog.schema.model) that groups a collection of model versions.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::registered_models::v1::*;
#[async_trait]
pub trait RegisteredModelHandler<Cx = crate::api::RequestContext>: Send + Sync + 'static {
    /// List registered models
    ///
    /// List registered models within the specified parent catalog and schema. If
    /// the caller is the metastore admin, all registered models are returned in the
    /// response. Otherwise, the caller must have USE_CATALOG on the parent catalog
    /// and USE_SCHEMA on the parent schema, and the model must either be owned by
    /// the caller or the caller must have a privilege on the model.
    async fn list_registered_models(
        &self,
        request: ListRegisteredModelsRequest,
        context: Cx,
    ) -> Result<ListRegisteredModelsResponse>;
    /// Create a registered model
    ///
    /// Creates a new registered model. The caller must be a metastore admin or have
    /// the CREATE_MODEL privilege on the parent catalog and schema.
    async fn create_registered_model(
        &self,
        request: CreateRegisteredModelRequest,
        context: Cx,
    ) -> Result<RegisteredModel>;
    /// Get a registered model
    ///
    /// Gets a registered model from within a parent catalog and schema. For the
    /// fetch to succeed, the caller must be a metastore admin, the owner of the
    /// registered model, or have a privilege on the registered model.
    async fn get_registered_model(
        &self,
        request: GetRegisteredModelRequest,
        context: Cx,
    ) -> Result<RegisteredModel>;
    /// Update a registered model
    ///
    /// Updates the registered model that matches the supplied name.
    async fn update_registered_model(
        &self,
        request: UpdateRegisteredModelRequest,
        context: Cx,
    ) -> Result<RegisteredModel>;
    /// Delete a registered model
    ///
    /// Deletes the registered model that matches the supplied name. For the deletion
    /// to succeed, the caller must be the owner of the registered model.
    async fn delete_registered_model(
        &self,
        request: DeleteRegisteredModelRequest,
        context: Cx,
    ) -> Result<()>;
}
