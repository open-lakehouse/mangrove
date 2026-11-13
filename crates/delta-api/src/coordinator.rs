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
//! All port the arbitration, backfill and retry logic of the Unity Catalog OSS
//! reference implementation (`DeltaCommitRepository.java`). The notable
//! invariants:
//!
//! 1. Backfill **flags** commit rows rather than deleting them. The *live commit
//!    window* — what [`get_commits`](CommitCoordinator::get_commits) returns and
//!    what the unbackfilled-commit cap applies to — is the unflagged rows. Flagged
//!    rows always form one contiguous block at the bottom, so the table's
//!    backfilled version is the highest flagged row (or the oldest row minus one
//!    when none is flagged).
//! 2. Flagged rows are retained for [`DEFAULT_BACKFILLED_COMMITS_RETAINED`]
//!    versions below the backfilled version, purely so a retried `add-commit` can
//!    be answered from its file name: same name is a replay
//!    ([`CommitError::AlreadyAccepted`]), a different name a conflict
//!    ([`CommitError::VersionConflict`]). A retry older than the retention window
//!    cannot be decided from the catalog alone ([`CommitError::StateUnknown`]).
//! 3. The highest row is never pruned, so an onboarded table always reports its
//!    `latest_table_version`.
//! 4. First *posted* commit version is `>= 1`; version `0` is established
//!    out-of-band by create-table. `CommitInfo` fields must be positive / non-empty.
//! 5. No in-commit-timestamp monotonicity is enforced (client's responsibility).
//! 6. There is a cap on unbackfilled commits per table (OSS hardcodes 10); exceeding
//!    it rejects the commit with a resource-exhausted (429) error.
//!
//! Wire-format note: this matches the *shipped* UC OSS server, not the prose
//! `ManagedTablesSpec.md`, which disagree in places (path `/delta/preview/commits`
//! vs `/delta/commit`, field `latest_backfilled_version` vs `latest_published_version`).

use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, RwLock};

/// A ratified Delta commit, the coordinator's per-version record.
///
/// All fields must be positive / non-empty — see [`validate_commit_info`].
#[derive(Clone, Debug, PartialEq)]
pub struct CommitInfo {
    /// The commit version.
    pub version: i64,
    /// In-commit timestamp, in epoch milliseconds.
    pub timestamp: i64,
    /// UUID-based commit file name. Also the retry-deduplication handle: a
    /// re-sent `add-commit` carries the same file name.
    pub file_name: String,
    /// Size of the commit file, in bytes.
    pub file_size: i64,
    /// File modification timestamp, in epoch milliseconds.
    pub file_modification_timestamp: i64,
}

/// Default cap on the number of unbackfilled commits a table may accumulate
/// before further commits are rejected. Matches UC OSS `MAX_NUM_COMMITS_PER_TABLE`.
pub const DEFAULT_MAX_UNBACKFILLED_COMMITS: i64 = 10;

/// Default number of versions below the backfilled version whose (flagged) commit
/// rows are retained for retry deduplication. Matches UC OSS
/// `NUM_BACKFILLED_COMMITS_RETAINED`.
pub const DEFAULT_BACKFILLED_COMMITS_RETAINED: i64 = 100;

