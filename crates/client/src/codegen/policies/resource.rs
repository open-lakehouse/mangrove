// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::builders::*;
use super::client::PolicyServiceClient;
use unitycatalog_common::models::policies::v1::*;
/// A client scoped to a single `policy`.
#[derive(Clone)]
pub struct PolicyClient {
    pub(crate) policy_name: String,
    pub(crate) client: PolicyServiceClient,
}
impl PolicyClient {
    /// Create a client bound to the resource's name components.
    pub fn new(policy_name: impl Into<String>, client: PolicyServiceClient) -> Self {
        Self {
            policy_name: policy_name.into(),
            client,
        }
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.policy_name
    }
    /// The fully-qualified name of this resource.
    pub fn full_name(&self) -> String {
        self.policy_name.clone()
    }
    /// Get a policy
    ///
    /// Gets the policy that matches the supplied name, defined on the specified securable.
    pub fn get(&self) -> GetPolicyBuilder {
        GetPolicyBuilder::new(
            self.client.clone(),
            &self.policy_name,
            &self.policy_name,
            &self.policy_name,
        )
    }
    /// Update a policy
    ///
    /// Updates the policy that matches the supplied name, defined on the specified securable.
    pub fn update(&self) -> UpdatePolicyBuilder {
        UpdatePolicyBuilder::new(
            self.client.clone(),
            &self.policy_name,
            &self.policy_name,
            &self.policy_name,
        )
    }
    /// Delete a policy
    ///
    /// Deletes the policy that matches the supplied name, defined on the specified securable.
    pub fn delete(&self) -> DeletePolicyBuilder {
        DeletePolicyBuilder::new(
            self.client.clone(),
            &self.policy_name,
            &self.policy_name,
            &self.policy_name,
        )
    }
}
