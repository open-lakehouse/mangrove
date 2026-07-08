//! The UC catalog-managed Delta table contract, the stored-property projection,
//! and the Delta-API ↔ UC column conversion.
//!
//! Ported from the Unity Catalog Java reference
//! (`service/delta/{UcManagedDeltaContract,DeltaConsts,DeltaPropertyMapper}.java`
//! and `utils/ColumnUtils.java`). The contract is the single source of truth that
//! `createStagingTable` advertises and that `createTable` / `updateTable` validate
//! against, so producer and consumer cannot drift.

use std::collections::BTreeMap;

use unitycatalog_common::models::tables::v1::{Column, ColumnTypeName};

use crate::rest::routers::delta::models::{
    DeltaArrayType, DeltaCreateTableRequest, DeltaDataType, DeltaDecimalType,
    DeltaDomainMetadataUpdates, DeltaMapType, DeltaProtocol, DeltaStructField, DeltaStructType,
};
use crate::{Error, Result};

// ===================================================================
// Contract constants (DeltaConsts + UcManagedDeltaContract)
// ===================================================================

/// Reader version 3 is the minimum that supports per-feature reader-features negotiation.
pub const REQUIRED_MIN_READER_VERSION: i32 = 3;
/// Writer version 7 is the minimum that supports per-feature writer-features negotiation.
pub const REQUIRED_MIN_WRITER_VERSION: i32 = 7;

// Table-feature spec names.
const FEATURE_CATALOG_MANAGED: &str = "catalogManaged";
const FEATURE_V2_CHECKPOINT: &str = "v2Checkpoint";
const FEATURE_VACUUM_PROTOCOL_CHECK: &str = "vacuumProtocolCheck";
const FEATURE_COLUMN_MAPPING: &str = "columnMapping";
const FEATURE_IN_COMMIT_TIMESTAMP: &str = "inCommitTimestamp";
const FEATURE_DOMAIN_METADATA: &str = "domainMetadata";
const FEATURE_ROW_TRACKING: &str = "rowTracking";

/// Required reader-features (the reader-writer subset of the required features).
///
/// `deletionVectors` is intentionally NOT required: it is a *recommended* Delta
/// feature, and requiring it on every managed table makes those tables
/// un-preview-able by the in-browser wasm query engine (`crates/query-wasm`),
/// which refuses `delta.enableDeletionVectors=true`. Writers may still enable it
/// explicitly; it is simply no longer mandated.
pub const REQUIRED_READER_FEATURES: &[&str] = &[
    FEATURE_CATALOG_MANAGED,
    FEATURE_V2_CHECKPOINT,
    FEATURE_VACUUM_PROTOCOL_CHECK,
];
/// Required writer-features (reader-writer features + the writer-only `inCommitTimestamp`).
pub const REQUIRED_WRITER_FEATURES: &[&str] = &[
    FEATURE_CATALOG_MANAGED,
    FEATURE_V2_CHECKPOINT,
    FEATURE_VACUUM_PROTOCOL_CHECK,
    FEATURE_IN_COMMIT_TIMESTAMP,
];
/// Suggested reader-features.
pub const SUGGESTED_READER_FEATURES: &[&str] = &[FEATURE_COLUMN_MAPPING];
/// Suggested writer-features.
pub const SUGGESTED_WRITER_FEATURES: &[&str] = &[
    FEATURE_COLUMN_MAPPING,
    FEATURE_DOMAIN_METADATA,
    FEATURE_ROW_TRACKING,
];

// Table-property keys.
pub const PROP_UC_TABLE_ID: &str = "io.unitycatalog.tableId";
pub const PROP_LAST_UPDATE_VERSION: &str = "delta.lastUpdateVersion";
pub const PROP_LAST_COMMIT_TIMESTAMP: &str = "delta.lastCommitTimestamp";
pub const PROP_CHECKPOINT_POLICY: &str = "delta.checkpointPolicy";
pub const PROP_ENABLE_DELETION_VECTORS: &str = "delta.enableDeletionVectors";
pub const PROP_ENABLE_IN_COMMIT_TIMESTAMPS: &str = "delta.enableInCommitTimestamps";
pub const PROP_CHECKPOINT_WRITE_STATS_AS_STRUCT: &str = "delta.checkpoint.writeStatsAsStruct";
pub const PROP_CHECKPOINT_WRITE_STATS_AS_JSON: &str = "delta.checkpoint.writeStatsAsJson";
const PROP_MIN_READER_VERSION: &str = "delta.minReaderVersion";
const PROP_MIN_WRITER_VERSION: &str = "delta.minWriterVersion";
const PROP_CLUSTERING_COLUMNS: &str = "delta.clusteringColumns";
const PROP_ROW_TRACKING_HIGH_WATER_MARK: &str = "delta.rowTracking.rowIdHighWaterMark";
const FEATURE_PREFIX: &str = "delta.feature.";
const FEATURE_SUPPORTED: &str = "supported";

