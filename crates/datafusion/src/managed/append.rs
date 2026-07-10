//! Append a batch of rows to an existing Unity Catalog catalog-managed Delta table.
//!
//! Drives the kernel write `Transaction` through our [`UnityCatalogCommitter`]:
//!
//! 1. `loadTable` → the catalog's ratified commit tail + latest version.
//! 2. Build a kernel snapshot from that tail (catalog is source of truth), over a
//!    credentialed object store + engine.
//! 3. `snapshot.transaction(committer, engine)` → write parquet via
//!    `DefaultEngine::write_parquet` → `add_files` → `commit`. The committer stages the
//!    commit and ratifies it via UC `updateTable add-commit`.
//!
//! ## Post-commit obligations (ManagedTablesSpec §"Write to the table", lines 75-77)
//!
//! After UC ratifies a commit the client **must** complete the commit lifecycle:
//!
//! - **Publish**: copy the ratified staged commit into `_delta_log/<v>.json` so
//!   non-catalog-aware readers see the new version, and so UC can stop tracking the
//!   unbackfilled tail (an unbounded tail eventually trips UC's `429`
//!   `ResourceExhaustedException` and blocks all further writes).
//! - **Backfill notify**: once published, tell UC via `set-latest-backfilled-version`.
//! - **Metrics**: report commit metrics so UC can schedule table maintenance.
//!
//! All three are **best-effort**: the commit is already ratified, so a publish/notify/
//! metrics failure must never be reported as a failed write — we log and move on, and the
//! next write (or the 429 backfill handler below) catches up.
//!
//! ## Retry (delta.yaml updateTable errors; ManagedTablesSpec commit-errors table)
//!
//! The commit is wrapped in a bounded retry loop:
//!
//! - **409 conflict** (`CommitVersionConflictException` / `AlreadyExistsException`): a
//!   concurrent writer took our version. Reload the table, rebuild the snapshot at the new
//!   latest version, re-stage, retry.
//! - **429 resource-exhausted** (`ResourceExhaustedException` / `TooManyRequestsException`):
//!   the unbackfilled tail is too long. Publish + backfill the pending tail, then retry.
//! - **500 commit-state-unknown** (`CommitStateUnknownException`): the outcome is ambiguous.
//!   Reload the table and look for the exact staged file we proposed — if present the commit
//!   landed (success, never re-commit); if absent, retry.

use std::sync::Arc;
use std::time::Duration;

use datafusion::arrow::array::RecordBatch;
use delta_kernel::engine::arrow_data::ArrowEngineData;
use delta_kernel::snapshot::{Snapshot as KernelSnapshot, SnapshotRef};
use delta_kernel::transaction::CommitResult;
use delta_kernel::{Engine, Version};
use delta_kernel_default_engine::DefaultEngineBuilder;
use tracing::{debug, info, warn};
use unitycatalog_client::DeltaV1Client;
use unitycatalog_common::models::delta::v1::{
    DeltaCommitReport, DeltaLoadTableResponse, DeltaReport, DeltaReportMetricsRequest,
    DeltaTableRequirement, DeltaTableUpdate, DeltaUpdateTableRequest,
};
use unitycatalog_object_store::{TableOperation, UnityObjectStoreFactory};
use url::Url;

use crate::catalog::{
    ManagedReadState, ensure_trailing_slash, resolve_managed_read_state, to_log_tail,
};

use super::committer::UnityCatalogCommitter;
use super::create::CreateManagedTableError;

/// Maximum number of commit attempts before giving up (1 initial try + retries).
const MAX_COMMIT_ATTEMPTS: u32 = 5;
/// Base backoff between retry attempts; jittered and capped per attempt.
const RETRY_BASE_BACKOFF: Duration = Duration::from_millis(50);
const RETRY_MAX_BACKOFF: Duration = Duration::from_secs(2);

