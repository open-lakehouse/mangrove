//! Test-only buffered collectors for [`DataFusionExecutor`].
//!
//! Thin wrappers around the SSA result-plan APIs that collect a `Vec<RecordBatch>`. Returned
//! batches carry whatever schema the SSA plan terminates at;
//! plans that need column-mapping renames / Delta field metadata bake those into the
//! terminal projection upstream of `Context::into_result_plan`.
//!
//! Named `testing` rather than `test_utils` to avoid colliding with the workspace
//! `test_utils` crate at import sites in tests that consume both. Gated by
//! `#[cfg(any(test, feature = "test-utils"))]`. The feature is activated for this crate's
//! own integration tests via a self dev-dependency in `Cargo.toml`, and for external test
//! crates (e.g. `acceptance`) by enabling `features = ["test-utils"]` on their
//! dev-dependency on this crate.

use datafusion::catalog::Session;
use delta_kernel::arrow::record_batch::RecordBatch;
use delta_kernel::sm_plans::errors::DeltaError;
use delta_kernel::sm_plans::ir::plan::ResultPlan;
use futures::TryStreamExt;

use crate::DataFusionExecutor;
use crate::error::DfResultIntoDelta;

/// Compile a [`ResultPlan`] via `executor`, plan + execute it against `session`, and **eagerly**
/// drain the resulting stream into a `Vec<RecordBatch>`. Suitable for SSA plans constructed
/// directly in tests (no coroutine required).
///
/// The library layer returns lazy `LogicalPlan`s /
/// [`SendableRecordBatchStream`](datafusion_physical_plan::SendableRecordBatchStream)s and never
/// buffers; this test-only helper is where eager materialization belongs. `session` is passed
/// explicitly (every call site has one) rather than reaching into the executor — the executor
/// tracks its session only for the internal SM drive.
pub async fn collect_ssa_result(
    session: &dyn Session,
    executor: &DataFusionExecutor<'_>,
    rp: ResultPlan,
) -> Result<Vec<RecordBatch>, DeltaError> {
    let logical = executor.compile_result_plan(&rp)?;
    let physical = session.create_physical_plan(&logical).await.into_delta()?;
    datafusion_physical_plan::execute_stream(physical, session.task_ctx())
        .into_delta()?
        .try_collect()
        .await
        .into_delta()
}
