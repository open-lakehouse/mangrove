// @generated — do not edit by hand.
//! Handler trait for [`FunctionHandler`].
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
//! Manage User-Defined Functions (UDFs) in the service.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::functions::v1::*;

#[async_trait]
pub trait FunctionHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// List functions
    ///
    /// List functions within the specified parent catalog and schema. If the caller is the metastore
    /// admin, all functions are returned in the response. Otherwise, the caller must have USE_CATALOG
    /// on the parent catalog and USE_SCHEMA on the parent schema, and the function must either be
    /// owned by the caller or have SELECT on the function.
    async fn list_functions(
        &self,
        request: ListFunctionsRequest,
        context: Cx,
    ) -> Result<ListFunctionsResponse>;
    /// Create a function
    ///
    /// Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION
    /// privilege on the parent catalog and schema.
    async fn create_function(
        &self,
        request: CreateFunctionRequest,
        context: Cx,
    ) -> Result<Function>;
    /// Get a function
    ///
    /// Gets a function from within a parent catalog and schema. For the fetch to succeed,
    /// the caller must be a metastore admin, the owner of the function, or have SELECT on
    /// the function.
    async fn get_function(&self, request: GetFunctionRequest, context: Cx) -> Result<Function>;
    /// Update a function
    ///
    /// Updates the function that matches the supplied name. Only the owner of the function
    /// can be updated.
    async fn update_function(
        &self,
        request: UpdateFunctionRequest,
        context: Cx,
    ) -> Result<Function>;
    /// Delete a function
    ///
    /// Deletes the function that matches the supplied name. For the deletion to succeed,
    /// the caller must be the owner of the function.
    async fn delete_function(&self, request: DeleteFunctionRequest, context: Cx) -> Result<()>;
}
