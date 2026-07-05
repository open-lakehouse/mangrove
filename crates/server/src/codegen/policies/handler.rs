// @generated — do not edit by hand.
//! Handler trait for [`PolicyHandler`].
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
//! Manage ABAC row-filter and column-mask policies bound to Unity Catalog securables.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::policies::v1::*;
#[async_trait]
pub trait PolicyHandler<Cx = crate::api::RequestContext>: Send + Sync + 'static {
    /// List policies
    ///
    /// Gets an array of policies defined on the specified securable. There is no guarantee
    /// of a specific ordering of the elements in the array.
    async fn list_policies(
        &self,
        request: ListPoliciesRequest,
        context: Cx,
    ) -> Result<ListPoliciesResponse>;
    /// Create a new policy
    ///
    /// Creates a new row-filter or column-mask policy on the specified securable.
    async fn create_policy(&self, request: CreatePolicyRequest, context: Cx) -> Result<PolicyInfo>;
    /// Get a policy
    ///
    /// Gets the policy that matches the supplied name, defined on the specified securable.
    async fn get_policy(&self, request: GetPolicyRequest, context: Cx) -> Result<PolicyInfo>;
    /// Update a policy
    ///
    /// Updates the policy that matches the supplied name, defined on the specified securable.
    async fn update_policy(&self, request: UpdatePolicyRequest, context: Cx) -> Result<PolicyInfo>;
    /// Delete a policy
    ///
    /// Deletes the policy that matches the supplied name, defined on the specified securable.
    async fn delete_policy(&self, request: DeletePolicyRequest, context: Cx) -> Result<()>;
}
