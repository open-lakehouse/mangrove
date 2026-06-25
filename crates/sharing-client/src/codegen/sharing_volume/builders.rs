// @generated — do not edit by hand.
#![allow(unused_mut)]
#![allow(unused_imports)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
use super::client::*;
use crate::Result;
use std::future::IntoFuture;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
/// Builder for volumes
pub struct ListVolumesBuilder {
    client: SharingVolumeClient,
    request: ListVolumesRequest,
}
impl ListVolumesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingVolumeClient`.
    pub(crate) fn new(
        client: SharingVolumeClient,
        share: impl Into<String>,
        schema: impl Into<String>,
    ) -> Self {
        let request = ListVolumesRequest {
            share: share.into(),
            schema: schema.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    /// Specifies a page token to use, from a previous response's next_page_token.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
}
impl IntoFuture for ListVolumesBuilder {
    type Output = Result<ListVolumesResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_volumes(&request).await })
    }
}
/// Builder for all volumes
pub struct ListAllVolumesBuilder {
    client: SharingVolumeClient,
    request: ListAllVolumesRequest,
}
impl ListAllVolumesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingVolumeClient`.
    pub(crate) fn new(client: SharingVolumeClient, share: impl Into<String>) -> Self {
        let request = ListAllVolumesRequest {
            share: share.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    /// Specifies a page token to use, from a previous response's next_page_token.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
}
impl IntoFuture for ListAllVolumesBuilder {
    type Output = Result<ListAllVolumesResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_all_volumes(&request).await })
    }
}
/// Builder for volume
pub struct GetVolumeBuilder {
    client: SharingVolumeClient,
    request: GetVolumeRequest,
}
impl GetVolumeBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingVolumeClient`.
    pub(crate) fn new(
        client: SharingVolumeClient,
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let request = GetVolumeRequest {
            share: share.into(),
            schema: schema.into(),
            name: name.into(),
        };
        Self { client, request }
    }
}
impl IntoFuture for GetVolumeBuilder {
    type Output = Result<SharingVolume>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_volume(&request).await })
    }
}
/// Builder for temporary volume credentials
pub struct GenerateTemporaryVolumeCredentialsBuilder {
    client: SharingVolumeClient,
    request: GenerateTemporaryVolumeCredentialsRequest,
}
impl GenerateTemporaryVolumeCredentialsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingVolumeClient`.
    pub(crate) fn new(
        client: SharingVolumeClient,
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let request = GenerateTemporaryVolumeCredentialsRequest {
            share: share.into(),
            schema: schema.into(),
            name: name.into(),
        };
        Self { client, request }
    }
}
impl IntoFuture for GenerateTemporaryVolumeCredentialsBuilder {
    type Output = Result<SharingTemporaryCredentials>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.generate_temporary_volume_credentials(&request).await })
    }
}
