// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::ExternalLocationHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::external_locations::v1::*;
pub async fn list_external_locations<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListExternalLocationsRequest,
) -> Result<::axum::Json<ListExternalLocationsResponse>>
where
    T: ExternalLocationHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_external_locations(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_external_location<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateExternalLocationRequest,
) -> Result<::axum::Json<ExternalLocation>>
where
    T: ExternalLocationHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_external_location(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_external_location<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetExternalLocationRequest,
) -> Result<::axum::Json<ExternalLocation>>
where
    T: ExternalLocationHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_external_location(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_external_location<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateExternalLocationRequest,
) -> Result<::axum::Json<ExternalLocation>>
where
    T: ExternalLocationHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_external_location(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_external_location<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteExternalLocationRequest,
) -> Result<()>
where
    T: ExternalLocationHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_external_location(request, context).await?;
    Ok(())
}
