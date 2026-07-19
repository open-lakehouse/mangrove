// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
#[cfg(not(target_arch = "wasm32"))]
use ::olai_http::CloudClient as Transport;
#[cfg(target_arch = "wasm32")]
use ::olai_http_wasm::WasmClient as Transport;
use unitycatalog_common::models::tables::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct TableServiceClient {
    pub(crate) client: Transport,
    pub(crate) base_url: Url,
}
impl TableServiceClient {
    /// Create a new client instance
    pub fn new(client: Transport, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// Gets an array of summaries for tables for a schema and catalog within the metastore. The table summaries returned are either:
    /// - summaries for tables (within the current metastore and parent catalog and schema), when the user is a metastore admin, or:
    /// - summaries for tables and schemas (within the current metastore and parent catalog) for which the user has ownership or the
    /// SELECT privilege on the table and ownership or USE_SCHEMA privilege on the schema, provided that the user also has ownership
    /// or the USE_CATALOG privilege on the parent catalog.
    ///
    /// There is no guarantee of a specific ordering of the elements in the array.
    pub async fn list_table_summaries(
        &self,
        request: &ListTableSummariesRequest,
    ) -> Result<ListTableSummariesResponse> {
        let mut url = self.base_url.join("table-summaries")?;
        url.query_pairs_mut()
            .append_pair("catalog_name", &request.catalog_name);
        if let Some(ref value) = request.schema_name_pattern {
            url.query_pairs_mut()
                .append_pair("schema_name_pattern", &value.to_string());
        }
        if let Some(ref value) = request.table_name_pattern {
            url.query_pairs_mut()
                .append_pair("table_name_pattern", &value.to_string());
        }
        if let Some(ref value) = request.max_results {
            url.query_pairs_mut()
                .append_pair("max_results", &value.to_string());
        }
        if let Some(ref value) = request.page_token {
            url.query_pairs_mut()
                .append_pair("page_token", &value.to_string());
        }
        if let Some(ref value) = request.include_manifest_capabilities {
            url.query_pairs_mut()
                .append_pair("include_manifest_capabilities", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Gets an array of all tables for the current metastore under the parent catalog and schema.
    ///
    /// The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table.
    /// For the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the
    /// parent catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a
    /// specific ordering of the elements in the array.
    pub async fn list_tables(&self, request: &ListTablesRequest) -> Result<ListTablesResponse> {
        let mut url = self.base_url.join("tables")?;
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
        if let Some(ref value) = request.include_delta_metadata {
            url.query_pairs_mut()
                .append_pair("include_delta_metadata", &value.to_string());
        }
        if let Some(ref value) = request.omit_columns {
            url.query_pairs_mut()
                .append_pair("omit_columns", &value.to_string());
        }
        if let Some(ref value) = request.omit_properties {
            url.query_pairs_mut()
                .append_pair("omit_properties", &value.to_string());
        }
        if let Some(ref value) = request.omit_username {
            url.query_pairs_mut()
                .append_pair("omit_username", &value.to_string());
        }
        if let Some(ref value) = request.include_browse {
            url.query_pairs_mut()
                .append_pair("include_browse", &value.to_string());
        }
        if let Some(ref value) = request.include_manifest_capabilities {
            url.query_pairs_mut()
                .append_pair("include_manifest_capabilities", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Create a table
    pub async fn create_table(&self, request: &CreateTableRequest) -> Result<Table> {
        let url = self.base_url.join("tables")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Get a table
    pub async fn get_table(&self, request: &GetTableRequest) -> Result<Table> {
        let formatted_path = format!("tables/{}", request.full_name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.include_delta_metadata {
            url.query_pairs_mut()
                .append_pair("include_delta_metadata", &value.to_string());
        }
        if let Some(ref value) = request.include_browse {
            url.query_pairs_mut()
                .append_pair("include_browse", &value.to_string());
        }
        if let Some(ref value) = request.include_manifest_capabilities {
            url.query_pairs_mut()
                .append_pair("include_manifest_capabilities", &value.to_string());
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Get boolean reflecting if table exists
    pub async fn get_table_exists(
        &self,
        request: &GetTableExistsRequest,
    ) -> Result<GetTableExistsResponse> {
        let formatted_path = format!("tables/{}/exists", request.full_name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Delete a table
    pub async fn delete_table(&self, request: &DeleteTableRequest) -> Result<()> {
        let formatted_path = format!("tables/{}", request.full_name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
}
