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
use unitycatalog_common::models::agent_skills::v0alpha1::*;
/// Builder for listing agent skills
pub struct ListAgentSkillsBuilder {
    client: AgentSkillServiceClient,
    request: ListAgentSkillsRequest,
}
impl ListAgentSkillsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentSkillServiceClient`.
    pub(crate) fn new(
        client: AgentSkillServiceClient,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> Self {
        let request = ListAgentSkillsRequest {
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
    /** Whether to include agent skills in the response for which the principal can
    only access selective metadata for.*/
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStr<'static, Result<AgentSkill>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_agent_skills(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.agent_skills.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.agent_skills.into_iter().map(Ok)))
        .try_flatten();
        stream.boxed()
    }
}
impl IntoFuture for ListAgentSkillsBuilder {
    type Output = Result<ListAgentSkillsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_agent_skills(&request).await })
    }
}
/// Builder for creating a agent skill
pub struct CreateAgentSkillBuilder {
    client: AgentSkillServiceClient,
    request: CreateAgentSkillRequest,
}
impl CreateAgentSkillBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentSkillServiceClient`.
    pub(crate) fn new(
        client: AgentSkillServiceClient,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        agent_skill_type: AgentSkillType,
    ) -> Self {
        let request = CreateAgentSkillRequest {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            name: name.into(),
            agent_skill_type: agent_skill_type as i32,
            ..Default::default()
        };
        Self { client, request }
    }
    /** The storage location of the skill directory on the cloud.

    Required for EXTERNAL skills; ignored (server-derived) for MANAGED skills.*/
    pub fn with_storage_location(mut self, storage_location: impl Into<Option<String>>) -> Self {
        self.request.storage_location = storage_location.into();
        self
    }
    /// A human-readable description of what the skill does and when to use it.
    pub fn with_description(mut self, description: impl Into<Option<String>>) -> Self {
        self.request.description = description.into();
        self
    }
    /// SPDX license identifier or free-form license text for the skill.
    pub fn with_license(mut self, license: impl Into<Option<String>>) -> Self {
        self.request.license = license.into();
        self
    }
    /// The tools the skill is permitted to use.
    pub fn with_allowed_tools<I>(mut self, allowed_tools: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.request.allowed_tools = allowed_tools.into_iter().collect();
        self
    }
    /// Arbitrary additional metadata declared by the skill.
    pub fn with_metadata<I, K, V>(mut self, metadata: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.request.metadata = metadata
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }
    /// User-provided free-form text description.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
}
impl IntoFuture for CreateAgentSkillBuilder {
    type Output = Result<AgentSkill>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_agent_skill(&request).await })
    }
}
/// Builder for getting a agent skill
pub struct GetAgentSkillBuilder {
    client: AgentSkillServiceClient,
    request: GetAgentSkillRequest,
}
impl GetAgentSkillBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentSkillServiceClient`.
    pub(crate) fn new(client: AgentSkillServiceClient, name: impl Into<String>) -> Self {
        let request = GetAgentSkillRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /** Whether to include agent skills in the response for which the principal can
    only access selective metadata for.*/
    pub fn with_include_browse(mut self, include_browse: impl Into<Option<bool>>) -> Self {
        self.request.include_browse = include_browse.into();
        self
    }
}
impl IntoFuture for GetAgentSkillBuilder {
    type Output = Result<AgentSkill>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_agent_skill(&request).await })
    }
}
/// Builder for updating a agent skill
pub struct UpdateAgentSkillBuilder {
    client: AgentSkillServiceClient,
    request: UpdateAgentSkillRequest,
}
impl UpdateAgentSkillBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentSkillServiceClient`.
    pub(crate) fn new(client: AgentSkillServiceClient, name: impl Into<String>) -> Self {
        let request = UpdateAgentSkillRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// New name for the agent skill.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    /// Updated description of what the skill does and when to use it.
    pub fn with_description(mut self, description: impl Into<Option<String>>) -> Self {
        self.request.description = description.into();
        self
    }
    /// Updated tools the skill is permitted to use.
    pub fn with_allowed_tools<I>(mut self, allowed_tools: I) -> Self
    where
        I: IntoIterator<Item = String>,
    {
        self.request.allowed_tools = allowed_tools.into_iter().collect();
        self
    }
    /// The comment attached to the agent skill.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// The identifier of the user who owns the agent skill.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
}
impl IntoFuture for UpdateAgentSkillBuilder {
    type Output = Result<AgentSkill>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_agent_skill(&request).await })
    }
}
/// Builder for deleting a agent skill
pub struct DeleteAgentSkillBuilder {
    client: AgentSkillServiceClient,
    request: DeleteAgentSkillRequest,
}
impl DeleteAgentSkillBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `AgentSkillServiceClient`.
    pub(crate) fn new(client: AgentSkillServiceClient, name: impl Into<String>) -> Self {
        let request = DeleteAgentSkillRequest { name: name.into() };
        Self { client, request }
    }
}
impl IntoFuture for DeleteAgentSkillBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_agent_skill(&request).await })
    }
}
