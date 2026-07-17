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

use delta_kernel::arrow::record_batch::RecordBatch;
use delta_kernel::sm_plans::errors::DeltaError;
use delta_kernel::sm_plans::ir::plan::ResultPlan;

use crate::DataFusionExecutor;

/// Compile a [`ResultPlan`] and collect it into a `Vec<RecordBatch>` via
/// [`DataFusionExecutor::collect_result_plan`]. Suitable for SSA plans constructed directly in
/// tests (no coroutine required).
pub async fn collect_ssa_result(
    executor: &DataFusionExecutor<'_>,
    rp: ResultPlan,
) -> Result<Vec<RecordBatch>, DeltaError> {
    executor.collect_result_plan(&rp).await
}
