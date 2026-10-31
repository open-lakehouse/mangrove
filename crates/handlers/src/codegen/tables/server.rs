// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::TableHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::tables::v1::*;
pub async fn list_table_summaries<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListTableSummariesRequest,
) -> Result<::axum::Json<ListTableSummariesResponse>>
where
    T: TableHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_table_summaries(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn list_tables<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListTablesRequest,
) -> Result<::axum::Json<ListTablesResponse>>
where
    T: TableHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_tables(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateTableRequest,
) -> Result<::axum::Json<Table>>
where
    T: TableHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_table(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetTableRequest,
) -> Result<::axum::Json<Table>>
where
    T: TableHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_table(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_table_exists<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetTableExistsRequest,
) -> Result<::axum::Json<GetTableExistsResponse>>
where
    T: TableHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_table_exists(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteTableRequest,
) -> Result<()>
where
    T: TableHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_table(request, context).await?;
    Ok(())
}
