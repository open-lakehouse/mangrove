// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::agents::v0alpha1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListAgentsRequest {
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
        Ok(ListAgentsRequest {
            catalog_name,
            schema_name,
            max_results,
            page_token,
            include_browse,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateAgentRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetAgentRequest {
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
            include_browse: Option<bool>,
        }
        let axum_extra::extract::Query(QueryParams { include_browse }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetAgentRequest {
            name,
            include_browse,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateAgentRequest {
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
        let axum::extract::Json::<UpdateAgentRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (
            new_name,
            invocation_protocol,
            endpoint,
            description,
            capabilities,
            input_schema,
            comment,
            owner,
        ) = (
            body.new_name,
            body.invocation_protocol,
            body.endpoint,
            body.description,
            body.capabilities,
            body.input_schema,
            body.comment,
            body.owner,
        );
        Ok(UpdateAgentRequest {
            name,
            new_name,
            invocation_protocol,
            endpoint,
            description,
            capabilities,
            input_schema,
            comment,
            owner,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteAgentRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path(name) = parts
            .extract::<axum::extract::Path<String>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(DeleteAgentRequest { name })
    }
}
