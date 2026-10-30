// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
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
        policy_type: Option<i32>,
        comment: Option<String>,
        to_principals: Option<Vec<String>>,
        except_principals: Option<Vec<String>>,
        for_securable_type: Option<String>,
        when_condition: Option<String>,
        update_mask: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update();
        request = request
            .with_policy_type(policy_type.and_then(<PolicyType as buffa::Enumeration>::from_i32));
        request = request.with_comment(comment);
        if let Some(to_principals) = to_principals {
            request = request.with_to_principals(to_principals);
        }
        if let Some(except_principals) = except_principals {
            request = request.with_except_principals(except_principals);
        }
        request = request.with_for_securable_type(for_securable_type);
        request = request.with_when_condition(when_condition);
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
