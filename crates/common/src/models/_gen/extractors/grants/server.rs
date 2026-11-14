// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]

use crate::models::grants::v1::*;
use axum::{RequestExt, RequestPartsExt};

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetGrantsRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((securable_type, full_name)) = parts
            .extract::<axum::extract::Path<(String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            principal: Option<String>,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
        }
        let axum_extra::extract::Query(QueryParams {
            principal,
            max_results,
            page_token,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetGrantsRequest {
            securable_type,
            full_name,
            principal,
            max_results,
            page_token,
            ..Default::default()
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateGrantsRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((securable_type, full_name)) = parts
            .extract::<axum::extract::Path<(String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<UpdateGrantsRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let changes = body.changes;
        Ok(UpdateGrantsRequest {
            securable_type,
            full_name,
            changes,
            ..Default::default()
        })
    }
}
