// @generated — do not edit by hand.
use super::builders::*;
use super::client::VolumeServiceClient;
use unitycatalog_common::models::volumes::v1::*;
/// A client scoped to a single `volume`.
#[derive(Clone)]
pub struct VolumeClient {
    pub(crate) catalog_name: String,
    pub(crate) schema_name: String,
    pub(crate) volume_name: String,
    pub(crate) client: VolumeServiceClient,
}
impl VolumeClient {
    /// Create a client bound to the resource's name components.
    pub fn new(
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        volume_name: impl Into<String>,
        client: VolumeServiceClient,
    ) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            volume_name: volume_name.into(),
            client,
        }
    }
    /// Create a `volume` client from its dot-joined full name (e.g. `"catalog_name.schema_name.volume_name"`).
    pub fn from_full_name(full_name: impl Into<String>, client: VolumeServiceClient) -> Self {
        let full_name = full_name.into();
        let mut parts = full_name.splitn(3usize, '.');
        let catalog_name = parts.next().unwrap_or_default();
        let schema_name = parts.next().unwrap_or_default();
        let volume_name = parts.next().unwrap_or_default();
        Self::new(catalog_name, schema_name, volume_name, client)
    }
    /// The `catalog_name` component of this resource's name.
    pub fn catalog_name(&self) -> &str {
        &self.catalog_name
    }
    /// The `schema_name` component of this resource's name.
    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.volume_name
    }
    /// The fully-qualified name of this resource (its dot-joined name components).
    pub fn full_name(&self) -> String {
        format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.volume_name
        )
    }
    pub fn get(&self) -> GetVolumeBuilder {
        GetVolumeBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.volume_name
            ),
        )
    }
    pub fn update(&self) -> UpdateVolumeBuilder {
        UpdateVolumeBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.volume_name
            ),
        )
    }
    pub fn delete(&self) -> DeleteVolumeBuilder {
        DeleteVolumeBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.volume_name
            ),
        )
    }
}
