// @generated — do not edit by hand.
#![allow(unused_imports)]
use crate::Result;
#[cfg(not(target_arch = "wasm32"))]
use ::olai_http::CloudClient as Transport;
#[cfg(target_arch = "wasm32")]
use ::olai_http_wasm::WasmClient as Transport;
use unitycatalog_common::models::tags::v1::*;
use url::Url;
/// HTTP client for service operations
#[derive(Clone)]
pub struct EntityTagAssignmentClient {
    pub(crate) client: Transport,
    pub(crate) base_url: Url,
}
impl EntityTagAssignmentClient {
    /// Create a new client instance
    pub fn new(client: Transport, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// List entity tag assignments
    ///
    /// Gets the tag assignments for the specified entity.
    pub async fn list_entity_tag_assignments(
        &self,
        request: &ListEntityTagAssignmentsRequest,
    ) -> Result<ListEntityTagAssignmentsResponse> {
        let formatted_path = format!(
            "entity-tag-assignments/{}/{}/tags",
            request.entity_type, request.entity_name
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
    /// Create an entity tag assignment
    ///
    /// Assigns a tag to a Unity Catalog entity.
    pub async fn create_entity_tag_assignment(
        &self,
        request: &CreateEntityTagAssignmentRequest,
    ) -> Result<EntityTagAssignment> {
        let url = self.base_url.join("entity-tag-assignments")?;
        let response = self.client.post(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Get an entity tag assignment
    ///
    /// Gets the tag assignment for the specified entity and tag key.
    pub async fn get_entity_tag_assignment(
        &self,
        request: &GetEntityTagAssignmentRequest,
    ) -> Result<EntityTagAssignment> {
        let formatted_path = format!(
            "entity-tag-assignments/{}/{}/tags/{}",
            request.entity_type, request.entity_name, request.tag_key
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Update an entity tag assignment
    ///
    /// Updates the tag assignment for the specified entity and tag key.
    pub async fn update_entity_tag_assignment(
        &self,
        request: &UpdateEntityTagAssignmentRequest,
    ) -> Result<EntityTagAssignment> {
        let formatted_path = format!(
            "entity-tag-assignments/{}/{}/tags/{}",
            request.entity_type, request.entity_name, request.tag_key
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.patch(url).json(request).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        let result = response.bytes().await?;
        Ok(serde_json::from_slice(&result)?)
    }
    /// Delete an entity tag assignment
    ///
    /// Deletes the tag assignment for the specified entity and tag key.
    pub async fn delete_entity_tag_assignment(
        &self,
        request: &DeleteEntityTagAssignmentRequest,
    ) -> Result<()> {
        let formatted_path = format!(
            "entity-tag-assignments/{}/{}/tags/{}",
            request.entity_type, request.entity_name, request.tag_key
        );
        let url = self.base_url.join(&formatted_path)?;
        let response = self.client.delete(url).send().await?;
        if !response.status().is_success() {
            return Err(crate::error::parse_error_response(response).await);
        }
        Ok(())
    }
}
