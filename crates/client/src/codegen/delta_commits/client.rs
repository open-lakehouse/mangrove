// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_common::models::delta_commits::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct DeltaCommitClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl DeltaCommitClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// Ratify a staged commit at the requested version (first-writer-wins), and/or
    /// notify the catalog that commits have been backfilled to the Delta log.
    pub async fn commit(&self, request: &CommitRequest) -> Result<()> {
        let url = self.base_url.join("delta/preview/commits")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
    /// Return ratified-but-unpublished commits for a table, plus the latest
    /// version the catalog tracks.
    pub async fn get_commits(&self, request: &GetCommitsRequest) -> Result<GetCommitsResponse> {
        let mut url = self.base_url.join("delta/preview/commits")?;
        url.query_pairs_mut()
            .append_pair("table_id", &request.table_id);
        url.query_pairs_mut()
            .append_pair("table_uri", &request.table_uri);
        url.query_pairs_mut()
            .append_pair("start_version", &request.start_version.to_string());
        if let Some(ref value) = request.end_version {
            url.query_pairs_mut()
                .append_pair("end_version", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
