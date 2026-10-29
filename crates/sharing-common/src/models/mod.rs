use std::{collections::HashMap, str::FromStr};

use delta_kernel::actions::{Metadata as KernelMetadata, Protocol as DeltaProtocol};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

// `empty_docs` is now suppressed inside the generated `_gen/mod.rs` itself.
#[allow(clippy::large_enum_variant)]
mod _gen;
pub use _gen::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResponseFormat {
    Parquet,
    Delta,
}

impl FromStr for ResponseFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "parquet" => Ok(ResponseFormat::Parquet),
            "delta" => Ok(ResponseFormat::Delta),
            _ => Err(Error::InvalidArgument(s.to_string())),
        }
    }
}

/// Delta reader features advertised by a Delta Sharing client in its
/// `delta-sharing-capabilities` header.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReaderFeature {
    CatalogManaged,
    ColumnMapping,
    DeletionVectors,
    TimestampWithoutTimezone,
    TypeWidening,
    V2Checkpoint,
    VariantType,
}

fn parse_reader_feature(s: &str) -> Result<ReaderFeature> {
    match s.to_lowercase().as_str() {
        "catalogmanaged" => Ok(ReaderFeature::CatalogManaged),
        "columnmapping" => Ok(ReaderFeature::ColumnMapping),
        "deletionvectors" => Ok(ReaderFeature::DeletionVectors),
        "timestampntz" => Ok(ReaderFeature::TimestampWithoutTimezone),
        "typewidening" => Ok(ReaderFeature::TypeWidening),
        "v2checkpoint" => Ok(ReaderFeature::V2Checkpoint),
        "varianttype" => Ok(ReaderFeature::VariantType),
        _ => Err(Error::InvalidArgument(s.to_string())),
    }
}

/// Parsed capabilities header
///
/// The key of the header is delta-sharing-capabilities, the value is semicolon
/// separated capabilities. Each capability is in the format of "key=value1,value2",
/// values are separated by commas.
///
/// Example: "responseformat=delta;readerfeatures=deletionvectors,columnmapping".
/// All keys and values should be case-insensitive when processed by the server.
pub struct DeltaSharingCapabilities {
    pub format: Option<Vec<ResponseFormat>>,
    pub reader_features: Option<Vec<ReaderFeature>>,
    pub include_end_stream_action: Option<bool>,
}

