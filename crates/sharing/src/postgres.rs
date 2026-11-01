//! Postgres-backed [`SharingStore`](crate::SharingStore) over the `uc_shares`,
//! `uc_recipients`, `uc_recipient_tokens` and `uc_share_grants` tables.
//!
//! Every multi-statement write runs in one transaction, so it is atomic.
//!
//! # Migrations
//!
//! The tables come from [`MIGRATOR`]. It is versioned in the 03xx range and meant
//! to be merged into the host's own migrator, so the database keeps one ordered
//! `_sqlx_migrations` ledger:
//!
//! ```ignore
//! let host = sqlx::migrate!("./migrations");
//! let mut migrations: Vec<_> = host.migrations.iter().cloned().collect();
//! migrations.extend(unitycatalog_sharing::postgres::MIGRATOR.iter().cloned());
//! migrations.sort_by_key(|m| m.version);
//! ```
//!
//! Hosts number their own migrations outside 03xx.

use async_trait::async_trait;
use sqlx::Row;
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgPool, PgRow};

use crate::sql::{affected, db};
use crate::{
    Error, RecipientRecord, RecipientStore, RecipientUpdate, Result, ShareObject, ShareReader,
    ShareRecord, ShareStore, TokenReader, TokenRecord, now_millis,
};

/// The migrations creating the sharing tables (version 0301), for the host to
/// merge into its own migrator.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations/postgres");

/// Postgres-backed [`SharingStore`](crate::SharingStore).
///
/// Holds its own clone of the shared [`PgPool`].
#[derive(Clone, Debug)]
pub struct PgSharingStore {
    pool: PgPool,
}

