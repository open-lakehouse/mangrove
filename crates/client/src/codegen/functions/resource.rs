// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::builders::*;
use super::client::FunctionServiceClient;
use unitycatalog_common::models::functions::v1::*;
/// A client scoped to a single `function`.
#[derive(Clone)]
pub struct FunctionClient {
    pub(crate) catalog_name: String,
    pub(crate) schema_name: String,
    pub(crate) function_name: String,
    pub(crate) client: FunctionServiceClient,
}
impl FunctionClient {
    /// Create a client bound to the resource's name components.
    pub fn new(
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        function_name: impl Into<String>,
        client: FunctionServiceClient,
    ) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            function_name: function_name.into(),
            client,
        }
    }
    /// Create a `function` client from its dot-joined full name (e.g. `"catalog_name.schema_name.function_name"`).
    pub fn from_full_name(full_name: impl Into<String>, client: FunctionServiceClient) -> Self {
        let full_name = full_name.into();
        let mut parts = full_name.splitn(3usize, '.');
        let catalog_name = parts.next().unwrap_or_default();
        let schema_name = parts.next().unwrap_or_default();
        let function_name = parts.next().unwrap_or_default();
        Self::new(catalog_name, schema_name, function_name, client)
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
        &self.function_name
    }
    /// The fully-qualified name of this resource (its dot-joined name components).
    pub fn full_name(&self) -> String {
        format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.function_name
        )
    }
    /// Get a function
    ///
    /// Gets a function from within a parent catalog and schema. For the fetch to succeed,
    /// the caller must be a metastore admin, the owner of the function, or have SELECT on
    /// the function.
    pub fn get(&self) -> GetFunctionBuilder {
        GetFunctionBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.function_name
            ),
        )
    }
    /// Update a function
    ///
    /// Updates the function that matches the supplied name. Only the owner of the function
    /// can be updated.
    pub fn update(&self) -> UpdateFunctionBuilder {
        UpdateFunctionBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.function_name
            ),
        )
    }
    /// Delete a function
    ///
    /// Deletes the function that matches the supplied name. For the deletion to succeed,
    /// the caller must be the owner of the function.
    pub fn delete(&self) -> DeleteFunctionBuilder {
        DeleteFunctionBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.function_name
            ),
        )
    }
}
