//! Delta catalog-managed commit coordinator.
//!
//! This defines the server-side contract for Delta's catalog-managed commits
//! ("commit coordinator") protocol: the catalog — not filesystem PUT-if-absent —
//! is the source of truth for which commit wins each table version. Writers stage
//! a commit file under `_delta_log/_staged_commits/<version>.<uuid>.json` and ask
//! the catalog to *ratify* it; readers ask the catalog for ratified-but-unpublished
//! commits and merge them with the published Delta log.
//!
//! [`CommitCoordinator`] is the backend-agnostic trait; [`InMemoryCommitCoordinator`]
//! is the default in-memory implementation. Persistent implementations (e.g. a
//! Postgres-backed one in `unitycatalog-postgres`) live in each backend crate.
//! Both faithfully port the arbitration and backfill
//! logic of the Unity Catalog OSS reference implementation
//! (`DeltaCommitRepository.java`'s `postCommitCore` → `handleOnboardingCommit` /
//! `handleNormalCommit` / `handleBackfillOnlyCommit`). The notable invariants:
//!
//! 1. The highest commit row is never deleted on backfill — when fully backfilled
//!    it is retained as a marker, so an onboarded table always keeps at least one
//!    row to report `latest_table_version`.
//! 2. [`get_commits`](CommitCoordinator::get_commits) excludes that marker row from
//!    the returned `commits`, but still reports its version as `latest_table_version`.
//! 3. First *posted* commit version is `>= 1`; version `0` is established
//!    out-of-band by create-table. `CommitInfo` fields must be positive / non-empty.
//! 4. No in-commit-timestamp monotonicity is enforced (client's responsibility).
//! 5. There is a cap on unbackfilled commits per table (OSS hardcodes 10); exceeding
//!    it rejects the commit with a resource-exhausted (429) error.
//!
//! Wire-format note: this matches the *shipped* UC OSS server, not the prose
//! `ManagedTablesSpec.md`, which disagree in places (path `/delta/preview/commits`
//! vs `/delta/commit`, field `latest_backfilled_version` vs `latest_published_version`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};

/// A ratified Delta commit, the coordinator's per-version record.
///
/// This is the hand-written replacement for the former proto-generated
/// `CommitInfo`; its fields (names and types) are identical, so backends'
/// SQL ↔ struct mapping is unaffected by the move. All fields must be positive /
/// non-empty — see [`validate_commit_info`].
#[derive(Clone, Debug, PartialEq)]
pub struct CommitInfo {
    /// The commit version.
    pub version: i64,
    /// In-commit timestamp, in epoch milliseconds.
    pub timestamp: i64,
    /// UUID-based commit file name.
    pub file_name: String,
    /// Size of the commit file, in bytes.
    pub file_size: i64,
    /// File modification timestamp, in epoch milliseconds.
    pub file_modification_timestamp: i64,
}

/// Default cap on the number of unbackfilled commits a table may accumulate
/// before further commits are rejected. Matches UC OSS `MAX_NUM_COMMITS_PER_TABLE`.
pub const DEFAULT_MAX_UNBACKFILLED_COMMITS: i64 = 10;

/// Error returned by a [`CommitCoordinator`].
///
/// Variants map onto the HTTP statuses the UC OSS commit API returns; the server
/// crate maps them onto its own error type (and thus the response status).
#[derive(Debug, thiserror::Error)]
pub enum CommitError {
    /// The requested version was already accepted (replay or lost the
    /// first-writer-wins race). Maps to 409.
    #[error("commit version conflict: {0}")]
    VersionConflict(String),

    /// Invalid request: missing fields, non-positive values, a version gap, or a
    /// backfill version beyond the latest commit. Maps to 400.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// The table has too many unbackfilled commits. Maps to 429.
    #[error("resource exhausted: {0}")]
    ResourceExhausted(String),

    /// An unexpected backend error (e.g. database failure). Maps to 500.
    #[error("commit coordinator backend error: {0}")]
    Backend(String),
}

/// Result type for commit-coordinator operations.
pub type CommitResult<T> = Result<T, CommitError>;

