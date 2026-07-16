// @generated — do not edit by hand.
//! Handler trait for [`SharingHandler`].
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
//! Service exposing the official APIs for Delta Sharing.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
#[async_trait]
pub trait SharingHandler<Cx = crate::DefaultContext>: Send + Sync + 'static {
    /// List shares accessible to a recipient.
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        context: Cx,
    ) -> Result<ListSharesResponse>;
    /// Get the metadata for a specific share.
    async fn get_share(&self, request: GetShareRequest, context: Cx) -> Result<Share>;
    /// List the schemas in a share.
    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
        context: Cx,
    ) -> Result<ListSchemasResponse>;
    /// List the tables in a given share's schema.
    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: Cx,
    ) -> Result<ListTablesResponse>;
    /// List all the tables under a share.
    ///
    /// A convenience over per-schema listing: returns every table across all
    /// schemas in the share.
    async fn list_all_tables(
        &self,
        request: ListAllTablesRequest,
        context: Cx,
    ) -> Result<ListAllTablesResponse>;
}
