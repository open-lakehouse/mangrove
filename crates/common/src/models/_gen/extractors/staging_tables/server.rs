// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports)]
use crate::models::staging_tables::v1::*;
use axum::RequestExt;
impl<S: Send + Sync> axum::extract::FromRequest<S> for CreateStagingTableRequest {
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