/// Required properties with fixed values (excludes `io.unitycatalog.tableId`, which is
/// per-table). Order is irrelevant; kept as a slice of pairs.
pub const REQUIRED_FIXED_PROPERTIES: &[(&str, &str)] = &[
    // `delta.enableDeletionVectors` is intentionally NOT required — see the note
    // on `REQUIRED_READER_FEATURES`.
    (PROP_CHECKPOINT_POLICY, "v2"),
    (PROP_ENABLE_IN_COMMIT_TIMESTAMPS, "true"),
    (PROP_CHECKPOINT_WRITE_STATS_AS_STRUCT, "true"),
    (PROP_CHECKPOINT_WRITE_STATS_AS_JSON, "true"),
];

/// Suggested properties advertised in the staging response. `None` values mean the
/// client generates the value at commit time.
pub fn suggested_properties() -> BTreeMap<String, Option<String>> {
    BTreeMap::from([
        (
            "delta.columnMapping.mode".to_string(),
            Some("name".to_string()),
        ),
        ("delta.columnMapping.maxColumnId".to_string(), None),
        (
            "delta.enableRowTracking".to_string(),
            Some("true".to_string()),
        ),
        (
            "delta.rowTracking.materializedRowIdColumnName".to_string(),
            None,
        ),
        (
            "delta.rowTracking.materializedRowCommitVersionColumnName".to_string(),
            None,
        ),
        (
            "delta.randomizeFilePrefixes".to_string(),
            Some("true".to_string()),
        ),
        (
            "delta.parquet.compression.codec".to_string(),
            Some("zstd".to_string()),
        ),
    ])
}

/// Required properties advertised in the staging response: the fixed values plus the
/// per-table `io.unitycatalog.tableId` bound to the allocated UUID.
pub fn required_properties(table_id: &str) -> BTreeMap<String, Option<String>> {
    let mut props: BTreeMap<String, Option<String>> = REQUIRED_FIXED_PROPERTIES
        .iter()
        .map(|(k, v)| (k.to_string(), Some(v.to_string())))
        .collect();
    props.insert(PROP_UC_TABLE_ID.to_string(), Some(table_id.to_string()));
    props
}

// ===================================================================
// Contract validation (MANAGED tables only)
// ===================================================================

/// Validate that the supplied protocol, domain-metadata, and properties satisfy the
/// UC catalog-managed Delta contract. Apply only to MANAGED tables; EXTERNAL tables
/// mirror what the client wrote and skip this. All failures map to
/// [`Error::InvalidArgument`] (HTTP 400, the spec's `INVALID_PARAMETER_VALUE`).
pub fn validate(
    protocol: &DeltaProtocol,
    domain_metadata: Option<&DeltaDomainMetadataUpdates>,
    properties: &BTreeMap<String, String>,
) -> Result<()> {
    validate_reader_feature_subset(protocol)?;
    validate_required_versions(protocol)?;
    validate_required_features(protocol)?;
    validate_domain_metadata_against_protocol(protocol, domain_metadata)?;
    validate_required_properties(properties)?;
    Ok(())
}

/// Every reader-feature must also be a writer-feature (Delta-spec rule).
fn validate_reader_feature_subset(protocol: &DeltaProtocol) -> Result<()> {
    let Some(reader) = protocol.reader_features.as_ref() else {
        return Ok(());
    };
    let writer: &[String] = protocol.writer_features.as_deref().unwrap_or(&[]);
    for rf in reader {
        if !writer.iter().any(|wf| wf == rf) {
            return Err(Error::invalid_argument(format!(
                "Feature '{rf}' is in reader-features but not writer-features. Per the Delta \
                 protocol, every reader-feature must also be a writer-feature."
            )));
        }
    }
    Ok(())
}

