// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::ExternalLocationClient;
use unitycatalog_common::models::external_locations::v1::*;
#[napi]
pub struct NapiExternalLocationClient {
    pub(crate) client: ExternalLocationClient,
}
#[napi]
impl NapiExternalLocationClient {
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
        url: Option<String>,
        credential_name: Option<String>,
        read_only: Option<bool>,
        owner: Option<String>,
        comment: Option<String>,
        new_name: Option<String>,
        force: Option<bool>,
        skip_validation: Option<bool>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update();
        request = request.with_url(url);
        request = request.with_credential_name(credential_name);
        request = request.with_read_only(read_only);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        request = request.with_new_name(new_name);
        request = request.with_force(force);
        request = request.with_skip_validation(skip_validation);
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
impl NapiExternalLocationClient {
    pub fn new(client: ExternalLocationClient) -> Self {
        Self { client }
    }
}
