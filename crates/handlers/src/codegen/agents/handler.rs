// @generated — do not edit by hand.
//! Handler trait for [`AgentHandler`].
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
//! Service for managing agents in Unity Catalog.
//!
//! Agents are remote invocable services (endpoint + invocation protocol) within a
//! schema. This is an early-stage (v0alpha1) surface aligned with the Open
//! Sharing agent community proposal.
use crate::Result;
use async_trait::async_trait;
use unitycatalog_common::models::agents::v0alpha1::*;

#[async_trait]
pub trait AgentHandler<Cx = crate::RequestContext>: Send + Sync + 'static {
    /// Lists agents.
    async fn list_agents(
        &self,
        request: ListAgentsRequest,
        context: Cx,
    ) -> Result<ListAgentsResponse>;
    async fn create_agent(&self, request: CreateAgentRequest, context: Cx) -> Result<Agent>;
    async fn get_agent(&self, request: GetAgentRequest, context: Cx) -> Result<Agent>;
    async fn update_agent(&self, request: UpdateAgentRequest, context: Cx) -> Result<Agent>;
    async fn delete_agent(&self, request: DeleteAgentRequest, context: Cx) -> Result<()>;
}
