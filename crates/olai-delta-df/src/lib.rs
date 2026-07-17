//! Async-native (sm_plans-driven) Delta DataFusion `TableProvider` scaffold for kernel SSA
//! plans. Ported from the kernel POC `delta-kernel-datafusion-engine`, reconciled to
//! mangrove's wasm pins (DataFusion 54, arrow-58, no tokio) and stripped of the deletion-vector
//! path so the scan needs no kernel `Engine` — see the crate manifest and
//! `handover-wasm-async-native-table-provider.md`.
//!
//! v1 gates the hard cases (deletion vectors, v2/UUID checkpoints, footer-probing checkpoints)
//! to `Unsupported` upstream in `query-wasm`'s `resolve.rs`, so the scan path here is the
//! DV-free lazy replay only: no `Arc<dyn Engine>` on the provider, exec, or plan-node structs,
//! and no `spawn_blocking`.

pub mod compile;
pub mod error;
pub mod exec;
pub mod executor;
pub mod provider;
pub mod session;
pub mod snapshot_build;
#[cfg(any(test, feature = "test-utils"))]
pub mod testing;

pub use executor::DataFusionExecutor;
pub use provider::{DeltaSsaScanConfig, DeltaSsaTableProvider};
pub use session::{
    DeltaEngineSessionExt, DeltaEngineSessionOptions, configure_delta_engine_config,
    delta_engine_session, delta_engine_session_config, install_delta_engine,
    validate_delta_engine_session,
};
pub use snapshot_build::build_snapshot_from_manifest;

// Re-export the kernel types on `build_snapshot_from_manifest`'s public signature so consumers
// (e.g. `query-wasm`) need not take a direct `delta_kernel` dependency and pin its features
// themselves — the kernel version/features are this crate's concern.
pub use delta_kernel::snapshot::SnapshotRef;
pub use delta_kernel::{FileMeta, Version};

/// A process-monotonic state-machine identity, replacing the POC's `uuid::Uuid::new_v4()` at
/// this crate's *SM-less* compile entry points (`compile_result_plan`, `execute_step`).
///
/// The `sm_id` is an opaque `(sm_id)` label the kernel stamps onto `Consume` handles for
/// tracing; its only requirement is uniqueness within a run. `Uuid::new_v4()` pulls
/// `getrandom`, which needs an explicit backend on `wasm32-unknown-unknown`; we mint the id from
/// an atomic counter via [`uuid::Uuid::from_u64_pair`] instead — deterministic and entropy-free.
///
/// Note: driving a real kernel `CoroutineSM` (e.g. `scan_state_machine()`) still mints its own
/// `sm_id` via `Uuid::new_v4()` *inside the kernel*, so the wasm cdylib must still enable
/// `getrandom`'s `wasm_js` backend + uuid's `js` feature (as `deltalake-wasm` already does).
/// This helper only keeps *our* SM-less paths entropy-free.
pub(crate) fn next_sm_id() -> uuid::Uuid {
    use core::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    uuid::Uuid::from_u64_pair(0, n)
}