impl DeltaSharingCapabilities {
    /// Parse a `delta-sharing-capabilities` header value.
    ///
    /// Parsing is lenient, as the header is client-supplied: empty or malformed
    /// segments (no `=`) and unknown keys are skipped, and unknown reader features
    /// are ignored rather than rejected, so a newer client never fails a request
    /// just by advertising more than this server knows. Only an unrecognized
    /// `responseformat` or a non-boolean `includeendstreamaction` is an error.
    pub fn parse_header(header: &str) -> Result<Self, Error> {
        let mut format = None;
        let mut reader_features = None;
        let mut include_end_stream_action = None;

        for capability in header.split(';') {
            let Some((key, value)) = capability.split_once('=') else {
                continue;
            };
            let key = key.trim().to_ascii_lowercase();
            let value = value.trim().to_ascii_lowercase();

            match key.as_str() {
                "responseformat" => {
                    format = Some(
                        value
                            .split(',')
                            .map(str::trim)
                            .filter(|s| !s.is_empty())
                            .map(|s| s.parse())
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                "readerfeatures" => {
                    reader_features = Some(
                        value
                            .split(',')
                            .filter_map(|s| parse_reader_feature(s.trim()).ok())
                            .collect(),
                    );
                }
                "includeendstreamaction" => {
                    include_end_stream_action = match value.as_str() {
                        "true" => Some(true),
                        "false" => Some(false),
                        _ => {
                            return Err(Error::InvalidArgument(format!(
                                "Invalid value for include_end_stream_action: {}",
                                value
                            )));
                        }
                    };
                }
                _ => {}
            }
        }

        Ok(Self {
            format,
            reader_features,
            include_end_stream_action,
        })
    }

    /// Whether the client accepts responses in `format`.
    ///
    /// A client that sends no `responseformat` accepts only `parquet`, the
    /// protocol's original (and default) format.
    pub fn accepts(&self, format: &ResponseFormat) -> bool {
        match &self.format {
            Some(formats) => formats.contains(format),
            None => *format == ResponseFormat::Parquet,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Format {
    /// Name of the encoding for files in this table
    pub provider: String,
    /// A map containing configuration options for the format
    #[serde(default)]
    pub options: HashMap<String, String>,
}

impl Default for Format {
    fn default() -> Self {
        Self {
            provider: String::from("parquet"),
            options: HashMap::new(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Unique identifier for this table
    pub id: String,
    /// User-provided identifier for this table
    pub name: Option<String>,
    /// User-provided description for this table
    pub description: Option<String>,
    /// Specification of the encoding for the files stored in the table
    pub format: Format,
    /// Schema of the table
    pub schema_string: String,
    /// Column names by which the data should be partitioned
    pub partition_columns: Vec<String>,
    /// The time when this metadata action is created, in milliseconds since the Unix epoch
    pub created_time: Option<i64>,
    /// Configuration options for the metadata action. These are parsed into `TableProperties`.
    #[serde(default)]
    pub configuration: HashMap<String, String>,
}

impl TryFrom<&KernelMetadata> for Metadata {
    type Error = Error;

    fn try_from(metadata: &KernelMetadata) -> Result<Self, Self::Error> {
        let bytes = serde_json::to_vec(metadata)?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}

impl TryFrom<KernelMetadata> for Metadata {
    type Error = Error;

    fn try_from(metadata: KernelMetadata) -> Result<Self, Self::Error> {
        (&metadata).try_into()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParquetProtocol {
    pub min_reader_version: i32,
}

impl From<&DeltaProtocol> for ParquetProtocol {
    fn from(protocol: &DeltaProtocol) -> Self {
        Self {
            min_reader_version: protocol.min_reader_version(),
        }
    }
}

impl From<DeltaProtocol> for ParquetProtocol {
    fn from(protocol: DeltaProtocol) -> Self {
        (&protocol).into()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ProtocolResponseData {
    /// Protocol in [Delta format]
    ///
    /// [Delta format]: https://github.com/delta-io/delta-sharing/blob/main/PROTOCOL.md#protocol-in-delta-format
    DeltaProtocol(DeltaProtocol),

    /// Protocol in [Parquet format]
    ///
    /// [Parquet format]: https://github.com/delta-io/delta-sharing/blob/main/PROTOCOL.md#protocol
    #[serde(untagged)]
    ParquetProtocol(ParquetProtocol),
}

impl ProtocolResponseData {
    pub fn min_reader_version(&self) -> i32 {
        match self {
            ProtocolResponseData::DeltaProtocol(protocol) => protocol.min_reader_version(),
            ProtocolResponseData::ParquetProtocol(protocol) => protocol.min_reader_version,
        }
    }

    pub fn min_writer_version(&self) -> Option<i32> {
        match self {
            ProtocolResponseData::DeltaProtocol(protocol) => Some(protocol.min_writer_version()),
            ProtocolResponseData::ParquetProtocol(_) => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeltaMetadata {
    pub delta_metadata: Metadata,
    pub version: Option<i64>,
    pub size: Option<i64>,
    pub num_files: Option<i64>,
}

impl TryFrom<&KernelMetadata> for DeltaMetadata {
    type Error = Error;

    fn try_from(kernel_metadata: &KernelMetadata) -> Result<Self, Self::Error> {
        let delta_metadata = Metadata::try_from(kernel_metadata)?;
        Ok(DeltaMetadata {
            delta_metadata,
            version: None,
            size: None,
            num_files: None,
        })
    }
}

impl TryFrom<KernelMetadata> for DeltaMetadata {
    type Error = Error;

    fn try_from(metadata: KernelMetadata) -> Result<Self, Self::Error> {
        (&metadata).try_into()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParquetMetadata {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub format: Format,
    pub schema_string: String,
    pub partition_columns: Vec<String>,
    pub configuration: HashMap<String, String>,
    pub version: Option<i64>,
    pub size: Option<i64>,
    pub num_files: Option<i64>,
}

impl TryFrom<&KernelMetadata> for ParquetMetadata {
    type Error = Error;

    fn try_from(metadata: &KernelMetadata) -> Result<Self, Self::Error> {
        let bytes = serde_json::to_vec(metadata)?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}

impl TryFrom<KernelMetadata> for ParquetMetadata {
    type Error = Error;

    fn try_from(metadata: KernelMetadata) -> Result<Self, Self::Error> {
        (&metadata).try_into()
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", untagged)]
pub enum MetadataResponseData {
    /// Protocol in [Delta format]
    ///
    /// [Delta format]: https://github.com/delta-io/delta-sharing/blob/main/PROTOCOL.md#protocol-in-delta-format
    DeltaMetadata(DeltaMetadata),

    /// Protocol in [Parquet format]
    ///
    /// [Parquet format]: https://github.com/delta-io/delta-sharing/blob/main/PROTOCOL.md#protocol
    ParquetMetadata(ParquetMetadata),
}

impl MetadataResponseData {
    pub fn id(&self) -> &str {
        match self {
            MetadataResponseData::DeltaMetadata(metadata) => &metadata.delta_metadata.id,
            MetadataResponseData::ParquetMetadata(metadata) => &metadata.id,
        }
    }

    pub fn partition_columns(&self) -> &[String] {
        match self {
            MetadataResponseData::DeltaMetadata(metadata) => {
                metadata.delta_metadata.partition_columns.as_ref()
            }
            MetadataResponseData::ParquetMetadata(metadata) => metadata.partition_columns.as_ref(),
        }
    }

    pub fn configuration(&self) -> &HashMap<String, String> {
        match self {
            MetadataResponseData::DeltaMetadata(metadata) => &metadata.delta_metadata.configuration,
            MetadataResponseData::ParquetMetadata(metadata) => &metadata.configuration,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MetadataResponse {
    Protocol(ProtocolResponseData),
    MetaData(MetadataResponseData),
}

/// A data file in a [parquet-format] query response: one `{"file": {...}}` line.
///
/// [parquet-format]: https://github.com/delta-io/delta-sharing/blob/main/PROTOCOL.md#file
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParquetFile {
    /// A (typically presigned, short-lived) URL the client reads the file from.
    pub url: String,
    /// A stable identifier for the file within the table.
    pub id: String,
    /// The file's partition values, keyed by partition column.
    #[serde(default)]
    pub partition_values: HashMap<String, String>,
    /// The file size in bytes.
    pub size: i64,
    /// The file's statistics as a JSON string (`numRecords`, …), when known.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stats: Option<String>,
    /// The table version the file was read at, for a version / timestamp query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<i64>,
    /// The commit timestamp (ms) of `version`, for a version / timestamp query.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    /// When `url` stops being valid, in milliseconds since the Unix epoch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration_timestamp: Option<i64>,
}

/// One line of a parquet-format `query` response: protocol, metadata, or a file.
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum QueryResponseLine {
    Protocol(ProtocolResponseData),
    MetaData(MetadataResponseData),
    File(ParquetFile),
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn test_protocol_response() {
        let raw_parquet = json!({
          "protocol": {
            "minReaderVersion": 1
          }
        });

        let response: MetadataResponse = serde_json::from_value(raw_parquet).unwrap();
        assert!(matches!(
            response,
            MetadataResponse::Protocol(ProtocolResponseData::ParquetProtocol(ParquetProtocol {
                min_reader_version: 1
            }))
        ));

        let raw_delta = json!({
          "protocol": {
            "deltaProtocol": {
              "minReaderVersion": 3,
              "minWriterVersion": 7,
              "readerFeatures": ["columnMapping"],
              "writerFeatures": ["columnMapping"]
            }
          }
        });

        let response: MetadataResponse = serde_json::from_value(raw_delta).unwrap();
        assert!(matches!(
            response,
            MetadataResponse::Protocol(ProtocolResponseData::DeltaProtocol(DeltaProtocol { .. }))
        ));
        let MetadataResponse::Protocol(ProtocolResponseData::DeltaProtocol(protocol)) = response
        else {
            panic!("Unexpected response");
        };
        assert_eq!(protocol.min_reader_version(), 3);
        assert_eq!(protocol.min_writer_version(), 7);
    }

    #[test]
    fn test_metadata_response() {
        let raw_parquet = json!({
          "metaData": {
            "version": 20,
            "size": 123456,
            "numFiles": 5,
            "deltaMetadata": {
              "partitionColumns": [
                "date"
              ],
              "format": {
                "provider": "parquet",
              },
              "schemaString": "{\"type\":\"struct\",\"fields\":[{\"name\":\"eventTime\",\"type\":\"timestamp\",\"nullable\":true,\"metadata\":{}},{\"name\":\"date\",\"type\":\"date\",\"nullable\":true,\"metadata\":{}}]}",
              "id": "f8d5c169-3d01-4ca3-ad9e-7dc3355aedb2",
              "configuration": {
                "enableChangeDataFeed": "true"
              }
            }
          }
        });

        let response: MetadataResponse = serde_json::from_value(raw_parquet).unwrap();
        assert!(matches!(
            response,
            MetadataResponse::MetaData(MetadataResponseData::DeltaMetadata(_))
        ));

        let MetadataResponse::MetaData(MetadataResponseData::DeltaMetadata(metadata)) = response
        else {
            panic!("Unexpected response");
        };
        assert_eq!(metadata.version, Some(20));
        assert_eq!(metadata.size, Some(123456));
        assert_eq!(metadata.delta_metadata.partition_columns.len(), 1);
    }

    #[test]
    fn test_share_fields_round_trip() {
        use open_sharing::v1::Share;

        // The generated serde accepts camelCase and snake_case; verify the new
        // spec fields (display_name/comment/properties) deserialize from
        // camelCase JSON as produced by Delta Sharing clients.
        let raw = json!({
            "name": "share1",
            "id": "abc-123",
            "displayName": "Share One",
            "comment": "a shared dataset",
            "properties": {"team": "data"}
        });
        let share: Share = serde_json::from_value(raw).unwrap();
        assert_eq!(share.name, "share1");
        assert_eq!(share.id.as_deref(), Some("abc-123"));
        assert_eq!(share.display_name.as_deref(), Some("Share One"));
        assert_eq!(share.comment.as_deref(), Some("a shared dataset"));
        assert_eq!(
            share.properties.get("team").map(String::as_str),
            Some("data")
        );
    }

    #[test]
    fn test_table_fields_round_trip() {
        use open_sharing::v1::Table;

        let raw = json!({
            "name": "table1",
            "schema": "schema1",
            "share": "share1",
            "location": "s3://bucket/path",
            "auxiliaryLocations": ["s3://bucket/aux"],
            "accessModes": ["url", "dir"]
        });
        let table: Table = serde_json::from_value(raw).unwrap();
        assert_eq!(table.name, "table1");
        assert_eq!(table.location.as_deref(), Some("s3://bucket/path"));
        assert_eq!(table.auxiliary_locations, vec!["s3://bucket/aux"]);
        assert_eq!(table.access_modes, vec!["url", "dir"]);
    }

    #[test]
    fn test_parse_capabilities_header_is_lenient() {
        // Malformed segments, unknown keys and unknown reader features must not
        // fail (or panic on) a request.
        let raw = "garbage;;readerfeatures=deletionvectors,somefuturefeature;unknown=x";
        let parsed = DeltaSharingCapabilities::parse_header(raw).unwrap();
        assert_eq!(parsed.format, None);
        assert_eq!(
            parsed.reader_features,
            Some(vec![ReaderFeature::DeletionVectors])
        );
        assert!(parsed.accepts(&ResponseFormat::Parquet));
        assert!(!parsed.accepts(&ResponseFormat::Delta));

        assert!(DeltaSharingCapabilities::parse_header("responseformat=avro").is_err());
    }

    #[test]
    fn test_query_response_file_line() {
        let raw = json!({
            "file": {
                "url": "https://bucket/part-0.parquet?sig=x",
                "id": "abc",
                "partitionValues": {"letter": "a"},
                "size": 783,
                "stats": "{\"numRecords\":1}",
                "expirationTimestamp": 1700000000000i64
            }
        });
        let line: QueryResponseLine = serde_json::from_value(raw).unwrap();
        let QueryResponseLine::File(file) = line else {
            panic!("expected a file line");
        };
        assert_eq!(file.size, 783);
        assert_eq!(
            file.partition_values.get("letter").map(String::as_str),
            Some("a")
        );
        assert_eq!(file.version, None);
    }

    #[test]
    fn test_parse_capabilities_header() {
        let raw = "responseformat=delta;readerfeatures=deletionvectors,columnmapping";
        let parsed = DeltaSharingCapabilities::parse_header(raw).unwrap();
        assert_eq!(parsed.format, Some(vec![ResponseFormat::Delta]));
        assert_eq!(
            parsed.reader_features,
            Some(vec![
                ReaderFeature::DeletionVectors,
                ReaderFeature::ColumnMapping
            ])
        );
    }
}
