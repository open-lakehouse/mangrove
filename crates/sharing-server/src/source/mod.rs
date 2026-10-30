//! The [`CatalogSource`] port: where shared assets come from.
//!
//! A share names assets by their full name in a source catalog
//! (`catalog.schema.table`). A catalog source resolves those names to storage
//! locations and formats, expands shared schemas into their tables, and — when
//! it can vend credentials — supplies the short-lived, downscoped storage
//! credentials the server reads with and hands on to recipients (*delegated*
//! access), so the sharing server stores no storage secrets.
//!
//! Implementations: [`fixed::StaticSource`] (assets declared in config) and,
//! with the `unity` feature, `unity::source::UnitySource` (a Unity Catalog REST
//! server — this one or Databricks). Iceberg REST and other catalogs plug in
//! here.

use std::sync::Arc;

use async_trait::async_trait;
use olai_storage_credentials::VendedCredential;
use open_sharing_api::catalog::DataFormat;

use crate::{Error, Result};

pub mod fixed;

/// A table in a source catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceTable {
    /// `catalog.schema.table`.
    pub full_name: String,
    /// The table's name within its schema.
    pub name: String,
    /// The catalog's id for the table, when it has one.
    pub id: Option<String>,
    /// The table's storage location.
    pub location: String,
    pub format: DataFormat,
}

/// A volume (or agent skill, which sits on a volume) in a source catalog.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceVolume {
    /// `catalog.schema.volume`.
    pub full_name: String,
    pub id: Option<String>,
    pub location: String,
}

/// A source catalog. See the module docs.
#[async_trait]
pub trait CatalogSource: Send + Sync + 'static {
    /// A table by full name. Missing → [`Error::NotFound`].
    async fn get_table(&self, full_name: &str) -> Result<SourceTable>;

    /// The tables of `catalog.schema`, in a stable order.
    async fn list_tables(&self, catalog: &str, schema: &str) -> Result<Vec<SourceTable>>;

    /// A volume by full name. Missing → [`Error::NotFound`].
    async fn get_volume(&self, full_name: &str) -> Result<SourceVolume>;

    /// A read-only temporary credential for a table, for delegated access.
    async fn vend_table(&self, _table: &SourceTable) -> Result<VendedCredential> {
        Err(Error::NotImplemented(
            "this catalog source does not vend credentials",
        ))
    }

    /// A read-only temporary credential for a volume, for delegated access.
    async fn vend_volume(&self, _volume: &SourceVolume) -> Result<VendedCredential> {
        Err(Error::NotImplemented(
            "this catalog source does not vend credentials",
        ))
    }
}

/// How the server reaches a source's storage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AccessMode {
    /// Ask the source catalog for temporary credentials; store no secrets.
    #[default]
    Delegated,
    /// Use the server's own stored credentials (external locations).
    Local,
}

/// A registered source and the catalog names it serves.
#[derive(Clone)]
pub struct RegisteredSource {
    pub name: String,
    /// Catalog names this source serves; `*` serves every catalog.
    pub catalogs: Vec<String>,
    pub access: AccessMode,
    pub source: Arc<dyn CatalogSource>,
}

impl RegisteredSource {
    fn serves(&self, catalog: &str) -> bool {
        self.catalogs.iter().any(|c| c == "*" || c == catalog)
    }
}

/// The configured sources, matched by catalog name in registration order.
#[derive(Clone, Default)]
pub struct SourceRegistry {
    sources: Vec<RegisteredSource>,
}

impl SourceRegistry {
    pub fn new(sources: Vec<RegisteredSource>) -> Self {
        Self { sources }
    }

    /// The source serving the catalog `full_name` lives in.
    pub fn for_name(&self, full_name: &str) -> Result<&RegisteredSource> {
        let catalog = full_name.split('.').next().unwrap_or_default();
        self.sources
            .iter()
            .find(|s| s.serves(catalog))
            .ok_or_else(|| {
                Error::invalid_argument(format!(
                    "no catalog source serves catalog '{catalog}' (of '{full_name}')"
                ))
            })
    }
}

/// Split a dotted full name into exactly `N` parts.
pub(crate) fn split_name<const N: usize>(full_name: &str) -> Result<[&str; N]> {
    let parts: Vec<_> = full_name.split('.').collect();
    parts
        .try_into()
        .ok()
        .filter(|parts: &[&str; N]| parts.iter().all(|p| !p.is_empty()))
        .ok_or_else(|| {
            Error::invalid_argument(format!(
                "'{full_name}' is not a {N}-part name ({})",
                ["catalog", "schema", "name"][..N].join(".")
            ))
        })
}
