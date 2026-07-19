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
use unitycatalog_common::models::functions::v1::*;
/// Builder for listing functions
pub struct ListFunctionsBuilder {
    client: FunctionServiceClient,
    request: ListFunctionsRequest,
}
impl ListFunctionsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `FunctionServiceClient`.
    pub(crate) fn new(
        client: FunctionServiceClient,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> Self {
        let request = ListFunctionsRequest {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
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
    /// Whether to include functions in the response for which the principal can only access selective metadata for.
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStr<'static, Result<Function>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_functions(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.functions.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.functions.into_iter().map(Ok)))
        .try_flatten();
        #[cfg(not(target_arch = "wasm32"))]
        let stream = stream.boxed();
        #[cfg(target_arch = "wasm32")]
        let stream = stream.boxed_local();
        stream
    }
}
impl IntoFuture for ListFunctionsBuilder {
    type Output = Result<ListFunctionsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_functions(&request).await })
    }
}
/// Builder for creating a function
pub struct CreateFunctionBuilder {
    client: FunctionServiceClient,
    request: CreateFunctionRequest,
}
impl CreateFunctionBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `FunctionServiceClient`.
    pub(crate) fn new(client: FunctionServiceClient, function_info: CreateFunction) -> Self {
        let request = CreateFunctionRequest {
            function_info: buffa::MessageField::some(function_info),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for CreateFunctionBuilder {
    type Output = Result<Function>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_function(&request).await })
    }
}
/// Builder for getting a function
pub struct GetFunctionBuilder {
    client: FunctionServiceClient,
    request: GetFunctionRequest,
}
impl GetFunctionBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `FunctionServiceClient`.
    pub(crate) fn new(client: FunctionServiceClient, name: impl Into<String>) -> Self {
        let request = GetFunctionRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetFunctionBuilder {
    type Output = Result<Function>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_function(&request).await })
    }
}
/// Builder for updating a function
pub struct UpdateFunctionBuilder {
    client: FunctionServiceClient,
    request: UpdateFunctionRequest,
}
impl UpdateFunctionBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `FunctionServiceClient`.
    pub(crate) fn new(client: FunctionServiceClient, name: impl Into<String>) -> Self {
        let request = UpdateFunctionRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Username of new owner of the function.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
}
impl IntoFuture for UpdateFunctionBuilder {
    type Output = Result<Function>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_function(&request).await })
    }
}
/// Builder for deleting a function
pub struct DeleteFunctionBuilder {
    client: FunctionServiceClient,
    request: DeleteFunctionRequest,
}
impl DeleteFunctionBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `FunctionServiceClient`.
    pub(crate) fn new(client: FunctionServiceClient, name: impl Into<String>) -> Self {
        let request = DeleteFunctionRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Force deletion even if the function is not empty.
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
}
impl IntoFuture for DeleteFunctionBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_function(&request).await })
    }
}
