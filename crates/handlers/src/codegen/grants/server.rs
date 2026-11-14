// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]

use super::handler::GrantHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::grants::v1::*;

pub async fn get_grants<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetGrantsRequest,
) -> Result<::axum::Json<GetGrantsResponse>>
where
    T: GrantHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_grants(request, context).await?;
    Ok(axum::Json(result))
}

pub async fn update_grants<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateGrantsRequest,
) -> Result<::axum::Json<UpdateGrantsResponse>>
where
    T: GrantHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_grants(request, context).await?;
    Ok(axum::Json(result))
}
