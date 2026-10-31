// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::ShareServiceClient;
use unitycatalog_common::models::shares::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::shares::ShareHandler<Cx> for ShareServiceClient {
    async fn list_shares(
        &self,
        request: ListSharesRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListSharesResponse> {
        ShareServiceClient::list_shares(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_share(
        &self,
        request: CreateShareRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Share> {
        ShareServiceClient::create_share(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_share(
        &self,
        request: GetShareRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Share> {
        ShareServiceClient::get_share(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_share(
        &self,
        request: UpdateShareRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Share> {
        ShareServiceClient::update_share(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_share(
        &self,
        request: DeleteShareRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        ShareServiceClient::delete_share(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_permissions(
        &self,
        request: GetPermissionsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<GetPermissionsResponse> {
        ShareServiceClient::get_permissions(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_permissions(
        &self,
        request: UpdatePermissionsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<UpdatePermissionsResponse> {
        ShareServiceClient::update_permissions(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
