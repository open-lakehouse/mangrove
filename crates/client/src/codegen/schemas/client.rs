// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
use olai_http::CloudClient;
use unitycatalog_common::models::schemas::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct SchemaServiceClient {
    pub(crate) client: CloudClient,
    pub(crate) base_url: Url,
}
impl SchemaServiceClient {
    /// Create a new client instance
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// Gets an array of schemas for a catalog in the metastore. If the caller is the metastore
    /// admin or the owner of the parent catalog, all schemas for the catalog will be retrieved.
    /// Otherwise, only schemas owned by the caller (or for which the caller has the USE_SCHEMA privilege)
    /// will be retrieved. There is no guarantee of a specific ordering of the elements in the array.
    pub async fn list_schemas(&self, request: &ListSchemasRequest) -> Result<ListSchemasResponse> {
        let mut url = self.base_url.join("schemas")?;
        url.query_pairs_mut()
            .append_pair("catalog_name", &request.catalog_name);
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
    /// Creates a new schema for catalog in the Metatastore. The caller must be a metastore admin,
    /// or have the CREATE_SCHEMA privilege in the parent catalog.
    pub async fn create_schema(&self, request: &CreateSchemaRequest) -> Result<Schema> {
        let url = self.base_url.join("schemas")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Gets the specified schema within the metastore.
    /// The caller must be a metastore admin, the owner of the schema,
    /// or a user that has the USE_SCHEMA privilege on the schema.
    pub async fn get_schema(&self, request: &GetSchemaRequest) -> Result<Schema> {
        let formatted_path = format!("schemas/{}", request.full_name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
    /// If the caller is a metastore admin, only the owner field can be changed in the update.
    /// If the name field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA
    /// privilege on the parent catalog.
    pub async fn update_schema(&self, request: &UpdateSchemaRequest) -> Result<Schema> {
        let formatted_path = format!("schemas/{}", request.full_name);
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Deletes the specified schema from the parent catalog. The caller must be the owner
    /// of the schema or an owner of the parent catalog.
    pub async fn delete_schema(&self, request: &DeleteSchemaRequest) -> Result<()> {
        let formatted_path = format!("schemas/{}", request.full_name);
        let mut url = self.base_url.join(&formatted_path)?;
        if let Some(ref value) = request.force {
            url.query_pairs_mut()
                .append_pair("force", &value.to_string());
        }
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
}
