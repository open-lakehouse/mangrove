// @generated — do not edit by hand.
#![allow(unused_mut)]
#![allow(unused_imports)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
type BoxStr<'a, T> = ::futures::stream::BoxStream<'a, T>;
use super::super::stream_paginated;
use super::client::*;
use crate::Result;
use futures::{StreamExt, TryStreamExt};
use std::future::IntoFuture;
use unitycatalog_common::models::providers::v1::*;
/// Builder for listing providers
pub struct ListProvidersBuilder {
    client: ProviderServiceClient,
    request: ListProvidersRequest,
}
impl ListProvidersBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ProviderServiceClient`.
    pub(crate) fn new(client: ProviderServiceClient) -> Self {
        let request = ListProvidersRequest {
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
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStr<'static, Result<Provider>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_providers(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.providers.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.providers.into_iter().map(Ok)))
        .try_flatten();
        stream.boxed()
    }
}
impl IntoFuture for ListProvidersBuilder {
    type Output = Result<ListProvidersResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_providers(&request).await })
    }
}
/// Builder for creating a provider
pub struct CreateProviderBuilder {
    client: ProviderServiceClient,
    request: CreateProviderRequest,
}
impl CreateProviderBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ProviderServiceClient`.
    pub(crate) fn new(
        client: ProviderServiceClient,
        name: impl Into<String>,
        authentication_type: ProviderAuthenticationType,
    ) -> Self {
        let request = CreateProviderRequest {
            name: name.into(),
            authentication_type: authentication_type as i32,
            ..Default::default()
        };
        Self { client, request }
    }
    /// Username of the provider owner.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    /// Description about the provider.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /** The recipient profile (credential file contents) used to connect to the
    sharing server, required for TOKEN authentication.*/
    pub fn with_recipient_profile_str(
        mut self,
        recipient_profile_str: impl Into<Option<String>>,
    ) -> Self {
        self.request.recipient_profile_str = recipient_profile_str.into();
        self
    }
    /// Provider properties as map of string key-value pairs.
    pub fn with_properties<I, K, V>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.request.properties = properties
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }
}
impl IntoFuture for CreateProviderBuilder {
    type Output = Result<Provider>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_provider(&request).await })
    }
}
/// Builder for getting a provider
pub struct GetProviderBuilder {
    client: ProviderServiceClient,
    request: GetProviderRequest,
}
impl GetProviderBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ProviderServiceClient`.
    pub(crate) fn new(client: ProviderServiceClient, name: impl Into<String>) -> Self {
        let request = GetProviderRequest { name: name.into() };
        Self { client, request }
    }
}
impl IntoFuture for GetProviderBuilder {
    type Output = Result<Provider>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_provider(&request).await })
    }
}
/// Builder for updating a provider
pub struct UpdateProviderBuilder {
    client: ProviderServiceClient,
    request: UpdateProviderRequest,
}
impl UpdateProviderBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ProviderServiceClient`.
    pub(crate) fn new(client: ProviderServiceClient, name: impl Into<String>) -> Self {
        let request = UpdateProviderRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// New name for the provider.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    /// Username of the provider owner.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    /// Description about the provider.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /** The recipient profile (credential file contents) used to connect to the
    sharing server.*/
    pub fn with_recipient_profile_str(
        mut self,
        recipient_profile_str: impl Into<Option<String>>,
    ) -> Self {
        self.request.recipient_profile_str = recipient_profile_str.into();
        self
    }
    /** Provider properties as map of string key-value pairs.

    When provided in update request, the specified properties will override the existing properties.
    To add and remove properties, one would need to perform a read-modify-write.*/
    pub fn with_properties<I, K, V>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.request.properties = properties
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }
}
impl IntoFuture for UpdateProviderBuilder {
    type Output = Result<Provider>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_provider(&request).await })
    }
}
/// Builder for deleting a provider
pub struct DeleteProviderBuilder {
    client: ProviderServiceClient,
    request: DeleteProviderRequest,
}
impl DeleteProviderBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ProviderServiceClient`.
    pub(crate) fn new(client: ProviderServiceClient, name: impl Into<String>) -> Self {
        let request = DeleteProviderRequest { name: name.into() };
        Self { client, request }
    }
}
impl IntoFuture for DeleteProviderBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_provider(&request).await })
    }
}
