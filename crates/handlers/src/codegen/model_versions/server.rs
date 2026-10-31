// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::ModelVersionHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::model_versions::v1::*;
pub async fn list_model_versions<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListModelVersionsRequest,
) -> Result<::axum::Json<ListModelVersionsResponse>>
where
    T: ModelVersionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_model_versions(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_model_version<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateModelVersionRequest,
) -> Result<::axum::Json<ModelVersion>>
where
    T: ModelVersionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_model_version(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_model_version<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetModelVersionRequest,
) -> Result<::axum::Json<ModelVersion>>
where
    T: ModelVersionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_model_version(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_model_version<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateModelVersionRequest,
) -> Result<::axum::Json<ModelVersion>>
where
    T: ModelVersionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_model_version(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_model_version<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteModelVersionRequest,
) -> Result<()>
where
    T: ModelVersionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_model_version(request, context).await?;
    Ok(())
}
pub async fn finalize_model_version<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: FinalizeModelVersionRequest,
) -> Result<::axum::Json<ModelVersion>>
where
    T: ModelVersionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.finalize_model_version(request, context).await?;
    Ok(axum::Json(result))
}
