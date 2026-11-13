//! SQLite-backed Delta [`CommitCoordinator`].
//!
//! Persists ratified commits in the SQLite `delta_commits` table and delegates
//! every arbitration decision to the storage-agnostic planner in
//! [`unitycatalog_delta_api::coordinator`] ([`plan_commit`], [`live_window`]), so
//! it follows exactly the rules of the in-memory and Postgres coordinators (a
//! port of UC OSS `DeltaCommitRepository`): backfill flags rows rather than
//! deleting them, flagged rows are retained for retry deduplication, and a
//! retried commit is recognized by its file name.
//!
//! Each `commit` runs as one `BEGIN IMMEDIATE` transaction, so contenders queue
//! on the write lock and then see the winner's row; the unique constraint on
//! `(table_id, commit_version)` remains the ultimate arbiter.
//!
//! Timestamps are stored as INTEGER epoch-millis — the same representation used
//! on the wire — so no timezone conversion is needed. The `is_backfilled_latest`
//! column predates retention; it now means "this row has been backfilled", and
//! rows written under the old layout read correctly under the new rule.

use sqlx::SqlitePool;
use unitycatalog_delta_api::coordinator::{
    CommitCoordinator, CommitError, CommitInfo, CommitLimits, CommitLogBounds, CommitResult,
    classify_existing_version, live_window, plan_commit,
};
use uuid::Uuid;

/// SQLite-backed Delta [`CommitCoordinator`] over the `delta_commits` table.
///
/// Holds its own clone of the shared [`SqlitePool`]; the object/association graph
/// store ([`SqliteGraphStore`](crate::SqliteGraphStore)) is an independent handle
/// on the same database.
#[derive(Clone)]
pub struct SqliteCommitCoordinator {
    pool: SqlitePool,
    limits: CommitLimits,
}

