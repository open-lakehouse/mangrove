//! Kernel SSA plan -> DataFusion [`datafusion_expr::LogicalPlan`] compilation.

use std::sync::Arc;

use datafusion_common::error::DataFusionError;
use datafusion_physical_expr_common::physical_expr::PhysicalExpr;
use uuid::Uuid;

// `stats` is the resolver's only live consumer. The `logical_to_physical` / predicate-rewrite
// helpers are unused on the DataFusion pushdown path (it prunes in *logical* space — the parquet
// `FilePruner` + `FieldIdPhysicalExprAdapterFactory` own the logical↔physical rename), but are kept
// and tested for a future kernel-predicate consumer; `allow(dead_code)` covers the unused halves.
#[allow(dead_code)]
pub mod column_mapping;
pub mod expr_translator;
mod json_parse;
pub mod logical;
pub mod stats;

pub use logical::compile_ssa;

/// Context shared by the compiler for leaf nodes that need runtime side state.
///
/// Carries only static / shared bits — no per-phase mutable accumulator and no kernel `Engine`
/// (nothing on this path resolves deletion vectors). The `Load` provider builds its file sources
/// from the DataFusion `Session`'s object store at scan time. Drained
/// consumer state for `Consume` steps flows directly out of
/// [`DataFusionExecutor::run_phase`](crate::executor::DataFusionExecutor) as a
/// [`EngineResponse::Consumer`](delta_kernel::sm_plans::state_machines::framework::step_payload::EngineResponse::Consumer)
/// after the executor finishes the sink locally.
#[derive(Clone)]
pub struct CompileContext {
    /// Owning state machine's identity. Stamped onto any `Consume` handle drained during the
    /// phase. Synthesized to `("standalone", "execute")` with a fresh `sm_id` for tests and
    /// SM-less entry points.
    pub sm_id: Uuid,
    pub sm_kind: &'static str,
    pub step_name: &'static str,
    /// Per-file statistics (keyed by raw `add.path`) to attach to the compiled `Load` leaf's
    /// per-file `PartitionedFile`s. `None` on every path except the provider's stats-enabled scan;
    /// `lower_load` clones it into the `LoadTableProvider`. See [`stats::FileStatsMap`].
    pub file_stats: Option<Arc<stats::FileStatsMap>>,
    /// Scan-global, **logical-named** filter-pushdown predicate to hand the per-file parquet
    /// `ParquetSource` (row-group / page pruning against the attached `file_stats`). `None` unless
    /// the provider lowered query filters to a `PhysicalExpr`; `lower_load` clones it into the
    /// `LoadTableProvider`, which applies it once onto the shared parquet source. Unlike
    /// [`Self::file_stats`] this is one expr shared by all files, not a per-file map.
    pub predicate: Option<Arc<dyn PhysicalExpr>>,
}

/// Optional per-scan side channels the provider threads into the compiled `Load` leaf.
///
/// Both default to `None`; a `Default`/empty value reproduces the plain compile exactly. Grouped
/// into one struct (rather than a growing positional argument list) so future channels are additive.
#[derive(Clone, Default)]
pub struct SideChannels {
    /// Per-file statistics keyed by raw `add.path`. See [`CompileContext::file_stats`].
    pub file_stats: Option<Arc<stats::FileStatsMap>>,
    /// Scan-global logical-named parquet pruning predicate. See [`CompileContext::predicate`].
    pub predicate: Option<Arc<dyn PhysicalExpr>>,
}

impl CompileContext {
    /// Build a context for SM-less inspection / standalone driving (benchmark plan printers,
    /// integration tests that lower a `ResultPlan` directly).
    pub fn new() -> Self {
        Self {
            sm_id: crate::next_sm_id(),
            sm_kind: "standalone",
            step_name: "execute",
            file_stats: None,
            predicate: None,
        }
    }
}

impl Default for CompileContext {
    fn default() -> Self {
        Self::new()
    }
}

pub(super) fn expand_projection_columns(
    columns: &[Arc<delta_kernel::expressions::Expression>],
    expected_output_fields: usize,
) -> Result<Vec<Arc<delta_kernel::expressions::Expression>>, DataFusionError> {
    let mut expanded = Vec::new();
    for (idx, expr) in columns.iter().enumerate() {
        let remaining_output = expected_output_fields
            .checked_sub(expanded.len())
            .ok_or_else(|| crate::error::plan_compilation("Projection expansion overflow"))?;
        let remaining_expr = columns.len() - idx;
        let extra_needed = remaining_output
            .checked_sub(remaining_expr)
            .ok_or_else(|| {
                crate::error::plan_compilation(format!(
                    "Projection has too many expressions: expected {expected_output_fields} output fields, got at least {}",
                    expanded.len() + remaining_expr
                ))
            })?;

        match expr.as_ref() {
            delta_kernel::expressions::Expression::Struct(children, _) => {
                let spread_extra = children.len().saturating_sub(1);
                if spread_extra > 0 && spread_extra <= extra_needed {
                    expanded.extend(children.iter().cloned());
                } else {
                    expanded.push(Arc::clone(expr));
                }
            }
            _ => expanded.push(Arc::clone(expr)),
        }
    }

    if expanded.len() != expected_output_fields {
        return Err(crate::error::plan_compilation(format!(
            "Projection output schema has {} fields but expanded to {} expressions",
            expected_output_fields,
            expanded.len()
        )));
    }
    Ok(expanded)
}
