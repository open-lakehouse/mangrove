// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::TagPolicyClient;
use unitycatalog_common::models::tags::v1::*;
#[napi]
pub struct NapiTagPolicyClient {
    pub(crate) client: TagPolicyClient,
}
#[napi]
impl NapiTagPolicyClient {
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
        tag_policy: napi::bindgen_prelude::Buffer,
        update_mask: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update(
            <TagPolicy as buffa::Message>::decode_from_slice(tag_policy.as_ref()).map_err(|e| {
                napi::Error::new(
                    napi::Status::GenericFailure,
                    format!("invalid {} payload: {e}", stringify!(TagPolicy)),
                )
            })?,
        );
        request = request.with_update_mask(update_mask);
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
impl NapiTagPolicyClient {
    pub fn new(client: TagPolicyClient) -> Self {
        Self { client }
    }
}
