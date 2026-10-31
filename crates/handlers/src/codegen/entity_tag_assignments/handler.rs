// @generated — do not edit by hand.
//! Handler trait for [`EntityTagAssignmentHandler`].
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
//! Manage assignments of tags to Unity Catalog entities.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::tags::v1::*;

#[async_trait]
pub trait EntityTagAssignmentHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// List entity tag assignments
    ///
    /// Gets the tag assignments for the specified entity.
    async fn list_entity_tag_assignments(
        &self,
        request: ListEntityTagAssignmentsRequest,
        context: Cx,
    ) -> Result<ListEntityTagAssignmentsResponse>;
    /// Create an entity tag assignment
    ///
    /// Assigns a tag to a Unity Catalog entity.
    async fn create_entity_tag_assignment(
        &self,
        request: CreateEntityTagAssignmentRequest,
        context: Cx,
    ) -> Result<EntityTagAssignment>;
    /// Get an entity tag assignment
    ///
    /// Gets the tag assignment for the specified entity and tag key.
    async fn get_entity_tag_assignment(
        &self,
        request: GetEntityTagAssignmentRequest,
        context: Cx,
    ) -> Result<EntityTagAssignment>;
    /// Update an entity tag assignment
    ///
    /// Updates the tag assignment for the specified entity and tag key.
    async fn update_entity_tag_assignment(
        &self,
        request: UpdateEntityTagAssignmentRequest,
        context: Cx,
    ) -> Result<EntityTagAssignment>;
    /// Delete an entity tag assignment
    ///
    /// Deletes the tag assignment for the specified entity and tag key.
    async fn delete_entity_tag_assignment(
        &self,
        request: DeleteEntityTagAssignmentRequest,
        context: Cx,
    ) -> Result<()>;
}
