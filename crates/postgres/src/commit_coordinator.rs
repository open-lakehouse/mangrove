//! Postgres-backed Delta [`CommitCoordinator`].
//!
//! Persists ratified commits in the `delta_commits` table and delegates every
//! arbitration decision to the storage-agnostic planner in
//! [`unitycatalog_delta_api::coordinator`] ([`plan_commit`], [`live_window`]), so
//! it follows exactly the rules of the in-memory and SQLite coordinators (a port
//! of UC OSS `DeltaCommitRepository`): backfill flags rows rather than deleting
//! them, flagged rows are retained for retry deduplication, and a retried commit
//! is recognized by its file name.
//!
//! The unique constraint on `(table_id, commit_version)` is the first-writer-wins
//! arbiter: each `commit` runs in a transaction, and a racing insert for the same
//! version fails with a unique violation. The winner's row then decides whether
//! the loser was a retry of the same commit or a conflicting one.
//!
//! The `is_backfilled_latest` column predates retention; it now means "this row
//! has been backfilled". Rows written under the old layout (backfilled rows
//! deleted, only a fully backfilled latest row flagged) read correctly under the
//! new rule, so no migration is needed.

use sqlx::PgPool;
use unitycatalog_delta_api::coordinator::{
    CommitCoordinator, CommitError, CommitInfo, CommitLimits, CommitLogBounds, CommitResult,
    classify_existing_version, live_window, plan_commit,
};
use uuid::Uuid;

/// Postgres-backed Delta [`CommitCoordinator`] over the `delta_commits` table.
///
/// Holds its own clone of the shared [`PgPool`]; the object/association graph
/// store ([`PgGraphStore`](crate::PgGraphStore)) is an independent handle on the
/// same database.
#[derive(Clone)]
pub struct PgCommitCoordinator {
    pool: PgPool,
    limits: CommitLimits,
}

impl PgCommitCoordinator {
    /// Construct a coordinator over a connection pool, with the UC OSS limits.
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            limits: CommitLimits::default(),
        }
    }

    /// Override the unbackfilled-commit cap and the backfilled-row retention.
    pub fn with_limits(mut self, limits: CommitLimits) -> Self {
        self.limits = limits;
        self
    }
}

/// Convert epoch-millis (the wire representation) to a `timestamptz` value.
fn millis_to_dt(millis: i64) -> CommitResult<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::from_timestamp_millis(millis)
        .ok_or_else(|| CommitError::InvalidArgument(format!("timestamp out of range: {millis}")))
}

/// Convert a `timestamptz` back to epoch-millis.
fn dt_to_millis(dt: chrono::DateTime<chrono::Utc>) -> i64 {
    dt.timestamp_millis()
}

/// Parse and validate a `table_id` UUID.
fn parse_table_id(table_id: &str) -> CommitResult<Uuid> {
    Uuid::parse_str(table_id)
        .map_err(|_| CommitError::InvalidArgument("table_id is not a valid UUID".to_string()))
}

/// Whether a sqlx error is a unique violation (the `(table_id, commit_version)`
/// first-writer-wins constraint).
fn is_unique_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(db_err)
        if db_err
            .try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
            .is_some_and(|pg| pg.code() == "23505"))
}

fn backend_err(e: sqlx::Error) -> CommitError {
    CommitError::Backend(e.to_string())
}

impl PgCommitCoordinator {
    /// Read a table's commit-log bounds.
    async fn bounds<'e, E>(&self, exec: E, table_id: Uuid) -> CommitResult<Option<CommitLogBounds>>
    where
        E: sqlx::PgExecutor<'e>,
    {
        let row = sqlx::query!(
            r#"SELECT MIN(commit_version) AS "first",
                      MAX(commit_version) AS "last",
                      MAX(commit_version) FILTER (WHERE is_backfilled_latest) AS "highest_backfilled"
               FROM delta_commits WHERE table_id = $1"#,
            table_id
        )
        .fetch_one(exec)
        .await
        .map_err(backend_err)?;
        Ok(match (row.first, row.last) {
            (Some(first), Some(last)) => Some(CommitLogBounds {
                first,
                last,
                highest_backfilled: row.highest_backfilled,
            }),
            _ => None,
        })
    }

