// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::TableClient;
use unitycatalog_common::models::tables::v1::*;
#[napi]
pub struct NapiTableClient {
    pub(crate) client: TableClient,
}
#[napi]
impl NapiTableClient {
    #[napi(catch_unwind)]
    pub async fn get(
        &self,
        include_delta_metadata: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.get();
        request = request.with_include_delta_metadata(include_delta_metadata);
        request = request.with_include_browse(include_browse);
        request = request.with_include_manifest_capabilities(include_manifest_capabilities);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn get_table_exists(&self) -> napi::Result<Buffer> {
        let mut request = self.client.get_table_exists();
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
impl NapiTableClient {
    pub fn new(client: TableClient) -> Self {
        Self { client }
    }
}
