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
use unitycatalog_common::models::tags::v1::*;
/// Builder for listing tag policies
pub struct ListTagPoliciesBuilder {
    client: TagPolicyServiceClient,
    request: ListTagPoliciesRequest,
}
impl ListTagPoliciesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyServiceClient`.
    pub(crate) fn new(client: TagPolicyServiceClient) -> Self {
        let request = ListTagPoliciesRequest {
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
    pub fn into_stream(self) -> BoxStr<'static, Result<TagPolicy>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_tag_policies(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.tag_policies.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.tag_policies.into_iter().map(Ok)))
        .try_flatten();
        #[cfg(not(target_arch = "wasm32"))]
        let stream = stream.boxed();
        #[cfg(target_arch = "wasm32")]
        let stream = stream.boxed_local();
        stream
    }
}
impl IntoFuture for ListTagPoliciesBuilder {
    type Output = Result<ListTagPoliciesResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_tag_policies(&request).await })
    }
}
/// Builder for creating a tag policy
pub struct CreateTagPolicyBuilder {
    client: TagPolicyServiceClient,
    request: CreateTagPolicyRequest,
}
impl CreateTagPolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyServiceClient`.
    pub(crate) fn new(client: TagPolicyServiceClient, tag_policy: TagPolicy) -> Self {
        let request = CreateTagPolicyRequest {
            tag_policy: buffa::MessageField::some(tag_policy),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for CreateTagPolicyBuilder {
    type Output = Result<TagPolicy>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_tag_policy(&request).await })
    }
}
/// Builder for getting a tag policy
pub struct GetTagPolicyBuilder {
    client: TagPolicyServiceClient,
    request: GetTagPolicyRequest,
}
impl GetTagPolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyServiceClient`.
    pub(crate) fn new(client: TagPolicyServiceClient, tag_key: impl Into<String>) -> Self {
        let request = GetTagPolicyRequest {
            tag_key: tag_key.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetTagPolicyBuilder {
    type Output = Result<TagPolicy>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_tag_policy(&request).await })
    }
}
/// Builder for updating a tag policy
pub struct UpdateTagPolicyBuilder {
    client: TagPolicyServiceClient,
    request: UpdateTagPolicyRequest,
}
impl UpdateTagPolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyServiceClient`.
    pub(crate) fn new(
        client: TagPolicyServiceClient,
        tag_key: impl Into<String>,
        tag_policy: TagPolicy,
    ) -> Self {
        let request = UpdateTagPolicyRequest {
            tag_key: tag_key.into(),
            tag_policy: buffa::MessageField::some(tag_policy),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The list of fields to update, as a comma-separated string.
    pub fn with_update_mask(mut self, update_mask: impl Into<Option<String>>) -> Self {
        self.request.update_mask = update_mask.into();
        self
    }
}
impl IntoFuture for UpdateTagPolicyBuilder {
    type Output = Result<TagPolicy>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_tag_policy(&request).await })
    }
}
/// Builder for deleting a tag policy
pub struct DeleteTagPolicyBuilder {
    client: TagPolicyServiceClient,
    request: DeleteTagPolicyRequest,
}
impl DeleteTagPolicyBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `TagPolicyServiceClient`.
    pub(crate) fn new(client: TagPolicyServiceClient, tag_key: impl Into<String>) -> Self {
        let request = DeleteTagPolicyRequest {
            tag_key: tag_key.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteTagPolicyBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_tag_policy(&request).await })
    }
}
