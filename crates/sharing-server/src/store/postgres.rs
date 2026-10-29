//! A Postgres [`ShareStore`] (sqlx), with embedded migrations.
//!
//! Migrations run only through [`migrate`] (the `migrate` subcommand), never on
//! startup; [`ensure_schema_current`] makes `serve` refuse to start on a schema
//! that is behind, ahead of, or diverged from this binary. Both hold a session
//! advisory lock, so on a transaction-pooled endpoint (PgBouncer, Neon's pooled
//! URL) point them at the direct, unpooled connection string.
//!
//! The migrator records applied migrations in `_sqlx_migrations`, so give the
//! sharing server its own database (or Neon branch) rather than sharing one with
//! another sqlx-migrated service.

use async_trait::async_trait;
use sqlx::postgres::{PgPool, PgPoolOptions, PgRow};
use sqlx::{Row, migrate::Migrator};

use super::{
    ExternalLocationRecord, RecipientRecord, SealedCredential, ShareObject, ShareRecord,
    ShareStore, TokenRecord, now_millis,
};
use crate::{Error, Result};

/// The embedded migrations.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations");

/// Connect a pool to `url`.
pub async fn connect(url: &str, max_connections: u32) -> Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(url)
        .await
        .map_err(|e| Error::generic(format!("connecting to postgres: {e}")))
}

/// Apply all pending migrations.
pub async fn migrate(pool: &PgPool) -> Result<()> {
    MIGRATOR
        .run(pool)
        .await
        .map_err(|e| Error::generic(format!("running migrations: {e}")))
}

/// Fail unless the database schema is exactly what this binary expects.
pub async fn ensure_schema_current(pool: &PgPool) -> Result<()> {
    let applied: Vec<(i64, Vec<u8>, bool)> = match sqlx::query_as(
        "SELECT version, checksum, success FROM _sqlx_migrations ORDER BY version",
    )
    .fetch_all(pool)
    .await
    {
        Ok(rows) => rows,
        Err(sqlx::Error::Database(e)) if e.code().as_deref() == Some("42P01") => Vec::new(),
        Err(e) => return Err(db(e)),
    };
    if let Some((version, _, _)) = applied.iter().find(|(_, _, ok)| !ok) {
        return Err(Error::generic(format!(
            "migration {version} is dirty (failed part-way); repair it and re-run `migrate`"
        )));
    }
    let expected: Vec<_> = MIGRATOR
        .iter()
        .filter(|m| !m.migration_type.is_down_migration())
        .collect();
    for migration in &expected {
        match applied.iter().find(|(v, _, _)| *v == migration.version) {
            None => {
                return Err(Error::generic(format!(
                    "migration {} ({}) is pending; run `sharing-server migrate`",
                    migration.version, migration.description
                )));
            }
            Some((_, checksum, _)) if checksum.as_slice() != &*migration.checksum => {
                return Err(Error::generic(format!(
                    "migration {} was modified after it was applied",
                    migration.version
                )));
            }
            Some(_) => {}
        }
    }
    if let Some((version, _, _)) = applied
        .iter()
        .find(|(v, _, _)| !expected.iter().any(|m| m.version == *v))
    {
        return Err(Error::generic(format!(
            "the database has migration {version}, unknown to this binary (is it newer?)"
        )));
    }
    Ok(())
}

/// A Postgres [`ShareStore`].
#[derive(Clone)]
pub struct PostgresStore {
    pool: PgPool,
}

impl PostgresStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}

/// Map a database error, turning unique violations into "already exists".
fn db(e: sqlx::Error) -> Error {
    match &e {
        sqlx::Error::RowNotFound => Error::NotFound,
        sqlx::Error::Database(d) if d.code().as_deref() == Some("23505") => {
            Error::invalid_argument("the resource already exists")
        }
        sqlx::Error::Database(d) if d.code().as_deref() == Some("23503") => {
            Error::invalid_argument(format!(
                "a referenced resource does not exist: {}",
                d.message()
            ))
        }
        _ => Error::generic(format!("database error: {e}")),
    }
}

