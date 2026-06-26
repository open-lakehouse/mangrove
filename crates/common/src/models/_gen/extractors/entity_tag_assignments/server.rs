// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::tags::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListEntityTagAssignmentsRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((entity_type, entity_name)) = parts
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
        Ok(ListEntityTagAssignmentsRequest {
            entity_type,
            entity_name,
            max_results,
            page_token,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateEntityTagAssignmentRequest {
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
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetEntityTagAssignmentRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((entity_type, entity_name, tag_key)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetEntityTagAssignmentRequest {
            entity_type,
            entity_name,
            tag_key,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdateEntityTagAssignmentRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((entity_type, entity_name, tag_key)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<UpdateEntityTagAssignmentRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (tag_assignment, update_mask) = (body.tag_assignment, body.update_mask);
        Ok(UpdateEntityTagAssignmentRequest {
            entity_type,
            entity_name,
            tag_key,
            tag_assignment,
            update_mask,
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeleteEntityTagAssignmentRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((entity_type, entity_name, tag_key)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(DeleteEntityTagAssignmentRequest {
            entity_type,
            entity_name,
            tag_key,
        })
    }
}