fn validate_required_versions(protocol: &DeltaProtocol) -> Result<()> {
    if protocol.min_reader_version < REQUIRED_MIN_READER_VERSION {
        return Err(Error::invalid_argument(format!(
            "MANAGED table minReaderVersion must be at least {REQUIRED_MIN_READER_VERSION} \
             (got {}).",
            protocol.min_reader_version
        )));
    }
    if protocol.min_writer_version < REQUIRED_MIN_WRITER_VERSION {
        return Err(Error::invalid_argument(format!(
            "MANAGED table minWriterVersion must be at least {REQUIRED_MIN_WRITER_VERSION} \
             (got {}).",
            protocol.min_writer_version
        )));
    }
    Ok(())
}

fn validate_required_features(protocol: &DeltaProtocol) -> Result<()> {
    let reader: &[String] = protocol.reader_features.as_deref().unwrap_or(&[]);
    let writer: &[String] = protocol.writer_features.as_deref().unwrap_or(&[]);
    for required in REQUIRED_READER_FEATURES {
        if !reader.iter().any(|f| f == required) {
            return Err(Error::invalid_argument(format!(
                "MANAGED table missing required reader-feature '{required}'."
            )));
        }
    }
    for required in REQUIRED_WRITER_FEATURES {
        if !writer.iter().any(|f| f == required) {
            return Err(Error::invalid_argument(format!(
                "MANAGED table missing required writer-feature '{required}'."
            )));
        }
    }
    Ok(())
}

fn validate_domain_metadata_against_protocol(
    protocol: &DeltaProtocol,
    domain_metadata: Option<&DeltaDomainMetadataUpdates>,
) -> Result<()> {
    let Some(dm) = domain_metadata else {
        return Ok(());
    };
    let writer: &[String] = protocol.writer_features.as_deref().unwrap_or(&[]);
    if dm.delta_clustering.is_some() && !writer.iter().any(|f| f == "clustering") {
        return Err(Error::invalid_argument(
            "domain-metadata.delta.clustering requires the 'clustering' writer feature.",
        ));
    }
    if dm.delta_row_tracking.is_some() && !writer.iter().any(|f| f == FEATURE_ROW_TRACKING) {
        return Err(Error::invalid_argument(
            "domain-metadata.delta.rowTracking requires the 'rowTracking' writer feature.",
        ));
    }
    Ok(())
}

fn validate_required_properties(properties: &BTreeMap<String, String>) -> Result<()> {
    for (key, expected) in REQUIRED_FIXED_PROPERTIES {
        match properties.get(*key) {
            Some(actual) if actual == expected => {}
            actual => {
                return Err(Error::invalid_argument(format!(
                    "MANAGED table required property '{key}' must be '{expected}' (got '{}').",
                    actual.map(String::as_str).unwrap_or("<missing>")
                )));
            }
        }
    }
    match properties.get(PROP_UC_TABLE_ID) {
        Some(v) if !v.is_blank_trim() => {}
        _ => {
            return Err(Error::invalid_argument(format!(
                "MANAGED table required property '{PROP_UC_TABLE_ID}' is missing."
            )));
        }
    }
    Ok(())
}

/// Cross-check the client's `io.unitycatalog.tableId` property against the
/// UC-allocated UUID for the table (the staging UUID at create time).
pub fn validate_table_id_property(
    properties: &BTreeMap<String, String>,
    expected_table_id: &str,
) -> Result<()> {
    let actual = properties.get(PROP_UC_TABLE_ID).ok_or_else(|| {
        Error::invalid_argument(format!("Properties does not contain {PROP_UC_TABLE_ID}."))
    })?;
    if actual != expected_table_id {
        return Err(Error::invalid_argument(format!(
            "the table id ({expected_table_id}) does not match the properties \
             {PROP_UC_TABLE_ID}({actual})."
        )));
    }
    Ok(())
}

trait BlankExt {
    fn is_blank_trim(&self) -> bool;
}
impl BlankExt for String {
    fn is_blank_trim(&self) -> bool {
        self.trim().is_empty()
    }
}

// ===================================================================
// Stored-property projection (DeltaPropertyMapper)
// ===================================================================

