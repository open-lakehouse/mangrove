//! Hand-written axum request extractors for the Delta Sharing **query** path
//! (`get_table_version`, `get_table_metadata`, `query_table`).
//!
//! These RPCs are deliberately not part of the generated `SharingService`
//! (their responses are newline-delimited JSON, a streaming contract the code
//! generator does not model — see `proto/sharing/open_sharing/v1/svc.proto`), so
//! their extractors are maintained here rather than generated alongside the
//! discovery extractors in `codegen/extractors`.

use axum::{RequestExt, RequestPartsExt};

use crate::Result;
use crate::models::open_sharing::v1::{
    GetTableMetadataRequest, GetTableVersionRequest, QueryTableRequest,
};

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableVersionRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            starting_timestamp: Option<String>,
        }
        let axum_extra::extract::Query(QueryParams { starting_timestamp }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetTableVersionRequest {
            share,
            schema,
            name,
            starting_timestamp,
            ..Default::default()
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableMetadataRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetTableMetadataRequest {
            share,
            schema,
            name,
            ..Default::default()
        })
    }
}

impl<S: Send + Sync> axum::extract::FromRequest<S> for QueryTableRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((share, schema, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<QueryTableRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(QueryTableRequest {
            share,
            schema,
            name,
            ..body
        })
    }
}
