// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::SharingHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
pub async fn list_shares<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSharesRequest,
) -> Result<::axum::Json<ListSharesResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_shares(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_share<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetShareRequest,
) -> Result<::axum::Json<Share>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_share(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_schemas<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSchemasRequest,
) -> Result<::axum::Json<ListSchemasResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_schemas(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_tables<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListTablesRequest,
) -> Result<::axum::Json<ListTablesResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_tables(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_all_tables<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListAllTablesRequest,
) -> Result<::axum::Json<ListAllTablesResponse>>
where
    T: SharingHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_all_tables(request, context).await?;
    Ok(axum::Json(result))
}
