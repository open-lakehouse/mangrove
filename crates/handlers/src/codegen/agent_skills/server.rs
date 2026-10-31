// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::AgentSkillHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
pub async fn list_agent_skills<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListAgentSkillsRequest,
) -> Result<::axum::Json<ListAgentSkillsResponse>>
where
    T: AgentSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_agent_skills(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_agent_skill<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateAgentSkillRequest,
) -> Result<::axum::Json<AgentSkill>>
where
    T: AgentSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_agent_skill(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_agent_skill<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetAgentSkillRequest,
) -> Result<::axum::Json<AgentSkill>>
where
    T: AgentSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_agent_skill(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_agent_skill<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateAgentSkillRequest,
) -> Result<::axum::Json<AgentSkill>>
where
    T: AgentSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_agent_skill(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_agent_skill<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteAgentSkillRequest,
) -> Result<()>
where
    T: AgentSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_agent_skill(request, context).await?;
    Ok(())
}
