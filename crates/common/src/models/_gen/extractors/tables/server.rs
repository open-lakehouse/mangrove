// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::tables::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListTableSummariesRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            catalog_name: String,
            #[serde(default)]
            schema_name_pattern: Option<String>,
            #[serde(default)]
            table_name_pattern: Option<String>,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
            #[serde(default)]
            include_manifest_capabilities: Option<bool>,
        }
        let axum_extra::extract::Query(QueryParams {
            catalog_name,
            schema_name_pattern,
            table_name_pattern,
            max_results,
            page_token,
            include_manifest_capabilities,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(ListTableSummariesRequest {
            catalog_name,
            schema_name_pattern,
            table_name_pattern,
            max_results,
            page_token,
            include_manifest_capabilities,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListTablesRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        #[derive(serde::Deserialize)]
        struct QueryParams {
            catalog_name: String,
            schema_name: String,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
            #[serde(default)]
            include_delta_metadata: Option<bool>,
            #[serde(default)]
            omit_columns: Option<bool>,
            #[serde(default)]
            omit_properties: Option<bool>,
            #[serde(default)]
            omit_username: Option<bool>,
            #[serde(default)]
            include_browse: Option<bool>,
            #[serde(default)]
            include_manifest_capabilities: Option<bool>,
        }
        let axum_extra::extract::Query(QueryParams {
            catalog_name,
            schema_name,
            max_results,
            page_token,
            include_delta_metadata,
            omit_columns,
            omit_properties,
            omit_username,
            include_browse,
            include_manifest_capabilities,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(ListTablesRequest {
            catalog_name,
            schema_name,
            max_results,
            page_token,
            include_delta_metadata,
            omit_columns,
            omit_properties,
            omit_username,
            include_browse,
            include_manifest_capabilities,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateTableRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableRequest {
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
            include_delta_metadata: Option<bool>,
            #[serde(default)]
            include_browse: Option<bool>,
            #[serde(default)]
            include_manifest_capabilities: Option<bool>,
        }
        let axum_extra::extract::Query(QueryParams {
            include_delta_metadata,
            include_browse,
            include_manifest_capabilities,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetTableRequest {
            full_name,
            include_delta_metadata,
            include_browse,
            include_manifest_capabilities,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetTableExistsRequest {
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
        Ok(GetTableExistsRequest { full_name })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteTableRequest {
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
        Ok(DeleteTableRequest { full_name })
    }
}
