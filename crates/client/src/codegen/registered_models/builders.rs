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
use unitycatalog_common::models::registered_models::v1::*;
/// Builder for listing registered models
pub struct ListRegisteredModelsBuilder {
    client: RegisteredModelServiceClient,
    request: ListRegisteredModelsRequest,
}
impl ListRegisteredModelsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RegisteredModelServiceClient`.
    pub(crate) fn new(client: RegisteredModelServiceClient) -> Self {
        let request = ListRegisteredModelsRequest {
            ..Default::default()
        };
        Self { client, request }
    }
    /// Name of parent catalog for models of interest.
    pub fn with_catalog_name(mut self, catalog_name: impl Into<Option<String>>) -> Self {
        self.request.catalog_name = catalog_name.into();
        self
    }
    /// Name of parent schema for models of interest.
    pub fn with_schema_name(mut self, schema_name: impl Into<Option<String>>) -> Self {
        self.request.schema_name = schema_name.into();
        self
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
    /** Whether to include registered models in the response for which the principal
    can only access selective metadata for.*/
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStr<'static, Result<RegisteredModel>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder
                    .client
                    .list_registered_models(&builder.request)
                    .await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.registered_models.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.registered_models.into_iter().map(Ok)))
        .try_flatten();
        stream.boxed()
    }
}
impl IntoFuture for ListRegisteredModelsBuilder {
    type Output = Result<ListRegisteredModelsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_registered_models(&request).await })
    }
}
/// Builder for creating a registered model
pub struct CreateRegisteredModelBuilder {
    client: RegisteredModelServiceClient,
    request: CreateRegisteredModelRequest,
}
impl CreateRegisteredModelBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RegisteredModelServiceClient`.
    pub(crate) fn new(
        client: RegisteredModelServiceClient,
        name: impl Into<String>,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> Self {
        let request = CreateRegisteredModelRequest {
            name: name.into(),
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
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
impl IntoFuture for CreateRegisteredModelBuilder {
    type Output = Result<RegisteredModel>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_registered_model(&request).await })
    }
}
/// Builder for getting a registered model
pub struct GetRegisteredModelBuilder {
    client: RegisteredModelServiceClient,
    request: GetRegisteredModelRequest,
}
impl GetRegisteredModelBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RegisteredModelServiceClient`.
    pub(crate) fn new(client: RegisteredModelServiceClient, full_name: impl Into<String>) -> Self {
        let request = GetRegisteredModelRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /** Whether to include registered models in the response for which the principal
    can only access selective metadata for.*/
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
}
impl IntoFuture for GetRegisteredModelBuilder {
    type Output = Result<RegisteredModel>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_registered_model(&request).await })
    }
}
/// Builder for updating a registered model
pub struct UpdateRegisteredModelBuilder {
    client: RegisteredModelServiceClient,
    request: UpdateRegisteredModelRequest,
}
impl UpdateRegisteredModelBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RegisteredModelServiceClient`.
    pub(crate) fn new(client: RegisteredModelServiceClient, full_name: impl Into<String>) -> Self {
        let request = UpdateRegisteredModelRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// New name for the registered model.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    /// User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// Username of new owner of the registered model.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
}
impl IntoFuture for UpdateRegisteredModelBuilder {
    type Output = Result<RegisteredModel>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_registered_model(&request).await })
    }
}
/// Builder for deleting a registered model
pub struct DeleteRegisteredModelBuilder {
    client: RegisteredModelServiceClient,
    request: DeleteRegisteredModelRequest,
}
impl DeleteRegisteredModelBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `RegisteredModelServiceClient`.
    pub(crate) fn new(client: RegisteredModelServiceClient, full_name: impl Into<String>) -> Self {
        let request = DeleteRegisteredModelRequest {
            full_name: full_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Force deletion even if the registered model still has model versions.
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
}
impl IntoFuture for DeleteRegisteredModelBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_registered_model(&request).await })
    }
}
