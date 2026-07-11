//! Hand-written serde models for the UC Delta REST API (`/delta/v1/...`).
//!
//! These mirror the component schemas in `openapi/delta.yaml` (vendored from the
//! Unity Catalog Java reference, `api/delta.yaml`). The wire format is kebab-case
//! JSON, so every struct uses `#[serde(rename_all = "kebab-case")]`; individual
//! fields whose JSON key is not a straight kebab-case of the Rust identifier
//! carry an explicit `#[serde(rename = "...")]`.
//!
//! This is a standalone REST protocol (not resource-CRUD), so — unlike most of
//! the workspace — it is not generated from proto. The pure wire DTOs live here
//! so every server implementation of the Delta API and every client shares a
//! single definition; the runtime error envelope (`DeltaApiError` +
//! `IntoResponse`) lives in the sibling [`crate::error`] module.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

// ===================================================================
// Enums
// ===================================================================

/// The type of a Delta table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeltaTableType {
    Managed,
    External,
}

/// The data source format of a table, as carried on `DeltaCreateTableRequest`.
///
/// Mirrors the canonical Unity Catalog `DataSourceFormat` set. Serializes to the
/// uppercase UC format strings (e.g. `"DELTA"`), matching what the Java OSS
/// server's createTable handler expects. The Delta REST API only operates on
/// Delta tables, so in practice this is always [`Delta`](Self::Delta); the wider
/// set is kept so the field round-trips any value the catalog might report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeltaDataSourceFormat {
    Delta,
    Iceberg,
    Hudi,
    Parquet,
    Csv,
    Json,
    Orc,
    Avro,
    Text,
    UnityCatalog,
    Deltasharing,
}

/// The permission level for a storage credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DeltaCredentialOperation {
    Read,
    ReadWrite,
}

// ===================================================================
// Shared credential types
// ===================================================================

/// Cloud provider-specific credential configuration. Only keys for the relevant
/// cloud provider are populated; the others are omitted (`None`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaStorageCredentialConfig {
    #[serde(rename = "s3.access-key-id", skip_serializing_if = "Option::is_none")]
    pub s3_access_key_id: Option<String>,
    #[serde(
        rename = "s3.secret-access-key",
        skip_serializing_if = "Option::is_none"
    )]
    pub s3_secret_access_key: Option<String>,
    #[serde(rename = "s3.session-token", skip_serializing_if = "Option::is_none")]
    pub s3_session_token: Option<String>,
    #[serde(rename = "azure.sas-token", skip_serializing_if = "Option::is_none")]
    pub azure_sas_token: Option<String>,
    #[serde(rename = "gcs.oauth-token", skip_serializing_if = "Option::is_none")]
    pub gcs_oauth_token: Option<String>,
}

/// Temporary storage credential with prefix and config.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaStorageCredential {
    /// Storage path prefix this credential applies to.
    pub prefix: String,
    pub operation: DeltaCredentialOperation,
    pub config: DeltaStorageCredentialConfig,
    /// Credential expiration time in epoch milliseconds.
    pub expiration_time_ms: i64,
}

/// Response carrying temporary cloud storage credentials.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaCredentialsResponse {
    pub storage_credentials: Vec<DeltaStorageCredential>,
}

// ===================================================================
// Configuration
// ===================================================================

/// Catalog configuration and supported endpoints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaCatalogConfig {
    /// List of supported endpoints.
    pub endpoints: Vec<String>,
    /// The negotiated protocol version (e.g., "1.0").
    pub protocol_version: String,
}

// ===================================================================
// Delta data types (schema)
// ===================================================================

