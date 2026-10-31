// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::SchemaHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::schemas::v1::*;
pub async fn list_schemas<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListSchemasRequest,
) -> Result<::axum::Json<ListSchemasResponse>>
where
    T: SchemaHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_schemas(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_schema<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateSchemaRequest,
) -> Result<::axum::Json<Schema>>
where
    T: SchemaHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_schema<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetSchemaRequest,
) -> Result<::axum::Json<Schema>>
where
    T: SchemaHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_schema<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateSchemaRequest,
) -> Result<::axum::Json<Schema>>
where
    T: SchemaHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_schema(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_schema<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteSchemaRequest,
) -> Result<()>
where
    T: SchemaHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_schema(request, context).await?;
    Ok(())
}
