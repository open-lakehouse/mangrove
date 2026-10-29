//! A Unity Catalog REST server as a [`CatalogSource`].
//!
//! Works against any server speaking the UC REST API — this repository's
//! `uc-server`, Unity Catalog OSS or Databricks. Delegated access goes through
//! the catalog's `temporary-table-credentials` / `temporary-volume-credentials`
//! endpoints, so the catalog's own authorization and downscoping apply.

use async_trait::async_trait;
use futures::TryStreamExt;
use open_sharing_api::catalog::DataFormat;
use unitycatalog_client::{
    TableOperation, TableReference, UnityCatalogClient, VolumeOperation, VolumeReference,
};
use unitycatalog_common::models::tables::v1::{DataSourceFormat, GetTableRequest, Table};
use unitycatalog_common::models::volumes::v1::GetVolumeRequest;
use unitycatalog_credentials::TemporaryCredential;
use url::Url;

use crate::source::{CatalogSource, SourceTable, SourceVolume, split_name};
use crate::{Error, Result};

/// A Unity Catalog REST server as a catalog source.
pub struct UnitySource {
    client: UnityCatalogClient,
}

impl UnitySource {
    /// A source for the UC REST API at `base_url` (`…/api/2.1/unity-catalog/`),
    /// authenticated with `token` when given.
    pub fn new(base_url: Url, token: Option<String>) -> Self {
        let client = match token {
            Some(token) => UnityCatalogClient::new_with_token(base_url, token),
            None => UnityCatalogClient::new_unauthenticated(base_url),
        };
        Self { client }
    }

    pub fn from_client(client: UnityCatalogClient) -> Self {
        Self { client }
    }
}

fn client_error(e: unitycatalog_client::Error) -> Error {
    if e.is_not_found() {
        Error::NotFound
    } else {
        Error::generic(format!("catalog source: {e}"))
    }
}

fn table(table: Table) -> Result<SourceTable> {
    let format = match table.data_source_format.as_known() {
        Some(DataSourceFormat::DELTA) => DataFormat::Delta,
        Some(DataSourceFormat::ICEBERG) => DataFormat::Iceberg,
        other => {
            return Err(Error::invalid_argument(format!(
                "table '{}' has unsupported format {other:?}",
                table.full_name
            )));
        }
    };
    let location = table.storage_location.ok_or_else(|| {
        Error::invalid_argument(format!(
            "table '{}' has no storage location",
            table.full_name
        ))
    })?;
    Ok(SourceTable {
        full_name: table.full_name,
        name: table.name,
        id: table.table_id,
        location,
        format,
    })
}

fn table_id(table: &SourceTable) -> Result<TableReference> {
    match &table.id {
        Some(id) => uuid::Uuid::parse_str(id)
            .map(TableReference::Id)
            .map_err(|_| Error::generic(format!("table '{}' has a malformed id", table.full_name))),
        None => Ok(TableReference::Name(table.full_name.clone())),
    }
}

#[async_trait]
impl CatalogSource for UnitySource {
    async fn get_table(&self, full_name: &str) -> Result<SourceTable> {
        split_name::<3>(full_name)?;
        let info = self
            .client
            .tables_client()
            .get_table(&GetTableRequest {
                full_name: full_name.to_string(),
                include_browse: Some(false),
                include_delta_metadata: Some(false),
                include_manifest_capabilities: Some(false),
                ..Default::default()
            })
            .await
            .map_err(client_error)?;
        table(info)
    }

    async fn list_tables(&self, catalog: &str, schema: &str) -> Result<Vec<SourceTable>> {
        let tables: Vec<Table> = self
            .client
            .list_tables(catalog, schema)
            .into_stream()
            .try_collect()
            .await
            .map_err(client_error)?;
        // Views and tables in formats the server cannot serve are not shareable;
        // skip them rather than failing the whole schema.
        let mut tables: Vec<_> = tables.into_iter().filter_map(|t| table(t).ok()).collect();
        tables.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(tables)
    }

    async fn get_volume(&self, full_name: &str) -> Result<SourceVolume> {
        split_name::<3>(full_name)?;
        let volume = self
            .client
            .volumes_client()
            .get_volume(&GetVolumeRequest {
                name: full_name.to_string(),
                include_browse: Some(false),
                ..Default::default()
            })
            .await
            .map_err(client_error)?;
        Ok(SourceVolume {
            full_name: volume.full_name,
            id: Some(volume.volume_id).filter(|id| !id.is_empty()),
            location: volume.storage_location,
        })
    }

    async fn vend_table(&self, table: &SourceTable) -> Result<TemporaryCredential> {
        let (credential, _) = self
            .client
            .temporary_credentials()
            .temporary_table_credential(table_id(table)?, TableOperation::Read)
            .await
            .map_err(client_error)?;
        let mut credential = TemporaryCredential::from(credential);
        if credential.url.is_empty() {
            credential.url = table.location.clone();
        }
        Ok(credential)
    }

    async fn vend_volume(&self, volume: &SourceVolume) -> Result<TemporaryCredential> {
        let reference = match &volume.id {
            Some(id) => uuid::Uuid::parse_str(id)
                .map(VolumeReference::Id)
                .map_err(|_| {
                    Error::generic(format!("volume '{}' has a malformed id", volume.full_name))
                })?,
            None => VolumeReference::Name(volume.full_name.clone()),
        };
        let (credential, _) = self
            .client
            .temporary_credentials()
            .temporary_volume_credential(reference, VolumeOperation::Read)
            .await
            .map_err(client_error)?;
        let mut credential = TemporaryCredential::from(credential);
        if credential.url.is_empty() {
            credential.url = volume.location.clone();
        }
        Ok(credential)
    }
}
