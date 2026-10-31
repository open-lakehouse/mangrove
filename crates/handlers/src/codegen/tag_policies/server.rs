// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::TagPolicyHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::tags::v1::*;
pub async fn list_tag_policies<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListTagPoliciesRequest,
) -> Result<::axum::Json<ListTagPoliciesResponse>>
where
    T: TagPolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_tag_policies(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_tag_policy<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateTagPolicyRequest,
) -> Result<::axum::Json<TagPolicy>>
where
    T: TagPolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_tag_policy(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_tag_policy<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetTagPolicyRequest,
) -> Result<::axum::Json<TagPolicy>>
where
    T: TagPolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_tag_policy(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_tag_policy<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateTagPolicyRequest,
) -> Result<::axum::Json<TagPolicy>>
where
    T: TagPolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_tag_policy(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_tag_policy<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteTagPolicyRequest,
) -> Result<()>
where
    T: TagPolicyHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_tag_policy(request, context).await?;
    Ok(())
}
