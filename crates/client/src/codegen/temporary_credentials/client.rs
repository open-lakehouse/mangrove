// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_common::models::temporary_credentials::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct TemporaryCredentialClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl TemporaryCredentialClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// Generate a new set of credentials for a table.
    pub async fn generate_temporary_table_credentials(
        &self,
        request: &GenerateTemporaryTableCredentialsRequest,
    ) -> Result<TemporaryCredential> {
        let url = self.base_url.join("temporary-table-credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Generate a new set of credentials for a path.
    pub async fn generate_temporary_path_credentials(
        &self,
        request: &GenerateTemporaryPathCredentialsRequest,
    ) -> Result<TemporaryCredential> {
        let url = self.base_url.join("temporary-path-credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Generate a new set of credentials for a volume.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true
    /// (default false). The caller must have the `EXTERNAL_USE_SCHEMA`
    /// privilege on the parent schema (granted by a catalog owner).
    pub async fn generate_temporary_volume_credentials(
        &self,
        request: &GenerateTemporaryVolumeCredentialsRequest,
    ) -> Result<TemporaryCredential> {
        let url = self.base_url.join("temporary-volume-credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Generate a new set of credentials for a model version.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true
    /// (default false). The caller must have the `EXTERNAL_USE_SCHEMA`
    /// privilege on the parent schema (granted by a catalog owner).
    pub async fn generate_temporary_model_version_credentials(
        &self,
        request: &GenerateTemporaryModelVersionCredentialsRequest,
    ) -> Result<TemporaryCredential> {
        let url = self.base_url.join("temporary-model-version-credentials")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
