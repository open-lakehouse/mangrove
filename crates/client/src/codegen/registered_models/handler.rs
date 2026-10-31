// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::RegisteredModelServiceClient;
use unitycatalog_common::models::registered_models::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::registered_models::RegisteredModelHandler<Cx>
    for RegisteredModelServiceClient
{
    async fn list_registered_models(
        &self,
        request: ListRegisteredModelsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListRegisteredModelsResponse> {
        RegisteredModelServiceClient::list_registered_models(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_registered_model(
        &self,
        request: CreateRegisteredModelRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<RegisteredModel> {
        RegisteredModelServiceClient::create_registered_model(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_registered_model(
        &self,
        request: GetRegisteredModelRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<RegisteredModel> {
        RegisteredModelServiceClient::get_registered_model(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_registered_model(
        &self,
        request: UpdateRegisteredModelRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<RegisteredModel> {
        RegisteredModelServiceClient::update_registered_model(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_registered_model(
        &self,
        request: DeleteRegisteredModelRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        RegisteredModelServiceClient::delete_registered_model(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
