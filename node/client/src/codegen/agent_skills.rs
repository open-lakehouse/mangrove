// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]

use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::AgentSkillClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;

#[napi]
pub struct NapiAgentSkillClient {
    pub(crate) client: AgentSkillClient,
}

#[napi]
impl NapiAgentSkillClient {
    #[napi(catch_unwind)]
    pub async fn get(&self, include_browse: Option<bool>) -> napi::Result<Buffer> {
        let request = self.client.get().with_include_browse(include_browse);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn update(
        &self,
        new_name: Option<String>,
        description: Option<String>,
        allowed_tools: Option<Vec<String>>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> napi::Result<Buffer> {
        let request = self
            .client
            .update()
            .with_new_name(new_name)
            .with_description(description)
            .with_allowed_tools(allowed_tools.unwrap_or_default())
            .with_comment(comment)
            .with_owner(owner);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn delete(&self) -> napi::Result<()> {
        let request = self.client.delete();
        request.await.default_error()
    }
}

impl NapiAgentSkillClient {
    pub fn new(client: AgentSkillClient) -> Self {
        Self { client }
    }
}
