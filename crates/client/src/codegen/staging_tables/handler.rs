// @generated — do not edit by hand.
#![allow(unused_imports)]

use super::client::StagingTableServiceClient;
use unitycatalog_common::models::staging_tables::v1::*;

#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::staging_tables::StagingTableHandler<Cx>
    for StagingTableServiceClient
{
    async fn create_staging_table(
        &self,
        request: CreateStagingTableRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<StagingTable> {
        StagingTableServiceClient::create_staging_table(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
