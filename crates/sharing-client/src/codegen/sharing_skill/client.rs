// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct SharingSkillClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl SharingSkillClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List the agent skills in a given share's schema.
    pub async fn list_skills(&self, request: &ListSkillsRequest) -> Result<ListSkillsResponse> {
        let formatted_path = format!("shares/{}/schemas/{}/skills", request.share, request.schema);
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
    /// List all the agent skills under a share, across all schemas.
    pub async fn list_all_skills(
        &self,
        request: &ListAllSkillsRequest,
    ) -> Result<ListAllSkillsResponse> {
        let formatted_path = format!("shares/{}/all-skills", request.share);
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
    /// Get the metadata for a single shared agent skill.
    pub async fn get_skill(&self, request: &GetSkillRequest) -> Result<SharingSkill> {
        let formatted_path = format!(
            "shares/{}/schemas/{}/skills/{}",
            request.share, request.schema, request.name
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Generate temporary credentials scoped to a shared skill's storage
    /// location, for direct file access via the cloud storage API.
    pub async fn generate_temporary_skill_credentials(
        &self,
        request: &GenerateTemporarySkillCredentialsRequest,
    ) -> Result<SharingTemporaryCredentials> {
        let formatted_path = format!(
            "shares/{}/schemas/{}/skills/{}/temporary-skill-credentials",
            request.share, request.schema, request.name
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
