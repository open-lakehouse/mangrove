// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::CredentialHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::credentials::v1::*;
pub async fn list_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: ListCredentialsRequest,
) -> Result<::axum::Json<ListCredentialsResponse>>
where
    T: CredentialHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.list_credentials(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn create_credential<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: CreateCredentialRequest,
) -> Result<::axum::Json<Credential>>
where
    T: CredentialHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.create_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn get_credential<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GetCredentialRequest,
) -> Result<::axum::Json<Credential>>
where
    T: CredentialHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.get_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn update_credential<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: UpdateCredentialRequest,
) -> Result<::axum::Json<Credential>>
where
    T: CredentialHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler.update_credential(request, context).await?;
    Ok(axum::Json(result))
}
pub async fn delete_credential<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: DeleteCredentialRequest,
) -> Result<()>
where
    T: CredentialHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    handler.delete_credential(request, context).await?;
    Ok(())
}
