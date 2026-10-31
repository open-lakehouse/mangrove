// @generated — do not edit by hand.
//! Handler trait for [`TagPolicyHandler`].
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
//! Manage governed tag definitions (tag policies).
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::tags::v1::*;
#[async_trait]
pub trait TagPolicyHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// List tag policies
    ///
    /// Gets an array of tag policies. There is no guarantee of a specific ordering
    /// of the elements in the array.
    async fn list_tag_policies(
        &self,
        request: ListTagPoliciesRequest,
        context: Cx,
    ) -> Result<ListTagPoliciesResponse>;
    /// Create a new tag policy
    ///
    /// Creates a new governed tag definition.
    async fn create_tag_policy(
        &self,
        request: CreateTagPolicyRequest,
        context: Cx,
    ) -> Result<TagPolicy>;
    /// Get a tag policy
    ///
    /// Gets the governed tag definition for the specified tag key.
    async fn get_tag_policy(&self, request: GetTagPolicyRequest, context: Cx) -> Result<TagPolicy>;
    /// Update a tag policy
    ///
    /// Updates the governed tag definition that matches the supplied tag key.
    async fn update_tag_policy(
        &self,
        request: UpdateTagPolicyRequest,
        context: Cx,
    ) -> Result<TagPolicy>;
    /// Delete a tag policy
    ///
    /// Deletes the governed tag definition that matches the supplied tag key.
    async fn delete_tag_policy(&self, request: DeleteTagPolicyRequest, context: Cx) -> Result<()>;
}
