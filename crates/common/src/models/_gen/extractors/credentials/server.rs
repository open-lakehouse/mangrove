// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::credentials::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListCredentialsRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            purpose: Option<Purpose>,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
        }
        let axum_extra::extract::Query(QueryParams {
            purpose,
            max_results,
            page_token,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(ListCredentialsRequest {
            purpose: purpose.map(|v| v as i32),
            max_results,
            page_token,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateCredentialRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Json(request) = req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(request)
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetCredentialRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts
            .extract::<axum::extract::Path<String>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetCredentialRequest { name })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateCredentialRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path(name) = parts
            .extract::<axum::extract::Path<String>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<UpdateCredentialRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (
            new_name,
            comment,
            read_only,
            owner,
            skip_validation,
            force,
            azure_service_principal,
            azure_managed_identity,
            azure_storage_key,
            aws_iam_role,
            databricks_gcp_service_account,
        ) = (
            body.new_name,
            body.comment,
            body.read_only,
            body.owner,
            body.skip_validation,
            body.force,
            body.azure_service_principal,
            body.azure_managed_identity,
            body.azure_storage_key,
            body.aws_iam_role,
            body.databricks_gcp_service_account,
        );
        Ok(UpdateCredentialRequest {
            name,
            new_name,
            comment,
            read_only,
            owner,
            skip_validation,
            force,
            azure_service_principal,
            azure_managed_identity,
            azure_storage_key,
            aws_iam_role,
            databricks_gcp_service_account,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteCredentialRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts
            .extract::<axum::extract::Path<String>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(DeleteCredentialRequest { name })
    }
}
