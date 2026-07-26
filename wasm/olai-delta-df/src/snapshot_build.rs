//! Async-native, engine-free Delta snapshot construction from a discovered `_delta_log` manifest.
//!
//! [`build_snapshot_from_manifest`] takes a caller-discovered `_delta_log` file set — the caller
//! lists it out-of-band (over plain HTTP, which has no listing, this is a HEAD probe) and hands it
//! here as kernel [`FileMeta`]s; this
//! builds the kernel [`LogSegment`] with **no directory listing** ([`LogSegment::from_listed_files`])
//! and resolves Protocol & Metadata by driving the [`SnapshotPm`] state machine through the
//! engine-free [`DataFusionExecutor`] over the caller's own async object store — the same store the
//! scan path reads through.
//!
//! # Why `async` (not `block_on`)
//!
//! Driving [`SnapshotPm`] reads real files over the object store — commit `.json` (a `Consume`
//! drain) and, for checkpointed tables, the checkpoint parquet footer (a `SchemaQuery`). Those
//! reads `.await` the store. On a browser target the store is `fetch`-backed, and a `fetch`
//! resolves *only* when control returns to the JS event loop. Wrapping the drive in
//! `futures::executor::block_on` parks the (single) worker thread, so the event loop never runs,
//! the fetch never settles, and construction hangs forever. So this builder is genuinely `async`
//! and `.await`s the drive; nothing here blocks. The future is `!Send` (the kernel SM is `!Send`),
//! which is fine on every driver we use — `wasm-bindgen-futures` in the browser, a current-thread
//! runtime in the native tests. Contrast the *scan* path ([`crate::DeltaSsaTableProvider`]), whose
//! SM drive performs no store IO for the tables that reach it and so is still driven with `block_on`
//! (see that type's `scan` for the exact caveat).
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
/// `session` must have the object store registered for `table_url`'s authority and carry the Delta
/// engine config (build it via [`crate::delta_engine_session`] / [`crate::DeltaEngineSessionExt`]).
/// The [`SnapshotPm`] SM is driven through a [`DataFusionExecutor`] over that same session, so its
/// log-replay reads go through the caller's store and its reconciliation `Consume` plans use the
/// caller's config.
///
/// Genuinely `async`: must NOT be `block_on`-ed — the drive awaits real object-store reads and a
/// blocked worker thread starves the `fetch` event loop and hangs (see the module docs). The
/// future is `!Send` (the kernel SM is `!Send`), which every driver we target tolerates.
///
/// [`SnapshotPm`]: delta_kernel::sm_plans::state_machines::snapshot::SnapshotPm
pub async fn build_snapshot_from_manifest(
    session: &SessionContext,
    table_url: &Url,
    manifest: Vec<FileMeta>,
    version: Version,
) -> DfResult<SnapshotRef> {
    // Normalize the table root to a directory URL (trailing `/`) BEFORE any `join`. `Url::join`
    // treats the last path segment as a file and REPLACES it unless the base ends in `/`, so a
    // caller-supplied root like `…/tables/<uuid>` (no trailing slash) would otherwise drop the
    // `<uuid>` segment: `join("_delta_log/")`
    // yields `…/tables/_delta_log/`, and the kernel then resolves every commit/data file against that
    // truncated root, producing a doubled `…/tables/_delta_log/…/tables/<uuid>/_delta_log/…json`
    // 404. Anchoring the trailing slash here fixes both the log root and the `Snapshot::from_parts`
    // table root below.
    let table_root = ensure_trailing_slash(table_url);
    let log_root = table_root
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

    // Drive the P&M SM over the caller's own session — it already has the table's object store
    // registered (that is how the caller reads the log) and, when built via `delta_engine_session`
    // / `with_delta_engine`, carries the reconciliation config the P&M `Consume` drain needs
    // (leaf-pushdown off / single partition / no stats). Assert that config here so a
    // misconfigured caller fails loudly rather than misplanning the reconciliation. We validate
    // against the session's own `schema_force_view_types` (construction is view-type-agnostic; the
    // scan provider enforces its own view-type contract).
    let state = session.state();
    let force_view_types = state
        .config_options()
        .execution
        .parquet
        .schema_force_view_types;
    crate::validate_delta_engine_session(&state, force_view_types)?;

    // Await the drive (never `block_on` — see module docs). The `!Send` SM future is awaited
    // directly here, making the whole `build_snapshot_from_manifest` future `!Send`.
    let snapshot = DataFusionExecutor::new()
        .build_snapshot_pm(&state, Arc::new(log_segment), table_root)
        .await
        .map_err(wrap_delta_err)?;
    Ok(snapshot)
}

/// Return `url` with a trailing `/` on its path so `Url::join` appends a child segment rather than
/// replacing the last one. Idempotent: a URL that already ends in `/` is returned unchanged.
fn ensure_trailing_slash(url: &Url) -> Url {
    if url.path().ends_with('/') {
        return url.clone();
    }
    let mut out = url.clone();
    out.set_path(&format!("{}/", url.path()));
    out
}
