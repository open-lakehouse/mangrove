// @generated — do not edit by hand.
#![allow(unused_imports)]

use super::client::ModelVersionServiceClient;
use unitycatalog_common::models::model_versions::v1::*;

#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::model_versions::ModelVersionHandler<Cx>
    for ModelVersionServiceClient
{
    async fn list_model_versions(
        &self,
        request: ListModelVersionsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListModelVersionsResponse> {
        ModelVersionServiceClient::list_model_versions(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn create_model_version(
        &self,
        request: CreateModelVersionRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ModelVersion> {
        ModelVersionServiceClient::create_model_version(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn get_model_version(
        &self,
        request: GetModelVersionRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ModelVersion> {
        ModelVersionServiceClient::get_model_version(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn update_model_version(
        &self,
        request: UpdateModelVersionRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ModelVersion> {
        ModelVersionServiceClient::update_model_version(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn delete_model_version(
        &self,
        request: DeleteModelVersionRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        ModelVersionServiceClient::delete_model_version(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn finalize_model_version(
        &self,
        request: FinalizeModelVersionRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ModelVersion> {
        ModelVersionServiceClient::finalize_model_version(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
