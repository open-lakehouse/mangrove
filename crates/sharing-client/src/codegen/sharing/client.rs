// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct SharingClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl SharingClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List shares accessible to a recipient.
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
    /// Get the metadata for a specific share.
    pub async fn get_share(&self, request: &GetShareRequest) -> Result<Share> {
        let formatted_path = format!("shares/{}", request.name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// List the schemas in a share.
    pub async fn list_schemas(&self, request: &ListSchemasRequest) -> Result<ListSchemasResponse> {
        let formatted_path = format!("shares/{}/schemas", request.share);
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
    /// List the tables in a given share's schema.
    pub async fn list_tables(&self, request: &ListTablesRequest) -> Result<ListTablesResponse> {
        let formatted_path = format!("shares/{}/schemas/{}/tables", request.share, request.name);
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
    /// List all the tables under a share.
    ///
    /// A convenience over per-schema listing: returns every table across all
    /// schemas in the share.
    pub async fn list_all_tables(
        &self,
        request: &ListAllTablesRequest,
    ) -> Result<ListAllTablesResponse> {
        let formatted_path = format!("shares/{}/all-tables", request.name);
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
}