/// Build the stored UC table-property map from a createTable request: client
/// properties first, then overlay protocol-derived (`delta.feature.*`,
/// `delta.minReaderVersion`/`minWriterVersion`), domain-metadata projections, the
/// version-0 commit timestamp, and `delta.lastUpdateVersion=0`.
pub fn build_stored_properties(req: &DeltaCreateTableRequest) -> BTreeMap<String, String> {
    let mut merged = req.properties.clone();
    derive_from_protocol(&mut merged, &req.protocol);
    if let Some(dm) = req.domain_metadata.as_ref() {
        derive_from_domain_metadata(&mut merged, dm);
    }
    merged.insert(
        PROP_LAST_COMMIT_TIMESTAMP.to_string(),
        req.last_commit_timestamp_ms.to_string(),
    );
    // createTable is always the version-0 commit.
    merged.insert(PROP_LAST_UPDATE_VERSION.to_string(), "0".to_string());
    merged
}

/// Overlay the `delta.feature.*` + version properties implied by a protocol block.
pub fn derive_from_protocol(props: &mut BTreeMap<String, String>, protocol: &DeltaProtocol) {
    props.insert(
        PROP_MIN_READER_VERSION.to_string(),
        protocol.min_reader_version.to_string(),
    );
    props.insert(
        PROP_MIN_WRITER_VERSION.to_string(),
        protocol.min_writer_version.to_string(),
    );
    for feature in protocol
        .reader_features
        .iter()
        .chain(protocol.writer_features.iter())
        .flatten()
    {
        props.insert(
            format!("{FEATURE_PREFIX}{feature}"),
            FEATURE_SUPPORTED.to_string(),
        );
    }
}

/// Overlay the table properties implied by a domain-metadata block.
pub fn derive_from_domain_metadata(
    props: &mut BTreeMap<String, String>,
    dm: &DeltaDomainMetadataUpdates,
) {
    if let Some(clustering) = dm.delta_clustering.as_ref() {
        // Encode as JSON so nested column paths don't collapse into dotted strings.
        if let Ok(json) = serde_json::to_string(&clustering.clustering_columns) {
            props.insert(PROP_CLUSTERING_COLUMNS.to_string(), json);
        }
    }
    if let Some(row_tracking) = dm.delta_row_tracking.as_ref() {
        props.insert(
            PROP_ROW_TRACKING_HIGH_WATER_MARK.to_string(),
            row_tracking.row_id_high_water_mark.to_string(),
        );
    }
}

// ===================================================================
// Column conversion (ColumnUtils)
// ===================================================================

/// Convert Delta API columns + partition-column names into UC [`Column`]s, mirroring
/// the reference `ColumnUtils.toColumnInfos` + `applyPartitionColumns`.
pub fn delta_columns_to_uc(
    columns: &DeltaStructType,
    partition_columns: Option<&[String]>,
) -> Result<Vec<Column>> {
    let partition_index = |name: &str| {
        partition_columns
            .and_then(|pcs| pcs.iter().position(|p| p.eq_ignore_ascii_case(name)))
            .map(|i| i as i32)
    };
    let cols = columns
        .fields
        .iter()
        .enumerate()
        .map(|(idx, f)| {
            Ok(Column {
                name: f.name.clone(),
                type_text: catalog_string(&f.data_type),
                type_json: serde_json::to_string(f)?,
                type_name: resolve_column_type_name(&f.data_type)? as i32,
                position: Some(idx as i32),
                nullable: Some(f.nullable),
                partition_index: partition_index(&f.name),
                ..Default::default()
            })
        })
        .collect::<Result<Vec<_>>>()?;

    // Every named partition column must exist in the schema.
    if let Some(pcs) = partition_columns {
        for pc in pcs {
            if !columns
                .fields
                .iter()
                .any(|f| f.name.eq_ignore_ascii_case(pc))
            {
                return Err(Error::invalid_argument(format!(
                    "partition column '{pc}' is not present in the table schema"
                )));
            }
        }
    }
    Ok(cols)
}

