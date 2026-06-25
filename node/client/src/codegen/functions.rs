// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::FunctionClient;
use unitycatalog_common::models::functions::v1::*;
#[napi]
pub struct NapiFunctionClient {
    pub(crate) client: FunctionClient,
}
#[napi]
impl NapiFunctionClient {
    #[napi(catch_unwind)]
    pub async fn get(&self) -> napi::Result<Buffer> {
        let mut request = self.client.get();
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn update(&self, owner: Option<String>) -> napi::Result<Buffer> {
        let mut request = self.client.update();
        request = request.with_owner(owner);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn delete(&self, force: Option<bool>) -> napi::Result<()> {
        let mut request = self.client.delete();
        request = request.with_force(force);
        request.await.default_error()
    }
}
impl NapiFunctionClient {
    pub fn new(client: FunctionClient) -> Self {
        Self { client }
    }
}
