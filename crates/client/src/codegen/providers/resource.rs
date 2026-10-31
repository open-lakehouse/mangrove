// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::builders::*;
use super::client::ProviderServiceClient;
use unitycatalog_common::models::providers::v1::*;
/// A client scoped to a single `provider`.
#[derive(Clone)]
pub struct ProviderClient {
    pub(crate) provider_name: String,
    pub(crate) client: ProviderServiceClient,
}
impl ProviderClient {
    /// Create a client bound to the resource's name components.
    pub fn new(provider_name: impl Into<String>, client: ProviderServiceClient) -> Self {
        Self {
            provider_name: provider_name.into(),
            client,
        }
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.provider_name
    }
    /// The fully-qualified name of this resource.
    pub fn full_name(&self) -> String {
        self.provider_name.clone()
    }
    /// Get a provider by name.
    pub fn get(&self) -> GetProviderBuilder {
        GetProviderBuilder::new(self.client.clone(), &self.provider_name)
    }
    /// Update a provider.
    pub fn update(&self) -> UpdateProviderBuilder {
        UpdateProviderBuilder::new(self.client.clone(), &self.provider_name)
    }
    /// Delete a provider.
    pub fn delete(&self) -> DeleteProviderBuilder {
        DeleteProviderBuilder::new(self.client.clone(), &self.provider_name)
    }
}
