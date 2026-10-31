// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]

use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::CredentialClient;
use unitycatalog_common::models::credentials::v1::*;

#[napi]
pub struct NapiCredentialClient {
    pub(crate) client: CredentialClient,
}

#[napi]
impl NapiCredentialClient {
    #[napi(catch_unwind)]
    pub async fn get(&self) -> napi::Result<Buffer> {
        let request = self.client.get();
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }

    #[napi(catch_unwind)]
    pub async fn update(
        &self,
        new_name: Option<String>,
        comment: Option<String>,
        read_only: Option<bool>,
        owner: Option<String>,
        skip_validation: Option<bool>,
        force: Option<bool>,
    ) -> napi::Result<Buffer> {
        let request = self
            .client
            .update()
            .with_new_name(new_name)
            .with_comment(comment)
            .with_read_only(read_only)
            .with_owner(owner)
            .with_skip_validation(skip_validation)
            .with_force(force);
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

impl NapiCredentialClient {
    pub fn new(client: CredentialClient) -> Self {
        Self { client }
    }
}
