// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
#[cfg(not(target_arch = "wasm32"))]
use ::olai_http::CloudClient as Transport;
#[cfg(target_arch = "wasm32")]
use ::olai_http_wasm::WasmClient as Transport;
use unitycatalog_common::models::tags::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct TagPolicyServiceClient {
    pub(crate) client: Transport,
    pub(crate) base_url: Url,
}
impl TagPolicyServiceClient {
    /// Create a new client instance
    pub fn new(client: Transport, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List tag policies
    ///
    /// Gets an array of tag policies. There is no guarantee of a specific ordering
    /// of the elements in the array.
    pub async fn list_tag_policies(
        &self,
        request: &ListTagPoliciesRequest,
    ) -> Result<ListTagPoliciesResponse> {
        let mut url = self.base_url.join("tag-policies")?;
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
    /// Create a new tag policy
    ///
    /// Creates a new governed tag definition.
    pub async fn create_tag_policy(&self, request: &CreateTagPolicyRequest) -> Result<TagPolicy> {
        let url = self.base_url.join("tag-policies")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Get a tag policy
    ///
    /// Gets the governed tag definition for the specified tag key.
    pub async fn get_tag_policy(&self, request: &GetTagPolicyRequest) -> Result<TagPolicy> {
        let formatted_path = format!("tag-policies/{}", request.tag_key);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Update a tag policy
    ///
    /// Updates the governed tag definition that matches the supplied tag key.
    pub async fn update_tag_policy(&self, request: &UpdateTagPolicyRequest) -> Result<TagPolicy> {
        let formatted_path = format!("tag-policies/{}", request.tag_key);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Delete a tag policy
    ///
    /// Deletes the governed tag definition that matches the supplied tag key.
    pub async fn delete_tag_policy(&self, request: &DeleteTagPolicyRequest) -> Result<()> {
        let formatted_path = format!("tag-policies/{}", request.tag_key);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
}
