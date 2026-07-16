// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::SharingSkillHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
pub async fn list_skills<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSkillsRequest,
) -> Result<::axum::Json<ListSkillsResponse>>
where
    T: SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_skills(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_all_skills<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListAllSkillsRequest,
) -> Result<::axum::Json<ListAllSkillsResponse>>
where
    T: SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_all_skills(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_skill<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetSkillRequest,
) -> Result<::axum::Json<SharingSkill>>
where
    T: SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_skill(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn generate_temporary_skill_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GenerateTemporarySkillCredentialsRequest,
) -> Result<::axum::Json<SharingTemporaryCredentials>>
where
    T: SharingSkillHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler
        .generate_temporary_skill_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
