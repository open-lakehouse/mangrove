// @generated — do not edit by hand.
#![allow(unused_imports)]

use super::client::GrantClient;
use unitycatalog_common::models::grants::v1::*;

#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::grants::GrantHandler<Cx> for GrantClient {
    async fn get_grants(
        &self,
        request: GetGrantsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<GetGrantsResponse> {
        GrantClient::get_grants(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn update_grants(
        &self,
        request: UpdateGrantsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<UpdateGrantsResponse> {
        GrantClient::update_grants(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
