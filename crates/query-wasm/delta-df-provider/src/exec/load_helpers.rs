//! Shared helpers for the streaming [`super::LoadExec`].
//!
//! DV-free (v1): the deletion-vector read path (kernel `DeletionVectorDescriptor::read` under
//! `spawn_blocking`, the `not_in_dv` UDF, and the parquet `_row_number` virtual column) is
//! dropped — v1 gates deletion vectors to `Unsupported` upstream. What remains is the plain
//! per-file `DataSourceExec` over DataFusion's async parquet/json source.

use std::sync::Arc;

use chrono::TimeZone;
use datafusion_common::error::DataFusionError;
use datafusion_common::{Result as DfResult, ScalarValue};
use datafusion_datasource::file::FileSource;
use datafusion_datasource::file_groups::FileGroup;
use datafusion_datasource::file_scan_config::FileScanConfigBuilder;
use datafusion_datasource::source::DataSourceExec;
use datafusion_datasource::{ListingTableUrl, PartitionedFile, TableSchema};
use datafusion_datasource_json::source::JsonSource;
use datafusion_datasource_parquet::source::ParquetSource;
use datafusion_execution::TaskContext;
use datafusion_execution::object_store::ObjectStoreUrl;
use datafusion_physical_plan::ExecutionPlan;
use delta_kernel::arrow::array::types::Int64Type;
use delta_kernel::arrow::array::{Array, ArrayRef, AsArray, RecordBatch};
use delta_kernel::arrow::datatypes::{
    FieldRef, Schema as ArrowSchema, SchemaRef as ArrowSchemaRef,
};
use delta_kernel::expressions::ColumnName;
use delta_kernel::sm_plans::ir::nodes::{FileType, LoadNode};
use url::Url;

use crate::exec::field_id_adapter::FieldIdPhysicalExprAdapterFactory;

/// DataFusion's parquet/json openers require a non-`None` batch size before
/// `create_file_opener` is called.
const DEFAULT_OPENER_BATCH_SIZE: usize = 8192;

/// Resolve a `LoadNode`'s `base_url`. Scan-emitted plans always set it; the runtime check
/// only catches IR-level misuse.
pub(crate) fn load_base_url(node: &LoadNode) -> Result<&Url, DataFusionError> {
    node.base_url
        .as_ref()
        .ok_or_else(|| crate::error::plan_compilation("LoadNode.base_url must be set"))
}

/// Join `path_str` onto the load node's `base_url`.
pub(crate) fn resolve_file_location(
    node: &LoadNode,
    path_str: &str,
) -> Result<Url, DataFusionError> {
    let base = load_base_url(node)?;
    base.join(path_str.trim()).map_err(|e| {
        crate::error::plan_compilation(format!(
            "LoadNode could not join base_url `{base}` with path `{path_str}`: {e}"
        ))
    })
}

/// Walk a nested struct path out of `batch`, returning the array at the leaf.
pub(crate) fn extract_column_array(
    batch: &RecordBatch,
    cn: &ColumnName,
) -> Result<ArrayRef, DataFusionError> {
    let mut parts = cn.path().iter();
    let head = parts
        .next()
        .ok_or_else(|| crate::error::plan_compilation(format!("empty column path `{cn}`")))?;
    let mut current = batch.column_by_name(head).cloned().ok_or_else(|| {
        crate::error::plan_compilation(format!(
            "batch schema {:?} missing top-level `{head}` while extracting `{cn}`",
            batch.schema(),
        ))
    })?;
    for seg in parts {
        let sa = current.as_struct_opt().ok_or_else(|| {
            crate::error::plan_compilation(format!(
                "expected struct while extracting `{cn}` segment `{seg}`"
            ))
        })?;
        current = sa.column_by_name(seg).cloned().ok_or_else(|| {
            crate::error::plan_compilation(format!(
                "struct missing `{seg}` while extracting `{cn}`"
            ))
        })?;
    }
    Ok(current)
}

/// Per-row inputs the open future captures: file URL, advisory size, and projected passthrough
/// values (in projected order). No deletion vector in the v1 DV-free path.
#[derive(Debug, Clone)]
pub(crate) struct RowInputs {
    pub url: Url,
    pub size: i64,
    pub partition_values: Vec<ScalarValue>,
}