fn affected(rows: u64) -> Result<()> {
    if rows == 0 {
        Err(Error::NotFound)
    } else {
        Ok(())
    }
}

const SHARE_COLUMNS: &str = "id, name, comment, objects, created_at, updated_at";

fn share(row: PgRow) -> Result<ShareRecord> {
    let objects: serde_json::Value = row.try_get("objects").map_err(db)?;
    Ok(ShareRecord {
        id: row.try_get("id").map_err(db)?,
        name: row.try_get("name").map_err(db)?,
        comment: row.try_get("comment").map_err(db)?,
        objects: serde_json::from_value(objects)?,
        created_at: row.try_get("created_at").map_err(db)?,
        updated_at: row.try_get("updated_at").map_err(db)?,
    })
}

fn recipient(row: &PgRow) -> Result<RecipientRecord> {
    Ok(RecipientRecord {
        id: row.try_get("id").map_err(db)?,
        name: row.try_get("name").map_err(db)?,
        comment: row.try_get("comment").map_err(db)?,
        created_at: row.try_get("created_at").map_err(db)?,
    })
}

const TOKEN_COLUMNS: &str = "t.id, t.recipient_id, t.token_hash, t.sealed_token, t.activation_code, t.expires_at, t.created_at";

fn token(row: &PgRow) -> Result<TokenRecord> {
    Ok(TokenRecord {
        id: row.try_get("id").map_err(db)?,
        recipient_id: row.try_get("recipient_id").map_err(db)?,
        token_hash: row.try_get("token_hash").map_err(db)?,
        sealed_token: row.try_get("sealed_token").map_err(db)?,
        activation_code: row.try_get("activation_code").map_err(db)?,
        expires_at: row.try_get("expires_at").map_err(db)?,
        created_at: row.try_get("created_at").map_err(db)?,
    })
}

#[async_trait]
impl ShareStore for PostgresStore {
    async fn create_share(&self, name: &str, comment: Option<String>) -> Result<ShareRecord> {
        let now = now_millis();
        let row = sqlx::query(&format!(
            "INSERT INTO sharing_shares (id, name, comment, objects, created_at, updated_at) \
             VALUES ($1, $2, $3, '[]'::jsonb, $4, $4) RETURNING {SHARE_COLUMNS}"
        ))
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(name)
        .bind(comment)
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        share(row)
    }

    async fn get_share(&self, name: &str) -> Result<ShareRecord> {
        let row = sqlx::query(&format!(
            "SELECT {SHARE_COLUMNS} FROM sharing_shares WHERE name = $1"
        ))
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        share(row)
    }

