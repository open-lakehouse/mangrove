// @generated — do not edit by hand.
//! Handler trait for [`AgentSkillHandler`].
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
//! Service for managing agent skills in Unity Catalog.
//!
//! Agent skills are storage-backed directories (SKILL.md + optional resources)
//! within a schema. This is an early-stage (v0alpha1) surface aligned with the
//! Open Sharing agent-skill community proposal.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
#[async_trait]
pub trait AgentSkillHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// Lists agent skills.
    async fn list_agent_skills(
        &self,
        request: ListAgentSkillsRequest,
        context: Cx,
    ) -> Result<ListAgentSkillsResponse>;
    async fn create_agent_skill(
        &self,
        request: CreateAgentSkillRequest,
        context: Cx,
    ) -> Result<AgentSkill>;
    async fn get_agent_skill(
        &self,
        request: GetAgentSkillRequest,
        context: Cx,
    ) -> Result<AgentSkill>;
    async fn update_agent_skill(
        &self,
        request: UpdateAgentSkillRequest,
        context: Cx,
    ) -> Result<AgentSkill>;
    async fn delete_agent_skill(&self, request: DeleteAgentSkillRequest, context: Cx)
    -> Result<()>;
}
