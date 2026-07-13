// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
use crate::error::NapiErrorExt;
use buffa::Message;
use napi::bindgen_prelude::Buffer;
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::StagingTableClient;
use unitycatalog_common::models::staging_tables::v1::*;
#[napi]
pub struct NapiStagingTableClient {
    pub(crate) client: StagingTableClient,
}
#[napi]
impl NapiStagingTableClient {}
impl NapiStagingTableClient {
    pub fn new(client: StagingTableClient) -> Self {
        Self { client }
    }
}
