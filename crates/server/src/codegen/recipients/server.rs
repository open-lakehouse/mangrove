// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::RecipientHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::recipients::v1::*;
pub async fn list_recipients<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListRecipientsRequest,
) -> Result<::axum::Json<ListRecipientsResponse>>
where
    T: RecipientHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_recipients(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_recipient<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateRecipientRequest,
) -> Result<::axum::Json<Recipient>>
where
    T: RecipientHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_recipient<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetRecipientRequest,
) -> Result<::axum::Json<Recipient>>
where
    T: RecipientHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_recipient<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateRecipientRequest,
) -> Result<::axum::Json<Recipient>>
where
    T: RecipientHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_recipient(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_recipient<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteRecipientRequest,
) -> Result<()>
where
    T: RecipientHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_recipient(request, context).await?;
    Ok(())
}
pub async fn rotate_recipient_token<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: RotateRecipientTokenRequest,
) -> Result<::axum::Json<Recipient>>
where
    T: RecipientHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.rotate_recipient_token(request, context).await?;
    Ok(axum::Json(result))
}