/// Reconstruct the Delta API `columns` from stored UC [`Column`]s for `loadTable`.
/// Uses each column's stored `type_json` (the original Delta type), falling back to
/// the bare type name string when absent.
pub fn uc_columns_to_delta(columns: &[Column]) -> DeltaStructType {
    let mut sorted: Vec<&Column> = columns.iter().collect();
    sorted.sort_by_key(|c| c.position.unwrap_or(i32::MAX));
    let fields = sorted
        .into_iter()
        .map(|c| {
            let data_type = serde_json::from_str::<DeltaDataType>(&c.type_json)
                .unwrap_or_else(|_| DeltaDataType::Primitive(c.type_text.clone()));
            DeltaStructField {
                name: c.name.clone(),
                data_type,
                nullable: c.nullable.unwrap_or(true),
                metadata: Default::default(),
            }
        })
        .collect();
    DeltaStructType {
        type_tag: Default::default(),
        fields,
    }
}

/// Map a Delta data type to the UC [`ColumnTypeName`], mirroring
/// `ColumnUtils.resolveColumnTypeName`. Primitive names follow the Delta schema
/// serialization format; `decimal(p,s)` strings resolve to `Decimal`.
fn resolve_column_type_name(data_type: &DeltaDataType) -> Result<ColumnTypeName> {
    Ok(match data_type {
        DeltaDataType::Array(_) => ColumnTypeName::Array,
        DeltaDataType::Map(_) => ColumnTypeName::Map,
        DeltaDataType::Struct(_) => ColumnTypeName::Struct,
        DeltaDataType::Decimal(_) => ColumnTypeName::Decimal,
        DeltaDataType::Primitive(p) => primitive_type_name(p)?,
    })
}

fn primitive_type_name(primitive: &str) -> Result<ColumnTypeName> {
    if primitive.starts_with("decimal") {
        return Ok(ColumnTypeName::Decimal);
    }
    Ok(match primitive {
        "boolean" => ColumnTypeName::Boolean,
        "byte" => ColumnTypeName::Byte,
        "short" => ColumnTypeName::Short,
        "integer" => ColumnTypeName::Int,
        "long" => ColumnTypeName::Long,
        "float" => ColumnTypeName::Float,
        "double" => ColumnTypeName::Double,
        "date" => ColumnTypeName::Date,
        "timestamp" => ColumnTypeName::Timestamp,
        "timestamp_ntz" => ColumnTypeName::TimestampNtz,
        "string" => ColumnTypeName::String,
        "binary" => ColumnTypeName::Binary,
        "variant" => ColumnTypeName::Variant,
        other => {
            return Err(Error::invalid_argument(format!(
                "unsupported Delta primitive type '{other}'"
            )));
        }
    })
}

