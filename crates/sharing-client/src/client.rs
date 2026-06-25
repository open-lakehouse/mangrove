use chrono::{DateTime, Utc};
use futures::stream::BoxStream;
use futures::{StreamExt, TryStreamExt};
use itertools::Itertools;
use olai_http::CloudClient;

use super::utils::stream_paginated;
use crate::codegen::sharing::SharingClient;
use crate::codegen::sharing::builders::GetShareBuilder;
use crate::models::open_sharing::v1::*;
use crate::models::{MetadataResponse, MetadataResponseData, ProtocolResponseData};
use crate::{Error, Result};

#[derive(Clone)]
pub struct DeltaSharingClient {
    client: CloudClient,
    base_url: url::Url,
    discovery: SharingClient,
}

impl DeltaSharingClient {
    pub fn new(client: CloudClient, base_url: url::Url) -> Self {
        let base_url = base_url.join("api/v1/delta-sharing/").unwrap();
        Self {
            discovery: SharingClient::new(client.clone(), base_url.clone()),
            client,
            base_url,
        }
    }

    pub fn new_with_prefix(
        client: CloudClient,
        base_url: url::Url,
        prefix: impl Into<String>,
    ) -> Self {
        let prefix = prefix.into();
        let base_url = if !prefix.ends_with('/') {
            base_url.join(&format!("{}/", prefix)).unwrap()
        } else {
            base_url.join(&prefix).unwrap()
        };
        Self {
            discovery: SharingClient::new(client.clone(), base_url.clone()),
            client,
            base_url,
        }
    }

    pub fn list_shares(&self, max_results: impl Into<Option<i32>>) -> BoxStream<'_, Result<Share>> {
        let max_results = max_results.into();
        stream_paginated(max_results, move |mut max_results, page_token| async move {
            let request = ListSharesRequest {
                max_results,
                page_token,
                ..Default::default()
            };
            let res = self.discovery.list_shares(&request).await?;

            // Update max_results for next page based on items received
            if let Some(ref mut remaining) = max_results {
                *remaining -= res.items.len() as i32;
                if *remaining <= 0 {
                    max_results = Some(0);
                }
            }

            Ok((res.items, max_results, res.next_page_token))
        })
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub fn get_share(&self, name: impl Into<String>) -> GetShareBuilder {
        GetShareBuilder::new(self.discovery.clone(), name)
    }

    pub fn list_share_schemas(
        &self,
        share: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<Schema>> {
        let share = share.into();
        let max_results = max_results.into();
        stream_paginated(
            (share, max_results),
            move |(share, mut max_results), page_token| async move {
                let request = ListSchemasRequest {
                    share: share.clone(),
                    max_results,
                    page_token,
                    ..Default::default()
                };
                let res = self
                    .discovery
                    .list_schemas(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;

                // Update max_results for next page based on items received
                if let Some(ref mut remaining) = max_results {
                    *remaining -= res.items.len() as i32;
                    if *remaining <= 0 {
                        max_results = Some(0);
                    }
                }

                Ok((res.items, (share, max_results), res.next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub fn list_all_tables(
        &self,
        share: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<Table>> {
        let share = share.into();
        let max_results = max_results.into();
        stream_paginated(
            (share, max_results),
            move |(share, mut max_results), page_token| async move {
                let request = ListAllTablesRequest {
                    name: share.clone(),
                    max_results,
                    page_token,
                    ..Default::default()
                };
                let res = self
                    .discovery
                    .list_all_tables(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;

                // Update max_results for next page based on items received
                if let Some(ref mut remaining) = max_results {
                    *remaining -= res.items.len() as i32;
                    if *remaining <= 0 {
                        max_results = Some(0);
                    }
                }

                Ok((res.items, (share, max_results), res.next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub fn list_schema_tables(
        &self,
        share: impl Into<String>,
        schema: impl Into<String>,
        max_results: impl Into<Option<i32>>,
    ) -> BoxStream<'_, Result<Table>> {
        let share = share.into();
        let schema = schema.into();
        let max_results = max_results.into();
        stream_paginated(
            (share, schema, max_results),
            move |(share, schema, mut max_results), page_token| async move {
                let request = ListTablesRequest {
                    share: share.clone(),
                    name: schema.clone(),
                    max_results,
                    page_token,
                    ..Default::default()
                };
                let res = self
                    .discovery
                    .list_tables(&request)
                    .await
                    .map_err(|e| Error::generic(e.to_string()))?;

                // Update max_results for next page based on items received
                if let Some(ref mut remaining) = max_results {
                    *remaining -= res.items.len() as i32;
                    if *remaining <= 0 {
                        max_results = Some(0);
                    }
                }

                Ok((res.items, (share, schema, max_results), res.next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.into_iter().map(Ok)))
        .try_flatten()
        .boxed()
    }

    pub async fn get_table_version(
        &self,
        share: impl Into<String>,
        schema: impl Into<String>,
        table: impl Into<String>,
        starting_timestamp: Option<DateTime<Utc>>,
    ) -> Result<u64> {
        let mut url = self.base_url.join(&format!(
            "shares/{}/schemas/{}/tables/{}/version",
            share.into(),
            schema.into(),
            table.into()
        ))?;
        if let Some(ts) = starting_timestamp {
            url.query_pairs_mut()
                .append_pair("startingTimestamp", &ts.to_rfc3339());
        }
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let headers = response.headers();
        let version = headers
            .get("Delta-Table-Version")
            .ok_or(Error::generic("Delta-Table-Version header not found"))?;
        let version = version
            .to_str()
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .ok_or(Error::generic("Invalid version header"))?;
        Ok(version)
    }

    pub async fn get_table_metadata(
        &self,
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
    ) -> Result<(ProtocolResponseData, MetadataResponseData)> {
        let url = self.base_url.join(&format!(
            "shares/{}/schemas/{}/tables/{}/metadata",
            share.into(),
            schema.into(),
            name.into()
        ))?;
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        let result = response.bytes().await?;
        // split newlines and parse each as json
        let line_data: Vec<MetadataResponse> = result
            .split(|c| *c == b'\n')
            .map(::serde_json::from_slice)
            .try_collect()?;
        let mut protocol = None;
        let mut metadata = None;
        for item in line_data {
            match item {
                MetadataResponse::Protocol(p) => protocol = Some(p),
                MetadataResponse::MetaData(m) => metadata = Some(m),
            }
        }
        if protocol.is_none() {
            return Err(Error::generic("Protocol not found"));
        }
        if metadata.is_none() {
            return Err(Error::generic("Metadata not found"));
        }
        Ok((protocol.unwrap(), metadata.unwrap()))
    }

    /// Query a table's data files, returning the raw newline-delimited JSON
    /// response (Protocol + Metadata + file entries).
    ///
    /// The query path is served as `application/x-ndjson`, which the generated
    /// JSON client cannot model, so this issues the request directly.
    pub async fn query_table(
        &self,
        share: impl Into<String>,
        schema: impl Into<String>,
        name: impl Into<String>,
        request: &QueryTableRequest,
    ) -> Result<Vec<u8>> {
        let url = self.base_url.join(&format!(
            "shares/{}/schemas/{}/tables/{}/query",
            share.into(),
            schema.into(),
            name.into()
        ))?;
        let response = self.client.post(url).json(request).send().await?;
        response.error_for_status_ref()?;
        Ok(response.bytes().await?.to_vec())
    }
}
