//! Postgres-backed Delta [`CommitCoordinator`].
//!
//! Implements the same arbitration/backfill state machine as the in-memory
//! coordinator in `unitycatalog-delta-api` (a port of UC OSS `postCommitCore`),
//! but persists ratified commits in the `delta_commits` table. The unique
//! constraint on `(table_id, commit_version)` is the real first-writer-wins
//! arbiter: each `commit` runs in a transaction, and a racing insert for the same
//! version fails with a unique violation that maps to
//! [`CommitError::VersionConflict`].
//!
//! See [`unitycatalog_delta_api::coordinator`] for the invariants
//! (never-delete-highest backfill marker, unbackfilled cap, field validation,
//! `latest_table_version` sentinels).

use unitycatalog_delta_api::coordinator::{
    CommitCoordinator, CommitError, CommitInfo, CommitResult, DEFAULT_MAX_UNBACKFILLED_COMMITS,
    validate_commit_info,
};
use uuid::Uuid;

use crate::GraphStore;

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

/// Map a sqlx error to a [`CommitError`], translating a unique violation on the
/// `(table_id, commit_version)` constraint into a version conflict.
fn map_sqlx_err(e: sqlx::Error) -> CommitError {
    if let sqlx::Error::Database(db_err) = &e
        && db_err
            .try_downcast_ref::<sqlx::postgres::PgDatabaseError>()
            .is_some_and(|pg| pg.code() == "23505")
    {
        return CommitError::VersionConflict(
            "commit version was already accepted by another writer".to_string(),
        );
    }
    CommitError::Backend(e.to_string())
}

/// The (first, last) ratified commit versions for a table, if any rows exist.
struct VersionBounds {
    first: i64,
    last: i64,
}

impl GraphStore {
    /// Read the min/max ratified commit version for a table within a transaction.
    async fn commit_bounds(
        &self,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        table_id: Uuid,
    ) -> CommitResult<Option<VersionBounds>> {
        let row = sqlx::query!(
            r#"SELECT MIN(commit_version) AS "first", MAX(commit_version) AS "last"
               FROM delta_commits WHERE table_id = $1"#,
            table_id
        )
        .fetch_one(&mut **txn)
        .await
        .map_err(map_sqlx_err)?;

        match (row.first, row.last) {
            (Some(first), Some(last)) => Ok(Some(VersionBounds { first, last })),
            _ => Ok(None),
        }
    }