/// A Delta column data type.
///
/// On the wire, primitive types are bare JSON strings (e.g. `"long"`, `"string"`,
/// `"decimal(10,2)"`) while complex types are JSON objects carrying a `type`
/// discriminator (`"array"`, `"map"`, `"struct"`, `"decimal"`). serde's `untagged`
/// representation drives the split: a JSON string matches [`DeltaDataType::Primitive`];
/// a JSON object matches one of the object variants. Each object variant carries its
/// own `type` constant so it both round-trips and self-identifies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum DeltaDataType {
    /// Bare-string primitive type, e.g. `"long"`, `"string"`, `"decimal(10,2)"`.
    Primitive(String),
    Array(Box<DeltaArrayType>),
    Map(Box<DeltaMapType>),
    Struct(Box<DeltaStructType>),
    /// Object form of decimal (`{"type":"decimal","precision":..,"scale":..}`).
    /// The string form `"decimal(p,s)"` is carried by [`DeltaDataType::Primitive`].
    Decimal(DeltaDecimalType),
}

/// The `type` discriminator constant for an array data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArrayTypeTag {
    #[default]
    Array,
}

/// The `type` discriminator constant for a map data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MapTypeTag {
    #[default]
    Map,
}

/// The `type` discriminator constant for a decimal data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DecimalTypeTag {
    #[default]
    Decimal,
}

/// Object form of an array type (`{"type":"array","element-type":..,"contains-null":..}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaArrayType {
    #[serde(rename = "type", default)]
    pub type_tag: ArrayTypeTag,
    pub element_type: DeltaDataType,
    pub contains_null: bool,
}

/// Object form of a map type (`{"type":"map","key-type":..,"value-type":..}`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaMapType {
    #[serde(rename = "type", default)]
    pub type_tag: MapTypeTag,
    pub key_type: DeltaDataType,
    pub value_type: DeltaDataType,
    pub value_contains_null: bool,
}

/// Object form of a decimal type (`{"type":"decimal","precision":..,"scale":..}`).
///
/// The compact string form `"decimal(p,s)"` is carried by
/// [`DeltaDataType::Primitive`] instead.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaDecimalType {
    #[serde(rename = "type", default)]
    pub type_tag: DecimalTypeTag,
    pub precision: i32,
    pub scale: i32,
}

/// A field in a [`DeltaStructType`]: name, type, nullability, and metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaStructField {
    pub name: String,
    #[serde(rename = "type")]
    pub data_type: DeltaDataType,
    pub nullable: bool,
    /// Arbitrary column metadata (e.g. `comment`, `delta.columnMapping.id`).
    /// Values can be strings, numbers, booleans, or nested objects.
    pub metadata: BTreeMap<String, serde_json::Value>,
}

/// Struct type containing named fields. Carries the `type: "struct"`
/// discriminator on the wire (it is the schema container for tables), so it is
/// emitted/accepted explicitly even when used as a bare `columns` value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaStructType {
    #[serde(rename = "type", default = "struct_type_tag")]
    pub type_tag: StructTypeTag,
    pub fields: Vec<DeltaStructField>,
}

fn struct_type_tag() -> StructTypeTag {
    StructTypeTag::Struct
}

/// The constant `"struct"` discriminator for [`DeltaStructType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StructTypeTag {
    #[default]
    Struct,
}

// ===================================================================
// Protocol
// ===================================================================

/// Delta table protocol specification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaProtocol {
    pub min_reader_version: i32,
    pub min_writer_version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reader_features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writer_features: Option<Vec<String>>,
}

/// Suggested protocol advertised in the staging response (no version fields,
/// only feature hints).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaSuggestedProtocol {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reader_features: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub writer_features: Option<Vec<String>>,
}

// ===================================================================
// Domain metadata
// ===================================================================

/// Configuration for Clustered Tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaClusteringDomainMetadata {
    /// Clustering column paths. Each inner array is a path from root schema to a
    /// column.
    #[serde(rename = "clusteringColumns")]
    pub clustering_columns: Vec<Vec<String>>,
}

/// Configuration for Row Tracking.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaRowTrackingDomainMetadata {
    /// The highest fresh Row ID assigned so far.
    #[serde(rename = "rowIdHighWaterMark")]
    pub row_id_high_water_mark: i64,
}

