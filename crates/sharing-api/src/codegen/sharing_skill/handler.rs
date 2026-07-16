// @generated — do not edit by hand.
//! Handler trait for [`SharingSkillHandler`].
//!
//! Implement this trait to provide a custom backend for this service, then mount the
//! generated handler functions (in the sibling `server` module) onto an `axum::Router`
//! with your implementation as state.
//!
//! # Composability
//!
//! A single struct can implement multiple handler traits to serve multiple
//! services. Use [`axum::Router::merge`] to compose per-service routers together.
//!
//! Open Sharing agent-skill APIs: discovery and credential vending for shared
//! storage-backed agent skills.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
#[async_trait]
pub trait SharingSkillHandler<Cx = crate::DefaultContext>: Send + Sync + 'static {
    /// List the agent skills in a given share's schema.
    async fn list_skills(
        &self,
        request: ListSkillsRequest,
        context: Cx,
    ) -> Result<ListSkillsResponse>;
    /// List all the agent skills under a share, across all schemas.
    async fn list_all_skills(
        &self,
        request: ListAllSkillsRequest,
        context: Cx,
    ) -> Result<ListAllSkillsResponse>;
    /// Get the metadata for a single shared agent skill.
    async fn get_skill(&self, request: GetSkillRequest, context: Cx) -> Result<SharingSkill>;
    /// Generate temporary credentials scoped to a shared skill's storage
    /// location, for direct file access via the cloud storage API.
    async fn generate_temporary_skill_credentials(
        &self,
        request: GenerateTemporarySkillCredentialsRequest,
        context: Cx,
    ) -> Result<SharingTemporaryCredentials>;
}
