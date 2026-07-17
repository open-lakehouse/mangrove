//! The narrow-waist column-mapping artifact: [`ColumnMappingResolver`].
//!
//! Delta column mapping means the *logical* schema a table exposes (`id`, `name`, …) can differ
//! from the *physical* schema stored in the parquet files (`col-1a2b…`, …). The kernel already
//! owns the physical→logical translation of the **data** path (its scan `ProjectNode` renames
//! columns as it emits rows — see `crate::compile::logical::project`). But two *side channels* also
//! need the logical↔physical name relation:
//!
//!   * **predicate rewrite** (filter pushdown): a filter written against logical names must be
//!     rewritten to physical names before it is handed to the kernel scan builder;
//!   * **statistics remap**: per-file stats the kernel carries under **physical** leaf names
//!     (`add.stats_parsed.{minValues,maxValues,nullCount}.<physical>`) must be placed at the
//!     **logical** output positions DataFusion prunes against.
//!
//! Rather than re-derive that relation independently in each side channel (the delta-rs
//! `DeltaScanNext` anti-pattern — it computes it three separate ways across three files), this
//! resolver computes the leaf-path relation **once per scan** and both consumers share it. That is
//! the "narrow waist": one place builds the map; the data path stays the kernel's `ProjectNode` and
//! the two side channels read this resolver.
//!
//! # Scope (deliberate non-goals)
//!
//! * This resolver is **scan-global and name-only**. It does *not* subsume
//!   [`FieldIdPhysicalExprAdapterFactory`](crate::exec::field_id_adapter): that adapter is
//!   genuinely *per-file* (schema evolution can shorten the physical schema file-to-file) and does
//!   array-level reshape. See the `// TODO(narrow-waist)` note there — the honest waist is *two
//!   mechanisms, with the name relation computed once and shared by the side channels*, not one.
//! * It has **no consumers yet** (landed ahead of the predicate-rewrite and stats stages). Its unit
//!   tests are the only exercise for now.
//!
//! # How the relation is built
//!
//! The kernel resolves both `scan.logical_schema()` and `scan.physical_schema()` up front; the
//! physical schema is `logical.make_physical(mode)` — the *same structural shape*, with each field
//! renamed to its `physical_name(mode)`. So the resolver walks the two [`StructType`]s in **lockstep
//! by position**, pairing logical and physical fields at every level and recording one entry per
//! **leaf** (primitive) column: `(logical ColumnName path, physical ColumnName path)`. Nested
//! structs contribute dotted paths (`outer.inner`); arrays and maps are treated as opaque leaves
//! (their element/entry names are not part of the column-mapping name relation the side channels
//! use — data skipping does not descend into them, matching the kernel's own
//! `GetReferencedFields`).
//!
//! Because `make_physical` preserves order and structure, positional lockstep is exact and needs no
//! field-id join. (The per-file [`FieldIdPhysicalExprAdapter`](crate::exec::field_id_adapter) *does*
//! join by `PARQUET:field_id` because a file's physical schema may be a shortened/reordered subset;
//! the scan-global schemas here are not.)

use std::collections::HashMap;

use delta_kernel::expressions::ColumnName;
use delta_kernel::scan::Scan;
use delta_kernel::schema::{DataType, StructType};

/// Leaf-name subpaths under a per-file `add.stats_parsed` struct.
const STATS_MIN: &str = "minValues";
const STATS_MAX: &str = "maxValues";
const STATS_NULL_COUNT: &str = "nullCount";

/// Precomputed, scan-global logical↔physical leaf-name relation for a single Delta scan.
///
/// Built once via [`ColumnMappingResolver::from_scan`] and shared by the predicate-rewrite and
/// statistics side channels so both agree on nesting and on which physical name backs each logical
/// leaf. See the module docs for the design rationale.
#[derive(Debug, Clone)]
pub(crate) struct ColumnMappingResolver {
    /// Leaf entries in logical output order. Each pairs a logical leaf column with its physical
    /// leaf column (same nesting depth).
    leaves: Vec<LeafMapping>,
    /// `logical path -> index into `leaves``, for O(1) `logical_to_physical`.
    by_logical: HashMap<Vec<String>, usize>,
    /// `physical path -> index into `leaves``, for O(1) `physical_to_logical`.
    by_physical: HashMap<Vec<String>, usize>,
}

