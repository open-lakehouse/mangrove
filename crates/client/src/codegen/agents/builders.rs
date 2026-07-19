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
use unitycatalog_common::models::agents::v0alpha1::*;
/// Builder for listing agents
pub struct ListAgentsBuilder {
    client: AgentServiceClient,
    request: ListAgentsRequest,
}
impl ListAgentsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentServiceClient`.
    pub(crate) fn new(
        client: AgentServiceClient,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> Self {
        let request = ListAgentsRequest {
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
    /** Whether to include agents in the response for which the principal can only
    access selective metadata for.*/
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStr<'static, Result<Agent>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_agents(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.agents.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.agents.into_iter().map(Ok)))
        .try_flatten();
        #[cfg(not(target_arch = "wasm32"))]
        let stream = stream.boxed();
        #[cfg(target_arch = "wasm32")]
        let stream = stream.boxed_local();
        stream
    }
}
impl IntoFuture for ListAgentsBuilder {
    type Output = Result<ListAgentsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_agents(&request).await })
    }
}
/// Builder for creating a agent
pub struct CreateAgentBuilder {
    client: AgentServiceClient,
    request: CreateAgentRequest,
}
impl CreateAgentBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentServiceClient`.
    pub(crate) fn new(
        client: AgentServiceClient,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        invocation_protocol: InvocationProtocol,
        endpoint: impl Into<String>,
    ) -> Self {
        let request = CreateAgentRequest {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            name: name.into(),
            invocation_protocol: buffa::EnumValue::Known(invocation_protocol),
            endpoint: endpoint.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// An LLM-readable description of what the agent does and the inputs it expects.
    pub fn with_description(mut self, description: impl Into<Option<String>>) -> Self {
        self.request.description = description.into();
        self
    }
    /// Capability identifiers advertised by the agent.
    pub fn with_capabilities<I>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.request.capabilities = capabilities.into_iter().collect();
        self
    }
    /// A JSON Schema (encoded as a JSON string) describing the expected input.
    pub fn with_input_schema(mut self, input_schema: impl Into<Option<String>>) -> Self {
        self.request.input_schema = input_schema.into();
        self
    }
    /// User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
}
impl IntoFuture for CreateAgentBuilder {
    type Output = Result<Agent>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_agent(&request).await })
    }
}
/// Builder for getting a agent
pub struct GetAgentBuilder {
    client: AgentServiceClient,
    request: GetAgentRequest,
}
impl GetAgentBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentServiceClient`.
    pub(crate) fn new(client: AgentServiceClient, name: impl Into<String>) -> Self {
        let request = GetAgentRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /** Whether to include agents in the response for which the principal can only
    access selective metadata for.*/
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
}
impl IntoFuture for GetAgentBuilder {
    type Output = Result<Agent>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_agent(&request).await })
    }
}
/// Builder for updating a agent
pub struct UpdateAgentBuilder {
    client: AgentServiceClient,
    request: UpdateAgentRequest,
}
impl UpdateAgentBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentServiceClient`.
    pub(crate) fn new(client: AgentServiceClient, name: impl Into<String>) -> Self {
        let request = UpdateAgentRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// New name for the agent.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    /// The protocol a recipient uses to invoke the agent.
    pub fn with_invocation_protocol(
        mut self,
        invocation_protocol: impl Into<Option<InvocationProtocol>>,
    ) -> Self {
        self.request.invocation_protocol = invocation_protocol.into().map(buffa::EnumValue::Known);
        self
    }
    /// The agent's invocation endpoint URL.
    pub fn with_endpoint(mut self, endpoint: impl Into<Option<String>>) -> Self {
        self.request.endpoint = endpoint.into();
        self
    }
    /// Updated LLM-readable description.
    pub fn with_description(mut self, description: impl Into<Option<String>>) -> Self {
        self.request.description = description.into();
        self
    }
    /// Updated capability identifiers advertised by the agent.
    pub fn with_capabilities<I>(mut self, capabilities: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.request.capabilities = capabilities.into_iter().collect();
        self
    }
    /// Updated JSON Schema (encoded as a JSON string) describing the expected input.
    pub fn with_input_schema(mut self, input_schema: impl Into<Option<String>>) -> Self {
        self.request.input_schema = input_schema.into();
        self
    }
    /// The comment attached to the agent.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// The identifier of the user who owns the agent.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
}
impl IntoFuture for UpdateAgentBuilder {
    type Output = Result<Agent>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_agent(&request).await })
    }
}
/// Builder for deleting a agent
pub struct DeleteAgentBuilder {
    client: AgentServiceClient,
    request: DeleteAgentRequest,
}
impl DeleteAgentBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentServiceClient`.
    pub(crate) fn new(client: AgentServiceClient, name: impl Into<String>) -> Self {
        let request = DeleteAgentRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteAgentBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_agent(&request).await })
    }
}
