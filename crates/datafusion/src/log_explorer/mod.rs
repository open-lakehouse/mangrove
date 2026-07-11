//! Expose a Delta table's transaction log as data.
//!
//! Two DataFusion [`TableProvider`](datafusion::catalog::TableProvider)s, both
//! driven **through delta-kernel** (never by parsing `_delta_log` files
//! directly — that would bypass checkpoint schema, sidecars, log compaction, and
//! replay semantics):
//!
//! - [`RawLogProvider`] — every action across commits and checkpoints, *as
//!   written* (add / remove / metaData / protocol / txn / commitInfo / cdc /
//!   sidecar / checkpointMetadata / domainMetadata), tagged with whether the
//!   batch came from a commit or a checkpoint. The log *as history*. Backed by
//!   `Snapshot::log_segment().read_actions(..)`.
//! - [`ReconciledLogProvider`] — the result of kernel **log replay**: the
//!   surviving add-files at the snapshot's version (add/remove tombstoning,
//!   protocol+metadata resolution applied). The *effective* state. Backed by
//!   `Snapshot::scan_builder().scan_metadata(..)`, per the
//!   [`scan_row_schema`](delta_kernel::scan::scan_row_schema).
//!
//! Both providers carry the delta-kernel [`Engine`](delta_kernel::Engine) used to
//! read the log, so they are self-contained and depend on no session extension.
//! [`build_default_engine`] constructs a
//! [`DefaultEngine`](delta_kernel_default_engine::DefaultEngine) over a resolved
//! object store — the same construction the server uses for its
//! `DeltaLogReplayProvider`.
//!
//! This is the native seam behind the planned Delta Log Explorer UI; the wasm
//! engine (`crates/query-wasm`) can reuse the same provider types with a
//! fetch-backed store and the facade's DataFusion engine (see
//! `WASM_QUERY_PREVIEW.md`).

use std::sync::Arc;

use delta_kernel::Engine;
use delta_kernel_default_engine::DefaultEngine;
use object_store::DynObjectStore;

mod raw;
mod reconciled;

pub use raw::{COMMIT_MARKER_COLUMN, RawLogProvider};
pub use reconciled::ReconciledLogProvider;

/// Build a delta-kernel [`DefaultEngine`] over `store`.
///
/// The engine manages its own background executor. Mirrors the server's
/// `build_engine`; callers that already hold a credentialed object store (the
/// native server via its object-store factory, the wasm engine via its fetch
/// store) pass it straight in.
pub fn build_default_engine(store: Arc<DynObjectStore>) -> Arc<dyn Engine> {
    Arc::new(DefaultEngine::builder(store).build())
}