    /// Insert a ratified commit. A unique violation surfaces as a version conflict.
    async fn insert_commit(
        &self,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        table_id: Uuid,
        info: &CommitInfo,
    ) -> CommitResult<()> {
        sqlx::query!(
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
        .await
        .map_err(map_sqlx_err)?;
        Ok(())
    }

    /// Backfill commits up to `up_to`, preserving the highest-version row.
    ///
    /// Port of UC OSS `backfillCommits`: deletes rows with version
    /// `<= min(up_to, last - 1)`; if `up_to >= last`, the last row is kept and
    /// marked as the backfilled-latest marker instead of being deleted.
    async fn backfill(
        &self,
        txn: &mut sqlx::Transaction<'_, sqlx::Postgres>,
        table_id: Uuid,
        last: i64,
        up_to: i64,
    ) -> CommitResult<()> {
        let delete_up_to = up_to.min(last - 1);
        sqlx::query!(
            "DELETE FROM delta_commits WHERE table_id = $1 AND commit_version <= $2",
            table_id,
            delete_up_to
        )
        .execute(&mut **txn)
        .await
        .map_err(map_sqlx_err)?;
        if up_to >= last {
            sqlx::query!(
                "UPDATE delta_commits SET is_backfilled_latest = true \
                 WHERE table_id = $1 AND commit_version = $2",
                table_id,
                last
            )
            .execute(&mut **txn)
            .await
            .map_err(map_sqlx_err)?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl CommitCoordinator for GraphStore {
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
        let table_id = parse_table_id(table_id)?;

        let mut txn = self.pool.begin().await.map_err(map_sqlx_err)?;
        let bounds = self.commit_bounds(&mut txn, table_id).await?;

        match (bounds, commit_info) {
            // No commits yet.
            (None, Some(info)) => {
                // Onboarding commit: accept the first posted commit as-is.
                self.insert_commit(&mut txn, table_id, &info).await?;
            }
            (None, None) => {
                return Err(CommitError::InvalidArgument(
                    "cannot backfill a table with no commits".to_string(),
                ));
            }

            // Backfill-only notification.
            (Some(b), None) => {
                let lbv = latest_backfilled_version.expect("checked above");
                if lbv > b.last {
                    return Err(CommitError::InvalidArgument(format!(
                        "latest_backfilled_version {lbv} is greater than the latest commit {}",
                        b.last
                    )));
                }
                self.backfill(&mut txn, table_id, b.last, lbv).await?;
            }

            // Normal commit to an existing table.
            (Some(b), Some(info)) => {
                let version = info.version;
                if version <= b.last {
                    return Err(CommitError::VersionConflict(format!(
                        "commit version {version} was already accepted; current table version is {}",
                        b.last
                    )));
                }
                if version > b.last + 1 {
                    return Err(CommitError::InvalidArgument(format!(
                        "commit version must be the next version after {}, but got {version}",
                        b.last
                    )));
                }
                if let Some(lbv) = latest_backfilled_version
                    && lbv > b.last
                {
                    return Err(CommitError::InvalidArgument(format!(
                        "latest_backfilled_version {lbv} is greater than the latest commit {}",
                        b.last
                    )));
                }

                // Enforce the unbackfilled-commit cap. The effective backfilled
                // watermark accounts for a backfill piggy-backed on this request.
                let from_state = b.first - 1;
                let eff_backfill = match latest_backfilled_version {
                    Some(lbv) => from_state.max(lbv),
                    None => from_state,
                };
                let expected_count = version - (eff_backfill + 1) + 1;
                if expected_count > DEFAULT_MAX_UNBACKFILLED_COMMITS {
                    return Err(CommitError::ResourceExhausted(format!(
                        "max number of unbackfilled commits per table reached: {} (limit {})",
                        expected_count, DEFAULT_MAX_UNBACKFILLED_COMMITS
                    )));
                }

                self.insert_commit(&mut txn, table_id, &info).await?;
                if let Some(lbv) = latest_backfilled_version {
                    self.backfill(&mut txn, table_id, b.last, lbv).await?;
                }
            }
        }

        txn.commit().await.map_err(map_sqlx_err)?;
        Ok(())
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
        let table_id = parse_table_id(table_id)?;

        let mut conn = self.pool.acquire().await.map_err(map_sqlx_err)?;

        // latest_table_version (0 when no commits) and the oldest retained version.
        let bounds_row = sqlx::query!(
            r#"SELECT MIN(commit_version) AS "first", MAX(commit_version) AS "last"
               FROM delta_commits WHERE table_id = $1"#,
            table_id
        )
        .fetch_one(&mut *conn)
        .await
        .map_err(map_sqlx_err)?;
        let latest_table_version = bounds_row.last.unwrap_or(0);
        let oldest = bounds_row.first.unwrap_or(0);

        // Pagination window, matching UC OSS / the in-memory backend.
        let window_start = start_version.max(oldest);
        let paginated_end = window_start + DEFAULT_MAX_UNBACKFILLED_COMMITS - 1;
        let effective_end = end_version.unwrap_or(i64::MAX).min(paginated_end);

        let rows = sqlx::query!(
            "SELECT commit_version, commit_timestamp, commit_filename, commit_filesize, \
                    commit_file_modification_timestamp \
             FROM delta_commits \
             WHERE table_id = $1 AND commit_version >= $2 AND commit_version <= $3 \
               AND is_backfilled_latest = false \
             ORDER BY commit_version ASC",
            table_id,
            window_start,
            effective_end
        )
        .fetch_all(&mut *conn)
        .await
        .map_err(map_sqlx_err)?;

        let commits = rows
            .into_iter()
            .map(|row| CommitInfo {
                version: row.commit_version,
                timestamp: dt_to_millis(row.commit_timestamp),
                file_name: row.commit_filename,
                file_size: row.commit_filesize,
                file_modification_timestamp: dt_to_millis(row.commit_file_modification_timestamp),
            })
            .collect::<Vec<_>>();

        if effective_end < latest_table_version && effective_end == paginated_end {
            tracing::debug!(
                %table_id,
                effective_end,
                latest_table_version,
                "get_commits truncated to pagination window"
            );
        }

        Ok((commits, latest_table_version))
    }
}
