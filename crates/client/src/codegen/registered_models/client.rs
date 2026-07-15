// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_common::models::registered_models::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct RegisteredModelServiceClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl RegisteredModelServiceClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List registered models
    ///
    /// List registered models within the specified parent catalog and schema. If
    /// the caller is the metastore admin, all registered models are returned in the
    /// response. Otherwise, the caller must have USE_CATALOG on the parent catalog
    /// and USE_SCHEMA on the parent schema, and the model must either be owned by
    /// the caller or the caller must have a privilege on the model.
    pub async fn list_registered_models(
        &self,
        request: &ListRegisteredModelsRequest,
    ) -> Result<ListRegisteredModelsResponse> {
        let mut url = self.base_url.join("models")?;
        if let Some(ref value) = request.catalog_name {
            url.query_pairs_mut()
                .append_pair("catalog_name", &value.to_string());
        }
        if let Some(ref value) = request.schema_name {
            url.query_pairs_mut()
                .append_pair("schema_name", &value.to_string());
        }
        if let Some(ref value) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &value.to_string());
        }
        if let Some(ref value) = request.page_token {
            url.query_pairs_mut()
                .append_pair("page_token", &value.to_string());
        }
        if let Some(ref value) = request.include_browse {
            url.query_pairs_mut()
                .append_pair("include_browse", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Create a registered model
    ///
    /// Creates a new registered model. The caller must be a metastore admin or have
    /// the CREATE_MODEL privilege on the parent catalog and schema.
    pub async fn create_registered_model(
        &self,
        request: &CreateRegisteredModelRequest,
    ) -> Result<RegisteredModel> {
        let url = self.base_url.join("models")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Get a registered model
    ///
    /// Gets a registered model from within a parent catalog and schema. For the
    /// fetch to succeed, the caller must be a metastore admin, the owner of the
    /// registered model, or have a privilege on the registered model.
    pub async fn get_registered_model(
        &self,
        request: &GetRegisteredModelRequest,
    ) -> Result<RegisteredModel> {
        let formatted_path = format!("models/{}", request.full_name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.include_browse {
            url.query_pairs_mut()
                .append_pair("include_browse", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Update a registered model
    ///
    /// Updates the registered model that matches the supplied name.
    pub async fn update_registered_model(
        &self,
        request: &UpdateRegisteredModelRequest,
    ) -> Result<RegisteredModel> {
        let formatted_path = format!("models/{}", request.full_name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Delete a registered model
    ///
    /// Deletes the registered model that matches the supplied name. For the deletion
    /// to succeed, the caller must be the owner of the registered model.
    pub async fn delete_registered_model(
        &self,
        request: &DeleteRegisteredModelRequest,
    ) -> Result<()> {
        let formatted_path = format!("models/{}", request.full_name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.force {
            url.query_pairs_mut()
                .append_pair("force", &value.to_string());
        }
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
}