/// Backend-agnostic Delta commit coordinator.
///
/// Implementations persist ratified commits per table and arbitrate the
/// first-writer-wins race for each version. See the module docs for the rules,
/// which are a port of UC OSS `postCommitCore`.
#[async_trait::async_trait]
pub trait CommitCoordinator: Send + Sync + 'static {
    /// Ratify a commit and/or record a backfill notification for `table_id`.
    ///
    /// At least one of `commit_info` / `latest_backfilled_version` must be set.
    async fn commit(
        &self,
        table_id: &str,
        commit_info: Option<CommitInfo>,
        latest_backfilled_version: Option<i64>,
    ) -> CommitResult<()>;

    /// Return ratified-but-unpublished commits for `table_id` in
    /// `[start_version, end_version]`, plus `latest_table_version`.
    ///
    /// The returned commits exclude the internal backfilled-latest marker row.
    /// `latest_table_version` is `0` for a managed table with no commits, else the
    /// highest tracked version.
    async fn get_commits(
        &self,
        table_id: &str,
        start_version: i64,
        end_version: Option<i64>,
    ) -> CommitResult<(Vec<CommitInfo>, i64)>;
}

#[async_trait::async_trait]
impl<T: CommitCoordinator> CommitCoordinator for Arc<T> {
    async fn commit(
        &self,
        table_id: &str,
        commit_info: Option<CommitInfo>,
        latest_backfilled_version: Option<i64>,
    ) -> CommitResult<()> {
        self.as_ref()
            .commit(table_id, commit_info, latest_backfilled_version)
            .await
    }

    async fn get_commits(
        &self,
        table_id: &str,
        start_version: i64,
        end_version: Option<i64>,
    ) -> CommitResult<(Vec<CommitInfo>, i64)> {
        self.as_ref()
            .get_commits(table_id, start_version, end_version)
            .await
    }
}

/// Auxiliary trait for handlers that carry a [`CommitCoordinator`].
pub trait ProvidesCommitCoordinator: Send + Sync + 'static {
    fn commit_coordinator(&self) -> &dyn CommitCoordinator;
}

/// Validate that all `CommitInfo` fields are positive / non-empty, matching UC
/// OSS `validateCommitInfo`. Shared by all backends.
pub fn validate_commit_info(info: &CommitInfo) -> CommitResult<()> {
    if info.version <= 0 {
        return Err(CommitError::InvalidArgument(
            "commit_info.version must be positive".to_string(),
        ));
    }
    if info.timestamp <= 0 {
        return Err(CommitError::InvalidArgument(
            "commit_info.timestamp must be positive".to_string(),
        ));
    }
    if info.file_name.is_empty() {
        return Err(CommitError::InvalidArgument(
            "commit_info.file_name must not be empty".to_string(),
        ));
    }
    if info.file_size <= 0 {
        return Err(CommitError::InvalidArgument(
            "commit_info.file_size must be positive".to_string(),
        ));
    }
    if info.file_modification_timestamp <= 0 {
        return Err(CommitError::InvalidArgument(
            "commit_info.file_modification_timestamp must be positive".to_string(),
        ));
    }
    Ok(())
}

/// A ratified commit plus the marker flag used during backfill.
#[derive(Debug, Clone)]
struct StoredCommit {
    info: CommitInfo,
    /// `true` once this is the highest commit *and* it has been backfilled —
    /// it is retained as a version marker but hidden from `get_commits`.
    is_backfilled_latest: bool,
}

/// Per-table ratified-commit state, ordered by version.
#[derive(Debug, Default)]
struct TableCommitState {
    commits: std::collections::BTreeMap<i64, StoredCommit>,
}

impl TableCommitState {
    fn first_version(&self) -> Option<i64> {
        self.commits.keys().next().copied()
    }

    fn last_version(&self) -> Option<i64> {
        self.commits.keys().next_back().copied()
    }
}

/// In-memory [`CommitCoordinator`].
///
/// Storage is a map of per-table state, each entry behind its own `Mutex`. The
/// per-table mutex is held across the whole read-validate-mutate sequence so
/// first-writer-wins is atomic — this is the in-memory equivalent of the Postgres
/// backend's unique `(table_id, commit_version)` constraint.
#[derive(Debug)]
pub struct InMemoryCommitCoordinator {
    tables: RwLock<HashMap<String, Arc<Mutex<TableCommitState>>>>,
    max_unbackfilled_commits: i64,
}

impl Default for InMemoryCommitCoordinator {
    fn default() -> Self {
        Self::new(DEFAULT_MAX_UNBACKFILLED_COMMITS)
    }
}

impl InMemoryCommitCoordinator {
    /// Create a coordinator with the given unbackfilled-commit cap.
    pub fn new(max_unbackfilled_commits: i64) -> Self {
        Self {
            tables: RwLock::new(HashMap::new()),
            max_unbackfilled_commits,
        }
    }

    /// Get the per-table state mutex, creating it if absent.
    fn table_state(&self, table_id: &str) -> Arc<Mutex<TableCommitState>> {
        if let Some(state) = self
            .tables
            .read()
            .expect("tables lock poisoned")
            .get(table_id)
        {
            return state.clone();
        }
        self.tables
            .write()
            .expect("tables lock poisoned")
            .entry(table_id.to_string())
            .or_default()
            .clone()
    }
}

