//! Build per-file DataFusion [`Statistics`] from the kernel's metadata-only stats state machine.
//!
//! The kernel `Scan::scan_stats_metadata_state_machine()` terminal carries one row per live data
//! file with a top-level `stats` struct (physical leaf names):
//!
//! ```text
//! stats: STRUCT<
//!   numRecords:  i64,
//!   nullCount:   STRUCT<physicalLeaf -> i64>,          // every leaf
//!   minValues:   STRUCT<physicalLeaf -> nativeType>?,  // skipping-eligible primitives only; may be absent
//!   maxValues:   STRUCT<physicalLeaf -> nativeType>?,  // same
//!   tightBounds: bool,
//! >
//! ```
//!
//! This module remaps those **physical** leaf names to their **logical** output positions (via the
//! Stage-3 [`ColumnMappingResolver`]) and collapses nested-struct leaf stats to **top-level logical
//! columns** (matching delta-rs), producing one [`ColumnStatistics`] per top-level logical column in
//! `scan.logical_schema()` order. DataFusion prunes against these once they are attached to each
//! file's [`PartitionedFile`](datafusion_datasource::PartitionedFile); the outer projection is
//! spliced *above* the scan, so the vector must align to the **full pre-projection** logical schema
//! (DataFusion projects the statistics itself).
//!
//! # Precision policy (delta-rs parity)
//!
//! * `num_rows` = `Exact(numRecords)`; `total_byte_size` = `Absent` (kernel stats carry no decoded
//!   byte size — `add.size` is the *file* size, not the Arrow size, so it is deliberately not used).
//! * `null_count` = `Exact` (the kernel `nullCount` struct covers every leaf).
//! * `min_value`/`max_value` = `Exact` iff the file's `tightBounds` is true, else `Inexact`;
//!   `Absent` when the leaf is not min/max-eligible, the `minValues`/`maxValues` struct is absent,
//!   or the top-level column is a struct (top-level collapse emits no bounds for structs).
//! * `distinct_count`, `sum_value`, `byte_size` = `Absent`.
//!
//! Every path degrades to all-`Absent` / an absent map entry rather than panicking when stats are
//! missing (a table with no eligible columns, or `.with_stats` yielding an empty struct).

use std::collections::HashMap;
use std::sync::Arc;

use datafusion_common::stats::Precision;
use datafusion_common::{ColumnStatistics, ScalarValue, Statistics};
use delta_kernel::arrow::array::types::Int64Type;
use delta_kernel::arrow::array::{Array, ArrayRef, AsArray, RecordBatch, StructArray};
use delta_kernel::expressions::ColumnName;
use delta_kernel::scan::Scan;

use crate::compile::column_mapping::{ColumnMappingResolver, LeafMapping};

/// Sub-field names inside the terminal `stats` struct.
const NUM_RECORDS: &str = "numRecords";
const TIGHT_BOUNDS: &str = "tightBounds";
const NULL_COUNT: &str = "nullCount";
const MIN_VALUES: &str = "minValues";
const MAX_VALUES: &str = "maxValues";

/// The terminal row's top-level column carrying the parsed stats struct.
const STATS_COLUMN: &str = "stats";
/// The terminal row's top-level column carrying the raw `add.path`.
const PATH_COLUMN: &str = "path";

/// Per-file statistics keyed by the **raw `add.path` string** (as the metadata-stats terminal emits
/// it). The per-file attach layer looks up by the same raw path off its own upstream row, so no URL
/// resolution round-trip is needed on either side. `Arc<Statistics>` because `PartitionedFile`
/// stores `Option<Arc<Statistics>>` directly.
pub(crate) type FileStatsMap = HashMap<String, Arc<Statistics>>;

/// Build a `raw add.path -> Arc<Statistics>` map from the metadata-stats terminal batches.
///
/// The key is the **raw, un-normalized `add.path` string** exactly as the terminal emits it; the
/// per-file attach layer looks it up by the same raw path it reads off its own upstream row, so no
/// URL resolution/decoding round-trip is needed on either side (sidestepping any
/// `base_url`-vs-`table_root` divergence).
///
/// Files whose `stats` struct is null (no stats recorded) are omitted from the map — the per-file
/// layer then leaves `PartitionedFile.statistics = None`, which is correct (unknown).
// Wired into the provider `scan()` path in a following commit (the threading seam lands first).
#[allow(dead_code)]
pub(crate) fn build_file_statistics(scan: &Scan, stats_batches: &[RecordBatch]) -> FileStatsMap {
    let resolver = ColumnMappingResolver::from_scan(scan);
    let top_level: Vec<String> = scan
        .logical_schema()
        .fields()
        .map(|f| f.name.clone())
        .collect();
    build_from_parts(&resolver, &top_level, stats_batches)
}