/// One logical↔physical leaf-column pairing.
#[derive(Debug, Clone)]
pub(crate) struct LeafMapping {
    /// Logical (table-facing) column name, e.g. `["outer", "id"]`.
    pub logical: ColumnName,
    /// Physical (on-disk) column name, e.g. `["outer", "col-1a2b"]`.
    pub physical: ColumnName,
}

impl ColumnMappingResolver {
    /// Build the resolver from a kernel [`Scan`], pairing its logical and physical schemas in
    /// lockstep. Correct for every column-mapping mode (`none`/`id`/`name`) because the physical
    /// schema is always `logical.make_physical(mode)` — in `none` mode the two schemas are equal and
    /// every leaf maps to itself.
    pub(crate) fn from_scan(scan: &Scan) -> Self {
        Self::from_schemas(scan.logical_schema(), scan.physical_schema())
    }

    /// Build from a logical/physical [`StructType`] pair directly (the seam the unit tests drive;
    /// [`from_scan`](Self::from_scan) is the production entry point).
    pub(crate) fn from_schemas(logical: &StructType, physical: &StructType) -> Self {
        let mut leaves = Vec::new();
        walk_lockstep(
            logical,
            physical,
            &mut Vec::new(),
            &mut Vec::new(),
            &mut leaves,
        );

        let mut by_logical = HashMap::with_capacity(leaves.len());
        let mut by_physical = HashMap::with_capacity(leaves.len());
        for (i, leaf) in leaves.iter().enumerate() {
            by_logical.insert(leaf.logical.path().to_vec(), i);
            by_physical.insert(leaf.physical.path().to_vec(), i);
        }
        Self {
            leaves,
            by_logical,
            by_physical,
        }
    }

    /// All leaf pairings in logical output order. Both side channels iterate this so they agree on
    /// nesting.
    pub(crate) fn leaves(&self) -> &[LeafMapping] {
        &self.leaves
    }

    /// Map a logical leaf column to its physical name. Returns `None` if `logical` is not a known
    /// leaf (e.g. a non-leaf struct path, or a name not in the table).
    ///
    /// Used by the filter-pushdown side channel to rewrite predicates written against logical names
    /// into physical names.
    pub(crate) fn logical_to_physical(&self, logical: &ColumnName) -> Option<ColumnName> {
        self.by_logical
            .get(logical.path())
            .map(|&i| self.leaves[i].physical.clone())
    }

    /// Map a physical leaf column back to its logical name. Returns `None` if `physical` is not a
    /// known leaf.
    ///
    /// Used by the statistics side channel to place a per-file stat (carried under a physical leaf
    /// name) at its logical output position.
    pub(crate) fn physical_to_logical(&self, physical: &ColumnName) -> Option<ColumnName> {
        self.by_physical
            .get(physical.path())
            .map(|&i| self.leaves[i].logical.clone())
    }

    /// Resolve the physical stats path for a logical leaf under a given stats kind, i.e.
    /// `<kind>.<physical leaf path…>` (e.g. `minValues.outer.col-1a2b`). Prefix with `add.stats_parsed`
    /// via [`ColumnName::join`] at the call site to get the full checkpoint path
    /// (`add.stats_parsed.minValues.<physical>`). Returns `None` if `logical` is not a known leaf.
    fn physical_stats_path_for(&self, logical: &ColumnName, kind: &str) -> Option<ColumnName> {
        let physical = self.logical_to_physical(logical)?;
        let mut path = Vec::with_capacity(physical.path().len() + 1);
        path.push(kind.to_string());
        path.extend(physical.path().iter().cloned());
        Some(ColumnName::new(path))
    }

    /// `minValues.<physical leaf path>` for a logical leaf (see [`physical_stats_path_for`](Self::physical_stats_path_for)).
    pub(crate) fn physical_min_path(&self, logical: &ColumnName) -> Option<ColumnName> {
        self.physical_stats_path_for(logical, STATS_MIN)
    }

    /// `maxValues.<physical leaf path>` for a logical leaf.
    pub(crate) fn physical_max_path(&self, logical: &ColumnName) -> Option<ColumnName> {
        self.physical_stats_path_for(logical, STATS_MAX)
    }

    /// `nullCount.<physical leaf path>` for a logical leaf.
    pub(crate) fn physical_null_count_path(&self, logical: &ColumnName) -> Option<ColumnName> {
        self.physical_stats_path_for(logical, STATS_NULL_COUNT)
    }
}

