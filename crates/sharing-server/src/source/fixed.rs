//! A [`CatalogSource`] whose assets are declared up front (in config).
//!
//! For standalone deployments without a catalog, demos and tests: each table and
//! volume is listed with its full name and storage location. A static source
//! cannot vend credentials, so it is paired with local storage access.

use std::collections::BTreeMap;

use async_trait::async_trait;
use open_sharing_api::catalog::DataFormat;
use serde::{Deserialize, Serialize};

use super::{CatalogSource, SourceTable, SourceVolume, split_name};
use crate::{Error, Result};

/// A declared table.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct StaticTable {
    pub location: String,
    /// The table format; only `delta` is served so far.
    #[serde(default = "delta")]
    pub format: String,
}

fn delta() -> String {
    "delta".to_string()
}

/// A declared volume.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct StaticVolume {
    pub location: String,
}

/// Assets declared up front, keyed by full name (`catalog.schema.name`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
pub struct StaticSource {
    #[serde(default)]
    pub tables: BTreeMap<String, StaticTable>,
    #[serde(default)]
    pub volumes: BTreeMap<String, StaticVolume>,
}

impl StaticSource {
    fn table(full_name: &str, table: &StaticTable) -> Result<SourceTable> {
        let [_, _, name] = split_name::<3>(full_name)?;
        let format = match table.format.to_ascii_lowercase().as_str() {
            "delta" => DataFormat::Delta,
            "iceberg" => DataFormat::Iceberg,
            other => {
                return Err(Error::invalid_argument(format!(
                    "table '{full_name}' has unsupported format '{other}'"
                )));
            }
        };
        Ok(SourceTable {
            full_name: full_name.to_string(),
            name: name.to_string(),
            id: None,
            location: table.location.clone(),
            format,
        })
    }
}

#[async_trait]
impl CatalogSource for StaticSource {
    async fn get_table(&self, full_name: &str) -> Result<SourceTable> {
        let table = self.tables.get(full_name).ok_or(Error::NotFound)?;
        Self::table(full_name, table)
    }

    async fn list_tables(&self, catalog: &str, schema: &str) -> Result<Vec<SourceTable>> {
        let prefix = format!("{catalog}.{schema}.");
        self.tables
            .iter()
            .filter(|(name, _)| {
                name.strip_prefix(&prefix)
                    .is_some_and(|rest| !rest.is_empty() && !rest.contains('.'))
            })
            .map(|(name, table)| Self::table(name, table))
            .collect()
    }

    async fn get_volume(&self, full_name: &str) -> Result<SourceVolume> {
        let volume = self.volumes.get(full_name).ok_or(Error::NotFound)?;
        Ok(SourceVolume {
            full_name: full_name.to_string(),
            id: None,
            location: volume.location.clone(),
        })
    }
}