/// Append `batch` as a new commit to the existing managed table `catalog.schema.table`.
///
/// Returns the committed table version. `engine_info` is recorded in the commit's `commitInfo`.
/// On success the ratified commit is published, backfilled, and its metrics reported on a
/// best-effort basis (a failure of any of those does not fail the already-ratified write).
pub async fn append_to_managed_table(
    factory: Arc<UnityObjectStoreFactory>,
    catalog: &str,
    schema: &str,
    table: &str,
    batch: RecordBatch,
    engine_info: &str,
) -> Result<Version, CreateManagedTableError> {
    let client = Arc::new(factory.unity_client().delta_v1());
    let num_rows = batch.num_rows() as i64;

    // The credentialed store + engine are stable across retries (the table exists, so we vend
    // by name). Build them once.
    let uc_store = factory
        .for_table(
            format!("{catalog}.{schema}.{table}"),
            TableOperation::ReadWrite,
        )
        .await?;
    let engine = DefaultEngineBuilder::new(uc_store.root()).build();

    let mut attempt: u32 = 0;
    loop {
        attempt += 1;

        // 1. Load the catalog's ratified state and build a snapshot from it.
        let loaded = client.load_table(catalog, schema, table).await?;
        let (table_id, location) = table_id_and_location(catalog, schema, table, &loaded)?;
        let snapshot = build_snapshot(&loaded, &location, &engine)?;

        // 2. Write the batch through a transaction committed by our catalog committer.
        let committer =
            UnityCatalogCommitter::new(client.clone(), catalog, schema, table, table_id.clone());
        let mut txn = snapshot
            .transaction(Box::new(committer.clone()), &engine)
            .map_err(CreateManagedTableError::Kernel)?
            .with_engine_info(engine_info);

        let write_context = txn
            .unpartitioned_write_context()
            .map_err(CreateManagedTableError::Kernel)?;
        let add_metadata = engine
            .write_parquet(&ArrowEngineData::new(batch.clone()), &write_context)
            .await
            .map_err(CreateManagedTableError::Kernel)?;
        txn.add_files(add_metadata);

        let outcome = match txn.commit(&engine) {
            Ok(CommitResult::CommittedTransaction(c)) => {
                let version = c.commit_version();
                info!(version, attempt, "committed managed-table append");
                let post = c.post_commit_snapshot().cloned();
                run_post_commit(
                    &client, catalog, schema, table, &table_id, &engine, post, version, num_rows,
                )
                .await;
                return Ok(version);
            }
            // 409: a concurrent writer ratified our version first. Reload + rebuild + retry.
            Ok(CommitResult::ConflictedTransaction(c)) => {
                warn!(
                    conflict_version = c.conflict_version(),
                    attempt, "commit conflict; reloading and retrying"
                );
                RetryOutcome::Retry
            }
            // Retryable (kernel IO error): recover the typed UC error and dispatch on it.
            Ok(CommitResult::RetryableTransaction(rt)) => {
                handle_typed_retry(
                    &client, catalog, schema, table, &committer, &engine, attempt, rt.error,
                )
                .await?
            }
            // Generic Err: same dispatch (the committer collapses UC errors into a kernel Err).
            Err(kernel_err) => {
                handle_typed_retry(
                    &client, catalog, schema, table, &committer, &engine, attempt, kernel_err,
                )
                .await?
            }
        };

        match outcome {
            RetryOutcome::Succeeded(version) => return Ok(version),
            RetryOutcome::Retry => {
                if attempt >= MAX_COMMIT_ATTEMPTS {
                    return Err(CreateManagedTableError::other(format!(
                        "managed-table commit failed after {attempt} attempts"
                    )));
                }
                backoff(attempt).await;
            }
        }
    }
}

/// The result of inspecting a failed commit attempt.
enum RetryOutcome {
    /// A `CommitStateUnknown` check confirmed the commit actually landed at this version.
    Succeeded(Version),
    /// Retry the commit (after the caller's backoff).
    Retry,
}

/// Inspect the typed UC error the committer recorded for the just-failed attempt and decide
/// whether to retry, fail, or treat the commit as already-landed.
///
/// `kernel_err` is the kernel error to surface when the failure is not a recognised UC error
/// (or attempts are exhausted).
#[allow(clippy::too_many_arguments)]
async fn handle_typed_retry(
    client: &DeltaV1Client,
    catalog: &str,
    schema: &str,
    table: &str,
    committer: &UnityCatalogCommitter,
    engine: &dyn Engine,
    attempt: u32,
    kernel_err: delta_kernel::Error,
) -> Result<RetryOutcome, CreateManagedTableError> {
    let Some(uc_err) = committer.take_last_error() else {
        // No typed UC error → a kernel-side / IO failure. Retry while attempts remain.
        if attempt >= MAX_COMMIT_ATTEMPTS {
            return Err(CreateManagedTableError::Kernel(kernel_err));
        }
        warn!(attempt, error = %kernel_err, "retryable kernel commit error; retrying");
        return Ok(RetryOutcome::Retry);
    };

    // 500 CommitStateUnknown: the commit may or may not have landed. Reload and look for the
    // exact staged file we proposed — never blind-retry (duplicate commit) nor blind-fail
    // (falsely failed write).
    if uc_err.is_commit_state_unknown() {
        return match commit_landed(client, catalog, schema, table, committer).await? {
            Some(version) => {
                info!(
                    version,
                    "commit-state-unknown resolved: proposed commit present; success"
                );
                Ok(RetryOutcome::Succeeded(version))
            }
            None if attempt >= MAX_COMMIT_ATTEMPTS => Err(CreateManagedTableError::Client(uc_err)),
            None => {
                warn!(
                    attempt,
                    "commit-state-unknown: proposed commit absent; retrying"
                );
                Ok(RetryOutcome::Retry)
            }
        };
    }

    // 429 ResourceExhausted: the unbackfilled tail is too long. Publish + backfill it, then retry.
    if uc_err.is_resource_exhausted() {
        if attempt >= MAX_COMMIT_ATTEMPTS {
            return Err(CreateManagedTableError::Client(uc_err));
        }
        warn!(
            attempt,
            "resource-exhausted (429); backfilling pending tail before retry"
        );
        if let Err(e) = backfill_pending_tail(client, catalog, schema, table, engine).await {
            warn!(error = %e, "backfill before retry failed; retrying anyway");
        }
        return Ok(RetryOutcome::Retry);
    }

    // Any other UC error is permanent (400 invalid, 403 denied, 404 missing, …).
    Err(CreateManagedTableError::Client(uc_err))
}

