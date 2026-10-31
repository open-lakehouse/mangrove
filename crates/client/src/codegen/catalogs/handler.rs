// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::CatalogServiceClient;
use unitycatalog_common::models::catalogs::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::catalogs::CatalogHandler<Cx>
    for CatalogServiceClient
{
    async fn list_catalogs(
        &self,
        request: ListCatalogsRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListCatalogsResponse> {
        CatalogServiceClient::list_catalogs(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_catalog(
        &self,
        request: CreateCatalogRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Catalog> {
        CatalogServiceClient::create_catalog(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_catalog(
        &self,
        request: GetCatalogRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Catalog> {
        CatalogServiceClient::get_catalog(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_catalog(
        &self,
        request: UpdateCatalogRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Catalog> {
        CatalogServiceClient::update_catalog(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_catalog(
        &self,
        request: DeleteCatalogRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        CatalogServiceClient::delete_catalog(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
