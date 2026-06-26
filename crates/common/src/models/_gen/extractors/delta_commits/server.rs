// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::delta_commits::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequest<S> for CommitRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetCommitsRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            table_id: String,
            table_uri: String,
            start_version: i64,
            #[serde(default)]
            end_version: Option<i64>,
        }
        let axum_extra::extract::Query(QueryParams {
            table_id,
            table_uri,
            start_version,
            end_version,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetCommitsRequest {
            table_id,
            table_uri,
            start_version,
            end_version,
        })
    }
}
