// @generated — do not edit by hand.
//! Handler trait for [`SchemaHandler`].
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
//! A schema (also called a database) is the second layer of Unity Catalog’s three-level namespace.
//! A schema organizes tables, views and functions. To access (or list) a table or view in a schema,
//! users must have the USE_SCHEMA data permission on the schema and its parent catalog, and they must
//! have the SELECT permission on the table or view.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::schemas::v1::*;
#[async_trait]
pub trait SchemaHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// Gets an array of schemas for a catalog in the metastore. If the caller is the metastore
    /// admin or the owner of the parent catalog, all schemas for the catalog will be retrieved.
    /// Otherwise, only schemas owned by the caller (or for which the caller has the USE_SCHEMA privilege)
    /// will be retrieved. There is no guarantee of a specific ordering of the elements in the array.
    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
        context: Cx,
    ) -> Result<ListSchemasResponse>;
    /// Creates a new schema for catalog in the Metatastore. The caller must be a metastore admin,
    /// or have the CREATE_SCHEMA privilege in the parent catalog.
    async fn create_schema(&self, request: CreateSchemaRequest, context: Cx) -> Result<Schema>;
    /// Gets the specified schema within the metastore.
    /// The caller must be a metastore admin, the owner of the schema,
    /// or a user that has the USE_SCHEMA privilege on the schema.
    async fn get_schema(&self, request: GetSchemaRequest, context: Cx) -> Result<Schema>;
    /// Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
    /// If the caller is a metastore admin, only the owner field can be changed in the update.
    /// If the name field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA
    /// privilege on the parent catalog.
    async fn update_schema(&self, request: UpdateSchemaRequest, context: Cx) -> Result<Schema>;
    /// Deletes the specified schema from the parent catalog. The caller must be the owner
    /// of the schema or an owner of the parent catalog.
    async fn delete_schema(&self, request: DeleteSchemaRequest, context: Cx) -> Result<()>;
}