impl PgSharingStore {
    /// A store over `pool`.
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// The pool the store runs on.
    pub fn pool(&self) -> &PgPool {
        &self.pool
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

const TOKEN_COLUMNS: &str = "t.id, t.recipient_id, t.token_hash, t.sealed_token, t.activation_code_hash, t.expires_at, t.created_at";

fn token(row: &PgRow) -> Result<TokenRecord> {
    Ok(TokenRecord {
        id: row.try_get("id").map_err(db)?,
        recipient_id: row.try_get("recipient_id").map_err(db)?,
        token_hash: row.try_get("token_hash").map_err(db)?,
        sealed_token: row.try_get("sealed_token").map_err(db)?,
        activation_code_hash: row.try_get("activation_code_hash").map_err(db)?,
        expires_at: row.try_get("expires_at").map_err(db)?,
        created_at: row.try_get("created_at").map_err(db)?,
    })
}

#[async_trait]
impl ShareReader for PgSharingStore {
    async fn get_share(&self, name: &str) -> Result<ShareRecord> {
        let row = sqlx::query(&format!(
            "SELECT {SHARE_COLUMNS} FROM uc_shares WHERE name = $1"
        ))
        .bind(name)
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        share(row)
    }

    async fn shares_of(&self, recipient_id: &str) -> Result<Vec<ShareRecord>> {
        sqlx::query(&format!(
            "SELECT {} FROM uc_shares s JOIN uc_share_grants g ON g.share_id = s.id \
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
            "SELECT EXISTS (SELECT 1 FROM uc_share_grants g JOIN uc_shares s ON s.id = g.share_id \
             WHERE s.name = $1 AND g.recipient_id = $2) AS granted",
        )
        .bind(share)
        .bind(recipient_id)
        .fetch_one(&self.pool)
        .await
        .map_err(db)?;
        row.try_get("granted").map_err(db)
    }
}

#[async_trait]
impl ShareStore for PgSharingStore {
    async fn create_share(&self, name: &str, comment: Option<String>) -> Result<ShareRecord> {
        let now = now_millis();
        let row = sqlx::query(&format!(
            "INSERT INTO uc_shares (id, name, comment, objects, created_at, updated_at) \
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

    async fn list_shares(&self) -> Result<Vec<ShareRecord>> {
        sqlx::query(&format!(
            "SELECT {SHARE_COLUMNS} FROM uc_shares ORDER BY name"
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
            "UPDATE uc_shares SET objects = $2, comment = $3, updated_at = $4 \
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
        let done = sqlx::query("DELETE FROM uc_shares WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        affected(done.rows_affected())
    }

    async fn grant(&self, share: &str, recipient: &str) -> Result<()> {
        let done = sqlx::query(
            "INSERT INTO uc_share_grants (share_id, recipient_id) \
             SELECT s.id, r.id FROM uc_shares s, uc_recipients r \
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
            "DELETE FROM uc_share_grants g USING uc_shares s, uc_recipients r \
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
            "SELECT r.name FROM uc_share_grants g \
             JOIN uc_shares s ON s.id = g.share_id \
             JOIN uc_recipients r ON r.id = g.recipient_id \
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
}

#[async_trait]
impl TokenReader for PgSharingStore {
    async fn token_by_hash(&self, token_hash: &[u8]) -> Result<(TokenRecord, RecipientRecord)> {
        let row = sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS}, r.name, r.comment, r.created_at AS recipient_created_at \
             FROM uc_recipient_tokens t JOIN uc_recipients r ON r.id = t.recipient_id \
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
}

// Every multi-statement write runs in one transaction, so it is atomic.
#[async_trait]
impl RecipientStore for PgSharingStore {
    async fn create_recipient(&self, r: RecipientRecord, t: TokenRecord) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db)?;
        sqlx::query(
            "INSERT INTO uc_recipients (id, name, comment, created_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(&r.id)
        .bind(&r.name)
        .bind(&r.comment)
        .bind(r.created_at)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        insert_token(&mut tx, &t).await?;
        tx.commit().await.map_err(db)
    }

    async fn get_recipient(&self, name: &str) -> Result<RecipientRecord> {
        let row =
            sqlx::query("SELECT id, name, comment, created_at FROM uc_recipients WHERE name = $1")
                .bind(name)
                .fetch_one(&self.pool)
                .await
                .map_err(db)?;
        recipient(&row)
    }

    async fn list_recipients(&self) -> Result<Vec<RecipientRecord>> {
        sqlx::query("SELECT id, name, comment, created_at FROM uc_recipients ORDER BY name")
            .fetch_all(&self.pool)
            .await
            .map_err(db)?
            .iter()
            .map(recipient)
            .collect()
    }

    async fn update_recipient(
        &self,
        name: &str,
        update: RecipientUpdate,
    ) -> Result<RecipientRecord> {
        let mut tx = self.pool.begin().await.map_err(db)?;
        let row = sqlx::query(
            "UPDATE uc_recipients SET comment = coalesce($2, comment) WHERE name = $1 \
             RETURNING id, name, comment, created_at",
        )
        .bind(name)
        .bind(update.comment)
        .fetch_one(&mut *tx)
        .await
        .map_err(db)?;
        let record = recipient(&row)?;
        if let Some(expires_at) = update.expires_at {
            sqlx::query(
                "UPDATE uc_recipient_tokens SET expires_at = $2 WHERE id = \
                 (SELECT id FROM uc_recipient_tokens WHERE recipient_id = $1 \
                  ORDER BY created_at DESC, id DESC LIMIT 1)",
            )
            .bind(&record.id)
            .bind(expires_at)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        }
        tx.commit().await.map_err(db)?;
        Ok(record)
    }

    async fn delete_recipient(&self, name: &str) -> Result<()> {
        // Tokens and grants cascade.
        let done = sqlx::query("DELETE FROM uc_recipients WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        affected(done.rows_affected())
    }

    async fn tokens_for_recipient(&self, recipient_id: &str) -> Result<Vec<TokenRecord>> {
        sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM uc_recipient_tokens t \
             WHERE t.recipient_id = $1 ORDER BY t.created_at DESC, t.id DESC"
        ))
        .bind(recipient_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db)?
        .iter()
        .map(token)
        .collect()
    }

    async fn rotate_token(&self, t: TokenRecord, existing_expire_at: Option<i64>) -> Result<()> {
        let mut tx = self.pool.begin().await.map_err(db)?;
        if let Some(at) = existing_expire_at {
            sqlx::query(
                "UPDATE uc_recipient_tokens SET expires_at = $2 \
                 WHERE recipient_id = $1 AND (expires_at IS NULL OR expires_at > $2)",
            )
            .bind(&t.recipient_id)
            .bind(at)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
            sqlx::query(
                "DELETE FROM uc_recipient_tokens WHERE recipient_id = $1 AND expires_at <= $2",
            )
            .bind(&t.recipient_id)
            .bind(t.created_at)
            .execute(&mut *tx)
            .await
            .map_err(db)?;
        } else {
            sqlx::query("DELETE FROM uc_recipient_tokens WHERE recipient_id = $1")
                .bind(&t.recipient_id)
                .execute(&mut *tx)
                .await
                .map_err(db)?;
        }
        insert_token(&mut tx, &t).await?;
        tx.commit().await.map_err(db)
    }

    async fn redeem_activation(&self, code_hash: &[u8], now: i64) -> Result<TokenRecord> {
        let mut tx = self.pool.begin().await.map_err(db)?;
        let row = sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM uc_recipient_tokens t \
             WHERE t.activation_code_hash = $1 AND (t.expires_at IS NULL OR t.expires_at > $2)"
        ))
        .bind(code_hash)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db)?
        .ok_or(Error::NotFound)?;
        let redeemed = token(&row)?;
        // Conditional on the code still being there: of concurrent redemptions,
        // only the first to update matches.
        let done = sqlx::query(
            "UPDATE uc_recipient_tokens SET sealed_token = NULL, activation_code_hash = NULL \
             WHERE id = $1 AND activation_code_hash = $2",
        )
        .bind(&redeemed.id)
        .bind(code_hash)
        .execute(&mut *tx)
        .await
        .map_err(db)?;
        affected(done.rows_affected())?;
        tx.commit().await.map_err(db)?;
        Ok(redeemed)
    }

    async fn sealed_tokens(&self) -> Result<Vec<TokenRecord>> {
        sqlx::query(&format!(
            "SELECT {TOKEN_COLUMNS} FROM uc_recipient_tokens t WHERE t.sealed_token IS NOT NULL"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(db)?
        .iter()
        .map(token)
        .collect()
    }

    async fn reseal_token(&self, token_id: &str, current: &[u8], sealed: Vec<u8>) -> Result<bool> {
        let done = sqlx::query(
            "UPDATE uc_recipient_tokens SET sealed_token = $3 WHERE id = $1 AND sealed_token = $2",
        )
        .bind(token_id)
        .bind(current)
        .bind(sealed)
        .execute(&self.pool)
        .await
        .map_err(db)?;
        Ok(done.rows_affected() == 1)
    }
}

async fn insert_token(conn: &mut sqlx::PgConnection, t: &TokenRecord) -> Result<()> {
    sqlx::query(
        "INSERT INTO uc_recipient_tokens (id, recipient_id, token_hash, sealed_token, \
         activation_code_hash, expires_at, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&t.id)
    .bind(&t.recipient_id)
    .bind(&t.token_hash)
    .bind(&t.sealed_token)
    .bind(&t.activation_code_hash)
    .bind(t.expires_at)
    .bind(t.created_at)
    .execute(conn)
    .await
    .map_err(db)?;
    Ok(())
}
