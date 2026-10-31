// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::CatalogHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::catalogs::v1::*;
pub async fn list_catalogs<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListCatalogsRequest,
) -> Result<::axum::Json<ListCatalogsResponse>>
where
    T: CatalogHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_catalogs(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_catalog<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateCatalogRequest,
) -> Result<::axum::Json<Catalog>>
where
    T: CatalogHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_catalog<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetCatalogRequest,
) -> Result<::axum::Json<Catalog>>
where
    T: CatalogHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_catalog<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateCatalogRequest,
) -> Result<::axum::Json<Catalog>>
where
    T: CatalogHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_catalog(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_catalog<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteCatalogRequest,
) -> Result<()>
where
    T: CatalogHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_catalog(request, context).await?;
    Ok(())
}