/// Domain metadata entries keyed by domain name.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaDomainMetadataUpdates {
    #[serde(rename = "delta.clustering", skip_serializing_if = "Option::is_none")]
    pub delta_clustering: Option<DeltaClusteringDomainMetadata>,
    #[serde(rename = "delta.rowTracking", skip_serializing_if = "Option::is_none")]
    pub delta_row_tracking: Option<DeltaRowTrackingDomainMetadata>,
}

// ===================================================================
// Uniform / Iceberg
// ===================================================================

/// Iceberg-specific conversion metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaIcebergMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub converted_delta_version: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub converted_delta_timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_converted_delta_version: Option<i64>,
}

/// UniForm conversion metadata. Present only for Uniform (Delta + Iceberg) tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaUniformMetadata {
    pub iceberg: DeltaIcebergMetadata,
}

// ===================================================================
// Commit
// ===================================================================

/// Delta commit information for CCv2.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaCommit {
    pub version: i64,
    /// In-commit timestamp, in epoch milliseconds.
    pub timestamp: i64,
    /// UUID-based commit file name.
    pub file_name: String,
    pub file_size: i64,
    /// File modification timestamp, in epoch milliseconds.
    pub file_modification_timestamp: i64,
}

// ===================================================================
// Table lifecycle
// ===================================================================

/// Request to create a staging table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeltaCreateStagingTableRequest {
    pub name: String,
}

/// Response from creating a staging table. Advertises the UC catalog-managed
/// contract (required / suggested protocol + properties) plus READ_WRITE
/// credentials for writing the initial commit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaStagingTableResponse {
    pub table_id: String,
    pub table_type: DeltaTableType,
    pub location: String,
    pub storage_credentials: Vec<DeltaStorageCredential>,
    pub required_protocol: DeltaProtocol,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_protocol: Option<DeltaSuggestedProtocol>,
    /// Properties that must be set; null values mean any valid value is allowed.
    pub required_properties: BTreeMap<String, Option<String>>,
    /// Properties that should be set whenever possible; null values mean the
    /// client generates the value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_properties: Option<BTreeMap<String, Option<String>>>,
}

/// Request to create a Delta table (managed or external).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaCreateTableRequest {
    pub name: String,
    pub location: String,
    pub table_type: DeltaTableType,
    /// Data source format — `"DELTA"` for Delta tables. Required by the running
    /// Java UC OSS server (`ghcr.io/roeap/unitycatalog:v0.0.0-dev-3`), whose
    /// createTable handler reads it from the request body. (Newer server code
    /// hardcodes `DELTA` in `DeltaCreateTableMapper` and ignores this field, and
    /// the `delta.yaml` schema omits it — a spec/impl drift; sending it satisfies
    /// both.)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_source_format: Option<DeltaDataSourceFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub columns: DeltaStructType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_columns: Option<Vec<String>>,
    pub protocol: DeltaProtocol,
    pub properties: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_metadata: Option<DeltaDomainMetadataUpdates>,
    /// Timestamp of version 0 (the commit the client wrote before calling this
    /// endpoint), in epoch milliseconds.
    pub last_commit_timestamp_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uniform: Option<DeltaUniformMetadata>,
}

/// Complete table metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaTableMetadata {
    /// Entity tag for optimistic concurrency control.
    pub etag: String,
    pub table_type: DeltaTableType,
    pub table_uuid: String,
    pub location: String,
    pub created_time: i64,
    pub updated_time: i64,
    pub columns: DeltaStructType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_columns: Option<Vec<String>>,
    pub properties: BTreeMap<String, String>,
    /// The version of the last commit that changed table metadata
    /// (`delta.lastUpdateVersion`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit_version: Option<i64>,
    /// Timestamp of the last commit that changed table metadata, in epoch
    /// milliseconds (`delta.lastCommitTimestamp`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_commit_timestamp_ms: Option<i64>,
}

/// Response from `loadTable` / `createTable` / `updateTable`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaLoadTableResponse {
    pub metadata: DeltaTableMetadata,
    /// All unbackfilled CCv2 commits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commits: Option<Vec<DeltaCommit>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uniform: Option<DeltaUniformMetadata>,
    /// The latest ratified table version tracked by the server, including
    /// data-only commits.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_table_version: Option<i64>,
}

