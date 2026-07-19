// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
#[cfg(not(target_arch = "wasm32"))]
use ::olai_http::CloudClient as Transport;
#[cfg(target_arch = "wasm32")]
use ::olai_http_wasm::WasmClient as Transport;
use unitycatalog_common::models::policies::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct PolicyServiceClient {
    pub(crate) client: Transport,
    pub(crate) base_url: Url,
}
impl PolicyServiceClient {
    /// Create a new client instance
    pub fn new(client: Transport, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List policies
    ///
    /// Gets an array of policies defined on the specified securable. There is no guarantee
    /// of a specific ordering of the elements in the array.
    pub async fn list_policies(
        &self,
        request: &ListPoliciesRequest,
    ) -> Result<ListPoliciesResponse> {
        let formatted_path = format!(
            "policies/{}/{}",
            request.on_securable_type, request.on_securable_fullname
        );
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.include_inherited {
            url.query_pairs_mut()
                .append_pair("include_inherited", &value.to_string());
        }
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
    /// Create a new policy
    ///
    /// Creates a new row-filter or column-mask policy on the specified securable.
    pub async fn create_policy(&self, request: &CreatePolicyRequest) -> Result<PolicyInfo> {
        let formatted_path = format!(
            "policies/{}/{}",
            request.on_securable_type, request.on_securable_fullname
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Get a policy
    ///
    /// Gets the policy that matches the supplied name, defined on the specified securable.
    pub async fn get_policy(&self, request: &GetPolicyRequest) -> Result<PolicyInfo> {
        let formatted_path = format!(
            "policies/{}/{}/{}",
            request.on_securable_type, request.on_securable_fullname, request.name
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Update a policy
    ///
    /// Updates the policy that matches the supplied name, defined on the specified securable.
    pub async fn update_policy(&self, request: &UpdatePolicyRequest) -> Result<PolicyInfo> {
        let formatted_path = format!(
            "policies/{}/{}/{}",
            request.on_securable_type, request.on_securable_fullname, request.name
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Delete a policy
    ///
    /// Deletes the policy that matches the supplied name, defined on the specified securable.
    pub async fn delete_policy(&self, request: &DeletePolicyRequest) -> Result<()> {
        let formatted_path = format!(
            "policies/{}/{}/{}",
            request.on_securable_type, request.on_securable_fullname, request.name
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
}
