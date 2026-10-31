// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::ShareHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::shares::v1::*;
pub async fn list_shares<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSharesRequest,
) -> Result<::axum::Json<ListSharesResponse>>
where
    T: ShareHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_shares(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_share<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateShareRequest,
) -> Result<::axum::Json<Share>>
where
    T: ShareHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_share<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetShareRequest,
) -> Result<::axum::Json<Share>>
where
    T: ShareHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_share<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateShareRequest,
) -> Result<::axum::Json<Share>>
where
    T: ShareHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_share<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteShareRequest,
) -> Result<()>
where
    T: ShareHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_share(request, context).await?;
    Ok(())
}
pub async fn get_permissions<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetPermissionsRequest,
) -> Result<::axum::Json<GetPermissionsResponse>>
where
    T: ShareHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_permissions(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_permissions<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdatePermissionsRequest,
) -> Result<::axum::Json<UpdatePermissionsResponse>>
where
    T: ShareHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_permissions(request, context).await?;
    Ok(axum::Json(result))
}
