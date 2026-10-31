// @generated — do not edit by hand.
//! Handler trait for [`TableHandler`].
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
//! Service for managing tables in Unity Catalog.
//! Tables represent structured data stored in a schema, supporting managed and external storage formats.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::tables::v1::*;

#[async_trait]
pub trait TableHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// Gets an array of summaries for tables for a schema and catalog within the metastore. The table summaries returned are either:
    /// - summaries for tables (within the current metastore and parent catalog and schema), when the user is a metastore admin, or:
    /// - summaries for tables and schemas (within the current metastore and parent catalog) for which the user has ownership or the
    /// SELECT privilege on the table and ownership or USE_SCHEMA privilege on the schema, provided that the user also has ownership
    /// or the USE_CATALOG privilege on the parent catalog.
    ///
    /// There is no guarantee of a specific ordering of the elements in the array.
    async fn list_table_summaries(
        &self,
        request: ListTableSummariesRequest,
        context: Cx,
    ) -> Result<ListTableSummariesResponse>;
    /// Gets an array of all tables for the current metastore under the parent catalog and schema.
    ///
    /// The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table.
    /// For the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the
    /// parent catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a
    /// specific ordering of the elements in the array.
    async fn list_tables(
        &self,
        request: ListTablesRequest,
        context: Cx,
    ) -> Result<ListTablesResponse>;
    /// Create a table
    async fn create_table(&self, request: CreateTableRequest, context: Cx) -> Result<Table>;
    /// Get a table
    async fn get_table(&self, request: GetTableRequest, context: Cx) -> Result<Table>;
    /// Get boolean reflecting if table exists
    async fn get_table_exists(
        &self,
        request: GetTableExistsRequest,
        context: Cx,
    ) -> Result<GetTableExistsResponse>;
    /// Delete a table
    async fn delete_table(&self, request: DeleteTableRequest, context: Cx) -> Result<()>;
}
