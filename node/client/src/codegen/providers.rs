// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::ProviderClient;
use unitycatalog_common::models::providers::v1::*;
#[napi]
pub struct NapiProviderClient {
    pub(crate) client: ProviderClient,
}
#[napi]
impl NapiProviderClient {
    #[napi(catch_unwind)]
    pub async fn get(&self) -> napi::Result<Buffer> {
        let mut request = self.client.get();
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn update(
        &self,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
        recipient_profile_str: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        request = request.with_recipient_profile_str(recipient_profile_str);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
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
impl NapiProviderClient {
    pub fn new(client: ProviderClient) -> Self {
        Self { client }
    }
}
