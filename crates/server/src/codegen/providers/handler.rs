// @generated — do not edit by hand.
//! Handler trait for [`ProviderHandler`].
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
//! Providers
//!
//! A provider represents an organization that shares data with this metastore.
//! It is the inbound counterpart of a recipient: registered from a share
//! activation/credential file and used to access shares offered by an upstream
//! Delta Sharing server.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::providers::v1::*;
#[async_trait]
pub trait ProviderHandler<Cx = crate::api::RequestContext>: Send + Sync + 'static {
    /// List providers.
    async fn list_providers(
        &self,
        request: ListProvidersRequest,
        context: Cx,
    ) -> Result<ListProvidersResponse>;
    /// Create a new provider.
    async fn create_provider(
        &self,
        request: CreateProviderRequest,
        context: Cx,
    ) -> Result<Provider>;
    /// Get a provider by name.
    async fn get_provider(&self, request: GetProviderRequest, context: Cx) -> Result<Provider>;
    /// Update a provider.
    async fn update_provider(
        &self,
        request: UpdateProviderRequest,
        context: Cx,
    ) -> Result<Provider>;
    /// Delete a provider.
    async fn delete_provider(&self, request: DeleteProviderRequest, context: Cx) -> Result<()>;
}
