// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::open_sharing::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListVolumesRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((share, schema)) = parts
            .extract::<axum::extract::Path<(String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
        }
        let axum_extra::extract::Query(QueryParams {
            max_results,
            page_token,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(ListVolumesRequest {
            share,
            schema,
            max_results,
            page_token,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListAllVolumesRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(share) = parts
            .extract::<axum::extract::Path<String>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
        }
        let axum_extra::extract::Query(QueryParams {
            max_results,
            page_token,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(ListAllVolumesRequest {
            share,
            max_results,
            page_token,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetVolumeRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetVolumeRequest {
            share,
            schema,
            name,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for GenerateTemporaryVolumeCredentialsRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, _body) = req.into_parts();
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GenerateTemporaryVolumeCredentialsRequest {
            share,
            schema,
            name,
        })
    }
}
