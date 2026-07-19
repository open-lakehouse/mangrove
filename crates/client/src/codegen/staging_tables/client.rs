// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
#[cfg(not(target_arch = "wasm32"))]
use ::olai_http::CloudClient as Transport;
#[cfg(target_arch = "wasm32")]
use ::olai_http_wasm::WasmClient as Transport;
use unitycatalog_common::models::staging_tables::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct StagingTableServiceClient {
    pub(crate) client: Transport,
    pub(crate) base_url: Url,
}
impl StagingTableServiceClient {
    /// Create a new client instance
    pub fn new(client: Transport, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// Creates a new staging table, allocating an immutable table id and a storage
    /// location under the parent schema/catalog managed storage root. The caller
    /// must have the CREATE privilege on the parent schema.
    pub async fn create_staging_table(
        &self,
        request: &CreateStagingTableRequest,
    ) -> Result<StagingTable> {
        let url = self.base_url.join("staging-tables")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
}
