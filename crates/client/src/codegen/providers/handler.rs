// @generated — do not edit by hand.
#![allow(unused_imports)]

use super::client::ProviderServiceClient;
use unitycatalog_common::models::providers::v1::*;

#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::providers::ProviderHandler<Cx>
    for ProviderServiceClient
{
    async fn list_providers(
        &self,
        request: ListProvidersRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListProvidersResponse> {
        ProviderServiceClient::list_providers(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn create_provider(
        &self,
        request: CreateProviderRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Provider> {
        ProviderServiceClient::create_provider(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn get_provider(
        &self,
        request: GetProviderRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Provider> {
        ProviderServiceClient::get_provider(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn update_provider(
        &self,
        request: UpdateProviderRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Provider> {
        ProviderServiceClient::update_provider(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn delete_provider(
        &self,
        request: DeleteProviderRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        ProviderServiceClient::delete_provider(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