    async fn list_shares(&self) -> Result<Vec<ShareRecord>> {
        sqlx::query(&format!(
            "SELECT {SHARE_COLUMNS} FROM sharing_shares ORDER BY name"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(db)?
        .into_iter()
        .map(share)
        .collect()
    }

    async fn update_share(
        &self,
        name: &str,
        objects: Vec<ShareObject>,
        comment: Option<String>,
    ) -> Result<ShareRecord> {
        let row = sqlx::query(&format!(
            "UPDATE sharing_shares SET objects = $2, comment = $3, updated_at = $4 \
             WHERE name = $1 RETURNING {SHARE_COLUMNS}"
        ))
        .bind(name)
        .bind(serde_json::to_value(&objects)?)
        .bind(comment)
        .bind(now_millis())
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        share(row)
    }

    async fn delete_share(&self, name: &str) -> Result<()> {
        let done = sqlx::query("DELETE FROM sharing_shares WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        affected(done.rows_affected())
    }

    async fn create_recipient(
        &self,
        name: &str,
        comment: Option<String>,
    ) -> Result<RecipientRecord> {
        let row = sqlx::query(
            "INSERT INTO sharing_recipients (id, name, comment, created_at) \
             VALUES ($1, $2, $3, $4) RETURNING id, name, comment, created_at",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(name)
        .bind(comment)
        .bind(now_millis())
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        recipient(&row)
    }

    async fn get_recipient(&self, name: &str) -> Result<RecipientRecord> {
        let row = sqlx::query(
            "SELECT id, name, comment, created_at FROM sharing_recipients WHERE name = $1",
        )
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        recipient(&row)
    }

    async fn list_recipients(&self) -> Result<Vec<RecipientRecord>> {
        sqlx::query("SELECT id, name, comment, created_at FROM sharing_recipients ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(db)?
            .iter()
            .map(recipient)
            .collect()
    }

    async fn delete_recipient(&self, name: &str) -> Result<()> {
        // Tokens and grants cascade.
        let done = sqlx::query("DELETE FROM sharing_recipients WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        affected(done.rows_affected())
    }

    async fn insert_token(&self, t: TokenRecord) -> Result<()> {
        sqlx::query(
            "INSERT INTO sharing_recipient_tokens \
             (id, recipient_id, token_hash, sealed_token, activation_code, expires_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(t.id)
        .bind(t.recipient_id)
        .bind(t.token_hash)
        .bind(t.sealed_token)
        .bind(t.activation_code)
        .bind(t.expires_at)
        .bind(t.created_at)
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn token_by_hash(&self, token_hash: &[u8]) -> Result<(TokenRecord, RecipientRecord)> {
        let row = sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS}, r.name, r.comment, r.created_at AS recipient_created_at \
             FROM sharing_recipient_tokens t JOIN sharing_recipients r ON r.id = t.recipient_id \
             WHERE t.token_hash = $1"
        ))
        .bind(token_hash)
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        let t = token(&row)?;
        let r = RecipientRecord {
            id: t.recipient_id.clone(),
            name: row.try_get("name").map_err(db)?,
            comment: row.try_get("comment").map_err(db)?,
            created_at: row.try_get("recipient_created_at").map_err(db)?,
        };
        Ok((t, r))
    }

    async fn token_by_activation_code(&self, code: &str) -> Result<TokenRecord> {
        let row = sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM sharing_recipient_tokens t WHERE t.activation_code = $1"
        ))
        .bind(code)
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        token(&row)
    }

    async fn mark_activated(&self, token_id: &str) -> Result<()> {
        let done = sqlx::query(
            "UPDATE sharing_recipient_tokens SET sealed_token = NULL, activation_code = NULL \
             WHERE id = $1",
        )
        .bind(token_id)
        .execute(&self.pool)
        .await
        .map_err(db)?;
        affected(done.rows_affected())
    }

    async fn tokens_for_recipient(&self, recipient_id: &str) -> Result<Vec<TokenRecord>> {
        sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM sharing_recipient_tokens t \
             WHERE t.recipient_id = $1 ORDER BY t.created_at DESC"
        ))
        .bind(recipient_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?
        .iter()
        .map(token)
        .collect()
    }

    async fn delete_tokens(&self, recipient_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM sharing_recipient_tokens WHERE recipient_id = $1")
            .bind(recipient_id)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        Ok(())
    }

    async fn sealed_tokens(&self) -> Result<Vec<TokenRecord>> {
        sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM sharing_recipient_tokens t WHERE t.sealed_token IS NOT NULL"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(db)?
        .iter()
        .map(token)
        .collect()
    }

    async fn update_sealed_token(&self, token_id: &str, sealed: Vec<u8>) -> Result<()> {
        let done =
            sqlx::query("UPDATE sharing_recipient_tokens SET sealed_token = $2 WHERE id = $1")
                .bind(token_id)
                .bind(sealed)
                .execute(&self.pool)
                .await
                .map_err(db)?;
        affected(done.rows_affected())
    }

    async fn grant(&self, share: &str, recipient: &str) -> Result<()> {
        let done = sqlx::query(
            "INSERT INTO sharing_grants (share_id, recipient_id) \
             SELECT s.id, r.id FROM sharing_shares s, sharing_recipients r \
             WHERE s.name = $1 AND r.name = $2 ON CONFLICT DO NOTHING",
        )
        .bind(share)
        .bind(recipient)
        .execute(&self.pool)
        .await
        .map_err(db)?;
        if done.rows_affected() == 0 {
            // Either already granted, or the share / recipient does not exist.
            self.get_share(share).await?;
            self.get_recipient(recipient).await?;
        }
        Ok(())
    }

    async fn revoke(&self, share: &str, recipient: &str) -> Result<()> {
        sqlx::query(
            "DELETE FROM sharing_grants g USING sharing_shares s, sharing_recipients r \
             WHERE g.share_id = s.id AND g.recipient_id = r.id AND s.name = $1 AND r.name = $2",
        )
        .bind(share)
        .bind(recipient)
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn recipients_of(&self, share: &str) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT r.name FROM sharing_grants g \
             JOIN sharing_shares s ON s.id = g.share_id \
             JOIN sharing_recipients r ON r.id = g.recipient_id \
             WHERE s.name = $1 ORDER BY r.name",
        )
        .bind(share)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?;
        rows.iter()
            .map(|row| row.try_get("name").map_err(db))
            .collect()
    }

    async fn shares_of(&self, recipient_id: &str) -> Result<Vec<ShareRecord>> {
        sqlx::query(&format!(
            "SELECT {} FROM sharing_shares s JOIN sharing_grants g ON g.share_id = s.id \
             WHERE g.recipient_id = $1 ORDER BY s.name",
            SHARE_COLUMNS
                .split(", ")
                .map(|c| format!("s.{c}"))
                .collect::<Vec<_>>()
                .join(", ")
        ))
        .bind(recipient_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?
        .into_iter()
        .map(share)
        .collect()
    }

    async fn is_granted(&self, share: &str, recipient_id: &str) -> Result<bool> {
        let row = sqlx::query(
            "SELECT EXISTS (SELECT 1 FROM sharing_grants g JOIN sharing_shares s ON s.id = g.share_id \
             WHERE s.name = $1 AND g.recipient_id = $2) AS granted",
        )
        .bind(share)
        .bind(recipient_id)
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        row.try_get("granted").map_err(db)
    }

    async fn put_credential(&self, c: SealedCredential) -> Result<()> {
        sqlx::query(
            "INSERT INTO sharing_credentials (name, sealed) VALUES ($1, $2) \
             ON CONFLICT (name) DO UPDATE SET sealed = EXCLUDED.sealed",
        )
        .bind(c.name)
        .bind(c.sealed)
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn get_credential(&self, name: &str) -> Result<SealedCredential> {
        let row = sqlx::query("SELECT name, sealed FROM sharing_credentials WHERE name = $1")
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .map_err(db)?;
        Ok(SealedCredential {
            name: row.try_get("name").map_err(db)?,
            sealed: row.try_get("sealed").map_err(db)?,
        })
    }

    async fn list_credentials(&self) -> Result<Vec<SealedCredential>> {
        sqlx::query("SELECT name, sealed FROM sharing_credentials ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(db)?
            .iter()
            .map(|row| {
                Ok(SealedCredential {
                    name: row.try_get("name").map_err(db)?,
                    sealed: row.try_get("sealed").map_err(db)?,
                })
            })
            .collect()
    }

    async fn delete_credential(&self, name: &str) -> Result<()> {
        let done = sqlx::query("DELETE FROM sharing_credentials WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        affected(done.rows_affected())
    }

    async fn put_external_location(&self, l: ExternalLocationRecord) -> Result<()> {
        sqlx::query(
            "INSERT INTO sharing_external_locations (name, url, credential_name) VALUES ($1, $2, $3) \
             ON CONFLICT (name) DO UPDATE SET url = EXCLUDED.url, credential_name = EXCLUDED.credential_name",
        )
        .bind(l.name)
        .bind(l.url)
        .bind(l.credential_name)
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(())
    }

    async fn list_external_locations(&self) -> Result<Vec<ExternalLocationRecord>> {
        sqlx::query(
            "SELECT name, url, credential_name FROM sharing_external_locations ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db)?
        .iter()
        .map(|row| {
            Ok(ExternalLocationRecord {
                name: row.try_get("name").map_err(db)?,
                url: row.try_get("url").map_err(db)?,
                credential_name: row.try_get("credential_name").map_err(db)?,
            })
        })
        .collect()
    }

    async fn delete_external_location(&self, name: &str) -> Result<()> {
        let done = sqlx::query("DELETE FROM sharing_external_locations WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        affected(done.rows_affected())
    }
}