/// Request to rename a table within the same catalog and schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaRenameTableRequest {
    pub new_name: String,
}

// ===================================================================
// Update table (requirements + actions)
// ===================================================================

/// A pre-condition that must hold for the update to apply.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum DeltaTableRequirement {
    /// `assert-table-uuid`: the table UUID must match `uuid`.
    AssertTableUuid { uuid: String },
    /// `assert-etag`: the table etag must match `etag`.
    AssertEtag { etag: String },
}

/// A single update action. Tagged by the `action` discriminator, matching the
/// `propertyName: action` + `const` mapping in `delta.yaml`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "kebab-case")]
pub enum DeltaTableUpdate {
    SetProperties {
        updates: BTreeMap<String, String>,
    },
    RemoveProperties {
        removals: Vec<String>,
    },
    SetColumns {
        columns: DeltaStructType,
    },
    SetTableComment {
        comment: String,
    },
    AddCommit {
        commit: DeltaCommit,
        #[serde(skip_serializing_if = "Option::is_none")]
        uniform: Option<DeltaUniformMetadata>,
    },
    SetLatestBackfilledVersion {
        #[serde(rename = "latest-published-version")]
        latest_published_version: i64,
    },
    SetProtocol {
        protocol: DeltaProtocol,
    },
    SetDomainMetadata {
        updates: DeltaDomainMetadataUpdates,
    },
    RemoveDomainMetadata {
        domains: Vec<String>,
    },
    SetPartitionColumns {
        #[serde(rename = "partition-columns")]
        partition_columns: Vec<String>,
    },
    UpdateMetadataSnapshotVersion {
        #[serde(rename = "last-commit-version")]
        last_commit_version: i64,
        #[serde(rename = "last-commit-timestamp-ms")]
        last_commit_timestamp_ms: i64,
    },
}

/// Request to update a table with requirements and updates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeltaUpdateTableRequest {
    pub requirements: Vec<DeltaTableRequirement>,
    pub updates: Vec<DeltaTableUpdate>,
}

// ===================================================================
// Metrics
// ===================================================================

/// Histogram tracking file counts and total bytes across size ranges.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaFileSizeHistogram {
    pub sorted_bin_boundaries: Vec<i64>,
    pub file_counts: Vec<i64>,
    pub total_bytes: Vec<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_version: Option<i64>,
}

/// Commit report metrics.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaCommitReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_files_added: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_bytes_added: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_files_removed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_bytes_removed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_rows_inserted: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_rows_removed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_rows_updated: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size_histogram: Option<DeltaFileSizeHistogram>,
}

/// The metrics being reported.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaReport {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit_report: Option<DeltaCommitReport>,
}

/// Request to report commit metrics (telemetry) for a table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct DeltaReportMetricsRequest {
    /// Table UUID. Must match the table identified by the path.
    pub table_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub report: Option<DeltaReport>,
}

// ===================================================================
// Errors (the /delta/v1 envelope)
// ===================================================================

/// Error type identifier returned in Delta API error responses. Mirrors the
/// `DeltaErrorType` enum in `delta.yaml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeltaErrorType {
    BadRequestException,
    InvalidParameterValueException,
    UnsupportedTableFormatException,
    NotAuthorizedException,
    PermissionDeniedException,
    NotFoundException,
    NoSuchCatalogException,
    NoSuchSchemaException,
    NoSuchTableException,
    AlreadyExistsException,
    CommitVersionConflictException,
    UpdateRequirementConflictException,
    ResourceExhaustedException,
    TooManyRequestsException,
    CommitStateUnknownException,
    InternalServerErrorException,
    NotImplementedException,
}

