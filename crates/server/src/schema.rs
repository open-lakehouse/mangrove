//! The startup schema check.
//!
//! `serve` (and [`crate::embed::start`]) never migrate a durable backend; the
//! operator runs `migrate` first. These checks make a skipped or failed `migrate`
//! a startup error instead of a stream of request-time SQL errors: the database
//! must hold exactly the migrations embedded in this build (none pending, none
//! dirty, none modified, none unknown).
//!
//! The checks only read. A database without the `_sqlx_migrations` ledger has
//! nothing applied, so every migration is pending.

use sqlx::migrate::{AppliedMigration, Migrator};
use unitycatalog_common::{Error, Result};

/// Fail unless the Postgres database at `pool` holds exactly `migrator`'s
/// migrations.
#[cfg(feature = "postgres")]
pub(crate) async fn ensure_current_postgres(
    migrator: &Migrator,
    pool: &sqlx::PgPool,
) -> Result<()> {
    use sqlx::migrate::Migrate as _;

    let mut conn = pool.acquire().await.map_err(db)?;
    let ledger: Option<String> = sqlx::query_scalar("SELECT to_regclass('_sqlx_migrations')::text")
        .fetch_one(&mut *conn)
        .await
        .map_err(db)?;
    if ledger.is_none() {
        return check_applied(migrator, None, &[]);
    }
    let dirty = conn.dirty_version().await.map_err(migrate)?;
    let applied = conn.list_applied_migrations().await.map_err(migrate)?;
    check_applied(migrator, dirty, &applied)
}

/// Fail unless the SQLite database at `pool` holds exactly `migrator`'s
/// migrations.
#[cfg(feature = "sqlite")]
pub(crate) async fn ensure_current_sqlite(
    migrator: &Migrator,
    pool: &sqlx::SqlitePool,
) -> Result<()> {
    use sqlx::migrate::Migrate as _;

    let mut conn = pool.acquire().await.map_err(db)?;
    let ledger: Option<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name = '_sqlx_migrations'",
    )
    .fetch_optional(&mut *conn)
    .await
    .map_err(db)?;
    if ledger.is_none() {
        return check_applied(migrator, None, &[]);
    }
    let dirty = conn.dirty_version().await.map_err(migrate)?;
    let applied = conn.list_applied_migrations().await.map_err(migrate)?;
    check_applied(migrator, dirty, &applied)
}

/// Fail unless `applied` is exactly `migrator`'s up migrations and nothing is
/// `dirty` (a `migrate` that failed part-way).
fn check_applied(
    migrator: &Migrator,
    dirty: Option<i64>,
    applied: &[AppliedMigration],
) -> Result<()> {
    if let Some(version) = dirty {
        return Err(Error::Generic(format!(
            "database migration {version} is dirty (a `migrate` failed part-way); \
             repair it and re-run `migrate`"
        )));
    }
    let expected: Vec<_> = migrator
        .iter()
        .filter(|m| !m.migration_type.is_down_migration())
        .collect();
    if let Some(unknown) = applied
        .iter()
        .find(|a| !expected.iter().any(|m| m.version == a.version))
    {
        return Err(Error::Generic(format!(
            "database has migration {} applied, unknown to this build (is the database newer?)",
            unknown.version
        )));
    }
    let mut pending = Vec::new();
    for migration in &expected {
        match applied.iter().find(|a| a.version == migration.version) {
            Some(a) if a.checksum != migration.checksum => {
                return Err(Error::Generic(format!(
                    "database migration {} ({}) was modified after it was applied",
                    migration.version, migration.description
                )));
            }
            Some(_) => {}
            None => pending.push(migration.version),
        }
    }
    if !pending.is_empty() {
        return Err(Error::Generic(format!(
            "database schema is behind: {} migration(s) pending ({pending:?}); run `migrate` first",
            pending.len()
        )));
    }
    Ok(())
}

fn db(e: sqlx::Error) -> Error {
    Error::Generic(format!("checking the database schema: {e}"))
}

fn migrate(e: sqlx::migrate::MigrateError) -> Error {
    Error::Generic(format!("checking the database schema: {e}"))
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use sqlx::migrate::{Migration, MigrationType};

    use super::*;

    fn migrator() -> Migrator {
        let migration = |version, sql: &'static str| {
            Migration::new(
                version,
                Cow::Borrowed("test"),
                MigrationType::Simple,
                Cow::Borrowed(sql),
                false,
            )
        };
        Migrator {
            migrations: Cow::Owned(vec![
                migration(1, "CREATE TABLE a (x INT)"),
                migration(2, "CREATE TABLE b (x INT)"),
            ]),
            ..Migrator::DEFAULT
        }
    }

    fn applied(migrator: &Migrator, versions: &[i64]) -> Vec<AppliedMigration> {
        migrator
            .iter()
            .filter(|m| versions.contains(&m.version))
            .map(|m| AppliedMigration {
                version: m.version,
                checksum: m.checksum.clone(),
            })
            .collect()
    }

    #[test]
    fn current_schema_passes() {
        let m = migrator();
        check_applied(&m, None, &applied(&m, &[1, 2])).unwrap();
    }

    #[test]
    fn pending_migrations_fail() {
        let m = migrator();
        let err = check_applied(&m, None, &applied(&m, &[1])).unwrap_err();
        assert!(
            err.to_string().contains("1 migration(s) pending ([2])"),
            "{err}"
        );
        let err = check_applied(&m, None, &[]).unwrap_err();
        assert!(err.to_string().contains("2 migration(s) pending"), "{err}");
    }

    #[test]
    fn dirty_migration_fails() {
        let m = migrator();
        let err = check_applied(&m, Some(2), &applied(&m, &[1, 2])).unwrap_err();
        assert!(err.to_string().contains("dirty"), "{err}");
    }

    #[test]
    fn modified_migration_fails() {
        let m = migrator();
        let mut rows = applied(&m, &[1, 2]);
        rows[1].checksum = Cow::Owned(vec![0; 4]);
        let err = check_applied(&m, None, &rows).unwrap_err();
        assert!(err.to_string().contains("modified"), "{err}");
    }

    #[test]
    fn unknown_migration_fails() {
        let m = migrator();
        let mut rows = applied(&m, &[1, 2]);
        rows.push(AppliedMigration {
            version: 3,
            checksum: Cow::Owned(vec![]),
        });
        let err = check_applied(&m, None, &rows).unwrap_err();
        assert!(err.to_string().contains("unknown to this build"), "{err}");
    }
}
