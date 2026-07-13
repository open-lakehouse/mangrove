// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::policies::v1::*;
use axum::{RequestExt, RequestPartsExt};
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for ListPoliciesRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((on_securable_type, on_securable_fullname)) = parts
            .extract::<axum::extract::Path<(String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        #[derive(serde::Deserialize)]
        struct QueryParams {
            #[serde(default)]
            include_inherited: Option<bool>,
            #[serde(default)]
            max_results: Option<i32>,
            #[serde(default)]
            page_token: Option<String>,
        }
        let axum_extra::extract::Query(QueryParams {
            include_inherited,
            max_results,
            page_token,
        }) = parts
            .extract::<axum_extra::extract::Query<QueryParams>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(ListPoliciesRequest {
            on_securable_type,
            on_securable_fullname,
            include_inherited,
            max_results,
            page_token,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreatePolicyRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((on_securable_type, on_securable_fullname)) = parts
            .extract::<axum::extract::Path<(String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<CreatePolicyRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let policy_info = body.policy_info;
        Ok(CreatePolicyRequest {
            on_securable_type,
            on_securable_fullname,
            policy_info,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for GetPolicyRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((on_securable_type, on_securable_fullname, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(GetPolicyRequest {
            on_securable_type,
            on_securable_fullname,
            name,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequest<S> for UpdatePolicyRequest {
    type Rejection = axum::response::Response;
    async fn from_request(
        mut req: axum::extract::Request<axum::body::Body>,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let (mut parts, body) = req.into_parts();
        let axum::extract::Path((on_securable_type, on_securable_fullname, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let body_req = axum::extract::Request::from_parts(parts, body);
        let axum::extract::Json::<UpdatePolicyRequest>(body) = body_req
            .extract()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        let (policy_info, update_mask) = (body.policy_info, body.update_mask);
        Ok(UpdatePolicyRequest {
            on_securable_type,
            on_securable_fullname,
            name,
            policy_info,
            update_mask,
            ..Default::default()
        })
    }
}
impl<S: Send + Sync> axum::extract::FromRequestParts<S> for DeletePolicyRequest {
    type Rejection = axum::response::Response;
    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let axum::extract::Path((on_securable_type, on_securable_fullname, name)) = parts
            .extract::<axum::extract::Path<(String, String, String)>>()
            .await
            .map_err(axum::response::IntoResponse::into_response)?;
        Ok(DeletePolicyRequest {
            on_securable_type,
            on_securable_fullname,
            name,
            ..Default::default()
        })
    }
}
