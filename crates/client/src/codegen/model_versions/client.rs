// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
#[cfg(not(target_arch = "wasm32"))]
use ::olai_http::CloudClient as Transport;
#[cfg(target_arch = "wasm32")]
use ::olai_http_wasm::WasmClient as Transport;
use unitycatalog_common::models::model_versions::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct ModelVersionServiceClient {
    pub(crate) client: Transport,
    pub(crate) base_url: Url,
}
impl ModelVersionServiceClient {
    /// Create a new client instance
    pub fn new(client: Transport, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List model versions
    ///
    /// List the model versions of the specified registered model. If the caller is
    /// the metastore admin, all model versions are returned. Otherwise, the caller
    /// must have the appropriate privileges on the parent model.
    pub async fn list_model_versions(
        &self,
        request: &ListModelVersionsRequest,
    ) -> Result<ListModelVersionsResponse> {
        let formatted_path = format!("models/{}/versions", request.full_name);
        let mut url = self.base_url.join(&formatted_path)?;
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
    /// Create a model version
    ///
    /// Creates a new model version in PENDING_REGISTRATION status. The server
    /// assigns the version number and a storage location for the artifacts. The
    /// caller must be a metastore admin or the owner of the parent registered model.
    pub async fn create_model_version(
        &self,
        request: &CreateModelVersionRequest,
    ) -> Result<ModelVersion> {
        let url = self.base_url.join("models/versions")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Get a model version
    ///
    /// Gets a model version by its parent model name and version number.
    pub async fn get_model_version(
        &self,
        request: &GetModelVersionRequest,
    ) -> Result<ModelVersion> {
        let formatted_path = format!("models/{}/versions/{}", request.full_name, request.version);
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
    /// Update a model version
    ///
    /// Updates the model version that matches the supplied name and version.
    pub async fn update_model_version(
        &self,
        request: &UpdateModelVersionRequest,
    ) -> Result<ModelVersion> {
        let formatted_path = format!("models/{}/versions/{}", request.full_name, request.version);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Delete a model version
    ///
    /// Deletes the model version that matches the supplied name and version. For the
    /// deletion to succeed, the caller must be the owner of the parent registered
    /// model.
    pub async fn delete_model_version(&self, request: &DeleteModelVersionRequest) -> Result<()> {
        let formatted_path = format!("models/{}/versions/{}", request.full_name, request.version);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
    /// Finalize a model version
    ///
    /// Transitions a model version to READY once all artifacts have been written to
    /// its storage location.
    pub async fn finalize_model_version(
        &self,
        request: &FinalizeModelVersionRequest,
    ) -> Result<ModelVersion> {
        let formatted_path = format!(
            "models/{}/versions/{}/finalize",
            request.full_name, request.version
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
