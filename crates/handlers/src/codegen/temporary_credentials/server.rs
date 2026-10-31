// @generated — do not edit by hand.
#![allow(unused_mut, clippy::too_many_arguments)]
use super::handler::TemporaryCredentialHandler;
use crate::Result;
use axum::extract::State;
use unitycatalog_common::models::temporary_credentials::v1::*;
pub async fn generate_temporary_table_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GenerateTemporaryTableCredentialsRequest,
) -> Result<::axum::Json<TemporaryCredential>>
where
    T: TemporaryCredentialHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler
        .generate_temporary_table_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
pub async fn generate_temporary_path_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GenerateTemporaryPathCredentialsRequest,
) -> Result<::axum::Json<TemporaryCredential>>
where
    T: TemporaryCredentialHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler
        .generate_temporary_path_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
pub async fn generate_temporary_volume_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GenerateTemporaryVolumeCredentialsRequest,
) -> Result<::axum::Json<TemporaryCredential>>
where
    T: TemporaryCredentialHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler
        .generate_temporary_volume_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
pub async fn generate_temporary_model_version_credentials<T, Cx>(
    State(handler): State<T>,
    context: Cx,
    request: GenerateTemporaryModelVersionCredentialsRequest,
) -> Result<::axum::Json<TemporaryCredential>>
where
    T: TemporaryCredentialHandler<Cx> + Clone + Send + Sync + 'static,
    Cx: axum::extract::FromRequestParts<T> + Send,
{
    let result = handler
        .generate_temporary_model_version_credentials(request, context)
        .await?;
    Ok(axum::Json(result))
}