impl DeltaErrorType {
    /// Whether this error type denotes a missing resource (404 family).
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            DeltaErrorType::NotFoundException
                | DeltaErrorType::NoSuchCatalogException
                | DeltaErrorType::NoSuchSchemaException
                | DeltaErrorType::NoSuchTableException
        )
    }

    /// Whether this error type denotes a name collision (`AlreadyExistsException`).
    pub fn is_already_exists(&self) -> bool {
        matches!(self, DeltaErrorType::AlreadyExistsException)
    }

    /// Whether this error type denotes a commit-version conflict that the client
    /// should resolve by rebuilding its snapshot and retrying.
    pub fn is_commit_conflict(&self) -> bool {
        matches!(self, DeltaErrorType::CommitVersionConflictException)
    }

    /// Whether this error type denotes an unmet `assert-etag`/`assert-table-uuid`
    /// requirement; the client should reload the table and retry.
    pub fn is_update_requirement_conflict(&self) -> bool {
        matches!(self, DeltaErrorType::UpdateRequirementConflictException)
    }

    /// Whether the request was throttled or hit a resource limit (429 family).
    /// The client should back off (and, for `ResourceExhaustedException`,
    /// backfill pending commits) before retrying.
    pub fn is_resource_exhausted(&self) -> bool {
        matches!(
            self,
            DeltaErrorType::ResourceExhaustedException | DeltaErrorType::TooManyRequestsException
        )
    }

    /// Whether the commit outcome is unknown; the client must check table state
    /// before retrying to avoid duplicate commits.
    pub fn is_commit_state_unknown(&self) -> bool {
        matches!(self, DeltaErrorType::CommitStateUnknownException)
    }

    /// Whether the table is not Delta, or is a Delta table this `/delta/v1`
    /// endpoint does not support (`UnsupportedTableFormatException`, a 400). Per
    /// the spec the client should fall back to the legacy UC table API.
    pub fn is_unsupported_table_format(&self) -> bool {
        matches!(self, DeltaErrorType::UnsupportedTableFormatException)
    }

    /// Whether the requested `/delta/v1` functionality is not supported by the
    /// server (`NotImplementedException`, a 501). The client should fall back to
    /// the legacy UC table API.
    pub fn is_not_implemented(&self) -> bool {
        matches!(self, DeltaErrorType::NotImplementedException)
    }
}

/// The JSON error payload (`DeltaErrorModel` in `delta.yaml`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaErrorModel {
    pub message: String,
    #[serde(rename = "type")]
    pub error_type: DeltaErrorType,
    /// HTTP response code.
    pub code: u16,
    /// Stack trace, only populated when the server runs in debug mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<Vec<String>>,
}

/// The JSON wrapper for all Delta API error responses (`DeltaErrorResponse`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeltaErrorResponse {
    pub error: DeltaErrorModel,
}

#[cfg(test)]
mod tests {
    //! Round-trip tests over the example payloads in `openapi/delta.yaml`.
    //!
    //! Each test deserializes a spec example into our hand-written model and
    //! re-serializes it, asserting the JSON survives a round-trip with the
    //! kebab-case keys, enum casing, and tagged-union discriminators intact.
    //! This is the Phase-1 parity gate: it pins our models to the vendored spec.

    use super::*;
    use serde_json::{Value, json};

    /// Deserialize `json` into `T`, re-serialize, and assert structural equality
    /// with the original JSON value (key order independent).
    fn round_trip<T>(value: Value)
    where
        T: Serialize + for<'de> Deserialize<'de>,
    {
        let parsed: T = serde_json::from_value(value.clone())
            .unwrap_or_else(|e| panic!("deserialize failed: {e}\njson: {value:#}"));
        let reserialized = serde_json::to_value(&parsed).expect("serialize");
        assert_eq!(
            reserialized, value,
            "round-trip mismatch\n  expected: {value:#}\n  got:      {reserialized:#}"
        );
    }