/// Core of [`build_file_statistics`], parameterized on the already-built resolver and top-level
/// logical column names — the seam the unit tests drive directly (a real [`Scan`] needs a snapshot).
fn build_from_parts(
    resolver: &ColumnMappingResolver,
    top_level: &[String],
    stats_batches: &[RecordBatch],
) -> FileStatsMap {
    // Per top-level logical column, the leaves that collapse into it (in resolver leaf order).
    let mut leaves_by_top: HashMap<&str, Vec<&LeafMapping>> = HashMap::new();
    for leaf in resolver.leaves() {
        if let Some(top) = leaf.logical.path().first() {
            leaves_by_top.entry(top.as_str()).or_default().push(leaf);
        }
    }

    let mut out: FileStatsMap = HashMap::new();
    for batch in stats_batches {
        let Some((path_arr, stats_arr)) = terminal_columns(batch) else {
            continue;
        };
        for row in 0..batch.num_rows() {
            if path_arr.is_null(row) || stats_arr.is_null(row) {
                continue;
            }
            let path = path_arr.as_string::<i32>().value(row).to_string();
            let stats = build_row_statistics(stats_arr, row, top_level, &leaves_by_top);
            out.insert(path, Arc::new(stats));
        }
    }
    out
}

/// Pull the `path` (Utf8) and `stats` (Struct) columns off a terminal batch, if both are present
/// with the expected types. Returns `None` (batch skipped) otherwise — e.g. a `.with_stats`-less
/// drive whose terminal has no `stats` column.
fn terminal_columns(batch: &RecordBatch) -> Option<(&ArrayRef, &StructArray)> {
    let path_arr = batch.column_by_name(PATH_COLUMN)?;
    path_arr.as_string_opt::<i32>()?;
    let stats_arr = batch.column_by_name(STATS_COLUMN)?.as_struct_opt()?;
    Some((path_arr, stats_arr))
}

/// Build the [`Statistics`] for one terminal row (one data file).
fn build_row_statistics(
    stats_arr: &StructArray,
    row: usize,
    top_level: &[String],
    leaves_by_top: &HashMap<&str, Vec<&LeafMapping>>,
) -> Statistics {
    let num_rows = struct_field(stats_arr, NUM_RECORDS)
        .filter(|a| !a.is_null(row))
        .and_then(|a| usize::try_from(a.as_primitive::<Int64Type>().value(row)).ok())
        .map_or(Precision::Absent, Precision::Exact);

    let tight_bounds = struct_field(stats_arr, TIGHT_BOUNDS)
        .filter(|a| !a.is_null(row))
        .map(|a| a.as_boolean().value(row))
        .unwrap_or(false);

    // The nested stat sub-structs (any may be absent — e.g. no min/max-eligible column).
    let null_count_struct = struct_field(stats_arr, NULL_COUNT).and_then(|a| a.as_struct_opt());
    let min_struct = struct_field(stats_arr, MIN_VALUES).and_then(|a| a.as_struct_opt());
    let max_struct = struct_field(stats_arr, MAX_VALUES).and_then(|a| a.as_struct_opt());

    let column_statistics = top_level
        .iter()
        .map(|top| {
            let leaves = leaves_by_top
                .get(top.as_str())
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            collapse_column(
                leaves,
                row,
                tight_bounds,
                null_count_struct,
                min_struct,
                max_struct,
            )
        })
        .collect();

    Statistics {
        num_rows,
        total_byte_size: Precision::Absent,
        column_statistics,
    }
}

