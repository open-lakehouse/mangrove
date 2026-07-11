//! Portable UC column model used by the managed-table contract.
//!
//! The Delta business logic derives UC columns from the Delta wire schema
//! (`delta_columns_to_uc`) and reconstructs the wire schema from stored columns
//! (`uc_columns_to_delta`). To keep this crate self-contained — free of
//! `unitycatalog-common` and its generated proto types — it owns a small
//! [`Column`] / [`ColumnTypeName`] pair mirroring the fields the contract needs.
//!
//! [`ColumnTypeName`]'s discriminants are identical to the generated
//! `unitycatalog_common::models::tables::v1::ColumnTypeName`, so `as i32`
//! round-trips across the boundary and each server's adapter can map between the
//! two by value. The adapter is where the mapping to the server's own column type
//! lives; the crate never sees it.

/// A Unity Catalog column, the portable shape exchanged with the backend port.
///
/// This is the subset of the UC `Column` message the managed-table contract
/// produces and consumes. The backend adapter maps between this and its own
/// column representation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Column {
    /// Name of the column.
    pub name: String,
    /// Full data type specification as SQL / catalog-string text.
    pub type_text: String,
    /// Full data type specification, JSON-serialized (the original Delta type).
    pub type_json: String,
    /// Ordinal position of the column (starting at 0).
    pub position: Option<i32>,
    /// Data type name.
    pub type_name: ColumnTypeName,
    /// User-provided free-form description.
    pub comment: Option<String>,
    /// Whether the field may be null.
    pub nullable: Option<bool>,
    /// Partition index for the column, if it is a partition column.
    pub partition_index: Option<i32>,
}

/// UC column data-type name.
///
/// Discriminants match the generated
/// `unitycatalog_common::models::tables::v1::ColumnTypeName` so `as i32` values
/// are interchangeable across the port boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(i32)]
pub enum ColumnTypeName {
    #[default]
    Unspecified = 0,
    Boolean = 1,
    Byte = 2,
    Short = 3,
    Int = 4,
    Long = 5,
    Float = 6,
    Double = 7,
    Date = 8,
    Timestamp = 9,
    String = 10,
    Binary = 11,
    Decimal = 12,
    Interval = 13,
    Array = 14,
    Struct = 15,
    Map = 16,
    Char = 17,
    Null = 18,
    UserDefinedType = 19,
    TimestampNtz = 20,
    Variant = 21,
    TableType = 22,
}

impl From<i32> for ColumnTypeName {
    /// The inverse of `as i32`. Unknown discriminants map to
    /// [`Unspecified`](Self::Unspecified), matching prost's open-enum behavior.
    fn from(v: i32) -> Self {
        match v {
            1 => Self::Boolean,
            2 => Self::Byte,
            3 => Self::Short,
            4 => Self::Int,
            5 => Self::Long,
            6 => Self::Float,
            7 => Self::Double,
            8 => Self::Date,
            9 => Self::Timestamp,
            10 => Self::String,
            11 => Self::Binary,
            12 => Self::Decimal,
            13 => Self::Interval,
            14 => Self::Array,
            15 => Self::Struct,
            16 => Self::Map,
            17 => Self::Char,
            18 => Self::Null,
            19 => Self::UserDefinedType,
            20 => Self::TimestampNtz,
            21 => Self::Variant,
            22 => Self::TableType,
            _ => Self::Unspecified,
        }
    }
}