    #[test]
    fn staging_table_response() {
        round_trip::<DeltaStagingTableResponse>(json!({
            "table-id": "123e4567-e89b-12d3-a456-426614174000",
            "table-type": "MANAGED",
            "location": "s3://bucket/warehouse/catalog/schema/table",
            "storage-credentials": [{
                "prefix": "s3://bucket/warehouse/catalog/schema/table/",
                "operation": "READ_WRITE",
                "config": {
                    "s3.access-key-id": "AK...example",
                    "s3.secret-access-key": "ExampleKey",
                    "s3.session-token": "token"
                },
                "expiration-time-ms": 1234567890000_i64
            }],
            "required-protocol": {
                "min-reader-version": 3,
                "min-writer-version": 7,
                "reader-features": ["deletionVectors", "vacuumProtocolCheck"],
                "writer-features": ["catalogManaged", "deletionVectors"]
            },
            "suggested-protocol": {
                "reader-features": ["typeWidening"],
                "writer-features": ["domainMetadata", "rowTracking"]
            },
            "required-properties": { "delta.checkpointPolicy": "v2" },
            "suggested-properties": {
                "delta.rowTracking.materializedRowIdColumnName": null,
                "delta.rowTracking.materializedRowCommitVersionColumnName": null
            }
        }));
    }

    #[test]
    fn create_table_request_with_decimal_and_partition() {
        round_trip::<DeltaCreateTableRequest>(json!({
            "name": "sales",
            "location": "s3://bucket/warehouse/catalog/schema/sales",
            "table-type": "MANAGED",
            "columns": {
                "type": "struct",
                "fields": [
                    { "name": "id", "type": "long", "nullable": false, "metadata": {} },
                    {
                        "name": "amount",
                        "type": { "type": "decimal", "precision": 10, "scale": 2 },
                        "nullable": true,
                        "metadata": {}
                    }
                ]
            },
            "partition-columns": ["id"],
            "protocol": {
                "min-reader-version": 3,
                "min-writer-version": 7,
                "reader-features": ["deletionVectors"],
                "writer-features": ["deletionVectors", "invariants"]
            },
            "properties": { "delta.enableDeletionVectors": "true" },
            "last-commit-timestamp-ms": 1704067400000_i64
        }));
    }

    #[test]
    fn struct_type_carries_type_tag() {
        // A bare `columns` value round-trips with its `type: "struct"` tag.
        round_trip::<DeltaStructType>(json!({
            "type": "struct",
            "fields": [
                { "name": "id", "type": "long", "nullable": false, "metadata": {} },
                { "name": "name", "type": "string", "nullable": true, "metadata": {} }
            ]
        }));
    }

    #[test]
    fn nested_array_and_map_types() {
        round_trip::<DeltaStructField>(json!({
            "name": "tags",
            "type": {
                "type": "array",
                "element-type": "string",
                "contains-null": true
            },
            "nullable": true,
            "metadata": {}
        }));
        round_trip::<DeltaStructField>(json!({
            "name": "props",
            "type": {
                "type": "map",
                "key-type": "string",
                "value-type": "long",
                "value-contains-null": false
            },
            "nullable": true,
            "metadata": {}
        }));
    }

    #[test]
    fn load_table_response_with_commits() {
        round_trip::<DeltaLoadTableResponse>(json!({
            "metadata": {
                "etag": "etag-1",
                "table-type": "MANAGED",
                "table-uuid": "123e4567-e89b-12d3-a456-426614174000",
                "location": "s3://bucket/warehouse/catalog/schema/table",
                "created-time": 1705600000000_i64,
                "updated-time": 1705600000000_i64,
                "columns": { "type": "struct", "fields": [] },
                "properties": { "delta.checkpointPolicy": "v2" },
                "last-commit-version": 0,
                "last-commit-timestamp-ms": 1704067400000_i64
            },
            "commits": [{
                "version": 1,
                "timestamp": 1704067200000_i64,
                "file-name": "00000000-0000-0000-0000-00000000002a.json",
                "file-size": 2048,
                "file-modification-timestamp": 1704067200000_i64
            }],
            "latest-table-version": 1
        }));
    }

