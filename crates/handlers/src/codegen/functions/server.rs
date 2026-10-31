// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::FunctionHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::functions::v1::*;
pub async fn list_functions<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListFunctionsRequest,
) -> Result<::axum::Json<ListFunctionsResponse>>
where
    T: FunctionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_functions(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_function<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateFunctionRequest,
) -> Result<::axum::Json<Function>>
where
    T: FunctionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_function(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_function<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetFunctionRequest,
) -> Result<::axum::Json<Function>>
where
    T: FunctionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_function(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_function<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateFunctionRequest,
) -> Result<::axum::Json<Function>>
where
    T: FunctionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_function(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_function<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteFunctionRequest,
) -> Result<()>
where
    T: FunctionHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_function(request, context).await?;
    Ok(())
}