    /// Read the row (flagged or not) at `version`.
    async fn row_at<'e, E>(
        &self,
        exec: E,
        table_id: Uuid,
        version: i64,
    ) -> CommitResult<Option<CommitInfo>>
    where
        E: sqlx::PgExecutor<'e>,
    {
        let row = sqlx::query!(
            "SELECT commit_version, commit_timestamp, commit_filename, commit_filesize, \
                    commit_file_modification_timestamp \
             FROM delta_commits WHERE table_id = $1 AND commit_version = $2",
            table_id,
            version
        )
        .fetch_optional(exec)
        .await
        .map_err(backend_err)?;
        Ok(row.map(|row| CommitInfo {
            version: row.commit_version,
            timestamp: dt_to_millis(row.commit_timestamp),
            file_name: row.commit_filename,
            file_size: row.commit_filesize,
            file_modification_timestamp: dt_to_millis(row.commit_file_modification_timestamp),
        }))
    }

    /// Plan and execute one request inside `txn`. Returns `Ok(Err(info))` when
    /// the insert lost the first-writer-wins race, so the caller can classify it
    /// against the winner's row outside the aborted transaction.
    async fn commit_in(
        &self,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        table_id: Uuid,
        commit_info: Option<CommitInfo>,
        latest_backfilled_version: Option<i64>,
    ) -> CommitResult<Result<(), CommitInfo>> {
        let bounds = self.bounds(&mut **txn, table_id).await?;
        let stored = match &commit_info {
            Some(info) => self.row_at(&mut **txn, table_id, info.version).await?,
            None => None,
        };
        let plan = plan_commit(
            bounds,
            commit_info,
            latest_backfilled_version,
            stored.as_ref(),
            self.limits,
        )?;

        if let Some(info) = &plan.insert {
            let inserted = sqlx::query!(
                "INSERT INTO delta_commits \
                 (table_id, commit_version, commit_filename, commit_filesize, \
                  commit_file_modification_timestamp, commit_timestamp) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
                table_id,
                info.version,
                info.file_name,
                info.file_size,
                millis_to_dt(info.file_modification_timestamp)?,
                millis_to_dt(info.timestamp)?,
            )
            .execute(&mut **txn)
            .await;
            match inserted {
                Ok(_) => {}
                Err(e) if is_unique_violation(&e) => return Ok(Err(info.clone())),
                Err(e) => return Err(backend_err(e)),
            }
        }
        if let Some(b) = plan.backfill {
            sqlx::query!(
                "UPDATE delta_commits SET is_backfilled_latest = true, updated_at = now() \
                 WHERE table_id = $1 AND commit_version <= $2 AND is_backfilled_latest = false",
                table_id,
                b.flag_through
            )
            .execute(&mut **txn)
            .await
            .map_err(backend_err)?;
            sqlx::query!(
                "DELETE FROM delta_commits \
                 WHERE table_id = $1 AND commit_version < $2 AND is_backfilled_latest = true",
                table_id,
                b.prune_below
            )
            .execute(&mut **txn)
            .await
            .map_err(backend_err)?;
        }
        Ok(Ok(()))
    }
}

#[async_trait::async_trait]
impl CommitCoordinator for PgCommitCoordinator {
    async fn commit(
        &self,
        table_id: &str,
        commit_info: Option<CommitInfo>,
        latest_backfilled_version: Option<i64>,
    ) -> CommitResult<()> {
        let table_id = parse_table_id(table_id)?;
        let mut txn = self.pool.begin().await.map_err(backend_err)?;
        match self
            .commit_in(&mut txn, table_id, commit_info, latest_backfilled_version)
            .await?
        {
            Ok(()) => txn.commit().await.map_err(backend_err),
            Err(lost) => {
                // Lost the insert race: the transaction is aborted, so read the
                // winner's row on a fresh connection to tell a concurrent retry
                // of the same commit from a conflicting one.
                drop(txn);
                let winner = self.row_at(&self.pool, table_id, lost.version).await?;
                Err(classify_existing_version(&lost, winner.as_ref(), i64::MIN))
            }
        }
    }

    async fn get_commits(
        &self,
        table_id: &str,
        start_version: i64,
        end_version: Option<i64>,
    ) -> CommitResult<(Vec<CommitInfo>, i64)> {
        let table_id = parse_table_id(table_id)?;
        let mut conn = self.pool.acquire().await.map_err(backend_err)?;
        let bounds = self.bounds(&mut *conn, table_id).await?;
        let latest_table_version = bounds.map_or(0, |b| b.last);
        let Some((from, to)) = live_window(bounds, start_version, end_version, self.limits)? else {
            return Ok((Vec::new(), latest_table_version));
        };

        let rows = sqlx::query!(
            "SELECT commit_version, commit_timestamp, commit_filename, commit_filesize, \
                    commit_file_modification_timestamp \
             FROM delta_commits \
             WHERE table_id = $1 AND commit_version >= $2 AND commit_version <= $3 \
               AND is_backfilled_latest = false \
             ORDER BY commit_version ASC",
            table_id,
            from,
            to
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(backend_err)?;

        let commits = rows
            .into_iter()
            .map(|row| CommitInfo {
                version: row.commit_version,
                timestamp: dt_to_millis(row.commit_timestamp),
                file_name: row.commit_filename,
                file_size: row.commit_filesize,
                file_modification_timestamp: dt_to_millis(row.commit_file_modification_timestamp),
            })
            .collect();
        Ok((commits, latest_table_version))
    }

    async fn find_commit(&self, table_id: &str, version: i64) -> CommitResult<Option<CommitInfo>> {
        let table_id = parse_table_id(table_id)?;
        self.row_at(&self.pool, table_id, version).await
    }
}
