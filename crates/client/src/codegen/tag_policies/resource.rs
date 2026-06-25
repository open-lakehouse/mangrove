// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::builders::*;
use super::client::TagPolicyServiceClient;
use unitycatalog_common::models::tags::v1::*;
/// A client scoped to a single `tag_policy`.
#[derive(Clone)]
pub struct TagPolicyClient {
    pub(crate) tag_policy_name: String,
    pub(crate) client: TagPolicyServiceClient,
}
impl TagPolicyClient {
    /// Create a client bound to the resource's name components.
    pub fn new(tag_policy_name: impl Into<String>, client: TagPolicyServiceClient) -> Self {
        Self {
            tag_policy_name: tag_policy_name.into(),
            client,
        }
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.tag_policy_name
    }
    /// The fully-qualified name of this resource.
    pub fn full_name(&self) -> String {
        self.tag_policy_name.clone()
    }
    /// Get a tag policy
    ///
    /// Gets the governed tag definition for the specified tag key.
    pub fn get(&self) -> GetTagPolicyBuilder {
        GetTagPolicyBuilder::new(self.client.clone(), &self.tag_policy_name)
    }
    /// Update a tag policy
    ///
    /// Updates the governed tag definition that matches the supplied tag key.
    pub fn update(&self, tag_policy: TagPolicy) -> UpdateTagPolicyBuilder {
        UpdateTagPolicyBuilder::new(self.client.clone(), &self.tag_policy_name, tag_policy)
    }
    /// Delete a tag policy
    ///
    /// Deletes the governed tag definition that matches the supplied tag key.
    pub fn delete(&self) -> DeleteTagPolicyBuilder {
        DeleteTagPolicyBuilder::new(self.client.clone(), &self.tag_policy_name)
    }
}