#[async_trait::async_trait]
impl CommitCoordinator for InMemoryCommitCoordinator {
    async fn commit(
        &self,
        table_id: &str,
        commit_info: Option<CommitInfo>,
        latest_backfilled_version: Option<i64>,
    ) -> CommitResult<()> {
        if commit_info.is_none() && latest_backfilled_version.is_none() {
            return Err(CommitError::InvalidArgument(
                "either commit_info or latest_backfilled_version must be provided".to_string(),
            ));
        }
        if let Some(info) = &commit_info {
            validate_commit_info(info)?;
        }

        let entry = self.table_state(table_id);
        let mut state = entry.lock().expect("commit state mutex poisoned");

        match (state.last_version(), commit_info) {
            // No commits yet.
            (None, Some(info)) => {
                // Onboarding commit: accept the first posted commit as-is.
                let version = info.version;
                state.commits.insert(
                    version,
                    StoredCommit {
                        info,
                        is_backfilled_latest: false,
                    },
                );
                Ok(())
            }
            (None, None) => Err(CommitError::InvalidArgument(
                "cannot backfill a table with no commits".to_string(),
            )),

            // Backfill-only notification.
            (Some(last), None) => {
                let lbv = latest_backfilled_version.expect("checked above");
                if lbv > last {
                    return Err(CommitError::InvalidArgument(format!(
                        "latest_backfilled_version {lbv} is greater than the latest commit {last}"
                    )));
                }
                backfill(&mut state, lbv);
                Ok(())
            }

            // Normal commit to an existing table.
            (Some(last), Some(info)) => {
                let version = info.version;
                if version <= last {
                    return Err(CommitError::VersionConflict(format!(
                        "commit version {version} was already accepted; current table version is {last}"
                    )));
                }
                if version > last + 1 {
                    return Err(CommitError::InvalidArgument(format!(
                        "commit version must be the next version after {last}, but got {version}"
                    )));
                }
                if let Some(lbv) = latest_backfilled_version
                    && lbv > last
                {
                    return Err(CommitError::InvalidArgument(format!(
                        "latest_backfilled_version {lbv} is greater than the latest commit {last}"
                    )));
                }

                // Enforce the unbackfilled-commit cap. The effective backfilled
                // watermark accounts for a backfill piggy-backed on this request.
                let eff_backfill = effective_backfilled_version(&state, latest_backfilled_version);
                let expected_count = version - (eff_backfill + 1) + 1;
                if expected_count > self.max_unbackfilled_commits {
                    return Err(CommitError::ResourceExhausted(format!(
                        "max number of unbackfilled commits per table reached: {} (limit {})",
                        expected_count, self.max_unbackfilled_commits
                    )));
                }

                state.commits.insert(
                    version,
                    StoredCommit {
                        info,
                        is_backfilled_latest: false,
                    },
                );
                if let Some(lbv) = latest_backfilled_version {
                    backfill(&mut state, lbv);
                }
                Ok(())
            }
        }
    }

    async fn get_commits(
        &self,
        table_id: &str,
        start_version: i64,
        end_version: Option<i64>,
    ) -> CommitResult<(Vec<CommitInfo>, i64)> {
        if start_version < 0 {
            return Err(CommitError::InvalidArgument(
                "start_version must be non-negative".to_string(),
            ));
        }
        if let Some(end) = end_version
            && end < start_version
        {
            return Err(CommitError::InvalidArgument(format!(
                "end_version {end} must be >= start_version {start_version}"
            )));
        }

        let Some(entry) = self
            .tables
            .read()
            .expect("tables lock poisoned")
            .get(table_id)
            .cloned()
        else {
            // Managed table with no commits yet.
            return Ok((Vec::new(), 0));
        };
        let state = entry.lock().expect("commit state mutex poisoned");

        let latest_table_version = state.last_version().unwrap_or(0);
        let oldest = state.first_version().unwrap_or(0);

        // Pagination window, matching UC OSS: keep at most `max` commits starting
        // from `max(start, oldest)`, intersected with `end_version`.
        let window_start = start_version.max(oldest);
        let paginated_end = window_start + self.max_unbackfilled_commits - 1;
        let effective_end = end_version.unwrap_or(i64::MAX).min(paginated_end);

        let commits: Vec<CommitInfo> = state
            .commits
            .range(window_start..=effective_end)
            .filter(|(_, c)| !c.is_backfilled_latest)
            .map(|(_, c)| c.info.clone())
            .collect();

        if effective_end < latest_table_version && effective_end == paginated_end {
            tracing::debug!(
                table_id,
                effective_end,
                latest_table_version,
                "get_commits truncated to pagination window"
            );
        }

        Ok((commits, latest_table_version))
    }
}

