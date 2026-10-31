// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::AgentHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::agents::v0alpha1::*;
pub async fn list_agents<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListAgentsRequest,
) -> Result<::axum::Json<ListAgentsResponse>>
where
    T: AgentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_agents(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_agent<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateAgentRequest,
) -> Result<::axum::Json<Agent>>
where
    T: AgentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_agent(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_agent<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetAgentRequest,
) -> Result<::axum::Json<Agent>>
where
    T: AgentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_agent(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_agent<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateAgentRequest,
) -> Result<::axum::Json<Agent>>
where
    T: AgentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_agent(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_agent<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteAgentRequest,
) -> Result<()>
where
    T: AgentHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_agent(request, context).await?;
    Ok(())
}
