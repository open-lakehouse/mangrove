// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::RecipientClient;
use unitycatalog_common::models::recipients::v1::*;
#[napi]
pub struct NapiRecipientClient {
    pub(crate) client: RecipientClient,
}
#[napi]
impl NapiRecipientClient {
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
        properties: Option<HashMap<String, String>>,
        expiration_time: Option<i64>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_expiration_time(expiration_time);
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
impl NapiRecipientClient {
    pub fn new(client: RecipientClient) -> Self {
        Self { client }
    }
}