/// If the staged commit the committer last proposed is present in the table's ratified tail (or
/// already published), return the version it landed at. Used to resolve a `CommitStateUnknown`.
async fn commit_landed(
    client: &DeltaV1Client,
    catalog: &str,
    schema: &str,
    table: &str,
    committer: &UnityCatalogCommitter,
) -> Result<Option<Version>, CreateManagedTableError> {
    let Some((version, file_name)) = committer.last_proposed() else {
        return Ok(None);
    };
    let landed_version: Version = version
        .try_into()
        .map_err(|_| CreateManagedTableError::other("negative proposed commit version"))?;
    let loaded = client.load_table(catalog, schema, table).await?;
    let in_tail = loaded
        .commits
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .any(|c| c.version == version && c.file_name == file_name);
    // It may already have been published (and dropped from the tail) if a previous attempt's
    // best-effort publish ran; treat "latest ratified version >= our version" as landed too.
    // Key only on `latest-table-version` (the true ratified-version field) — never
    // `metadata.last-commit-version`, which tracks only metadata-changing commits (A5).
    let published = loaded
        .latest_table_version
        .map(|v| v >= version)
        .unwrap_or(false);
    Ok((in_tail || published).then_some(landed_version))
}

/// Run the best-effort post-commit lifecycle: publish the ratified tail to `_delta_log/`,
/// notify UC via `set-latest-backfilled-version`, and report commit metrics. None of these
/// failing turns the (already ratified) commit into a failed write.
#[allow(clippy::too_many_arguments)]
async fn run_post_commit(
    client: &DeltaV1Client,
    catalog: &str,
    schema: &str,
    table: &str,
    table_id: &str,
    engine: &dyn Engine,
    post_commit_snapshot: Option<SnapshotRef>,
    version: Version,
    num_rows: i64,
) {
    match post_commit_snapshot {
        Some(snapshot) => {
            match publish_and_backfill(client, catalog, schema, table, table_id, engine, &snapshot)
                .await
            {
                Ok(published_to) => {
                    debug!(published_to, "published + backfilled managed-table commits")
                }
                Err(e) => {
                    warn!(error = %e, "best-effort publish/backfill failed; commit remains ratified")
                }
            }
        }
        // No post-commit snapshot (kernel didn't return one): publish is a maintenance step the
        // next write or the 429 handler will catch up; skip rather than rebuild here.
        None => debug!(
            version,
            "no post-commit snapshot; deferring publish to a later write"
        ),
    }

    // Metrics (best-effort).
    let request = DeltaReportMetricsRequest {
        table_id: table_id.to_string(),
        report: Some(DeltaReport {
            commit_report: Some(DeltaCommitReport {
                num_files_added: Some(1),
                num_rows_inserted: Some(num_rows),
                ..Default::default()
            }),
        }),
    };
    if let Err(e) = client
        .report_metrics(catalog, schema, table, &request)
        .await
    {
        warn!(version, error = %e, "best-effort report_metrics failed");
    }
}