/// Extract one upstream row into a [`RowInputs`]. `projected_passthrough` is the precomputed
/// set of `node.passthrough_columns` indices to materialize, in projected order.
pub(crate) fn extract_row_inputs(
    batch: &RecordBatch,
    row: usize,
    node: &LoadNode,
    projected_passthrough: &[usize],
) -> Result<RowInputs, DataFusionError> {
    let path_cn = &node.file_meta.path;
    let path_arr = extract_column_array(batch, path_cn)?;
    if path_arr.is_null(row) {
        return Err(crate::error::plan_compilation(format!(
            "LoadNode path column `{path_cn}` was NULL at upstream row {row}"
        )));
    }
    // Path columns are always Utf8 per kernel's scan_live_actions_schema.
    let url = resolve_file_location(node, path_arr.as_string::<i32>().value(row))?;

    // v1 is DV-free. The scan SSA ALWAYS attaches a `dv_ref` (a pointer to the
    // `deletionVector` descriptor column on the upstream), so its mere presence does NOT mean a
    // deletion vector is in use — the descriptor is null per-row for files without one. Reject
    // only if a row actually carries a non-null descriptor (belt-and-suspenders: `resolve.rs`
    // gates `delta.enableDeletionVectors` tables to Unsupported upstream, so this should never
    // fire; if it does, error rather than silently returning deleted rows).
    if let Some(dv_ref) = node.dv_ref.as_ref() {
        let dv_arr = extract_column_array(batch, &dv_ref.column)?;
        if !dv_arr.is_null(row) {
            return Err(crate::error::plan_compilation(format!(
                "row {row} carries a deletion-vector descriptor (column `{}`); deletion vectors \
                 are unsupported in the v1 wasm scan path and must be gated to Unsupported \
                 upstream (delta.enableDeletionVectors)",
                dv_ref.column
            )));
        }
    }

    let size = match node.file_meta.size.as_ref() {
        Some(sz_cn) => {
            let arr = extract_column_array(batch, sz_cn)?;
            if arr.is_null(row) {
                0
            } else {
                arr.as_primitive::<Int64Type>().value(row)
            }
        }
        None => 0,
    };
    let partition_values = projected_passthrough
        .iter()
        .map(|&i| {
            let cn = &node.passthrough_columns[i];
            let arr = extract_column_array(batch, cn)?;
            ScalarValue::try_from_array(arr.as_ref(), row)
        })
        .collect::<DfResult<Vec<_>>>()?;
    Ok(RowInputs {
        url,
        size,
        partition_values,
    })
}

/// Build a [`FileSource`] for the load. Layout is `[file, partition]`: file fields are split off
/// `full_schema`; passthrough fields become DataFusion *partition columns* (the per-file
/// constant-broadcast mechanism via `PartitionedFile.partition_values`). Projection is pushed
/// into the source via `with_projection_indices`.
///
/// Reconciled to DataFusion 54.0.0: the POC used the DF-main `TableSchema::builder(..)` plus a
/// `.with_virtual_columns(..)` call for the parquet `_row_number` virtual column. 54.0.0's
/// `TableSchema` is `from_file_schema(..).with_table_partition_cols(..)` (no `builder`), and it
/// has no `with_virtual_columns` — but the virtual column was deletion-vector-only, so the
/// DV-free path needs neither.
pub(crate) fn build_file_source(
    file_type: FileType,
    full_schema: &ArrowSchemaRef,
    file_field_count: usize,
    projection: Option<&[usize]>,
) -> DfResult<Arc<dyn FileSource>> {
    let (file_fields, passthrough_fields) = full_schema.fields().split_at(file_field_count);
    let file_arrow_schema: ArrowSchemaRef = Arc::new(
        ArrowSchema::new(file_fields.to_vec()).with_metadata(full_schema.metadata().clone()),
    );
    // Partition-col broadcast (`replace_columns_with_literals` / `ProjectionOpener`) synthesizes
    // each output array's `data_type` from the upstream-extracted `ScalarValue`, which never
    // carries kernel's `delta.columnMapping.*` / `PARQUET:field_id` metadata; we strip it here
    // and reapply per-batch in the metadata stamper.
    let stripped_passthrough_fields: Vec<FieldRef> = passthrough_fields
        .iter()
        .map(|f| Arc::new(strip_field_metadata_recursive(f.as_ref())))
        .collect();
    let table_schema = TableSchema::from_file_schema(file_arrow_schema)
        .with_table_partition_cols(stripped_passthrough_fields);
    let source: Arc<dyn FileSource> = match file_type {
        FileType::Parquet => Arc::new(ParquetSource::new(table_schema)),
        FileType::Json => Arc::new(JsonSource::new(table_schema)),
    };
    let source = source.with_batch_size(DEFAULT_OPENER_BATCH_SIZE);
    let Some(proj) = projection else {
        return Ok(source);
    };
    let projected_config = FileScanConfigBuilder::new(
        // Placeholder URL; `with_projection_indices` only mutates the file_source.
        ObjectStoreUrl::local_filesystem(),
        source,
    )
    .with_projection_indices(Some(proj.to_vec()))?
    .build();
    Ok(Arc::clone(projected_config.file_source()))
}

