// @generated — do not edit by hand.
#![allow(unused_mut)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
use super::client::*;
use crate::Result;
use std::future::IntoFuture;
use unitycatalog_sharing_client::models::open_sharing::v1::*;
/// Builder for skills
pub struct ListSkillsBuilder {
    client: SharingSkillClient,
    request: ListSkillsRequest,
}
impl ListSkillsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingSkillClient`.
    pub(crate) fn new(
        client: SharingSkillClient,
        share: impl Into<String>,
        schema: impl Into<String>,
    ) -> Self {
        let request = ListSkillsRequest {
            share: share.into(),
            schema: schema.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    /// Specifies a page token to use, from a previous response's next_page_token.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
}
impl IntoFuture for ListSkillsBuilder {
    type Output = Result<ListSkillsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_skills(&request).await })
    }
}
/// Builder for all skills
pub struct ListAllSkillsBuilder {
    client: SharingSkillClient,
    request: ListAllSkillsRequest,
}
impl ListAllSkillsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingSkillClient`.
    pub(crate) fn new(client: SharingSkillClient, share: impl Into<String>) -> Self {
        let request = ListAllSkillsRequest {
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
    /// Specifies a page token to use, from a previous response's next_page_token.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
}
impl IntoFuture for ListAllSkillsBuilder {
    type Output = Result<ListAllSkillsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_all_skills(&request).await })
    }
}
/// Builder for skill
pub struct GetSkillBuilder {
    client: SharingSkillClient,
    request: GetSkillRequest,
}
impl GetSkillBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingSkillClient`.
    pub(crate) fn new(
        client: SharingSkillClient,
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let request = GetSkillRequest {
            share: share.into(),
            schema: schema.into(),
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetSkillBuilder {
    type Output = Result<SharingSkill>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_skill(&request).await })
    }
}
/// Builder for temporary skill credentials
pub struct GenerateTemporarySkillCredentialsBuilder {
    client: SharingSkillClient,
    request: GenerateTemporarySkillCredentialsRequest,
}
impl GenerateTemporarySkillCredentialsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `SharingSkillClient`.
    pub(crate) fn new(
        client: SharingSkillClient,
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
    ) -> Self {
        let request = GenerateTemporarySkillCredentialsRequest {
            share: share.into(),
            schema: schema.into(),
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GenerateTemporarySkillCredentialsBuilder {
    type Output = Result<SharingTemporaryCredentials>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.generate_temporary_skill_credentials(&request).await })
    }
}
