// @generated — do not edit by hand.
use super::builders::*;
use super::client::ExternalLocationServiceClient;
use unitycatalog_common::models::external_locations::v1::*;
/// A client scoped to a single `external_location`.
#[derive(Clone)]
pub struct ExternalLocationClient {
    pub(crate) external_location_name: String,
    pub(crate) client: ExternalLocationServiceClient,
}
impl ExternalLocationClient {
    /// Create a client bound to the resource's name components.
    pub fn new(
        external_location_name: impl Into<String>,
        client: ExternalLocationServiceClient,
    ) -> Self {
        Self {
            external_location_name: external_location_name.into(),
            client,
        }
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.external_location_name
    }
    /// The fully-qualified name of this resource.
    pub fn full_name(&self) -> String {
        self.external_location_name.clone()
    }
    /// Get an external location
    pub fn get(&self) -> GetExternalLocationBuilder {
        GetExternalLocationBuilder::new(self.client.clone(), &self.external_location_name)
    }
    /// Update an external location
    pub fn update(&self) -> UpdateExternalLocationBuilder {
        UpdateExternalLocationBuilder::new(self.client.clone(), &self.external_location_name)
    }
    /// Delete an external location
    pub fn delete(&self) -> DeleteExternalLocationBuilder {
        DeleteExternalLocationBuilder::new(self.client.clone(), &self.external_location_name)
    }
}
