// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::client::VolumeServiceClient;
use unitycatalog_common::models::volumes::v1::*;
#[::async_trait::async_trait]
impl<Cx: Send + 'static> unitycatalog_handlers::volumes::VolumeHandler<Cx> for VolumeServiceClient {
    async fn list_volumes(
        &self,
        request: ListVolumesRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<ListVolumesResponse> {
        VolumeServiceClient::list_volumes(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn create_volume(
        &self,
        request: CreateVolumeRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Volume> {
        VolumeServiceClient::create_volume(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn get_volume(
        &self,
        request: GetVolumeRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Volume> {
        VolumeServiceClient::get_volume(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn update_volume(
        &self,
        request: UpdateVolumeRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<Volume> {
        VolumeServiceClient::update_volume(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
    async fn delete_volume(
        &self,
        request: DeleteVolumeRequest,
        _context: Cx,
    ) -> unitycatalog_handlers::Result<()> {
        VolumeServiceClient::delete_volume(self, &request)
            .await
            .map_err(::core::convert::Into::into)
    }
}
