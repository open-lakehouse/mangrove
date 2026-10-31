// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::VolumeHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::volumes::v1::*;
pub async fn list_volumes<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListVolumesRequest,
) -> Result<::axum::Json<ListVolumesResponse>>
where
    T: VolumeHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_volumes(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_volume<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateVolumeRequest,
) -> Result<::axum::Json<Volume>>
where
    T: VolumeHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_volume(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_volume<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetVolumeRequest,
) -> Result<::axum::Json<Volume>>
where
    T: VolumeHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_volume(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_volume<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateVolumeRequest,
) -> Result<::axum::Json<Volume>>
where
    T: VolumeHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_volume(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_volume<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteVolumeRequest,
) -> Result<()>
where
    T: VolumeHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_volume(request, context).await?;
    Ok(())
}
