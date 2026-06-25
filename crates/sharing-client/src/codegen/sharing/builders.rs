// @generated — do not edit by hand.
#![allow(unused_mut)]
#![allow(unused_imports)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
use super::client::*;
use crate::Result;
use std::future::IntoFuture;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
/// Builder for shares
pub struct ListSharesBuilder {
    client: SharingClient,
    request: ListSharesRequest,
}
impl ListSharesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingClient`.
    pub(crate) fn new(client: SharingClient) -> Self {
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
    /** Specifies a page token to use. Set pageToken to the nextPageToken returned
    by a previous list request to get the next page of results.*/
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
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
/// Builder for share
pub struct GetShareBuilder {
    client: SharingClient,
    request: GetShareRequest,
}
impl GetShareBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingClient`.
    pub(crate) fn new(client: SharingClient, name: impl Into<String>) -> Self {
        let request = GetShareRequest { name: name.into() };
        Self { client, request }
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
/// Builder for schemas
pub struct ListSchemasBuilder {
    client: SharingClient,
    request: ListSchemasRequest,
}
impl ListSchemasBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingClient`.
    pub(crate) fn new(client: SharingClient, share: impl Into<String>) -> Self {
        let request = ListSchemasRequest {
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
    /** Specifies a page token to use. Set pageToken to the nextPageToken returned
    by a previous list request to get the next page of results.*/
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
}
impl IntoFuture for ListSchemasBuilder {
    type Output = Result<ListSchemasResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_schemas(&request).await })
    }
}
/// Builder for tables
pub struct ListTablesBuilder {
    client: SharingClient,
    request: ListTablesRequest,
}
impl ListTablesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingClient`.
    pub(crate) fn new(
        client: SharingClient,
        share: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let request = ListTablesRequest {
            share: share.into(),
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
    /** Specifies a page token to use. Set pageToken to the nextPageToken returned
    by a previous list request to get the next page of results.*/
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
}
impl IntoFuture for ListTablesBuilder {
    type Output = Result<ListTablesResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_tables(&request).await })
    }
}
/// Builder for all tables
pub struct ListAllTablesBuilder {
    client: SharingClient,
    request: ListAllTablesRequest,
}
impl ListAllTablesBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingClient`.
    pub(crate) fn new(client: SharingClient, name: impl Into<String>) -> Self {
        let request = ListAllTablesRequest {
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
    /** Specifies a page token to use. Set pageToken to the nextPageToken returned
    by a previous list request to get the next page of results.*/
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
}
impl IntoFuture for ListAllTablesBuilder {
    type Output = Result<ListAllTablesResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_all_tables(&request).await })
    }
}
