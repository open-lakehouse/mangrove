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
        url: Option<String>,
        credential_name: Option<String>,
        read_only: Option<bool>,
        owner: Option<String>,
        comment: Option<String>,
        new_name: Option<String>,
        force: Option<bool>,
        skip_validation: Option<bool>,
    ) -> napi::Result<Buffer> {
        let request = self
            .client
            .update()
            .with_url(url)
            .with_credential_name(credential_name)
            .with_read_only(read_only)
            .with_owner(owner)
            .with_comment(comment)
            .with_new_name(new_name)
            .with_force(force)
            .with_skip_validation(skip_validation);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn delete(&self, force: Option<bool>) -> napi::Result<()> {
        let request = self.client.delete().with_force(force);
        request.await.default_error()
    }
}

impl NapiExternalLocationClient {
    pub fn new(client: ExternalLocationClient) -> Self {
        Self { client }
    }
}
