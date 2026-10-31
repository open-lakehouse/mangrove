// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::ExternalLocationServiceClient;
use unitycatalog_common::models::external_locations::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::external_locations::ExternalLocationHandler<Cx>
    for ExternalLocationServiceClient
{
    async fn list_external_locations(
        &self,
        request: ListExternalLocationsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListExternalLocationsResponse> {
        ExternalLocationServiceClient::list_external_locations(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_external_location(
        &self,
        request: CreateExternalLocationRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ExternalLocation> {
        ExternalLocationServiceClient::create_external_location(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_external_location(
        &self,
        request: GetExternalLocationRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ExternalLocation> {
        ExternalLocationServiceClient::get_external_location(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_external_location(
        &self,
        request: UpdateExternalLocationRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ExternalLocation> {
        ExternalLocationServiceClient::update_external_location(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_external_location(
        &self,
        request: DeleteExternalLocationRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        ExternalLocationServiceClient::delete_external_location(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