impl SqliteCommitCoordinator {
    /// Construct a coordinator over a connection pool, with the UC OSS limits.
    pub fn new(pool: SqlitePool) -> Self {
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

/// Parse and validate a `table_id` UUID, returning its raw bytes for storage.
fn parse_table_id(table_id: &str) -> CommitResult<Vec<u8>> {
    Uuid::parse_str(table_id)
        .map(|id| id.as_bytes().to_vec())
        .map_err(|_| CommitError::InvalidArgument("table_id is not a valid UUID".to_string()))
}

fn is_unique_violation(e: &sqlx::Error) -> bool {
    matches!(e, sqlx::Error::Database(db_err)
        if matches!(db_err.kind(), sqlx::error::ErrorKind::UniqueViolation))
}

fn backend_err(e: sqlx::Error) -> CommitError {
    CommitError::Backend(e.to_string())
}

impl SqliteCommitCoordinator {
    /// Read a table's commit-log bounds.
    async fn bounds(
        conn: &mut sqlx::SqliteConnection,
        table_id: &[u8],
    ) -> CommitResult<Option<CommitLogBounds>> {
        let row = sqlx::query!(
            r#"SELECT MIN(commit_version) AS "first?: i64",
                      MAX(commit_version) AS "last?: i64",
                      MAX(CASE WHEN is_backfilled_latest = 1 THEN commit_version END)
                          AS "highest_backfilled?: i64"
               FROM delta_commits WHERE table_id = ?"#,
            table_id
        )
        .fetch_one(&mut *conn)
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
    async fn row_at(
        conn: &mut sqlx::SqliteConnection,
        table_id: &[u8],
        version: i64,
    ) -> CommitResult<Option<CommitInfo>> {
        let row = sqlx::query!(
            "SELECT commit_version, commit_timestamp, commit_filename, commit_filesize, \
                    commit_file_modification_timestamp \
             FROM delta_commits WHERE table_id = ? AND commit_version = ?",
            table_id,
            version
        )
        .fetch_optional(&mut *conn)
        .await
        .map_err(backend_err)?;
        Ok(row.map(|row| CommitInfo {
            version: row.commit_version,
            timestamp: row.commit_timestamp,
            file_name: row.commit_filename,
            file_size: row.commit_filesize,
            file_modification_timestamp: row.commit_file_modification_timestamp,
        }))
    }

    /// Plan and execute one request. The caller owns the
    /// `BEGIN IMMEDIATE`/`COMMIT`/`ROLLBACK` lifecycle.
    async fn commit_txn(
        &self,
        conn: &mut sqlx::SqliteConnection,
        table_id: &[u8],
        commit_info: Option<CommitInfo>,
        latest_backfilled_version: Option<i64>,
    ) -> CommitResult<()> {
        let bounds = Self::bounds(conn, table_id).await?;
        let stored = match &commit_info {
            Some(info) => Self::row_at(conn, table_id, info.version).await?,
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
            let id = Uuid::now_v7().as_bytes().to_vec();
            let created_at = info.timestamp;
            let inserted = sqlx::query!(
                "INSERT INTO delta_commits \
                 (id, table_id, commit_version, commit_filename, commit_filesize, \
                  commit_file_modification_timestamp, commit_timestamp, created_at) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                id,
                table_id,
                info.version,
                info.file_name,
                info.file_size,
                info.file_modification_timestamp,
                info.timestamp,
                created_at,
            )
            .execute(&mut *conn)
            .await;
            match inserted {
                Ok(_) => {}
                // Unreachable under BEGIN IMMEDIATE (the planner saw no row),
                // kept so the constraint still decides if it ever happens.
                Err(e) if is_unique_violation(&e) => {
                    let winner = Self::row_at(conn, table_id, info.version).await?;
                    return Err(classify_existing_version(info, winner.as_ref(), i64::MIN));
                }
                Err(e) => return Err(backend_err(e)),
            }
        }
        if let Some(b) = plan.backfill {
            let now = chrono::Utc::now().timestamp_millis();
            sqlx::query!(
                "UPDATE delta_commits SET is_backfilled_latest = 1, updated_at = ? \
                 WHERE table_id = ? AND commit_version <= ? AND is_backfilled_latest = 0",
                now,
                table_id,
                b.flag_through
            )
            .execute(&mut *conn)
            .await
            .map_err(backend_err)?;
            sqlx::query!(
                "DELETE FROM delta_commits \
                 WHERE table_id = ? AND commit_version < ? AND is_backfilled_latest = 1",
                table_id,
                b.prune_below
            )
            .execute(&mut *conn)
            .await
            .map_err(backend_err)?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl CommitCoordinator for SqliteCommitCoordinator {
    async fn commit(
        &self,
        table_id: &str,
        commit_info: Option<CommitInfo>,
        latest_backfilled_version: Option<i64>,
    ) -> CommitResult<()> {
        let table_id = parse_table_id(table_id)?;

        // Run the read-then-write as a single `BEGIN IMMEDIATE` transaction so
        // the version check and the insert are atomic against other writers.
        // Under WAL a deferred transaction takes its read snapshot lazily and a
        // racing writer that commits first turns our later insert into a
        // snapshot conflict (SQLITE_BUSY_SNAPSHOT) rather than a clean unique
        // violation. Acquiring the write lock up front makes contenders wait on
        // `busy_timeout` and then observe the committed row, which the planner
        // classifies as a retry or a conflict.
        let mut conn = self.pool.acquire().await.map_err(backend_err)?;
        sqlx::query("BEGIN IMMEDIATE")
            .execute(&mut *conn)
            .await
            .map_err(backend_err)?;

        let result = self
            .commit_txn(&mut conn, &table_id, commit_info, latest_backfilled_version)
            .await;

        match &result {
            Ok(()) => {
                sqlx::query("COMMIT")
                    .execute(&mut *conn)
                    .await
                    .map_err(backend_err)?;
            }
            Err(_) => {
                // Best-effort rollback; surface the original error.
                let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
            }
        }
        result
    }

    async fn get_commits(
        &self,
        table_id: &str,
        start_version: i64,
        end_version: Option<i64>,
    ) -> CommitResult<(Vec<CommitInfo>, i64)> {
        let table_id = parse_table_id(table_id)?;
        let mut conn = self.pool.acquire().await.map_err(backend_err)?;
        let bounds = Self::bounds(&mut conn, &table_id).await?;
        let latest_table_version = bounds.map_or(0, |b| b.last);
        let Some((from, to)) = live_window(bounds, start_version, end_version, self.limits)? else {
            return Ok((Vec::new(), latest_table_version));
        };

        let rows = sqlx::query!(
            "SELECT commit_version, commit_timestamp, commit_filename, commit_filesize, \
                    commit_file_modification_timestamp \
             FROM delta_commits \
             WHERE table_id = ? AND commit_version >= ? AND commit_version <= ? \
               AND is_backfilled_latest = 0 \
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
                timestamp: row.commit_timestamp,
                file_name: row.commit_filename,
                file_size: row.commit_filesize,
                file_modification_timestamp: row.commit_file_modification_timestamp,
            })
            .collect();
        Ok((commits, latest_table_version))
    }

    async fn find_commit(&self, table_id: &str, version: i64) -> CommitResult<Option<CommitInfo>> {
        let table_id = parse_table_id(table_id)?;
        let mut conn = self.pool.acquire().await.map_err(backend_err)?;
        Self::row_at(&mut conn, &table_id, version).await
    }
}
