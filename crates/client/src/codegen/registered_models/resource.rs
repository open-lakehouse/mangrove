// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::builders::*;
use super::client::RegisteredModelServiceClient;
use unitycatalog_common::models::registered_models::v1::*;
/// A client scoped to a single `registered_model`.
#[derive(Clone)]
pub struct RegisteredModelClient {
    pub(crate) catalog_name: String,
    pub(crate) schema_name: String,
    pub(crate) registered_model_name: String,
    pub(crate) client: RegisteredModelServiceClient,
}
impl RegisteredModelClient {
    /// Create a client bound to the resource's name components.
    pub fn new(
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        registered_model_name: impl Into<String>,
        client: RegisteredModelServiceClient,
    ) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            registered_model_name: registered_model_name.into(),
            client,
        }
    }
    /// Create a `registered_model` client from its dot-joined full name (e.g. `"catalog_name.schema_name.registered_model_name"`).
    pub fn from_full_name(
        full_name: impl Into<String>,
        client: RegisteredModelServiceClient,
    ) -> Self {
        let full_name = full_name.into();
        let mut parts = full_name.splitn(3usize, '.');
        let catalog_name = parts.next().unwrap_or_default();
        let schema_name = parts.next().unwrap_or_default();
        let registered_model_name = parts.next().unwrap_or_default();
        Self::new(catalog_name, schema_name, registered_model_name, client)
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
        &self.registered_model_name
    }
    /// The fully-qualified name of this resource (its dot-joined name components).
    pub fn full_name(&self) -> String {
        format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.registered_model_name
        )
    }
    /// Get a registered model
    ///
    /// Gets a registered model from within a parent catalog and schema. For the
    /// fetch to succeed, the caller must be a metastore admin, the owner of the
    /// registered model, or have a privilege on the registered model.
    pub fn get(&self) -> GetRegisteredModelBuilder {
        GetRegisteredModelBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.registered_model_name
            ),
        )
    }
    /// Update a registered model
    ///
    /// Updates the registered model that matches the supplied name.
    pub fn update(&self) -> UpdateRegisteredModelBuilder {
        UpdateRegisteredModelBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.registered_model_name
            ),
        )
    }
    /// Delete a registered model
    ///
    /// Deletes the registered model that matches the supplied name. For the deletion
    /// to succeed, the caller must be the owner of the registered model.
    pub fn delete(&self) -> DeleteRegisteredModelBuilder {
        DeleteRegisteredModelBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.registered_model_name
            ),
        )
    }
}
