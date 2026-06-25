// @generated — do not edit by hand.
#![allow(unused_mut)]
#![allow(unused_imports)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
use super::client::*;
use crate::Result;
use std::future::IntoFuture;
use unitycatalog_common::models::staging_tables::v1::*;
/// Builder for creating a staging table
pub struct CreateStagingTableBuilder {
    client: StagingTableServiceClient,
    request: CreateStagingTableRequest,
}
impl CreateStagingTableBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `StagingTableServiceClient`.
    pub(crate) fn new(
        client: StagingTableServiceClient,
        name: impl Into<String>,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> Self {
        let request = CreateStagingTableRequest {
            name: name.into(),
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for CreateStagingTableBuilder {
    type Output = Result<StagingTable>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_staging_table(&request).await })
    }
}
