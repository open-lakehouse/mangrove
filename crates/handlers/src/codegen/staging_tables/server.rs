// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::StagingTableHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::staging_tables::v1::*;
pub async fn create_staging_table<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateStagingTableRequest,
) -> Result<::axum::Json<StagingTable>>
where
    T: StagingTableHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_staging_table(request, context).await?;
    Ok(axum::Json(result))
}
