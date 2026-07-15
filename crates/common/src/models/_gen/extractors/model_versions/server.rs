// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::model_versions::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListModelVersionsRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(full_name) =
            parts
                .extract::<axum::extract::Path<String>>()
                .await
                .map_err(axum::response::IntoResponse::into_response)?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
            #[serde(default)]
            include_browse: Option<bool>,
        }
        let axum_extra::extract::Query(QueryParams {
            max_results,
            page_token,
            include_browse,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(ListModelVersionsRequest {
            full_name,
            max_results,
            page_token,
            include_browse,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateModelVersionRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetModelVersionRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((full_name, version)) = parts
            .extract::<axum::extract::Path<(String, i64)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            include_browse: Option<bool>,
        }
        let axum_extra::extract::Query(QueryParams { include_browse }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetModelVersionRequest {
            full_name,
            version,
            include_browse,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateModelVersionRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((full_name, version)) = parts
            .extract::<axum::extract::Path<(String, i64)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<UpdateModelVersionRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let comment = body.comment;
        Ok(UpdateModelVersionRequest {
            full_name,
            version,
            comment,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteModelVersionRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((full_name, version)) = parts
            .extract::<axum::extract::Path<(String, i64)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(DeleteModelVersionRequest {
            full_name,
            version,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for FinalizeModelVersionRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, _body) = req.into_parts();
        let axum::extract::Path((full_name, version)) = parts
            .extract::<axum::extract::Path<(String, i64)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(FinalizeModelVersionRequest {
            full_name,
            version,
            ..Default::default()
        })
    }
}
