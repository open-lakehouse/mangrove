// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::TableServiceClient;
use unitycatalog_common::models::tables::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::tables::TableHandler<Cx> for TableServiceClient {
    async fn list_table_summaries(
        &self,
        request: ListTableSummariesRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListTableSummariesResponse> {
        TableServiceClient::list_table_summaries(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn list_tables(
        &self,
        request: ListTablesRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListTablesResponse> {
        TableServiceClient::list_tables(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_table(
        &self,
        request: CreateTableRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Table> {
        TableServiceClient::create_table(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_table(
        &self,
        request: GetTableRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Table> {
        TableServiceClient::get_table(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_table_exists(
        &self,
        request: GetTableExistsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<GetTableExistsResponse> {
        TableServiceClient::get_table_exists(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_table(
        &self,
        request: DeleteTableRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        TableServiceClient::delete_table(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