    #[test]
    fn update_table_request_add_commit() {
        round_trip::<DeltaUpdateTableRequest>(json!({
            "requirements": [
                { "type": "assert-table-uuid", "uuid": "123e4567-e89b-12d3-a456-426614174000" },
                { "type": "assert-etag", "etag": "etag-1" }
            ],
            "updates": [
                {
                    "action": "add-commit",
                    "commit": {
                        "version": 1,
                        "timestamp": 1704067200000_i64,
                        "file-name": "v.uuid1.json",
                        "file-size": 2048,
                        "file-modification-timestamp": 1704067200000_i64
                    }
                },
                { "action": "set-latest-backfilled-version", "latest-published-version": 0 }
            ]
        }));
    }

    #[test]
    fn update_table_request_metadata_actions() {
        round_trip::<DeltaUpdateTableRequest>(json!({
            "requirements": [],
            "updates": [
                { "action": "set-properties", "updates": { "k": "v" } },
                { "action": "remove-properties", "removals": ["old"] },
                { "action": "set-table-comment", "comment": "hello" },
                {
                    "action": "set-protocol",
                    "protocol": { "min-reader-version": 3, "min-writer-version": 7 }
                },
                { "action": "set-partition-columns", "partition-columns": ["id"] },
                {
                    "action": "update-metadata-snapshot-version",
                    "last-commit-version": 5,
                    "last-commit-timestamp-ms": 1704067400000_i64
                }
            ]
        }));
    }

    #[test]
    fn update_table_domain_metadata_actions() {
        round_trip::<DeltaUpdateTableRequest>(json!({
            "requirements": [],
            "updates": [
                {
                    "action": "set-domain-metadata",
                    "updates": {
                        "delta.clustering": { "clusteringColumns": [["id"], ["address", "city"]] },
                        "delta.rowTracking": { "rowIdHighWaterMark": 42 }
                    }
                },
                { "action": "remove-domain-metadata", "domains": ["delta.clustering"] }
            ]
        }));
    }

    #[test]
    fn credentials_response() {
        round_trip::<DeltaCredentialsResponse>(json!({
            "storage-credentials": [{
                "prefix": "s3://bucket/path/",
                "operation": "READ",
                "config": { "s3.access-key-id": "AK", "s3.secret-access-key": "SK" },
                "expiration-time-ms": 1234567890000_i64
            }]
        }));
    }

    #[test]
    fn catalog_config() {
        round_trip::<DeltaCatalogConfig>(json!({
            "endpoints": ["GET /v1/temporary-path-credentials"],
            "protocol-version": "1.0"
        }));
    }

    #[test]
    fn report_metrics_request() {
        round_trip::<DeltaReportMetricsRequest>(json!({
            "table-id": "123e4567-e89b-12d3-a456-426614174000",
            "report": {
                "commit-report": {
                    "num-files-added": 10,
                    "num-bytes-added": 104857600_i64,
                    "file-size-histogram": {
                        "sorted-bin-boundaries": [0, 1024, 2048],
                        "file-counts": [100, 40, 5],
                        "total-bytes": [104857600_i64, 167772160_i64, 83886080_i64],
                        "commit-version": 6
                    }
                }
            }
        }));
    }

    #[test]
    fn error_response_round_trips() {
        round_trip::<DeltaErrorResponse>(json!({
            "error": {
                "message": "table not found",
                "type": "NoSuchTableException",
                "code": 404
            }
        }));
    }

    #[test]
    fn error_response_with_stack_round_trips() {
        round_trip::<DeltaErrorResponse>(json!({
            "error": {
                "message": "boom",
                "type": "InternalServerErrorException",
                "code": 500,
                "stack": ["frame one", "frame two"]
            }
        }));
    }

    #[test]
    fn uniform_metadata_round_trips() {
        round_trip::<DeltaUniformMetadata>(json!({
            "iceberg": {
                "metadata-location": "s3://bucket/metadata/v1.json",
                "converted-delta-version": 42,
                "converted-delta-timestamp": 1704067200000_i64
            }
        }));
    }
}
