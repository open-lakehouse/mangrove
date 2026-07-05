// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use prost::Message;
use std::collections::HashMap;
use unitycatalog_client::PolicyClient;
use unitycatalog_common::models::policies::v1::*;
#[napi]
pub struct NapiPolicyClient {
    pub(crate) client: PolicyClient,
}
#[napi]
impl NapiPolicyClient {
    #[napi(catch_unwind)]
    pub async fn create_policy(
        &self,
        policy_info: napi::bindgen_prelude::Buffer,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_policy(
            <PolicyInfo as prost::Message>::decode(policy_info.as_ref()).map_err(|e| {
                napi::Error::new(
                    napi::Status::GenericFailure,
                    format!("invalid {} payload: {e}", stringify!(PolicyInfo)),
                )
            })?,
        );
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
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
        policy_info: napi::bindgen_prelude::Buffer,
        update_mask: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update(
            <PolicyInfo as prost::Message>::decode(policy_info.as_ref()).map_err(|e| {
                napi::Error::new(
                    napi::Status::GenericFailure,
                    format!("invalid {} payload: {e}", stringify!(PolicyInfo)),
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
impl NapiPolicyClient {
    pub fn new(client: PolicyClient) -> Self {
        Self { client }
    }
}
