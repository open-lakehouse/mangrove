// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_common::models::shares::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct ShareServiceClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl ShareServiceClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List shares.
    pub async fn list_shares(&self, request: &ListSharesRequest) -> Result<ListSharesResponse> {
        let mut url = self.base_url.join("shares")?;
        if let Some(ref value) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &value.to_string());
        }
        if let Some(ref value) = request.page_token {
            url.query_pairs_mut()
                .append_pair("page_token", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Create a new share.
    pub async fn create_share(&self, request: &CreateShareRequest) -> Result<Share> {
        let url = self.base_url.join("shares")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Get a share by name.
    pub async fn get_share(&self, request: &GetShareRequest) -> Result<Share> {
        let formatted_path = format!("shares/{}", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.include_shared_data {
            url.query_pairs_mut()
                .append_pair("include_shared_data", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Update a share.
    pub async fn update_share(&self, request: &UpdateShareRequest) -> Result<Share> {
        let formatted_path = format!("shares/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Deletes a share.
    pub async fn delete_share(&self, request: &DeleteShareRequest) -> Result<()> {
        let formatted_path = format!("shares/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
    /// Gets the permissions for a data share from the metastore.
    pub async fn get_permissions(
        &self,
        request: &GetPermissionsRequest,
    ) -> Result<GetPermissionsResponse> {
        let formatted_path = format!("shares/{}/permissions", request.name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &value.to_string());
        }
        if let Some(ref value) = request.page_token {
            url.query_pairs_mut()
                .append_pair("page_token", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Updates the permissions for a data share in the metastore.
    pub async fn update_permissions(
        &self,
        request: &UpdatePermissionsRequest,
    ) -> Result<UpdatePermissionsResponse> {
        let formatted_path = format!("shares/{}/permissions", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