/// Publish `snapshot`'s unpublished catalog commits to `_delta_log/<v>.json`, then notify UC
/// with `set-latest-backfilled-version`. Returns the version published up to.
async fn publish_and_backfill(
    client: &DeltaV1Client,
    catalog: &str,
    schema: &str,
    table: &str,
    table_id: &str,
    engine: &dyn Engine,
    snapshot: &SnapshotRef,
) -> Result<i64, CreateManagedTableError> {
    // A `UnityCatalogCommitter` is a catalog committer whose `publish` copies staged commits to
    // their published path (idempotent on already-published files). `publish` is a no-op when
    // nothing is unpublished, returning the same snapshot.
    let committer =
        UnityCatalogCommitter::new(Arc::new(client.clone()), catalog, schema, table, table_id);
    let published = snapshot
        .publish(engine, &committer)
        .map_err(CreateManagedTableError::Kernel)?;
    let published_to: i64 = published
        .version()
        .try_into()
        .map_err(|_| CreateManagedTableError::other("published version does not fit into i64"))?;

    let request = DeltaUpdateTableRequest {
        requirements: vec![DeltaTableRequirement::AssertTableUuid {
            uuid: table_id.to_string(),
        }],
        updates: vec![DeltaTableUpdate::SetLatestBackfilledVersion {
            latest_published_version: published_to,
        }],
    };
    client
        .update_table(catalog, schema, table, &request)
        .await?;
    debug!(published_to, "set-latest-backfilled-version");
    Ok(published_to)
}

/// Reload the table and publish + backfill its current unpublished tail. Used by the 429 handler
/// to drain the tail before retrying a commit.
async fn backfill_pending_tail(
    client: &DeltaV1Client,
    catalog: &str,
    schema: &str,
    table: &str,
    engine: &dyn Engine,
) -> Result<(), CreateManagedTableError> {
    let loaded = client.load_table(catalog, schema, table).await?;
    let (table_id, location) = table_id_and_location(catalog, schema, table, &loaded)?;
    let snapshot = build_snapshot(&loaded, &location, engine)?;
    publish_and_backfill(client, catalog, schema, table, &table_id, engine, &snapshot).await?;
    Ok(())
}

/// Extract the UC table id and trailing-slash-normalised location from a loadTable response.
fn table_id_and_location(
    catalog: &str,
    schema: &str,
    table: &str,
    loaded: &DeltaLoadTableResponse,
) -> Result<(String, Url), CreateManagedTableError> {
    let table_id = loaded
        .metadata
        .properties
        .get("io.unitycatalog.tableId")
        .cloned()
        .ok_or_else(|| {
            CreateManagedTableError::other(format!(
                "table {catalog}.{schema}.{table} is missing the required \
                 io.unitycatalog.tableId property (ManagedTablesSpec §table properties)"
            ))
        })?;
    let location = Url::parse(&ensure_trailing_slash(&loaded.metadata.location))
        .map_err(|e| CreateManagedTableError::other(format!("invalid table location: {e}")))?;
    Ok((table_id, location))
}

/// Build a kernel snapshot from the catalog's ratified state (the catalog is the source of
/// truth). A catalog-managed table always requires `max_catalog_version`.
///
/// Shares [`resolve_managed_read_state`] with the read path, so the latest-version
/// resolution is identical (and never substitutes `metadata.last-commit-version` — review
/// finding A5). The table is known managed here, so a `NotManaged` result is a hard error.
fn build_snapshot(
    loaded: &DeltaLoadTableResponse,
    location: &Url,
    engine: &dyn Engine,
) -> Result<SnapshotRef, CreateManagedTableError> {
    let (commits, latest) = match resolve_managed_read_state(loaded)
        .map_err(|e| CreateManagedTableError::other(e.to_string()))?
    {
        ManagedReadState::Managed { commits, latest } => (commits, latest),
        ManagedReadState::NotManaged => {
            return Err(CreateManagedTableError::other(format!(
                "table at '{location}' is not catalog-managed; cannot build a managed snapshot"
            )));
        }
    };
    KernelSnapshot::builder_for(location.as_str())
        .with_log_tail(
            to_log_tail(location, &commits)
                .map_err(|e| CreateManagedTableError::other(e.to_string()))?,
        )
        .with_max_catalog_version(latest)
        .build(engine)
        .map_err(CreateManagedTableError::Kernel)
}

/// Jittered, capped exponential backoff between commit attempts.
async fn backoff(attempt: u32) {
    let exp = RETRY_BASE_BACKOFF.saturating_mul(1u32 << attempt.min(5));
    let capped = exp.min(RETRY_MAX_BACKOFF);
    // Deterministic jitter from the attempt count avoids pulling in an RNG dependency.
    let jitter = Duration::from_millis((attempt as u64 * 7) % 25);
    tokio::time::sleep(capped + jitter).await;
}
