// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::CatalogClient;
use unitycatalog_common::models::catalogs::v1::*;
#[napi]
pub struct NapiCatalogClient {
    pub(crate) client: CatalogClient,
}
#[napi]
impl NapiCatalogClient {
    #[napi(catch_unwind)]
    pub async fn get(&self, include_browse: Option<bool>) -> napi::Result<Buffer> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn update(
        &self,
        owner: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        new_name: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update();
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_new_name(new_name);
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
impl NapiCatalogClient {
    pub fn new(client: CatalogClient) -> Self {
        Self { client }
    }
}