/// Error returned by a [`CommitCoordinator`].
///
/// Variants map onto the HTTP statuses the UC OSS commit API returns. Convert into
/// a [`DeltaBackendError`](crate::error::DeltaBackendError) with `From`.
///
/// `#[non_exhaustive]`: match with a wildcard arm.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum CommitError {
    /// Another commit holds the requested version (lost the first-writer-wins
    /// race). Maps to 409.
    #[error("commit version conflict: {0}")]
    VersionConflict(String),

    /// The commit is a retry of one already accepted at this version (same file
    /// name). Not a failure: the request's work is already done, and the caller
    /// answers with the current table state.
    #[error("commit already accepted")]
    AlreadyAccepted,

    /// The outcome cannot be decided from the catalog: a retry of a version whose
    /// record is no longer retained. Maps to 500 `CommitStateUnknownException`.
    #[error("commit state unknown: {0}")]
    StateUnknown(String),

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
/// which are a port of UC OSS `DeltaCommitRepository`.
#[async_trait::async_trait]
pub trait CommitCoordinator: Send + Sync + 'static {
    /// Ratify a commit and/or record a backfill notification for `table_id`.
    ///
    /// At least one of `commit_info` / `latest_backfilled_version` must be set.
    /// A retry of an already-ratified commit (same version and file name) returns
    /// [`CommitError::AlreadyAccepted`] and changes nothing.
    async fn commit(
        &self,
        table_id: &str,
        commit_info: Option<CommitInfo>,
        latest_backfilled_version: Option<i64>,
    ) -> CommitResult<()>;

    /// Return ratified-but-unpublished commits for `table_id` in
    /// `[start_version, end_version]`, plus `latest_table_version`.
    ///
    /// The returned commits are the live window only (backfilled rows are
    /// excluded). `latest_table_version` is `0` for a managed table with no
    /// commits, else the highest tracked version.
    async fn get_commits(
        &self,
        table_id: &str,
        start_version: i64,
        end_version: Option<i64>,
    ) -> CommitResult<(Vec<CommitInfo>, i64)>;

    /// The ratified commit at `version`, if the coordinator still holds a record
    /// of it — backfilled-but-retained rows included.
    ///
    /// Used to recognize a retried `add-commit` before any other check runs. The
    /// default returns `Ok(None)` (no lookup), which only means a replay is
    /// detected later, by [`commit`](Self::commit).
    async fn find_commit(&self, table_id: &str, version: i64) -> CommitResult<Option<CommitInfo>> {
        let _ = (table_id, version);
        Ok(None)
    }
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

    async fn find_commit(&self, table_id: &str, version: i64) -> CommitResult<Option<CommitInfo>> {
        self.as_ref().find_commit(table_id, version).await
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

/// Classify a commit at a version the table already holds (`version <= last`):
/// the stored row decides between a replay and a conflict, and a version below
/// the retained rows cannot be decided. Shared by all backends so they answer
/// retries identically.
///
/// `stored` is the row at `version` (flagged or not), `oldest_retained` the lowest
/// version the backend still holds.
pub fn classify_existing_version(
    incoming: &CommitInfo,
    stored: Option<&CommitInfo>,
    oldest_retained: i64,
) -> CommitError {
    match stored {
        Some(row) if row.file_name == incoming.file_name => CommitError::AlreadyAccepted,
        Some(_) => CommitError::VersionConflict(format!(
            "commit version {} was already accepted by another commit",
            incoming.version
        )),
        None if incoming.version < oldest_retained => CommitError::StateUnknown(format!(
            "commit version {} is no longer retained by the catalog; compare the staged \
             commit with the published log",
            incoming.version
        )),
        None => CommitError::VersionConflict(format!(
            "commit version {} was already accepted",
            incoming.version
        )),
    }
}

/// A ratified commit plus its backfill flag.
#[derive(Debug, Clone)]
struct StoredCommit {
    info: CommitInfo,
    /// `true` once the commit has been published to `_delta_log` and reported via
    /// backfill. Flagged rows are outside the live window and kept only for
    /// retry deduplication.
    backfilled: bool,
}

/// Per-table ratified-commit state, ordered by version.
#[derive(Debug, Default)]
struct TableCommitState {
    commits: BTreeMap<i64, StoredCommit>,
}

impl TableCommitState {
    fn first_version(&self) -> Option<i64> {
        self.commits.keys().next().copied()
    }

    fn last_version(&self) -> Option<i64> {
        self.commits.keys().next_back().copied()
    }

    /// The table's backfilled version: the highest flagged row, or the oldest row
    /// minus one when none is flagged (everything below it was pruned). Port of
    /// UC OSS `backfilledVersionOf`.
    fn backfilled_through(&self) -> i64 {
        self.commits
            .iter()
            .rev()
            .find(|(_, c)| c.backfilled)
            .map(|(v, _)| *v)
            .or_else(|| self.first_version().map(|f| f - 1))
            .unwrap_or(-1)
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
    backfilled_commits_retained: i64,
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
            backfilled_commits_retained: DEFAULT_BACKFILLED_COMMITS_RETAINED,
        }
    }

    /// Override how many versions below the backfilled version keep their row for
    /// retry deduplication.
    pub fn with_backfilled_commits_retained(mut self, retained: i64) -> Self {
        self.backfilled_commits_retained = retained;
        self
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

    /// The synchronous core of [`CommitCoordinator::commit`]. Never awaits, so a
    /// caller holding its own lock (e.g. the in-memory Delta backend applying a
    /// commit and a metadata change atomically) can call it in its critical
    /// section.
    pub fn commit_sync(
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
            // No commits yet: an onboarding commit is accepted as-is.
            (None, Some(info)) => {
                state.commits.insert(
                    info.version,
                    StoredCommit {
                        info,
                        backfilled: false,
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
                self.backfill(&mut state, lbv);
                Ok(())
            }

            // Commit to an existing table.
            (Some(last), Some(info)) => {
                let version = info.version;
                if version <= last {
                    let stored = state.commits.get(&version).map(|c| &c.info);
                    let oldest = state.first_version().unwrap_or(version);
                    return Err(classify_existing_version(&info, stored, oldest));
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

                // The unbackfilled-commit cap applies to the window after this
                // request: everything above the higher of the current backfilled
                // version and a backfill piggy-backed on this request.
                let effective = state
                    .backfilled_through()
                    .max(latest_backfilled_version.unwrap_or(i64::MIN));
                if version - effective > self.max_unbackfilled_commits {
                    return Err(CommitError::ResourceExhausted(format!(
                        "max number of unbackfilled commits per table reached: {} (limit {})",
                        version - effective,
                        self.max_unbackfilled_commits
                    )));
                }

                state.commits.insert(
                    version,
                    StoredCommit {
                        info,
                        backfilled: false,
                    },
                );
                if let Some(lbv) = latest_backfilled_version {
                    self.backfill(&mut state, lbv);
                }
                Ok(())
            }
        }
    }

    /// Flag commits up to `up_to` as backfilled and prune flagged rows that have
    /// aged out of the retention window. The backfilled version only moves
    /// forward, and the highest row is never pruned.
    fn backfill(&self, state: &mut TableCommitState, up_to: i64) {
        let Some(last) = state.last_version() else {
            return;
        };
        let through = state.backfilled_through().max(up_to);
        for (_, c) in state.commits.range_mut(..=through) {
            c.backfilled = true;
        }
        let prune_below = (through - self.backfilled_commits_retained).min(last);
        let stale: Vec<i64> = state
            .commits
            .range(..prune_below)
            .map(|(v, _)| *v)
            .collect();
        for v in stale {
            state.commits.remove(&v);
        }
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
        self.commit_sync(table_id, commit_info, latest_backfilled_version)
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
        // The live window starts just above the backfilled version.
        let oldest_live = state.backfilled_through() + 1;

        // Pagination window, matching UC OSS: keep at most `max` commits starting
        // from `max(start, oldest live)`, intersected with `end_version`.
        let window_start = start_version.max(oldest_live);
        let paginated_end = window_start + self.max_unbackfilled_commits - 1;
        let effective_end = end_version.unwrap_or(i64::MAX).min(paginated_end);

        let commits: Vec<CommitInfo> = if window_start > effective_end {
            Vec::new()
        } else {
            state
                .commits
                .range(window_start..=effective_end)
                .filter(|(_, c)| !c.backfilled)
                .map(|(_, c)| c.info.clone())
                .collect()
        };

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

    async fn find_commit(&self, table_id: &str, version: i64) -> CommitResult<Option<CommitInfo>> {
        let Some(entry) = self
            .tables
            .read()
            .expect("tables lock poisoned")
            .get(table_id)
            .cloned()
        else {
            return Ok(None);
        };
        let state = entry.lock().expect("commit state mutex poisoned");
        Ok(state.commits.get(&version).map(|c| c.info.clone()))
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

    fn other_commit(version: i64) -> CommitInfo {
        CommitInfo {
            file_name: format!("{version:020}.ffff-other.json"),
            ..commit_info(version)
        }
    }

    #[tokio::test]
    async fn retry_of_accepted_commit_is_already_accepted() {
        let cc = InMemoryCommitCoordinator::default();
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();
        cc.commit("t", Some(commit_info(2)), None).await.unwrap();
        for v in [1, 2] {
            let err = cc
                .commit("t", Some(commit_info(v)), None)
                .await
                .unwrap_err();
            assert!(matches!(err, CommitError::AlreadyAccepted), "{err:?}");
        }
        let (commits, latest) = cc.get_commits("t", 0, None).await.unwrap();
        assert_eq!(latest, 2);
        assert_eq!(commits.len(), 2, "a replay changes nothing");
    }

    #[tokio::test]
    async fn different_commit_at_taken_version_conflicts() {
        let cc = InMemoryCommitCoordinator::default();
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();
        cc.commit("t", Some(commit_info(2)), None).await.unwrap();
        for v in [1, 2] {
            let err = cc
                .commit("t", Some(other_commit(v)), None)
                .await
                .unwrap_err();
            assert!(matches!(err, CommitError::VersionConflict(_)), "{err:?}");
        }
    }

    #[tokio::test]
    async fn retry_is_recognized_after_backfill_within_retention() {
        let cc = InMemoryCommitCoordinator::default();
        for v in 1..=3 {
            cc.commit("t", Some(commit_info(v)), None).await.unwrap();
        }
        cc.commit("t", None, Some(3)).await.unwrap();
        let err = cc
            .commit("t", Some(commit_info(2)), None)
            .await
            .unwrap_err();
        assert!(matches!(err, CommitError::AlreadyAccepted), "{err:?}");
        assert_eq!(
            cc.find_commit("t", 2).await.unwrap(),
            Some(commit_info(2)),
            "backfilled rows stay findable"
        );
    }

    #[tokio::test]
    async fn retry_older_than_retention_is_state_unknown() {
        let cc = InMemoryCommitCoordinator::default().with_backfilled_commits_retained(1);
        for v in 1..=4 {
            cc.commit("t", Some(commit_info(v)), None).await.unwrap();
        }
        // Backfilled through 4, retaining 1 version below: rows < 3 are pruned.
        cc.commit("t", None, Some(4)).await.unwrap();
        assert_eq!(cc.find_commit("t", 1).await.unwrap(), None);
        let err = cc
            .commit("t", Some(commit_info(1)), None)
            .await
            .unwrap_err();
        assert!(matches!(err, CommitError::StateUnknown(_)), "{err:?}");
        // The latest row is never pruned.
        let (_, latest) = cc.get_commits("t", 0, None).await.unwrap();
        assert_eq!(latest, 4);
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
    async fn fully_backfilled_marker_does_not_count_against_the_cap() {
        // After a full backfill the latest row is flagged (backfilled), so a
        // whole fresh window of `max` commits fits above it.
        let cc = InMemoryCommitCoordinator::new(3);
        cc.commit("t", Some(commit_info(1)), None).await.unwrap();
        cc.commit("t", None, Some(1)).await.unwrap();
        for v in 2..=4 {
            cc.commit("t", Some(commit_info(v)), None).await.unwrap();
        }
        let err = cc
            .commit("t", Some(commit_info(5)), None)
            .await
            .unwrap_err();
        assert!(matches!(err, CommitError::ResourceExhausted(_)));
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
                // Distinct file names: competing writers, not retries.
                let info = CommitInfo {
                    file_name: format!("00000000000000000002.{}.json", uuid::Uuid::now_v7()),
                    ..commit_info(2)
                };
                cc.commit("t", Some(info), None).await
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
