// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::AgentClient;
use unitycatalog_common::models::agents::v0alpha1::*;
#[napi]
pub struct NapiAgentClient {
    pub(crate) client: AgentClient,
}
#[napi]
impl NapiAgentClient {
    #[napi(catch_unwind)]
    pub async fn get(&self, include_browse: Option<bool>) -> napi::Result<Buffer> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn update(
        &self,
        new_name: Option<String>,
        invocation_protocol: Option<i32>,
        endpoint: Option<String>,
        description: Option<String>,
        capabilities: Option<Vec<String>>,
        input_schema: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_invocation_protocol(
            invocation_protocol.and_then(<InvocationProtocol as buffa::Enumeration>::from_i32),
        );
        request = request.with_endpoint(endpoint);
        request = request.with_description(description);
        if let Some(capabilities) = capabilities {
            request = request.with_capabilities(capabilities);
        }
        request = request.with_input_schema(input_schema);
        request = request.with_comment(comment);
        request = request.with_owner(owner);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn delete(&self) -> napi::Result<()> {
        let mut request = self.client.delete();
        request.await.default_error()
    }
}
impl NapiAgentClient {
    pub fn new(client: AgentClient) -> Self {
        Self { client }
    }
}