/// Recursively strip per-field metadata so the resulting field's `data_type` matches what
/// DataFusion's partition-col broadcast produces at runtime.
fn strip_field_metadata_recursive(
    field: &delta_kernel::arrow::datatypes::Field,
) -> delta_kernel::arrow::datatypes::Field {
    use delta_kernel::arrow::datatypes::{DataType, Field, Fields};
    let strip_inner = |f: &Arc<Field>| Arc::new(strip_field_metadata_recursive(f.as_ref()));
    let stripped_dt = match field.data_type() {
        DataType::Struct(fs) => DataType::Struct(Fields::from_iter(fs.iter().map(strip_inner))),
        DataType::List(inner) => DataType::List(strip_inner(inner)),
        DataType::LargeList(inner) => DataType::LargeList(strip_inner(inner)),
        DataType::FixedSizeList(inner, n) => DataType::FixedSizeList(strip_inner(inner), *n),
        DataType::Map(entry, sorted) => DataType::Map(strip_inner(entry), *sorted),
        other => other.clone(),
    };
    Field::new(field.name(), stripped_dt, field.is_nullable())
}

/// Parquet decoding uses field-id/column-mapping aware adaptation; JSON keeps DataFusion's
/// default name-based adapter.
pub(crate) fn adapter_factory_for(
    file_type: FileType,
) -> Option<Arc<dyn datafusion_physical_expr_adapter::PhysicalExprAdapterFactory>> {
    match file_type {
        FileType::Parquet => Some(Arc::new(FieldIdPhysicalExprAdapterFactory)),
        FileType::Json => None,
    }
}

/// Translate `url` + `size` into a DataFusion [`ObjectStoreUrl`] + [`PartitionedFile`] pair.
/// `size <= 0` means "unknown" and the caller must resolve it via [`resolve_size_if_unknown`]
/// before opening (parquet's footer reader needs a non-zero size).
pub(crate) fn into_partitioned_file(
    url: &Url,
    size: i64,
) -> Result<(ObjectStoreUrl, PartitionedFile), DataFusionError> {
    let listing = ListingTableUrl::parse(url.as_str())?;
    let object_store_url = listing.object_store();
    let store_path = listing.prefix().clone();
    let object_meta = delta_kernel::object_store::ObjectMeta {
        location: store_path,
        last_modified: chrono::Utc.timestamp_nanos(0),
        size: u64::try_from(size.max(0)).unwrap_or(0),
        e_tag: None,
        version: None,
    };
    Ok((
        object_store_url,
        PartitionedFile::new_from_meta(object_meta),
    ))
}

/// HEAD-resolve `pf.object_meta.size` if it's currently unknown (0). No-op otherwise.
pub(crate) async fn resolve_size_if_unknown(
    pf: &mut PartitionedFile,
    object_store: Arc<dyn delta_kernel::object_store::ObjectStore>,
) -> Result<(), DataFusionError> {
    use delta_kernel::object_store::ObjectStoreExt;
    if pf.object_meta.size > 0 {
        return Ok(());
    }
    let meta = object_store
        .head(&pf.object_meta.location)
        .await
        .map_err(|e| {
            crate::error::internal_error(format!(
                "object-store HEAD failed for `{}`: {e}",
                pf.object_meta.location
            ))
        })?;
    pf.object_meta.size = meta.size;
    Ok(())
}

/// Per-file [`ExecutionPlan`]: a bare `DataSourceExec` over the projection-pushed `file_source`.
/// (The POC also had a DV branch — `DataSourceExec (+_row_number)` → `FilterExec(not_in_dv)` →
/// `ProjectionExec` — which this v1 DV-free port omits.)
pub(crate) async fn build_per_file_plan(
    inputs: RowInputs,
    file_source: Arc<dyn FileSource>,
    file_type: FileType,
    output_schema: &ArrowSchemaRef,
    task_context: &TaskContext,
) -> Result<Arc<dyn ExecutionPlan>, DataFusionError> {
    let RowInputs {
        url,
        size,
        partition_values,
    } = inputs;
    let (object_store_url, mut partitioned_file) = into_partitioned_file(&url, size)?;
    partitioned_file.partition_values = partition_values;
    let object_store = task_context.runtime_env().object_store(&object_store_url)?;
    resolve_size_if_unknown(&mut partitioned_file, object_store).await?;
    let config = FileScanConfigBuilder::new(object_store_url, file_source)
        .with_file_group(FileGroup::from_iter([partitioned_file]))
        .with_expr_adapter(adapter_factory_for(file_type))
        .build();
    let plan: Arc<dyn ExecutionPlan> = Arc::new(DataSourceExec::new(Arc::new(config)));

    debug_assert_eq!(
        plan.schema().fields().len(),
        output_schema.fields().len(),
        "build_per_file_plan: DataSourceExec schema mismatch with output_schema"
    );
    Ok(plan)
}
