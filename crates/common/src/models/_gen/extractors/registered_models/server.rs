// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::registered_models::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListRegisteredModelsRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            catalog_name: Option<String>,
            #[serde(default)]
            schema_name: Option<String>,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
            #[serde(default)]
            include_browse: Option<bool>,
        }
        let axum_extra::extract::Query(QueryParams {
            catalog_name,
            schema_name,
            max_results,
            page_token,
            include_browse,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(ListRegisteredModelsRequest {
            catalog_name,
            schema_name,
            max_results,
            page_token,
            include_browse,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateRegisteredModelRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetRegisteredModelRequest {
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
            include_browse: Option<bool>,
        }
        let axum_extra::extract::Query(QueryParams { include_browse }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetRegisteredModelRequest {
            full_name,
            include_browse,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateRegisteredModelRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path(full_name) =
            parts
                .extract::<axum::extract::Path<String>>()
                .await
                .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<UpdateRegisteredModelRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (new_name, comment, owner) = (body.new_name, body.comment, body.owner);
        Ok(UpdateRegisteredModelRequest {
            full_name,
            new_name,
            comment,
            owner,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteRegisteredModelRequest {
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
            force: Option<bool>,
        }
        let axum_extra::extract::Query(QueryParams { force }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(DeleteRegisteredModelRequest {
            full_name,
            force,
            ..Default::default()
        })
    }
}
