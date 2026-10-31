// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::ProviderHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::providers::v1::*;
pub async fn list_providers<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListProvidersRequest,
) -> Result<::axum::Json<ListProvidersResponse>>
where
    T: ProviderHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_providers(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_provider<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateProviderRequest,
) -> Result<::axum::Json<Provider>>
where
    T: ProviderHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_provider(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_provider<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetProviderRequest,
) -> Result<::axum::Json<Provider>>
where
    T: ProviderHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_provider(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_provider<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateProviderRequest,
) -> Result<::axum::Json<Provider>>
where
    T: ProviderHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_provider(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_provider<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteProviderRequest,
) -> Result<()>
where
    T: ProviderHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_provider(request, context).await?;
    Ok(())
}
