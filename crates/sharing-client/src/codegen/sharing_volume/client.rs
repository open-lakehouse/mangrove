// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct SharingVolumeClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl SharingVolumeClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List the volumes in a given share's schema.
    pub async fn list_volumes(&self, request: &ListVolumesRequest) -> Result<ListVolumesResponse> {
        let formatted_path = format!(
            "shares/{}/schemas/{}/volumes",
            request.share, request.schema
        );
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
    /// List all the volumes under a share, across all schemas.
    pub async fn list_all_volumes(
        &self,
        request: &ListAllVolumesRequest,
    ) -> Result<ListAllVolumesResponse> {
        let formatted_path = format!("shares/{}/all-volumes", request.share);
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
    /// Get the metadata for a single shared volume.
    pub async fn get_volume(&self, request: &GetVolumeRequest) -> Result<SharingVolume> {
        let formatted_path = format!(
            "shares/{}/schemas/{}/volumes/{}",
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
    /// Generate temporary credentials scoped to a shared volume's storage
    /// location, for direct file access via the cloud storage API.
    pub async fn generate_temporary_volume_credentials(
        &self,
        request: &GenerateTemporaryVolumeCredentialsRequest,
    ) -> Result<SharingTemporaryCredentials> {
        let formatted_path = format!(
            "shares/{}/schemas/{}/volumes/{}/temporary-volume-credentials",
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
