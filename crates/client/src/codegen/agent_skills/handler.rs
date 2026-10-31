// @generated — do not edit by hand.
#![allow(unused_imports)]

use super::client::AgentSkillServiceClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;

#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::agent_skills::AgentSkillHandler<Cx>
    for AgentSkillServiceClient
{
    async fn list_agent_skills(
        &self,
        request: ListAgentSkillsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListAgentSkillsResponse> {
        AgentSkillServiceClient::list_agent_skills(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn create_agent_skill(
        &self,
        request: CreateAgentSkillRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<AgentSkill> {
        AgentSkillServiceClient::create_agent_skill(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn get_agent_skill(
        &self,
        request: GetAgentSkillRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<AgentSkill> {
        AgentSkillServiceClient::get_agent_skill(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn update_agent_skill(
        &self,
        request: UpdateAgentSkillRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<AgentSkill> {
        AgentSkillServiceClient::update_agent_skill(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn delete_agent_skill(
        &self,
        request: DeleteAgentSkillRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        AgentSkillServiceClient::delete_agent_skill(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