/// Collapse the leaves of one top-level logical column into a single [`ColumnStatistics`]
/// (delta-rs top-level parity):
///
/// * `null_count` — the sum of every contributing leaf's null count, `Exact` iff **all** leaves are
///   present and non-null in the `nullCount` struct, else `Absent`.
/// * `min_value`/`max_value` — populated only for a **primitive** top-level column (exactly one
///   leaf whose logical path is length 1). Struct columns collapse to `Absent` bounds.
fn collapse_column(
    leaves: &[&LeafMapping],
    row: usize,
    tight_bounds: bool,
    null_count_struct: Option<&StructArray>,
    min_struct: Option<&StructArray>,
    max_struct: Option<&StructArray>,
) -> ColumnStatistics {
    let mut col = ColumnStatistics::new_unknown();

    // --- null_count: sum across leaves, Exact only if every leaf contributes ---
    if !leaves.is_empty() {
        let mut total: i64 = 0;
        let mut all_present = true;
        for leaf in leaves {
            match leaf_i64(null_count_struct, &leaf.physical, row) {
                Some(n) => total += n,
                None => {
                    all_present = false;
                    break;
                }
            }
        }
        if let (true, Ok(n)) = (all_present, usize::try_from(total)) {
            col.null_count = Precision::Exact(n);
        }
    }

    // --- min/max: only for a single-leaf, top-level (length-1 path) primitive column ---
    if let [leaf] = leaves
        && leaf.logical.path().len() == 1
    {
        col.min_value = wrap_bound(leaf_scalar(min_struct, &leaf.physical, row), tight_bounds);
        col.max_value = wrap_bound(leaf_scalar(max_struct, &leaf.physical, row), tight_bounds);
    }
    col
}

/// `Exact` iff the file advertised tight bounds, else `Inexact`; `Absent` when the value is missing.
fn wrap_bound(value: Option<ScalarValue>, tight_bounds: bool) -> Precision<ScalarValue> {
    match value {
        Some(v) if tight_bounds => Precision::Exact(v),
        Some(v) => Precision::Inexact(v),
        None => Precision::Absent,
    }
}

/// Read the direct child array of a struct by name, if present.
fn struct_field<'a>(st: &'a StructArray, name: &str) -> Option<&'a ArrayRef> {
    st.column_by_name(name)
}

/// Walk a physical leaf path out of a stats sub-struct (`nullCount`/`minValues`/`maxValues`) and
/// return the leaf array at that path, or `None` if any segment is missing.
fn leaf_array<'a>(root: Option<&'a StructArray>, physical: &ColumnName) -> Option<&'a ArrayRef> {
    let mut current = root?;
    let (last, prefix) = physical.path().split_last()?;
    for seg in prefix {
        current = current.column_by_name(seg)?.as_struct_opt()?;
    }
    current.column_by_name(last)
}

/// Read a physical leaf's i64 value from a stats sub-struct (used for `nullCount`).
fn leaf_i64(root: Option<&StructArray>, physical: &ColumnName, row: usize) -> Option<i64> {
    let arr = leaf_array(root, physical)?;
    (!arr.is_null(row)).then(|| arr.as_primitive::<Int64Type>().value(row))
}