/// The SQL/catalog-string form of a Delta data type (mirrors `ColumnUtils.toCatalogString`).
fn catalog_string(data_type: &DeltaDataType) -> String {
    match data_type {
        DeltaDataType::Primitive(p) => p.clone(),
        DeltaDataType::Decimal(DeltaDecimalType {
            precision, scale, ..
        }) => format!("decimal({precision},{scale})"),
        DeltaDataType::Array(a) => {
            let DeltaArrayType { element_type, .. } = a.as_ref();
            format!("array<{}>", catalog_string(element_type))
        }
        DeltaDataType::Map(m) => {
            let DeltaMapType {
                key_type,
                value_type,
                ..
            } = m.as_ref();
            format!(
                "map<{},{}>",
                catalog_string(key_type),
                catalog_string(value_type)
            )
        }
        DeltaDataType::Struct(s) => {
            let inner = s
                .fields
                .iter()
                .map(|f| format!("{}:{}", f.name, catalog_string(&f.data_type)))
                .collect::<Vec<_>>()
                .join(",");
            format!("struct<{inner}>")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rest::routers::delta::models::{DeltaDataSourceFormat, DeltaProtocol};

    fn compliant_protocol() -> DeltaProtocol {
        DeltaProtocol {
            min_reader_version: 3,
            min_writer_version: 7,
            reader_features: Some(
                REQUIRED_READER_FEATURES
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
            writer_features: Some(
                REQUIRED_WRITER_FEATURES
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
            ),
        }
    }

    fn compliant_properties(table_id: &str) -> BTreeMap<String, String> {
        let mut p: BTreeMap<String, String> = REQUIRED_FIXED_PROPERTIES
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        p.insert(PROP_UC_TABLE_ID.to_string(), table_id.to_string());
        p
    }

    #[test]
    fn compliant_passes() {
        assert!(validate(&compliant_protocol(), None, &compliant_properties("id-1")).is_ok());
    }

    #[test]
    fn missing_feature_rejected() {
        let mut proto = compliant_protocol();
        // Drop every required feature, leaving only an unrelated one.
        proto.writer_features = Some(vec!["someOtherFeature".to_string()]);
        let err = validate(&proto, None, &compliant_properties("id-1")).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
    }

    #[test]
    fn low_version_rejected() {
        let mut proto = compliant_protocol();
        proto.min_writer_version = 5;
        let err = validate(&proto, None, &compliant_properties("id-1")).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
    }

    #[test]
    fn reader_not_subset_of_writer_rejected() {
        let mut proto = compliant_protocol();
        proto.reader_features = Some(vec!["someReaderOnly".to_string()]);
        let err = validate(&proto, None, &compliant_properties("id-1")).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
    }

    #[test]
    fn missing_fixed_property_rejected() {
        let mut props = compliant_properties("id-1");
        props.remove(PROP_CHECKPOINT_POLICY);
        let err = validate(&compliant_protocol(), None, &props).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
    }

    #[test]
    fn table_id_mismatch_rejected() {
        let props = compliant_properties("id-1");
        let err = validate_table_id_property(&props, "id-2").unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
        assert!(validate_table_id_property(&props, "id-1").is_ok());
    }

    #[test]
    fn build_stored_properties_derives_features() {
        let req = DeltaCreateTableRequest {
            name: "t".into(),
            location: "s3://b/t".into(),
            table_type: crate::rest::routers::delta::models::DeltaTableType::Managed,
            comment: None,
            columns: DeltaStructType {
                type_tag: Default::default(),
                fields: vec![],
            },
            partition_columns: None,
            protocol: compliant_protocol(),
            properties: compliant_properties("id-1"),
            domain_metadata: None,
            last_commit_timestamp_ms: 1700,
            uniform: None,
            data_source_format: Some(DeltaDataSourceFormat::Delta),
        };
        let props = build_stored_properties(&req);
        assert_eq!(
            props
                .get("delta.feature.catalogManaged")
                .map(String::as_str),
            Some("supported")
        );
        assert_eq!(
            props.get(PROP_LAST_UPDATE_VERSION).map(String::as_str),
            Some("0")
        );
        assert_eq!(
            props.get(PROP_LAST_COMMIT_TIMESTAMP).map(String::as_str),
            Some("1700")
        );
        assert_eq!(
            props.get(PROP_MIN_WRITER_VERSION).map(String::as_str),
            Some("7")
        );
    }

    #[test]
    fn column_conversion_roundtrips_primitive_and_partition() {
        let columns = DeltaStructType {
            type_tag: Default::default(),
            fields: vec![
                DeltaStructField {
                    name: "id".into(),
                    data_type: DeltaDataType::Primitive("long".into()),
                    nullable: false,
                    metadata: Default::default(),
                },
                DeltaStructField {
                    name: "amount".into(),
                    data_type: DeltaDataType::Decimal(DeltaDecimalType {
                        type_tag: Default::default(),
                        precision: 10,
                        scale: 2,
                    }),
                    nullable: true,
                    metadata: Default::default(),
                },
            ],
        };
        let uc = delta_columns_to_uc(&columns, Some(&["id".to_string()])).unwrap();
        assert_eq!(uc.len(), 2);
        assert_eq!(uc[0].name, "id");
        assert_eq!(uc[0].type_name, ColumnTypeName::Long as i32);
        assert_eq!(uc[0].partition_index, Some(0));
        assert_eq!(uc[1].type_text, "decimal(10,2)");
        assert_eq!(uc[1].type_name, ColumnTypeName::Decimal as i32);

        // Reverse: stored type_json should reconstruct the Delta types.
        let back = uc_columns_to_delta(&uc);
        assert_eq!(back.fields.len(), 2);
        assert_eq!(back.fields[0].name, "id");
    }

    #[test]
    fn unknown_partition_column_rejected() {
        let columns = DeltaStructType {
            type_tag: Default::default(),
            fields: vec![DeltaStructField {
                name: "id".into(),
                data_type: DeltaDataType::Primitive("long".into()),
                nullable: false,
                metadata: Default::default(),
            }],
        };
        let err = delta_columns_to_uc(&columns, Some(&["missing".to_string()])).unwrap_err();
        assert!(matches!(err, Error::InvalidArgument(_)), "{err:?}");
    }
}
