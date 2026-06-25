// @generated — do not edit by hand.
use super::builders::*;
use super::client::CredentialServiceClient;
use unitycatalog_common::models::credentials::v1::*;
/// A client scoped to a single `credential`.
#[derive(Clone)]
pub struct CredentialClient {
    pub(crate) credential_name: String,
    pub(crate) client: CredentialServiceClient,
}
impl CredentialClient {
    /// Create a client bound to the resource's name components.
    pub fn new(credential_name: impl Into<String>, client: CredentialServiceClient) -> Self {
        Self {
            credential_name: credential_name.into(),
            client,
        }
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.credential_name
    }
    /// The fully-qualified name of this resource.
    pub fn full_name(&self) -> String {
        self.credential_name.clone()
    }
    pub fn get(&self) -> GetCredentialBuilder {
        GetCredentialBuilder::new(self.client.clone(), &self.credential_name)
    }
    pub fn update(&self) -> UpdateCredentialBuilder {
        UpdateCredentialBuilder::new(self.client.clone(), &self.credential_name)
    }
    pub fn delete(&self) -> DeleteCredentialBuilder {
        DeleteCredentialBuilder::new(self.client.clone(), &self.credential_name)
    }
}
