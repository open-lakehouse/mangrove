// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::PolicyHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::policies::v1::*;
pub async fn list_policies<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListPoliciesRequest,
) -> Result<::axum::Json<ListPoliciesResponse>>
where
    T: PolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_policies(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_policy<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreatePolicyRequest,
) -> Result<::axum::Json<PolicyInfo>>
where
    T: PolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_policy(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_policy<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetPolicyRequest,
) -> Result<::axum::Json<PolicyInfo>>
where
    T: PolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_policy(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_policy<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdatePolicyRequest,
) -> Result<::axum::Json<PolicyInfo>>
where
    T: PolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_policy(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_policy<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeletePolicyRequest,
) -> Result<()>
where
    T: PolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_policy(request, context).await?;
    Ok(())
}
