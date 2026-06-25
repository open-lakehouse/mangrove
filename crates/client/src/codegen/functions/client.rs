// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_common::models::functions::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct FunctionServiceClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl FunctionServiceClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List functions
    ///
    /// List functions within the specified parent catalog and schema. If the caller is the metastore
    /// admin, all functions are returned in the response. Otherwise, the caller must have USE_CATALOG
    /// on the parent catalog and USE_SCHEMA on the parent schema, and the function must either be
    /// owned by the caller or have SELECT on the function.
    pub async fn list_functions(
        &self,
        request: &ListFunctionsRequest,
    ) -> Result<ListFunctionsResponse> {
        let mut url = self.base_url.join("functions")?;
        url.query_pairs_mut()
            .append_pair("catalog_name", &request.catalog_name);
        url.query_pairs_mut()
            .append_pair("schema_name", &request.schema_name);
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
    /// Create a function
    ///
    /// Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION
    /// privilege on the parent catalog and schema.
    pub async fn create_function(&self, request: &CreateFunctionRequest) -> Result<Function> {
        let url = self.base_url.join("functions")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Get a function
    ///
    /// Gets a function from within a parent catalog and schema. For the fetch to succeed,
    /// the caller must be a metastore admin, the owner of the function, or have SELECT on
    /// the function.
    pub async fn get_function(&self, request: &GetFunctionRequest) -> Result<Function> {
        let formatted_path = format!("functions/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Update a function
    ///
    /// Updates the function that matches the supplied name. Only the owner of the function
    /// can be updated.
    pub async fn update_function(&self, request: &UpdateFunctionRequest) -> Result<Function> {
        let formatted_path = format!("functions/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Delete a function
    ///
    /// Deletes the function that matches the supplied name. For the deletion to succeed,
    /// the caller must be the owner of the function.
    pub async fn delete_function(&self, request: &DeleteFunctionRequest) -> Result<()> {
        let formatted_path = format!("functions/{}", request.name);
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
