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
use unitycatalog_common::models::catalogs::v1::*;
/// Builder for listing catalogs
pub struct ListCatalogsBuilder {
    client: CatalogServiceClient,
    request: ListCatalogsRequest,
}
impl ListCatalogsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CatalogServiceClient`.
    pub(crate) fn new(client: CatalogServiceClient) -> Self {
        let request = ListCatalogsRequest {
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
    pub fn into_stream(self) -> BoxStr<'static, Result<Catalog>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_catalogs(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.catalogs.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.catalogs.into_iter().map(Ok)))
        .try_flatten();
        stream.boxed()
    }
}
impl IntoFuture for ListCatalogsBuilder {
    type Output = Result<ListCatalogsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_catalogs(&request).await })
    }
}
/// Builder for creating a catalog
pub struct CreateCatalogBuilder {
    client: CatalogServiceClient,
    request: CreateCatalogRequest,
}
impl CreateCatalogBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CatalogServiceClient`.
    pub(crate) fn new(client: CatalogServiceClient, name: impl Into<String>) -> Self {
        let request = CreateCatalogRequest {
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
    /// A map of key-value properties attached to the securable.
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
    /// Storage root URL for managed tables within catalog.
    pub fn with_storage_root(mut self, storage_root: impl Into<Option<String>>) -> Self {
        self.request.storage_root = storage_root.into();
        self
    }
    /** The name of delta sharing provider.

    A Delta Sharing catalog is a catalog that is based on a Delta share on a remote sharing server.*/
    pub fn with_provider_name(mut self, provider_name: impl Into<Option<String>>) -> Self {
        self.request.provider_name = provider_name.into();
        self
    }
    /// The name of the share under the share provider.
    pub fn with_share_name(mut self, share_name: impl Into<Option<String>>) -> Self {
        self.request.share_name = share_name.into();
        self
    }
}
impl IntoFuture for CreateCatalogBuilder {
    type Output = Result<Catalog>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_catalog(&request).await })
    }
}
/// Builder for getting a catalog
pub struct GetCatalogBuilder {
    client: CatalogServiceClient,
    request: GetCatalogRequest,
}
impl GetCatalogBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CatalogServiceClient`.
    pub(crate) fn new(client: CatalogServiceClient, name: impl Into<String>) -> Self {
        let request = GetCatalogRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Whether to include catalogs in the response for which the principal can only access selective metadata for
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
}
impl IntoFuture for GetCatalogBuilder {
    type Output = Result<Catalog>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_catalog(&request).await })
    }
}
/// Builder for updating a catalog
pub struct UpdateCatalogBuilder {
    client: CatalogServiceClient,
    request: UpdateCatalogRequest,
}
impl UpdateCatalogBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CatalogServiceClient`.
    pub(crate) fn new(client: CatalogServiceClient, name: impl Into<String>) -> Self {
        let request = UpdateCatalogRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Username of new owner of catalog.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    /// User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /** A map of key-value properties attached to the securable.

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
    /// Name of catalog.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
}
impl IntoFuture for UpdateCatalogBuilder {
    type Output = Result<Catalog>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_catalog(&request).await })
    }
}
/// Builder for deleting a catalog
pub struct DeleteCatalogBuilder {
    client: CatalogServiceClient,
    request: DeleteCatalogRequest,
}
impl DeleteCatalogBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CatalogServiceClient`.
    pub(crate) fn new(client: CatalogServiceClient, name: impl Into<String>) -> Self {
        let request = DeleteCatalogRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Force deletion even if the catalog is not empty.
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
}
impl IntoFuture for DeleteCatalogBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_catalog(&request).await })
    }
}