/// Read a physical leaf's value from a stats sub-struct as a [`ScalarValue`] (used for min/max).
fn leaf_scalar(
    root: Option<&StructArray>,
    physical: &ColumnName,
    row: usize,
) -> Option<ScalarValue> {
    let arr = leaf_array(root, physical)?;
    if arr.is_null(row) {
        return None;
    }
    ScalarValue::try_from_array(arr.as_ref(), row).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use delta_kernel::arrow::array::{BooleanArray, Int64Array, StringArray};
    use delta_kernel::arrow::datatypes::{DataType, Field, Fields};
    use delta_kernel::schema::{DataType as KernelDataType, StructField, StructType};

    /// A struct array from `(name, ArrayRef)` children (all nullable).
    fn struct_arr(children: Vec<(&str, ArrayRef)>) -> ArrayRef {
        let fields: Fields = children
            .iter()
            .map(|(name, arr)| Arc::new(Field::new(*name, arr.data_type().clone(), true)))
            .collect();
        let arrays: Vec<ArrayRef> = children.into_iter().map(|(_, a)| a).collect();
        Arc::new(StructArray::new(fields, arrays, None)) as ArrayRef
    }

    /// A one-row terminal batch: `path` (Utf8) + `stats` (struct of the given children).
    fn terminal_batch(path: &str, stats_children: Vec<(&str, ArrayRef)>) -> RecordBatch {
        let stats = struct_arr(stats_children);
        RecordBatch::try_from_iter(vec![
            (PATH_COLUMN, Arc::new(StringArray::from(vec![path])) as ArrayRef),
            (STATS_COLUMN, stats),
        ])
        .unwrap()
    }

    fn i64_arr(v: i64) -> ArrayRef {
        Arc::new(Int64Array::from(vec![v])) as ArrayRef
    }
    fn i64_null() -> ArrayRef {
        Arc::new(Int64Array::from(vec![None::<i64>])) as ArrayRef
    }
    fn bool_arr(v: bool) -> ArrayRef {
        Arc::new(BooleanArray::from(vec![v])) as ArrayRef
    }

    /// A flat two-column CM resolver: logical `id:Long, name:String` ↔ physical `col-a, col-b`.
    fn flat_resolver() -> (ColumnMappingResolver, Vec<String>) {
        let logical = StructType::try_new([
            StructField::nullable("id", KernelDataType::LONG),
            StructField::nullable("name", KernelDataType::STRING),
        ])
        .unwrap();
        let physical = StructType::try_new([
            StructField::nullable("col-a", KernelDataType::LONG),
            StructField::nullable("col-b", KernelDataType::STRING),
        ])
        .unwrap();
        let r = ColumnMappingResolver::from_schemas(&logical, &physical);
        (r, vec!["id".to_string(), "name".to_string()])
    }

    #[test]
    fn flat_stats_remapped_and_ordered() {
        let (resolver, top) = flat_resolver();
        // Physical names on disk; logical order is id, name.
        let batch = terminal_batch(
            "part-0.parquet",
            vec![
                (NUM_RECORDS, i64_arr(10)),
                (
                    NULL_COUNT,
                    struct_arr(vec![("col-a", i64_arr(2)), ("col-b", i64_arr(0))]),
                ),
                (
                    MIN_VALUES,
                    struct_arr(vec![
                        ("col-a", i64_arr(1)),
                        ("col-b", Arc::new(StringArray::from(vec!["apple"])) as ArrayRef),
                    ]),
                ),
                (
                    MAX_VALUES,
                    struct_arr(vec![
                        ("col-a", i64_arr(9)),
                        ("col-b", Arc::new(StringArray::from(vec!["pear"])) as ArrayRef),
                    ]),
                ),
                (TIGHT_BOUNDS, bool_arr(true)),
            ],
        );
        let map = build_from_parts(&resolver, &top, &[batch]);
        let stats = map.get("part-0.parquet").expect("path key hits");
        assert_eq!(stats.num_rows, Precision::Exact(10));
        assert_eq!(stats.total_byte_size, Precision::Absent);
        assert_eq!(stats.column_statistics.len(), 2);

        // Column 0 == logical `id` (physical col-a).
        let id = &stats.column_statistics[0];
        assert_eq!(id.null_count, Precision::Exact(2));
        assert_eq!(id.min_value, Precision::Exact(ScalarValue::Int64(Some(1))));
        assert_eq!(id.max_value, Precision::Exact(ScalarValue::Int64(Some(9))));
        assert_eq!(id.distinct_count, Precision::Absent);

        // Column 1 == logical `name` (physical col-b).
        let name = &stats.column_statistics[1];
        assert_eq!(name.null_count, Precision::Exact(0));
        assert_eq!(
            name.min_value,
            Precision::Exact(ScalarValue::from("apple"))
        );
        assert_eq!(name.max_value, Precision::Exact(ScalarValue::from("pear")));
    }

    #[test]
    fn loose_bounds_are_inexact_null_count_stays_exact() {
        let (resolver, top) = flat_resolver();
        let batch = terminal_batch(
            "f.parquet",
            vec![
                (NUM_RECORDS, i64_arr(3)),
                (
                    NULL_COUNT,
                    struct_arr(vec![("col-a", i64_arr(1)), ("col-b", i64_arr(1))]),
                ),
                (MIN_VALUES, struct_arr(vec![("col-a", i64_arr(0))])),
                (MAX_VALUES, struct_arr(vec![("col-a", i64_arr(5))])),
                (TIGHT_BOUNDS, bool_arr(false)),
            ],
        );
        let map = build_from_parts(&resolver, &top, &[batch]);
        let stats = map.get("f.parquet").unwrap();
        let id = &stats.column_statistics[0];
        assert_eq!(id.null_count, Precision::Exact(1)); // exact regardless of tightBounds
        assert_eq!(id.min_value, Precision::Inexact(ScalarValue::Int64(Some(0))));
        assert_eq!(id.max_value, Precision::Inexact(ScalarValue::Int64(Some(5))));
    }

    #[test]
    fn missing_min_max_struct_yields_absent_bounds() {
        let (resolver, top) = flat_resolver();
        // No min/max eligible column present => minValues/maxValues absent entirely.
        let batch = terminal_batch(
            "g.parquet",
            vec![
                (NUM_RECORDS, i64_arr(1)),
                (
                    NULL_COUNT,
                    struct_arr(vec![("col-a", i64_arr(0)), ("col-b", i64_arr(0))]),
                ),
                (TIGHT_BOUNDS, bool_arr(true)),
            ],
        );
        let map = build_from_parts(&resolver, &top, &[batch]);
        let stats = map.get("g.parquet").unwrap();
        for c in &stats.column_statistics {
            assert_eq!(c.min_value, Precision::Absent);
            assert_eq!(c.max_value, Precision::Absent);
        }
        assert_eq!(stats.column_statistics[0].null_count, Precision::Exact(0));
    }

    #[test]
    fn null_min_leaf_absent_but_null_count_present() {
        let (resolver, top) = flat_resolver();
        let batch = terminal_batch(
            "h.parquet",
            vec![
                (NUM_RECORDS, i64_arr(4)),
                (
                    NULL_COUNT,
                    struct_arr(vec![("col-a", i64_arr(4)), ("col-b", i64_arr(4))]),
                ),
                // col-a min present but NULL (e.g. all-null column) => Absent bound.
                (MIN_VALUES, struct_arr(vec![("col-a", i64_null())])),
                (MAX_VALUES, struct_arr(vec![("col-a", i64_null())])),
                (TIGHT_BOUNDS, bool_arr(true)),
            ],
        );
        let map = build_from_parts(&resolver, &top, &[batch]);
        let id = &map.get("h.parquet").unwrap().column_statistics[0];
        assert_eq!(id.null_count, Precision::Exact(4));
        assert_eq!(id.min_value, Precision::Absent);
    }

    #[test]
    fn nested_struct_collapses_to_top_level() {
        // logical: top:{ a:Long, b:Long }  ↔  physical: col-t:{ col-a, col-b }
        let logical_inner = StructType::try_new([
            StructField::nullable("a", KernelDataType::LONG),
            StructField::nullable("b", KernelDataType::LONG),
        ])
        .unwrap();
        let logical = StructType::try_new([StructField::nullable(
            "top",
            KernelDataType::Struct(Box::new(logical_inner)),
        )])
        .unwrap();
        let physical_inner = StructType::try_new([
            StructField::nullable("col-a", KernelDataType::LONG),
            StructField::nullable("col-b", KernelDataType::LONG),
        ])
        .unwrap();
        let physical = StructType::try_new([StructField::nullable(
            "col-t",
            KernelDataType::Struct(Box::new(physical_inner)),
        )])
        .unwrap();
        let resolver = ColumnMappingResolver::from_schemas(&logical, &physical);
        let top = vec!["top".to_string()];

        let batch = terminal_batch(
            "n.parquet",
            vec![
                (NUM_RECORDS, i64_arr(8)),
                (
                    NULL_COUNT,
                    struct_arr(vec![(
                        "col-t",
                        struct_arr(vec![("col-a", i64_arr(2)), ("col-b", i64_arr(3))]),
                    )]),
                ),
                (
                    MIN_VALUES,
                    struct_arr(vec![(
                        "col-t",
                        struct_arr(vec![("col-a", i64_arr(1)), ("col-b", i64_arr(1))]),
                    )]),
                ),
                (
                    MAX_VALUES,
                    struct_arr(vec![(
                        "col-t",
                        struct_arr(vec![("col-a", i64_arr(9)), ("col-b", i64_arr(9))]),
                    )]),
                ),
                (TIGHT_BOUNDS, bool_arr(true)),
            ],
        );
        let map = build_from_parts(&resolver, &top, &[batch]);
        let stats = map.get("n.parquet").unwrap();
        assert_eq!(stats.column_statistics.len(), 1);
        let top_col = &stats.column_statistics[0];
        // null_count collapses to the sum of both leaves (2 + 3).
        assert_eq!(top_col.null_count, Precision::Exact(5));
        // Struct top-level column carries NO min/max (top-level collapse policy).
        assert_eq!(top_col.min_value, Precision::Absent);
        assert_eq!(top_col.max_value, Precision::Absent);
    }

    #[test]
    fn null_stats_struct_omits_file() {
        let (resolver, top) = flat_resolver();
        // stats column all-null (no stats recorded for this file).
        let stats_fields: Fields = vec![Arc::new(Field::new(NUM_RECORDS, DataType::Int64, true))]
            .into_iter()
            .collect();
        let stats_arr = Arc::new(StructArray::new(
            stats_fields,
            vec![i64_null()],
            Some(vec![false].into()), // the struct itself is null at row 0
        )) as ArrayRef;
        let batch = RecordBatch::try_from_iter(vec![
            (PATH_COLUMN, Arc::new(StringArray::from(vec!["z.parquet"])) as ArrayRef),
            (STATS_COLUMN, stats_arr),
        ])
        .unwrap();
        let map = build_from_parts(&resolver, &top, &[batch]);
        assert!(map.is_empty(), "null stats struct => no map entry");
    }
}