/// Backfill commits up to `up_to`, preserving the highest-version row.
///
/// Port of UC OSS `backfillCommits`: deletes rows with version
/// `<= min(up_to, last - 1)`; if `up_to >= last`, the last row is kept and marked
/// as the backfilled-latest marker instead of being deleted.
fn backfill(state: &mut TableCommitState, up_to: i64) {
    let Some(last) = state.last_version() else {
        return;
    };
    let delete_up_to = up_to.min(last - 1);
    let to_remove: Vec<i64> = state
        .commits
        .range(..=delete_up_to)
        .map(|(v, _)| *v)
        .collect();
    for v in to_remove {
        state.commits.remove(&v);
    }
    if up_to >= last
        && let Some(c) = state.commits.get_mut(&last)
    {
        c.is_backfilled_latest = true;
    }
}

/// Compute the effective backfilled watermark for the cap check, accounting for a
/// backfill piggy-backed on the current request. Port of UC OSS
/// `getEffectiveBackfilledVersion`.
fn effective_backfilled_version(
    state: &TableCommitState,
    latest_backfilled_version: Option<i64>,
) -> i64 {
    // The oldest retained row marks the watermark: everything below it is already
    // backfilled. `first - 1` is the highest backfilled version implied by state.
    let from_state = state.first_version().map(|f| f - 1).unwrap_or(-1);
    match latest_backfilled_version {
        Some(lbv) => from_state.max(lbv),
        None => from_state,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn commit_info(version: i64) -> CommitInfo {
        CommitInfo {
            version,
            timestamp: 1000 + version,
            file_name: format!("{version:020}.0000-uuid.json"),
            file_size: 128,
            file_modification_timestamp: 2000 + version,
        }
    }

    #[tokio::test]
    async fn onboarding_then_normal_commits() {
        let cc = InMemoryCommitCoordinator::default();
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();
        cc.commit("t", Some(commit_info(2)), None).await.unwrap();
        let (commits, latest) = cc.get_commits("t", 0, None).await.unwrap();
        assert_eq!(latest, 2);
        assert_eq!(
            commits.iter().map(|c| c.version).collect::<Vec<_>>(),
            vec![1, 2]
        );
    }

    #[tokio::test]
    async fn backfill_only_with_no_prior_commit_is_invalid() {
        let cc = InMemoryCommitCoordinator::default();
        let err = cc.commit("t", None, Some(1)).await.unwrap_err();
        assert!(matches!(err, CommitError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn replay_version_conflicts() {
        let cc = InMemoryCommitCoordinator::default();
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();
        cc.commit("t", Some(commit_info(2)), None).await.unwrap();
        let err = cc
            .commit("t", Some(commit_info(2)), None)
            .await
            .unwrap_err();
        assert!(matches!(err, CommitError::VersionConflict(_)));
        let err = cc
            .commit("t", Some(commit_info(1)), None)
            .await
            .unwrap_err();
        assert!(matches!(err, CommitError::VersionConflict(_)));
    }

    #[tokio::test]
    async fn version_gap_is_invalid() {
        let cc = InMemoryCommitCoordinator::default();
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();
        let err = cc
            .commit("t", Some(commit_info(3)), None)
            .await
            .unwrap_err();
        assert!(matches!(err, CommitError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn field_validation() {
        let cc = InMemoryCommitCoordinator::default();
        let mut bad = commit_info(1);
        bad.version = 0;
        assert!(matches!(
            cc.commit("t", Some(bad), None).await.unwrap_err(),
            CommitError::InvalidArgument(_)
        ));
        let mut bad = commit_info(1);
        bad.file_name = String::new();
        assert!(matches!(
            cc.commit("t", Some(bad), None).await.unwrap_err(),
            CommitError::InvalidArgument(_)
        ));
        let mut bad = commit_info(1);
        bad.file_size = 0;
        assert!(matches!(
            cc.commit("t", Some(bad), None).await.unwrap_err(),
            CommitError::InvalidArgument(_)
        ));
    }

    #[tokio::test]
    async fn backfill_prunes_but_keeps_highest_as_marker() {
        let cc = InMemoryCommitCoordinator::default();
        for v in 1..=4 {
            cc.commit("t", Some(commit_info(v)), None).await.unwrap();
        }
        cc.commit("t", None, Some(4)).await.unwrap();
        let (commits, latest) = cc.get_commits("t", 0, None).await.unwrap();
        assert_eq!(latest, 4, "latest_table_version still reported from marker");
        assert!(commits.is_empty(), "marker row excluded from commits");

        cc.commit("t", Some(commit_info(5)), None).await.unwrap();
        let (commits, latest) = cc.get_commits("t", 0, None).await.unwrap();
        assert_eq!(latest, 5);
        assert_eq!(
            commits.iter().map(|c| c.version).collect::<Vec<_>>(),
            vec![5]
        );
    }

    #[tokio::test]
    async fn partial_backfill_prunes_only_below_watermark() {
        let cc = InMemoryCommitCoordinator::default();
        for v in 1..=5 {
            cc.commit("t", Some(commit_info(v)), None).await.unwrap();
        }
        cc.commit("t", None, Some(3)).await.unwrap();
        let (commits, latest) = cc.get_commits("t", 0, None).await.unwrap();
        assert_eq!(latest, 5);
        assert_eq!(
            commits.iter().map(|c| c.version).collect::<Vec<_>>(),
            vec![4, 5]
        );
    }

    #[tokio::test]
    async fn backfill_beyond_latest_is_invalid() {
        let cc = InMemoryCommitCoordinator::default();
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();
        let err = cc.commit("t", None, Some(2)).await.unwrap_err();
        assert!(matches!(err, CommitError::InvalidArgument(_)));
    }

    #[tokio::test]
    async fn unbackfilled_cap_is_enforced_and_reopened_by_backfill() {
        let cc = InMemoryCommitCoordinator::new(3);
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();
        cc.commit("t", Some(commit_info(2)), None).await.unwrap();
        cc.commit("t", Some(commit_info(3)), None).await.unwrap();
        let err = cc
            .commit("t", Some(commit_info(4)), None)
            .await
            .unwrap_err();
        assert!(matches!(err, CommitError::ResourceExhausted(_)));
        cc.commit("t", None, Some(2)).await.unwrap();
        cc.commit("t", Some(commit_info(4)), None).await.unwrap();
        let (_, latest) = cc.get_commits("t", 0, None).await.unwrap();
        assert_eq!(latest, 4);
    }

    #[tokio::test]
    async fn get_commits_unknown_table_reports_zero() {
        let cc = InMemoryCommitCoordinator::default();
        let (commits, latest) = cc.get_commits("missing", 0, None).await.unwrap();
        assert!(commits.is_empty());
        assert_eq!(latest, 0);
    }

    #[tokio::test]
    async fn get_commits_invalid_range() {
        let cc = InMemoryCommitCoordinator::default();
        assert!(matches!(
            cc.get_commits("t", 5, Some(2)).await.unwrap_err(),
            CommitError::InvalidArgument(_)
        ));
    }

    #[tokio::test]
    async fn get_commits_respects_end_version() {
        let cc = InMemoryCommitCoordinator::default();
        for v in 1..=3 {
            cc.commit("t", Some(commit_info(v)), None).await.unwrap();
        }
        let (commits, latest) = cc.get_commits("t", 0, Some(1)).await.unwrap();
        assert_eq!(latest, 3);
        assert_eq!(
            commits.iter().map(|c| c.version).collect::<Vec<_>>(),
            vec![1]
        );
    }

    #[tokio::test]
    async fn get_commits_caps_window_to_max() {
        let cc = InMemoryCommitCoordinator::new(2);
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();
        cc.commit("t", Some(commit_info(2)), None).await.unwrap();
        cc.commit("t", None, Some(1)).await.unwrap();
        cc.commit("t", Some(commit_info(3)), None).await.unwrap();
        let (commits, latest) = cc.get_commits("t", 0, None).await.unwrap();
        assert_eq!(latest, 3);
        assert_eq!(
            commits.iter().map(|c| c.version).collect::<Vec<_>>(),
            vec![2, 3]
        );
    }

    #[tokio::test]
    async fn first_writer_wins_under_concurrency() {
        let cc = Arc::new(InMemoryCommitCoordinator::default());
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();

        let mut handles = Vec::new();
        for _ in 0..16 {
            let cc = cc.clone();
            handles.push(tokio::spawn(async move {
                cc.commit("t", Some(commit_info(2)), None).await
            }));
        }
        let mut wins = 0;
        let mut conflicts = 0;
        for h in handles {
            match h.await.unwrap() {
                Ok(()) => wins += 1,
                Err(CommitError::VersionConflict(_)) => conflicts += 1,
                Err(e) => panic!("unexpected error: {e:?}"),
            }
        }
        assert_eq!(wins, 1, "exactly one writer wins version 2");
        assert_eq!(conflicts, 15, "all other writers conflict");
    }
}
