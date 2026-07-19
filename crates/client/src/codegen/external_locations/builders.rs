// @generated — do not edit by hand.
#![allow(unused_mut)]
#![allow(unused_imports)]
#[cfg(not(target_arch = "wasm32"))]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
#[cfg(target_arch = "wasm32")]
type BoxFut<'a, T> = ::futures::future::LocalBoxFuture<'a, T>;
#[cfg(not(target_arch = "wasm32"))]
type BoxStr<'a, T> = ::futures::stream::BoxStream<'a, T>;
#[cfg(target_arch = "wasm32")]
type BoxStr<'a, T> = ::futures::stream::LocalBoxStream<'a, T>;
use super::super::stream_paginated;
use super::client::*;
use crate::Result;
use futures::{StreamExt, TryStreamExt};
use std::future::IntoFuture;
use unitycatalog_common::models::external_locations::v1::*;
/// Builder for listing external locations
pub struct ListExternalLocationsBuilder {
    client: ExternalLocationServiceClient,
    request: ListExternalLocationsRequest,
}
impl ListExternalLocationsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ExternalLocationServiceClient`.
    pub(crate) fn new(client: ExternalLocationServiceClient) -> Self {
        let request = ListExternalLocationsRequest {
            ..Default::default()
        };
        Self { client, request }
    }
    /// The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    /// Opaque pagination token to go to next page based on previous query.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
    /// Whether to include schemas in the response for which the principal can only access selective metadata for
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStr<'static, Result<ExternalLocation>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder
                    .client
                    .list_external_locations(&builder.request)
                    .await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.external_locations.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.external_locations.into_iter().map(Ok)))
        .try_flatten();
        #[cfg(not(target_arch = "wasm32"))]
        let stream = stream.boxed();
        #[cfg(target_arch = "wasm32")]
        let stream = stream.boxed_local();
        stream
    }
}
impl IntoFuture for ListExternalLocationsBuilder {
    type Output = Result<ListExternalLocationsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_external_locations(&request).await })
    }
}
/// Builder for creating a external location
pub struct CreateExternalLocationBuilder {
    client: ExternalLocationServiceClient,
    request: CreateExternalLocationRequest,
}
impl CreateExternalLocationBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ExternalLocationServiceClient`.
    pub(crate) fn new(
        client: ExternalLocationServiceClient,
        name: impl Into<String>,
        url: impl Into<String>,
        credential_name: impl Into<String>,
    ) -> Self {
        let request = CreateExternalLocationRequest {
            name: name.into(),
            url: url.into(),
            credential_name: credential_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Indicates whether the external location is read-only.
    pub fn with_read_only(mut self, read_only: impl Into<Option<bool>>) -> Self {
        self.request.read_only = read_only.into();
        self
    }
    /// User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// Skips validation of the storage credential associated with the external location.
    pub fn with_skip_validation(mut self, skip_validation: impl Into<Option<bool>>) -> Self {
        self.request.skip_validation = skip_validation.into();
        self
    }
}
impl IntoFuture for CreateExternalLocationBuilder {
    type Output = Result<ExternalLocation>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_external_location(&request).await })
    }
}
/// Builder for getting a external location
pub struct GetExternalLocationBuilder {
    client: ExternalLocationServiceClient,
    request: GetExternalLocationRequest,
}
impl GetExternalLocationBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ExternalLocationServiceClient`.
    pub(crate) fn new(client: ExternalLocationServiceClient, name: impl Into<String>) -> Self {
        let request = GetExternalLocationRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetExternalLocationBuilder {
    type Output = Result<ExternalLocation>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_external_location(&request).await })
    }
}
/// Builder for updating a external location
pub struct UpdateExternalLocationBuilder {
    client: ExternalLocationServiceClient,
    request: UpdateExternalLocationRequest,
}
impl UpdateExternalLocationBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ExternalLocationServiceClient`.
    pub(crate) fn new(client: ExternalLocationServiceClient, name: impl Into<String>) -> Self {
        let request = UpdateExternalLocationRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Path URL of the external location.
    pub fn with_url(mut self, url: impl Into<Option<String>>) -> Self {
        self.request.url = url.into();
        self
    }
    /// Name of the storage credential used with this location.
    pub fn with_credential_name(mut self, credential_name: impl Into<Option<String>>) -> Self {
        self.request.credential_name = credential_name.into();
        self
    }
    /// Indicates whether the external location is read-only.
    pub fn with_read_only(mut self, read_only: impl Into<Option<bool>>) -> Self {
        self.request.read_only = read_only.into();
        self
    }
    /// owner of the external location.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    /// User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// new name of the external location.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    /// force update of the external location.
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
    /// Skips validation of the storage credential associated with the external location.
    pub fn with_skip_validation(mut self, skip_validation: impl Into<Option<bool>>) -> Self {
        self.request.skip_validation = skip_validation.into();
        self
    }
}
impl IntoFuture for UpdateExternalLocationBuilder {
    type Output = Result<ExternalLocation>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_external_location(&request).await })
    }
}
/// Builder for deleting a external location
pub struct DeleteExternalLocationBuilder {
    client: ExternalLocationServiceClient,
    request: DeleteExternalLocationRequest,
}
impl DeleteExternalLocationBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ExternalLocationServiceClient`.
    pub(crate) fn new(client: ExternalLocationServiceClient, name: impl Into<String>) -> Self {
        let request = DeleteExternalLocationRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Force deletion even if the external location is not empty.
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
}
impl IntoFuture for DeleteExternalLocationBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_external_location(&request).await })
    }
}
