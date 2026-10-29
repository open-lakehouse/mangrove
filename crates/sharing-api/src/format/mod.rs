//! The [`TableFormat`] port: serving one table format over the sharing protocol.
//!
//! A table format turns a shared table's storage into protocol responses: its
//! current version, its protocol + metadata, and the (presigned) data files of a
//! query. Formats are registered in a [`FormatRegistry`] keyed by
//! [`DataFormat`]; the service looks up the format a resolved asset declares.
//!
//! The Delta format (feature `delta`, on by default) reads the table through
//! `delta_kernel` directly — no query engine is involved in listing files.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use delta_kernel::Version;

use open_sharing_common::models::open_sharing::v1::JsonPredicate;

use crate::access::TableStorage;
use crate::catalog::{DataFormat, SharedAsset};
use crate::error::{Error, Result};

#[cfg(feature = "delta")]
pub mod delta;
pub mod predicate;

/// Which version of a table to read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VersionSpec {
    /// The latest version.
    #[default]
    Latest,
    /// A specific version.
    Version(Version),
    /// The latest version committed at or before this timestamp (ms since epoch).
    Timestamp(i64),
}

impl VersionSpec {
    /// Whether this reads table history rather than the latest version.
    pub fn is_time_travel(&self) -> bool {
        !matches!(self, VersionSpec::Latest)
    }
}

/// A snapshot query against a shared table.
#[derive(Debug, Clone, Default)]
pub struct QuerySpec {
    /// The version to read.
    pub at: VersionSpec,
    /// A best-effort predicate hint for skipping files.
    pub predicate: Option<JsonPredicate>,
    /// A best-effort hint on how many rows the recipient intends to read.
    pub limit_hint: Option<u64>,
}

/// A newline-delimited JSON table response and the table version it describes
/// (sent as the `Delta-Table-Version` response header).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableResponse {
    pub version: Version,
    pub body: Bytes,
}

/// Serves one table format. See the module docs.
///
/// Every method receives the resolved [`SharedAsset`] (which carries the
/// provider's history-sharing limits — a format must reject reads below
/// [`SharedAsset::start_version`]) and the [`TableStorage`] to read it with.
/// Responses are newline-delimited JSON in the protocol's parquet response format.
#[async_trait]
pub trait TableFormat: Send + Sync + 'static {
    /// The table's version: the latest, or with `starting_timestamp` (ms since
    /// epoch) the earliest version committed at or after it.
    async fn version(
        &self,
        asset: &SharedAsset,
        storage: &TableStorage,
        starting_timestamp: Option<i64>,
    ) -> Result<Version>;

    /// The protocol and metadata lines for the table at `at`.
    async fn metadata(
        &self,
        asset: &SharedAsset,
        storage: &TableStorage,
        at: VersionSpec,
    ) -> Result<TableResponse>;

    /// The protocol, metadata and file lines answering `query`.
    async fn query(
        &self,
        asset: &SharedAsset,
        storage: &TableStorage,
        query: QuerySpec,
    ) -> Result<TableResponse>;
}

/// The table formats a service can serve, keyed by [`DataFormat`].
#[derive(Clone, Default)]
pub struct FormatRegistry {
    formats: HashMap<DataFormat, Arc<dyn TableFormat>>,
}

impl FormatRegistry {
    /// An empty registry.
    pub fn empty() -> Self {
        Self::default()
    }

    /// The built-in formats enabled by this crate's features (Delta with `delta`).
    pub fn builtin() -> Self {
        #[allow(unused_mut)]
        let mut registry = Self::empty();
        #[cfg(feature = "delta")]
        registry.register(DataFormat::Delta, Arc::new(delta::DeltaFormat));
        registry
    }

    /// Register (or replace) the implementation serving `format`.
    pub fn register(&mut self, format: DataFormat, implementation: Arc<dyn TableFormat>) {
        self.formats.insert(format, implementation);
    }

    /// The implementation serving `format`.
    pub fn get(&self, format: DataFormat) -> Result<&Arc<dyn TableFormat>> {
        self.formats
            .get(&format)
            .ok_or(Error::NotImplemented("serving this table format"))
    }
}