/// Walk `logical` and `physical` structs in lockstep by position, appending one [`LeafMapping`] per
/// primitive leaf. `logical_path` / `physical_path` are the accumulated name stacks (mutated in
/// place, restored on return). Arrays, maps, and variants are treated as opaque leaves — the
/// column-mapping name relation the side channels use does not descend into them (matching the
/// kernel's data-skipping `GetReferencedFields`, which filters out array/map fields).
fn walk_lockstep(
    logical: &StructType,
    physical: &StructType,
    logical_path: &mut Vec<String>,
    physical_path: &mut Vec<String>,
    out: &mut Vec<LeafMapping>,
) {
    // `make_physical` preserves field order + structure, so pairing by position (zip) is exact.
    for (lf, pf) in logical.fields().zip(physical.fields()) {
        logical_path.push(lf.name.clone());
        physical_path.push(pf.name.clone());
        match (&lf.data_type, &pf.data_type) {
            (DataType::Struct(l_inner), DataType::Struct(p_inner)) => {
                walk_lockstep(l_inner, p_inner, logical_path, physical_path, out);
            }
            // Primitive / array / map / variant: a leaf in the name relation.
            _ => out.push(LeafMapping {
                logical: ColumnName::new(logical_path.iter().cloned()),
                physical: ColumnName::new(physical_path.iter().cloned()),
            }),
        }
        logical_path.pop();
        physical_path.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use delta_kernel::schema::{DataType, StructField, StructType};

    /// A logical struct field. The `id` / `physical` args document the intended column-mapping
    /// annotation, but the resolver reads the *resolved physical names* off the physical schema (the
    /// kernel has already applied `make_physical`), never the raw `delta.columnMapping.*` metadata —
    /// so these tests carry no metadata and pass the physical name explicitly on the physical side.
    fn logical_field(name: &str, dt: DataType, _id: i64, _physical: &str) -> StructField {
        StructField::nullable(name, dt)
    }

    /// A field named by its physical (on-disk) name — what a resolved physical schema field looks
    /// like after column mapping.
    fn plain_field(name: &str, dt: DataType) -> StructField {
        StructField::nullable(name, dt)
    }

    fn col(path: &[&str]) -> ColumnName {
        ColumnName::new(path.iter().copied())
    }

    // Flat rename: two top-level leaves, physical names differ from logical.
    #[test]
    fn flat_rename() {
        let logical = StructType::try_new([
            logical_field("id", DataType::LONG, 1, "col-a"),
            logical_field("name", DataType::STRING, 2, "col-b"),
        ])
        .unwrap();
        // Physical schema mirrors structure with physical names.
        let physical = StructType::try_new([
            plain_field("col-a", DataType::LONG),
            plain_field("col-b", DataType::STRING),
        ])
        .unwrap();

        let r = ColumnMappingResolver::from_schemas(&logical, &physical);
        assert_eq!(r.leaves().len(), 2);
        assert_eq!(r.logical_to_physical(&col(&["id"])), Some(col(&["col-a"])));
        assert_eq!(
            r.logical_to_physical(&col(&["name"])),
            Some(col(&["col-b"]))
        );
        assert_eq!(r.physical_to_logical(&col(&["col-a"])), Some(col(&["id"])));
        assert_eq!(
            r.physical_to_logical(&col(&["col-b"])),
            Some(col(&["name"]))
        );
    }

    // Nested struct: leaf paths are dotted; the outer struct itself is not a leaf.
    #[test]
    fn nested_struct_leaf_paths() {
        let logical_inner = StructType::try_new([
            logical_field("a", DataType::LONG, 10, "col-a"),
            logical_field("b", DataType::STRING, 11, "col-b"),
        ])
        .unwrap();
        let logical = StructType::try_new([logical_field(
            "outer",
            DataType::Struct(Box::new(logical_inner)),
            1,
            "col-outer",
        )])
        .unwrap();

        let physical_inner = StructType::try_new([
            plain_field("col-a", DataType::LONG),
            plain_field("col-b", DataType::STRING),
        ])
        .unwrap();
        let physical = StructType::try_new([plain_field(
            "col-outer",
            DataType::Struct(Box::new(physical_inner)),
        )])
        .unwrap();

        let r = ColumnMappingResolver::from_schemas(&logical, &physical);
        // Only the two primitive leaves — the `outer` struct is not itself a leaf.
        assert_eq!(r.leaves().len(), 2);
        assert_eq!(
            r.logical_to_physical(&col(&["outer", "a"])),
            Some(col(&["col-outer", "col-a"]))
        );
        assert_eq!(
            r.physical_to_logical(&col(&["col-outer", "col-b"])),
            Some(col(&["outer", "b"]))
        );
        // The struct path itself is not a leaf → no mapping.
        assert_eq!(r.logical_to_physical(&col(&["outer"])), None);
    }

    // "id" vs "name" mode are indistinguishable to the resolver: both produce a physical schema with
    // renamed fields, and the resolver only reads the resolved physical names. Same-shape rename ⇒
    // same mapping regardless of how the physical name was chosen.
    #[test]
    fn id_and_name_mode_both_map_by_resolved_physical_name() {
        // In practice `id` mode may keep numeric-ish physical names and `name` mode uses the
        // annotated physicalName; either way the physical schema carries the final name.
        let logical =
            StructType::try_new([logical_field("id", DataType::LONG, 7, "col-7")]).unwrap();
        let physical = StructType::try_new([plain_field("col-7", DataType::LONG)]).unwrap();
        let r = ColumnMappingResolver::from_schemas(&logical, &physical);
        assert_eq!(r.logical_to_physical(&col(&["id"])), Some(col(&["col-7"])));
    }

    // No column mapping (`none` mode): physical schema == logical schema, every leaf maps to itself.
    #[test]
    fn none_mode_identity() {
        let logical = StructType::try_new([
            plain_field("id", DataType::LONG),
            plain_field("name", DataType::STRING),
        ])
        .unwrap();
        let physical = logical.clone();
        let r = ColumnMappingResolver::from_schemas(&logical, &physical);
        assert_eq!(r.logical_to_physical(&col(&["id"])), Some(col(&["id"])));
        assert_eq!(r.physical_to_logical(&col(&["name"])), Some(col(&["name"])));
    }

    // Unknown / missing columns return None rather than panicking.
    #[test]
    fn missing_columns_return_none() {
        let logical =
            StructType::try_new([logical_field("id", DataType::LONG, 1, "col-a")]).unwrap();
        let physical = StructType::try_new([plain_field("col-a", DataType::LONG)]).unwrap();
        let r = ColumnMappingResolver::from_schemas(&logical, &physical);
        assert_eq!(r.logical_to_physical(&col(&["nonexistent"])), None);
        assert_eq!(r.physical_to_logical(&col(&["col-nope"])), None);
    }

    // Stats-path resolution: physical leaf path placed under the stats-kind subpath.
    #[test]
    fn stats_path_resolution() {
        let logical = StructType::try_new([
            logical_field("id", DataType::LONG, 1, "col-a"),
            logical_field("name", DataType::STRING, 2, "col-b"),
        ])
        .unwrap();
        let physical = StructType::try_new([
            plain_field("col-a", DataType::LONG),
            plain_field("col-b", DataType::STRING),
        ])
        .unwrap();
        let r = ColumnMappingResolver::from_schemas(&logical, &physical);

        assert_eq!(
            r.physical_min_path(&col(&["id"])),
            Some(col(&["minValues", "col-a"]))
        );
        assert_eq!(
            r.physical_max_path(&col(&["name"])),
            Some(col(&["maxValues", "col-b"]))
        );
        assert_eq!(
            r.physical_null_count_path(&col(&["id"])),
            Some(col(&["nullCount", "col-a"]))
        );
        assert_eq!(r.physical_min_path(&col(&["missing"])), None);
    }

    // Nested stats path: physical leaf path retains nesting under the stats kind.
    #[test]
    fn nested_stats_path() {
        let logical_inner =
            StructType::try_new([logical_field("a", DataType::LONG, 10, "col-a")]).unwrap();
        let logical = StructType::try_new([logical_field(
            "outer",
            DataType::Struct(Box::new(logical_inner)),
            1,
            "col-outer",
        )])
        .unwrap();
        let physical_inner = StructType::try_new([plain_field("col-a", DataType::LONG)]).unwrap();
        let physical = StructType::try_new([plain_field(
            "col-outer",
            DataType::Struct(Box::new(physical_inner)),
        )])
        .unwrap();
        let r = ColumnMappingResolver::from_schemas(&logical, &physical);
        assert_eq!(
            r.physical_min_path(&col(&["outer", "a"])),
            Some(col(&["minValues", "col-outer", "col-a"]))
        );
    }
}
