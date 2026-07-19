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
use unitycatalog_common::models::recipients::v1::*;
/// Builder for listing recipients
pub struct ListRecipientsBuilder {
    client: RecipientServiceClient,
    request: ListRecipientsRequest,
}
impl ListRecipientsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RecipientServiceClient`.
    pub(crate) fn new(client: RecipientServiceClient) -> Self {
        let request = ListRecipientsRequest {
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
    pub fn into_stream(self) -> BoxStr<'static, Result<Recipient>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_recipients(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.recipients.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.recipients.into_iter().map(Ok)))
        .try_flatten();
        #[cfg(not(target_arch = "wasm32"))]
        let stream = stream.boxed();
        #[cfg(target_arch = "wasm32")]
        let stream = stream.boxed_local();
        stream
    }
}
impl IntoFuture for ListRecipientsBuilder {
    type Output = Result<ListRecipientsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_recipients(&request).await })
    }
}
/// Builder for creating a recipient
pub struct CreateRecipientBuilder {
    client: RecipientServiceClient,
    request: CreateRecipientRequest,
}
impl CreateRecipientBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RecipientServiceClient`.
    pub(crate) fn new(
        client: RecipientServiceClient,
        name: impl Into<String>,
        authentication_type: AuthenticationType,
        owner: impl Into<String>,
    ) -> Self {
        let request = CreateRecipientRequest {
            name: name.into(),
            authentication_type: buffa::EnumValue::Known(authentication_type),
            owner: owner.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Description about the recipient.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /** Recipient properties as map of string key-value pairs.

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
    /// Expiration timestamp of the token, in epoch milliseconds.
    pub fn with_expiration_time(mut self, expiration_time: impl Into<Option<i64>>) -> Self {
        self.request.expiration_time = expiration_time.into();
        self
    }
}
impl IntoFuture for CreateRecipientBuilder {
    type Output = Result<Recipient>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_recipient(&request).await })
    }
}
/// Builder for getting a recipient
pub struct GetRecipientBuilder {
    client: RecipientServiceClient,
    request: GetRecipientRequest,
}
impl GetRecipientBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RecipientServiceClient`.
    pub(crate) fn new(client: RecipientServiceClient, name: impl Into<String>) -> Self {
        let request = GetRecipientRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetRecipientBuilder {
    type Output = Result<Recipient>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_recipient(&request).await })
    }
}
/// Builder for updating a recipient
pub struct UpdateRecipientBuilder {
    client: RecipientServiceClient,
    request: UpdateRecipientRequest,
}
impl UpdateRecipientBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RecipientServiceClient`.
    pub(crate) fn new(client: RecipientServiceClient, name: impl Into<String>) -> Self {
        let request = UpdateRecipientRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// New name for the recipient
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    /// Username of the recipient owner.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    /// Description about the recipient.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /** Recipient properties as map of string key-value pairs.

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
    /// Expiration timestamp of the token, in epoch milliseconds.
    pub fn with_expiration_time(mut self, expiration_time: impl Into<Option<i64>>) -> Self {
        self.request.expiration_time = expiration_time.into();
        self
    }
}
impl IntoFuture for UpdateRecipientBuilder {
    type Output = Result<Recipient>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_recipient(&request).await })
    }
}
/// Builder for deleting a recipient
pub struct DeleteRecipientBuilder {
    client: RecipientServiceClient,
    request: DeleteRecipientRequest,
}
impl DeleteRecipientBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RecipientServiceClient`.
    pub(crate) fn new(client: RecipientServiceClient, name: impl Into<String>) -> Self {
        let request = DeleteRecipientRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteRecipientBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_recipient(&request).await })
    }
}
