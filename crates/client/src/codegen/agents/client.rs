// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_common::models::agents::v0alpha1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct AgentServiceClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl AgentServiceClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// Lists agents.
    pub async fn list_agents(&self, request: &ListAgentsRequest) -> Result<ListAgentsResponse> {
        let mut url = self.base_url.join("agents")?;
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
    pub async fn create_agent(&self, request: &CreateAgentRequest) -> Result<Agent> {
        let url = self.base_url.join("agents")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn get_agent(&self, request: &GetAgentRequest) -> Result<Agent> {
        let formatted_path = format!("agents/{}", request.name);
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
    pub async fn update_agent(&self, request: &UpdateAgentRequest) -> Result<Agent> {
        let formatted_path = format!("agents/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    pub async fn delete_agent(&self, request: &DeleteAgentRequest) -> Result<()> {
        let formatted_path = format!("agents/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
}
