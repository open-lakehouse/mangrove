//! Async-native, engine-free Delta snapshot construction from a discovered `_delta_log` manifest.
//!
//! [`build_snapshot_from_manifest`] is the list-free / prime-free replacement for the eager
//! `deltalake-wasm` facade path (`PrimedStore::prime` + a synchronous `DataFusionEngine` driven by
//! `InlineExecutor`). The caller discovers the log file set out-of-band (mangrove's `resolve.rs`
//! HEAD-probes it, since plain HTTP has no listing) and hands it here as kernel [`FileMeta`]s; this
//! builds the kernel [`LogSegment`] with **no directory listing** ([`LogSegment::from_listed_files`])
//! and resolves Protocol & Metadata by driving the [`SnapshotPm`] state machine through the
//! engine-free [`DataFusionExecutor`] over the caller's own async object store — the same store the
//! scan path reads through.
//!
//! [`SnapshotPm`]: delta_kernel::sm_plans::state_machines::snapshot::SnapshotPm

use std::sync::Arc;

use datafusion::execution::context::SessionContext;
use datafusion_common::Result as DfResult;
use delta_kernel::log_segment::LogSegment;
use delta_kernel::path::ParsedLogPath;
use delta_kernel::snapshot::SnapshotRef;
use delta_kernel::{FileMeta, Version};
use url::Url;

use crate::DataFusionExecutor;
use crate::error::{plan_compilation, wrap_delta_err};

/// Build a kernel [`SnapshotRef`] for the table at `table_url`, pinned to `version`, from a
/// pre-discovered `_delta_log` manifest — **async-native, list-free, and engine-free**.
///
/// `manifest` is the set of `_delta_log` files (commit `.json` + any checkpoint `.parquet` parts)
/// the caller discovered, as kernel [`FileMeta`]s (absolute URLs + sizes). Entries whose filename
/// does not parse as a Delta log path are dropped, mirroring what a storage listing would ignore.
///
/// `session` must have the object store registered for `table_url`'s authority (the same session
/// the scan runs against). The [`SnapshotPm`] SM is driven through a
/// [`DataFusionExecutor::from_session`] so its log-replay reads go through that store.
///
/// The SM is `!Send`; the drive is confined to a `block_on` here so nothing `!Send` crosses the
/// `async` boundary and the returned `SnapshotRef` is `Send`.
///
/// [`SnapshotPm`]: delta_kernel::sm_plans::state_machines::snapshot::SnapshotPm
pub fn build_snapshot_from_manifest(
    session: &SessionContext,
    table_url: &Url,
    manifest: Vec<FileMeta>,
    version: Version,
) -> DfResult<SnapshotRef> {
    // The kernel joins the log root onto the table root; the trailing slash makes it append rather
    // than replace the last path segment.
    let log_root = table_url
        .join("_delta_log/")
        .map_err(|e| plan_compilation(format!("build_snapshot_from_manifest: log root: {e}")))?;

    // Classify each manifest entry into a ParsedLogPath; drop names that don't parse as a Delta
    // log path (`try_from` -> None). The kernel's `from_listed_files` accumulator then applies the
    // same file-type filtering the storage-listing path uses (`should_process_log_file`), so
    // unrecognized-but-parseable entries are handled identically to a real listing.
    let mut paths: Vec<ParsedLogPath> = Vec::with_capacity(manifest.len());
    for meta in manifest {
        if let Some(parsed) = ParsedLogPath::try_from(meta).map_err(wrap_delta_err)? {
            paths.push(parsed);
        }
    }

    let log_segment =
        LogSegment::from_listed_files(log_root, paths, Some(version)).map_err(wrap_delta_err)?;

    // Resolve the caller-registered object store for the table's authority, then drive the P&M SM
    // over a reconciliation-safe executor session that has that store registered. `new_with_store`
    // (not `from_session`) is required: the P&M `Consume` drain both runs the reconciliation plan
    // — which needs the leaf-pushdown / single-partition config — and reads the log/checkpoint
    // files over the store during the drive.
    let object_store_url =
        datafusion_datasource::ListingTableUrl::parse(table_url.as_str())?.object_store();
    let store = session.runtime_env().object_store(&object_store_url)?;

    // The SM is `!Send`/IO-free-CPU (it yields Consume/SchemaQuery ops the executor services
    // async), so `block_on` completes without cooperating with the outer runtime and confines the
    // `!Send` region — the returned `SnapshotRef` is `Send`. Mirrors the scan path's confinement.
    let executor = DataFusionExecutor::new_with_store(table_url, store);
    let snapshot = futures::executor::block_on(
        executor.build_snapshot_pm(Arc::new(log_segment), table_url.clone()),
    )
    .map_err(wrap_delta_err)?;
    Ok(snapshot)
}
