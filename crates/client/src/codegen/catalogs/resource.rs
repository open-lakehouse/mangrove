// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::builders::*;
use super::client::CatalogServiceClient;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::schemas::v1::*;
/// A client scoped to a single `catalog`.
#[derive(Clone)]
pub struct CatalogClient {
    pub(crate) catalog_name: String,
    pub(crate) client: CatalogServiceClient,
}
impl CatalogClient {
    /// Create a client bound to the resource's name components.
    pub fn new(catalog_name: impl Into<String>, client: CatalogServiceClient) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            client,
        }
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.catalog_name
    }
    /// The fully-qualified name of this resource.
    pub fn full_name(&self) -> String {
        self.catalog_name.clone()
    }
    /// Get a catalog
    ///
    /// Gets the specified catalog in a metastore. The caller must be a metastore admin,
    /// the owner of the catalog, or a user that has the USE_CATALOG privilege set for their account.
    pub fn get(&self) -> GetCatalogBuilder {
        GetCatalogBuilder::new(self.client.clone(), &self.catalog_name)
    }
    /// Update a catalog
    ///
    /// Updates the catalog that matches the supplied name. The caller must be either
    /// the owner of the catalog, or a metastore admin (when changing the owner field of the catalog).
    pub fn update(&self) -> UpdateCatalogBuilder {
        UpdateCatalogBuilder::new(self.client.clone(), &self.catalog_name)
    }
    /// Delete a catalog
    ///
    /// Deletes the catalog that matches the supplied name. The caller must
    /// be a metastore admin or the owner of the catalog.
    pub fn delete(&self) -> DeleteCatalogBuilder {
        DeleteCatalogBuilder::new(self.client.clone(), &self.catalog_name)
    }
    /// Access a `schema` within this resource.
    pub fn schema(&self, schema_name: impl Into<String>) -> crate::codegen::schemas::SchemaClient {
        crate::codegen::schemas::SchemaClient::new(
            &self.catalog_name,
            schema_name,
            crate::codegen::schemas::SchemaServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
        )
    }
    /// Create a `schema` within this resource.
    pub fn create_schema(
        &self,
        name: impl Into<String>,
    ) -> crate::codegen::schemas::CreateSchemaBuilder {
        crate::codegen::schemas::CreateSchemaBuilder::new(
            crate::codegen::schemas::SchemaServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            name,
            &self.catalog_name,
        )
    }
    /// List `schema` resources within this resource.
    pub fn list_schemas(&self) -> crate::codegen::schemas::ListSchemasBuilder {
        crate::codegen::schemas::ListSchemasBuilder::new(
            crate::codegen::schemas::SchemaServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            &self.catalog_name,
        )
    }
}
