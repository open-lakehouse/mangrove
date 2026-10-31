// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::FunctionServiceClient;
use unitycatalog_common::models::functions::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::functions::FunctionHandler<Cx>
    for FunctionServiceClient
{
    async fn list_functions(
        &self,
        request: ListFunctionsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListFunctionsResponse> {
        FunctionServiceClient::list_functions(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_function(
        &self,
        request: CreateFunctionRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Function> {
        FunctionServiceClient::create_function(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_function(
        &self,
        request: GetFunctionRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Function> {
        FunctionServiceClient::get_function(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_function(
        &self,
        request: UpdateFunctionRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Function> {
        FunctionServiceClient::update_function(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_function(
        &self,
        request: DeleteFunctionRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        FunctionServiceClient::delete_function(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
