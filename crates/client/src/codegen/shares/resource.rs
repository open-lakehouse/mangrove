// @generated — do not edit by hand.
use super::builders::*;
use super::client::ShareServiceClient;
use unitycatalog_common::models::shares::v1::*;
/// A client scoped to a single `share`.
#[derive(Clone)]
pub struct ShareClient {
    pub(crate) share_name: String,
    pub(crate) client: ShareServiceClient,
}
impl ShareClient {
    /// Create a client bound to the resource's name components.
    pub fn new(share_name: impl Into<String>, client: ShareServiceClient) -> Self {
        Self {
            share_name: share_name.into(),
            client,
        }
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.share_name
    }
    /// The fully-qualified name of this resource.
    pub fn full_name(&self) -> String {
        self.share_name.clone()
    }
    /// Get a share by name.
    pub fn get(&self) -> GetShareBuilder {
        GetShareBuilder::new(self.client.clone(), &self.share_name)
    }
    /// Update a share.
    pub fn update(&self) -> UpdateShareBuilder {
        UpdateShareBuilder::new(self.client.clone(), &self.share_name)
    }
    /// Deletes a share.
    pub fn delete(&self) -> DeleteShareBuilder {
        DeleteShareBuilder::new(self.client.clone(), &self.share_name)
    }
    /// Gets the permissions for a data share from the metastore.
    pub fn get_permissions(&self) -> GetPermissionsBuilder {
        GetPermissionsBuilder::new(self.client.clone(), &self.share_name)
    }
    /// Updates the permissions for a data share in the metastore.
    pub fn update_permissions(&self) -> UpdatePermissionsBuilder {
        UpdatePermissionsBuilder::new(self.client.clone(), &self.share_name)
    }
}
