// @generated — do not edit by hand.
#![allow(unused_imports)]

use super::client::AgentServiceClient;
use unitycatalog_common::models::agents::v0alpha1::*;

#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::agents::AgentHandler<Cx> for AgentServiceClient {
    async fn list_agents(
        &self,
        request: ListAgentsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListAgentsResponse> {
        AgentServiceClient::list_agents(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn create_agent(
        &self,
        request: CreateAgentRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Agent> {
        AgentServiceClient::create_agent(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn get_agent(
        &self,
        request: GetAgentRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Agent> {
        AgentServiceClient::get_agent(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn update_agent(
        &self,
        request: UpdateAgentRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Agent> {
        AgentServiceClient::update_agent(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn delete_agent(
        &self,
        request: DeleteAgentRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        AgentServiceClient::delete_agent(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
