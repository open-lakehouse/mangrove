// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::external_locations::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListExternalLocationsRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
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
        Ok(ListExternalLocationsRequest {
            max_results,
            page_token,
            include_browse,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateExternalLocationRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetExternalLocationRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts
            .extract::<axum::extract::Path<String>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetExternalLocationRequest { name })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateExternalLocationRequest {
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
        let axum::extract::Json::<UpdateExternalLocationRequest>(body) =
            body_req
                .extract()
                .await
                .map_err(axum::response::IntoResponse::into_response)?;
        let (url, credential_name, read_only, owner, comment, new_name, force, skip_validation) = (
            body.url,
            body.credential_name,
            body.read_only,
            body.owner,
            body.comment,
            body.new_name,
            body.force,
            body.skip_validation,
        );
        Ok(UpdateExternalLocationRequest {
            name,
            url,
            credential_name,
            read_only,
            owner,
            comment,
            new_name,
            force,
            skip_validation,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteExternalLocationRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts
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
        Ok(DeleteExternalLocationRequest { name, force })
    }
}
