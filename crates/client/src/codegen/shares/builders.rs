// @generated — do not edit by hand.
#![allow(unused_mut)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
type BoxStr<'a, T> = ::futures::stream::BoxStream<'a, T>;
use super::super::stream_paginated;
use super::client::*;
use crate::Result;
use futures::{StreamExt, TryStreamExt};
use std::future::IntoFuture;
use unitycatalog_common::models::shares::v1::*;
/// Builder for listing shares
pub struct ListSharesBuilder {
    client: ShareServiceClient,
    request: ListSharesRequest,
}
impl ListSharesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ShareServiceClient`.
    pub(crate) fn new(client: ShareServiceClient) -> Self {
        let request = ListSharesRequest {
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
    pub fn into_stream(self) -> BoxStr<'static, Result<Share>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_shares(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.shares.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.shares.into_iter().map(Ok)))
        .try_flatten();
        stream.boxed()
    }
}
impl IntoFuture for ListSharesBuilder {
    type Output = Result<ListSharesResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_shares(&request).await })
    }
}
/// Builder for creating a share
pub struct CreateShareBuilder {
    client: ShareServiceClient,
    request: CreateShareRequest,
}
impl CreateShareBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ShareServiceClient`.
    pub(crate) fn new(client: ShareServiceClient, name: impl Into<String>) -> Self {
        let request = CreateShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
}
impl IntoFuture for CreateShareBuilder {
    type Output = Result<Share>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_share(&request).await })
    }
}
/// Builder for getting a share
pub struct GetShareBuilder {
    client: ShareServiceClient,
    request: GetShareRequest,
}
impl GetShareBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ShareServiceClient`.
    pub(crate) fn new(client: ShareServiceClient, name: impl Into<String>) -> Self {
        let request = GetShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Query for data to include in the share.
    pub fn with_include_shared_data(
        mut self,
        include_shared_data: impl Into<Option<bool>>,
    ) -> Self {
        self.request.include_shared_data = include_shared_data.into();
        self
    }
}
impl IntoFuture for GetShareBuilder {
    type Output = Result<Share>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_share(&request).await })
    }
}
/// Builder for updating a share
pub struct UpdateShareBuilder {
    client: ShareServiceClient,
    request: UpdateShareRequest,
}
impl UpdateShareBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ShareServiceClient`.
    pub(crate) fn new(client: ShareServiceClient, name: impl Into<String>) -> Self {
        let request = UpdateShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Array of shared data object updates.
    pub fn with_updates<I>(mut self, updates: I) -> Self
    where
        I: IntoIterator<Item = DataObjectUpdate>,
    {
        self.request.updates = updates.into_iter().collect();
        self
    }
    /// A new name for the share.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    /// Owner of the share.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    /// User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
}
impl IntoFuture for UpdateShareBuilder {
    type Output = Result<Share>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_share(&request).await })
    }
}
/// Builder for deleting a share
pub struct DeleteShareBuilder {
    client: ShareServiceClient,
    request: DeleteShareRequest,
}
impl DeleteShareBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ShareServiceClient`.
    pub(crate) fn new(client: ShareServiceClient, name: impl Into<String>) -> Self {
        let request = DeleteShareRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteShareBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_share(&request).await })
    }
}
/// Builder for permissions
pub struct GetPermissionsBuilder {
    client: ShareServiceClient,
    request: GetPermissionsRequest,
}
impl GetPermissionsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ShareServiceClient`.
    pub(crate) fn new(client: ShareServiceClient, name: impl Into<String>) -> Self {
        let request = GetPermissionsRequest {
            name: name.into(),
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
}
impl IntoFuture for GetPermissionsBuilder {
    type Output = Result<GetPermissionsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_permissions(&request).await })
    }
}
/// Builder for permissions
pub struct UpdatePermissionsBuilder {
    client: ShareServiceClient,
    request: UpdatePermissionsRequest,
}
impl UpdatePermissionsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `ShareServiceClient`.
    pub(crate) fn new(client: ShareServiceClient, name: impl Into<String>) -> Self {
        let request = UpdatePermissionsRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Array of permissions change objects.
    pub fn with_changes<I>(mut self, changes: I) -> Self
    where
        I: IntoIterator<Item = PermissionsChange>,
    {
        self.request.changes = changes.into_iter().collect();
        self
    }
    /// Whether to return the latest permissions list of the share in the response.
    pub fn with_omit_permissions_list(
        mut self,
        omit_permissions_list: impl Into<Option<bool>>,
    ) -> Self {
        self.request.omit_permissions_list = omit_permissions_list.into();
        self
    }
}
impl IntoFuture for UpdatePermissionsBuilder {
    type Output = Result<UpdatePermissionsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_permissions(&request).await })
    }
}
