// @generated — do not edit by hand.
#![allow(unused_imports)]

use super::client::SchemaServiceClient;
use unitycatalog_common::models::schemas::v1::*;

#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::schemas::SchemaHandler<Cx> for SchemaServiceClient {
    async fn list_schemas(
        &self,
        request: ListSchemasRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListSchemasResponse> {
        SchemaServiceClient::list_schemas(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn create_schema(
        &self,
        request: CreateSchemaRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Schema> {
        SchemaServiceClient::create_schema(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn get_schema(
        &self,
        request: GetSchemaRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Schema> {
        SchemaServiceClient::get_schema(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn update_schema(
        &self,
        request: UpdateSchemaRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Schema> {
        SchemaServiceClient::update_schema(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }

    async fn delete_schema(
        &self,
        request: DeleteSchemaRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        SchemaServiceClient::delete_schema(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
