// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::RegisteredModelHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::registered_models::v1::*;
pub async fn list_registered_models<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListRegisteredModelsRequest,
) -> Result<::axum::Json<ListRegisteredModelsResponse>>
where
    T: RegisteredModelHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_registered_models(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_registered_model<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateRegisteredModelRequest,
) -> Result<::axum::Json<RegisteredModel>>
where
    T: RegisteredModelHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_registered_model(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_registered_model<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetRegisteredModelRequest,
) -> Result<::axum::Json<RegisteredModel>>
where
    T: RegisteredModelHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_registered_model(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_registered_model<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateRegisteredModelRequest,
) -> Result<::axum::Json<RegisteredModel>>
where
    T: RegisteredModelHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_registered_model(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_registered_model<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteRegisteredModelRequest,
) -> Result<()>
where
    T: RegisteredModelHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_registered_model(request, context).await?;
    Ok(())
}
