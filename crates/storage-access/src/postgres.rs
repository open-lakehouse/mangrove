//! Postgres-backed [`StorageAccessStore`] over the `uc_credentials` and
//! `uc_external_locations` tables.
//!
//! Every write that checks a rule before writing (overlap, credential existence,
//! credential in use) runs in a transaction holding the bundle's advisory lock,
//! so concurrent checks serialize. Locations are matched on their indexed
//! `url_key` ([`LocationKey`]): the locations covering a path are an `= ANY`
//! lookup over the path's ancestors, the locations under it a range scan.
//!
//! # Migrations
//!
//! The tables come from [`MIGRATOR`]. It is versioned in the 02xx range and meant
//! to be merged into the host's own migrator, so the database keeps one ordered
//! `_sqlx_migrations` ledger:
//!
//! ```ignore
//! let host = sqlx::migrate!("./migrations");
//! let mut migrations: Vec<_> = host.migrations.iter().cloned().collect();
//! migrations.extend(unitycatalog_storage_access::postgres::MIGRATOR.iter().cloned());
//! migrations.sort_by_key(|m| m.version);
//! ```
//!
//! Hosts number their own migrations outside 02xx.

use crate::rules::{credential_in_use_error, new_id, now_millis, overlap_error};
use crate::sql::{db, paginate, rewrap};
use crate::{
    CredentialSecrets, CredentialUpdate, Error, ExternalLocationUpdate, Governed, LocationKey,
    Page, Result, StorageAccessStore,
};
use async_trait::async_trait;
use olai_storage_credentials::StorageLocation;
use olai_store::encryption::EnvelopeEncryptor;
use sqlx::migrate::Migrator;
use sqlx::{PgConnection, PgPool, Postgres, QueryBuilder};
use unitycatalog_common::models::credentials::v1::{Credential, Purpose};
use unitycatalog_common::models::external_locations::v1::ExternalLocation;

/// The migrations creating the `uc_credentials` and `uc_external_locations`
/// tables (version 0201), for the host to merge into its own migrator.
pub static MIGRATOR: Migrator = sqlx::migrate!("./migrations/postgres");

/// Postgres-backed [`StorageAccessStore`].
///
/// Holds its own clone of the shared [`PgPool`]; credential secrets are sealed
/// with `encryptor`, bound to the credential id.
#[derive(Clone, Debug)]
pub struct PgStorageAccess {
    pool: PgPool,
    encryptor: EnvelopeEncryptor,
}

impl PgStorageAccess {
    /// A store over `pool`, sealing secrets with `encryptor`.
    pub fn new(pool: PgPool, encryptor: EnvelopeEncryptor) -> Self {
        Self { pool, encryptor }
    }
}

/// The advisory lock serializing the bundle's checked writes ("UCSTACCS").
const ADVISORY_LOCK: i64 = 0x5543_5354_4143_4353;

// Ids are `uuid` columns, read back as their hyphenated text.
const CREDENTIAL_COLUMNS: &str = "c.id::text AS id, c.name, c.purpose, c.read_only, \
    c.used_for_managed_storage, c.comment, c.owner, c.created_at, c.created_by, \
    c.updated_at, c.updated_by";

const LOCATION_COLUMNS: &str = "l.id::text AS id, l.name, l.url, \
    l.credential_id::text AS credential_id, c.name AS credential_name, l.read_only, \
    l.comment, l.owner, l.created_at, l.created_by, l.updated_at, l.updated_by";

#[derive(sqlx::FromRow)]
struct CredentialRow {
    id: String,
    name: String,
    purpose: i32,
    read_only: bool,
    used_for_managed_storage: bool,
    comment: Option<String>,
    owner: Option<String>,
    created_at: i64,
    created_by: Option<String>,
    updated_at: i64,
    updated_by: Option<String>,
}

impl From<CredentialRow> for Credential {
    fn from(row: CredentialRow) -> Self {
        Credential {
            full_name: row.name.clone(),
            name: row.name,
            id: Some(row.id),
            purpose: row.purpose.into(),
            read_only: row.read_only,
            used_for_managed_storage: row.used_for_managed_storage,
            comment: row.comment,
            owner: row.owner,
            created_at: Some(row.created_at),
            created_by: row.created_by,
            updated_at: Some(row.updated_at),
            updated_by: row.updated_by,
            ..Default::default()
        }
    }
}

#[derive(sqlx::FromRow)]
struct SealedCredentialRow {
    #[sqlx(flatten)]
    credential: CredentialRow,
    sealed: Vec<u8>,
}

#[derive(sqlx::FromRow)]
struct LocationRow {
    id: String,
    name: String,
    url: String,
    credential_id: String,
    credential_name: String,
    read_only: bool,
    comment: Option<String>,
    owner: Option<String>,
    created_at: i64,
    created_by: Option<String>,
    updated_at: i64,
    updated_by: Option<String>,
}

impl From<LocationRow> for ExternalLocation {
    fn from(row: LocationRow) -> Self {
        ExternalLocation {
            name: row.name,
            url: row.url,
            credential_name: row.credential_name,
            credential_id: row.credential_id,
            read_only: row.read_only,
            comment: row.comment,
            owner: row.owner,
            created_at: Some(row.created_at),
            created_by: row.created_by,
            updated_at: Some(row.updated_at),
            updated_by: row.updated_by,
            external_location_id: Some(row.id),
            ..Default::default()
        }
    }
}

/// Opens a transaction holding the bundle's advisory lock, so its rule checks and
/// writes are atomic against other writers. Dropping it uncommitted rolls back.
async fn write_txn(pool: &PgPool) -> Result<sqlx::Transaction<'static, Postgres>> {
    let mut txn = pool.begin().await.map_err(db)?;
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(ADVISORY_LOCK)
        .execute(&mut *txn)
        .await
        .map_err(db)?;
    Ok(txn)
}

async fn credential_by_name(conn: &mut PgConnection, name: &str) -> Result<CredentialRow> {
    sqlx::query_as(&format!(
        "SELECT {CREDENTIAL_COLUMNS} FROM uc_credentials c WHERE c.name = $1"
    ))
    .bind(name)
    .fetch_optional(conn)
    .await
    .map_err(db)?
    .ok_or(Error::NotFound)
}

async fn location_by_name(conn: &mut PgConnection, name: &str) -> Result<LocationRow> {
    sqlx::query_as(&format!(
        "SELECT {LOCATION_COLUMNS} FROM uc_external_locations l \
         JOIN uc_credentials c ON c.id = l.credential_id WHERE l.name = $1"
    ))
    .bind(name)
    .fetch_optional(conn)
    .await
    .map_err(db)?
    .ok_or(Error::NotFound)
}

/// The location whose key is among `key`'s ancestors: the one covering it.
async fn covering(conn: &mut PgConnection, key: &LocationKey) -> Result<Option<LocationRow>> {
    let ancestors: Vec<String> = key
        .ancestors()
        .iter()
        .map(|k| k.as_str().to_owned())
        .collect();
    sqlx::query_as(&format!(
        "SELECT {LOCATION_COLUMNS} FROM uc_external_locations l \
         JOIN uc_credentials c ON c.id = l.credential_id WHERE l.url_key = ANY($1) \
         ORDER BY length(l.url_key) DESC LIMIT 1"
    ))
    .bind(ancestors)
    .fetch_optional(conn)
    .await
    .map_err(db)
}

/// Refuses `url` (with key `key`) if it overlaps a location other than `except_id`.
async fn check_no_overlap(
    conn: &mut PgConnection,
    url: &str,
    key: &LocationKey,
    except_id: Option<&str>,
) -> Result<()> {
    let overlapping = match covering(conn, key).await? {
        Some(row) if Some(row.id.as_str()) != except_id => Some((row.name, row.url)),
        _ => {
            let (lower, upper) = key.descendants();
            sqlx::query_as::<_, (String, String)>(
                "SELECT name, url FROM uc_external_locations \
                 WHERE url_key >= $1 AND url_key < $2 AND id IS DISTINCT FROM $3::uuid \
                 LIMIT 1",
            )
            .bind(lower)
            .bind(upper)
            .bind(except_id)
            .fetch_optional(conn)
            .await
            .map_err(db)?
        }
    };
    match overlapping {
        Some((name, existing)) => Err(overlap_error(url, &name, &existing)),
        None => Ok(()),
    }
}

impl PgStorageAccess {
    async fn seal(&self, id: &str, secrets: &CredentialSecrets) -> Result<Vec<u8>> {
        crate::sql::seal(&self.encryptor, id, secrets).await
    }

    async fn open(&self, id: &str, sealed: &[u8]) -> Result<CredentialSecrets> {
        crate::sql::open(&self.encryptor, id, sealed).await
    }
}

#[async_trait]
impl StorageAccessStore for PgStorageAccess {
    async fn create_credential(&self, mut credential: Credential) -> Result<Credential> {
        let secrets = CredentialSecrets::take(&mut credential);
        let id = new_id();
        let sealed = self.seal(&id, &secrets).await?;
        let now = now_millis();
        let mut conn = self.pool.acquire().await.map_err(db)?;
        sqlx::query(
            "INSERT INTO uc_credentials (id, name, purpose, read_only, \
             used_for_managed_storage, comment, owner, sealed, created_at, created_by, \
             updated_at, updated_by) \
             VALUES ($1::uuid, $2, $3, $4, $5, $6, $7, $8, $9, $10, $9, $10)",
        )
        .bind(&id)
        .bind(&credential.name)
        .bind(credential.purpose.to_i32())
        .bind(credential.read_only)
        .bind(credential.used_for_managed_storage)
        .bind(&credential.comment)
        .bind(&credential.owner)
        .bind(&sealed)
        .bind(now)
        .bind(&credential.created_by)
        .execute(&mut *conn)
        .await
        .map_err(db)?;
        Ok(credential_by_name(&mut conn, &credential.name)
            .await?
            .into())
    }

    async fn get_credential(&self, name: &str) -> Result<Credential> {
        let mut conn = self.pool.acquire().await.map_err(db)?;
        Ok(credential_by_name(&mut conn, name).await?.into())
    }

    async fn list_credentials(
        &self,
        purpose: Option<Purpose>,
        page: Page,
    ) -> Result<(Vec<Credential>, Option<String>)> {
        let mut query = QueryBuilder::<Postgres>::new(format!(
            "SELECT {CREDENTIAL_COLUMNS} FROM uc_credentials c WHERE true"
        ));
        if let Some(purpose) = purpose {
            query.push(" AND c.purpose = ").push_bind(purpose as i32);
        }
        if let Some(token) = page.token {
            query.push(" AND c.name > ").push_bind(token);
        }
        query.push(" ORDER BY c.name");
        if let Some(max) = page.max_results {
            query.push(" LIMIT ").push_bind(max as i64 + 1);
        }
        let rows: Vec<CredentialRow> = query
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(db)?;
        Ok(paginate(rows, page.max_results, |r| r.name.clone()))
    }

    async fn update_credential(&self, name: &str, update: CredentialUpdate) -> Result<Credential> {
        let current = self.get_credential(name).await?;
        let id = current.id.unwrap_or_default();
        let sealed = match &update.secrets {
            Some(secrets) => Some(self.seal(&id, secrets).await?),
            None => None,
        };
        let mut txn = write_txn(&self.pool).await?;
        let result = sqlx::query(
            "UPDATE uc_credentials SET name = coalesce($1, name), \
             comment = coalesce($2, comment), read_only = coalesce($3, read_only), \
             owner = coalesce($4, owner), sealed = coalesce($5, sealed), updated_at = $6 \
             WHERE id = $7::uuid",
        )
        .bind(&update.new_name)
        .bind(&update.comment)
        .bind(update.read_only)
        .bind(&update.owner)
        .bind(&sealed)
        .bind(now_millis())
        .bind(&id)
        .execute(&mut *txn)
        .await
        .map_err(db)?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        let name = update.new_name.as_deref().unwrap_or(name);
        let updated = credential_by_name(&mut txn, name).await?;
        txn.commit().await.map_err(db)?;
        Ok(updated.into())
    }

    async fn delete_credential(&self, name: &str) -> Result<()> {
        let mut txn = write_txn(&self.pool).await?;
        let credential = credential_by_name(&mut txn, name).await?;
        let users: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM uc_external_locations WHERE credential_id = $1::uuid \
             ORDER BY name",
        )
        .bind(&credential.id)
        .fetch_all(&mut *txn)
        .await
        .map_err(db)?;
        if !users.is_empty() {
            return Err(credential_in_use_error(&credential.name, &users));
        }
        sqlx::query("DELETE FROM uc_credentials WHERE id = $1::uuid")
            .bind(&credential.id)
            .execute(&mut *txn)
            .await
            .map_err(db)?;
        txn.commit().await.map_err(db)
    }

    async fn create_external_location(
        &self,
        location: ExternalLocation,
    ) -> Result<ExternalLocation> {
        let key = LocationKey::parse(&location.url)?;
        let mut txn = write_txn(&self.pool).await?;
        check_no_overlap(&mut txn, &location.url, &key, None).await?;
        let credential = credential_by_name(&mut txn, &location.credential_name).await?;
        let now = now_millis();
        sqlx::query(
            "INSERT INTO uc_external_locations (id, name, url, url_key, credential_id, \
             read_only, comment, owner, created_at, created_by, updated_at, updated_by) \
             VALUES ($1::uuid, $2, $3, $4, $5::uuid, $6, $7, $8, $9, $10, $9, $10)",
        )
        .bind(new_id())
        .bind(&location.name)
        .bind(&location.url)
        .bind(key.as_str())
        .bind(&credential.id)
        .bind(location.read_only)
        .bind(&location.comment)
        .bind(&location.owner)
        .bind(now)
        .bind(&location.created_by)
        .execute(&mut *txn)
        .await
        .map_err(db)?;
        let created = location_by_name(&mut txn, &location.name).await?;
        txn.commit().await.map_err(db)?;
        Ok(created.into())
    }

    async fn get_external_location(&self, name: &str) -> Result<ExternalLocation> {
        let mut conn = self.pool.acquire().await.map_err(db)?;
        Ok(location_by_name(&mut conn, name).await?.into())
    }

    async fn list_external_locations(
        &self,
        page: Page,
    ) -> Result<(Vec<ExternalLocation>, Option<String>)> {
        let mut query = QueryBuilder::<Postgres>::new(format!(
            "SELECT {LOCATION_COLUMNS} FROM uc_external_locations l \
             JOIN uc_credentials c ON c.id = l.credential_id WHERE true"
        ));
        if let Some(token) = page.token {
            query.push(" AND l.name > ").push_bind(token);
        }
        query.push(" ORDER BY l.name");
        if let Some(max) = page.max_results {
            query.push(" LIMIT ").push_bind(max as i64 + 1);
        }
        let rows: Vec<LocationRow> = query
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .map_err(db)?;
        Ok(paginate(rows, page.max_results, |r| r.name.clone()))
    }

    async fn update_external_location(
        &self,
        name: &str,
        update: ExternalLocationUpdate,
    ) -> Result<ExternalLocation> {
        let key = update.url.as_deref().map(LocationKey::parse).transpose()?;
        let mut txn = write_txn(&self.pool).await?;
        let current = location_by_name(&mut txn, name).await?;
        if let (Some(url), Some(key)) = (&update.url, &key) {
            check_no_overlap(&mut txn, url, key, Some(&current.id)).await?;
        }
        let credential_id = match &update.credential_name {
            Some(credential) => Some(credential_by_name(&mut txn, credential).await?.id),
            None => None,
        };
        sqlx::query(
            "UPDATE uc_external_locations SET name = coalesce($1, name), \
             url = coalesce($2, url), url_key = coalesce($3, url_key), \
             credential_id = coalesce($4::uuid, credential_id), \
             read_only = coalesce($5, read_only), comment = coalesce($6, comment), \
             owner = coalesce($7, owner), updated_at = $8 WHERE id = $9::uuid",
        )
        .bind(&update.new_name)
        .bind(&update.url)
        .bind(key.as_ref().map(LocationKey::as_str))
        .bind(&credential_id)
        .bind(update.read_only)
        .bind(&update.comment)
        .bind(&update.owner)
        .bind(now_millis())
        .bind(&current.id)
        .execute(&mut *txn)
        .await
        .map_err(db)?;
        let name = update.new_name.as_deref().unwrap_or(name);
        let updated = location_by_name(&mut txn, name).await?;
        txn.commit().await.map_err(db)?;
        Ok(updated.into())
    }

    async fn delete_external_location(&self, name: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM uc_external_locations WHERE name = $1")
            .bind(name)
            .execute(&self.pool)
            .await
            .map_err(db)?;
        if result.rows_affected() == 0 {
            return Err(Error::NotFound);
        }
        Ok(())
    }

    async fn governing_location(&self, location: &StorageLocation) -> Result<Governed> {
        let mut conn = self.pool.acquire().await.map_err(db)?;
        let row = covering(&mut conn, &LocationKey::new(location))
            .await?
            .ok_or(Error::NotFound)?;
        let SealedCredentialRow { credential, sealed } = sqlx::query_as(&format!(
            "SELECT {CREDENTIAL_COLUMNS}, c.sealed FROM uc_credentials c \
             WHERE c.id = $1::uuid"
        ))
        .bind(&row.credential_id)
        .fetch_one(&mut *conn)
        .await
        .map_err(db)?;
        let secrets = self.open(&credential.id, &sealed).await?;
        let mut credential = Credential::from(credential);
        secrets.apply(&mut credential);
        Ok(Governed {
            location: row.into(),
            credential,
        })
    }

    async fn covering_location(
        &self,
        location: &StorageLocation,
    ) -> Result<Option<ExternalLocation>> {
        let mut conn = self.pool.acquire().await.map_err(db)?;
        Ok(covering(&mut conn, &LocationKey::new(location))
            .await?
            .map(Into::into))
    }

    async fn rewrap_secrets(&self) -> Result<usize> {
        let rows: Vec<(String, Vec<u8>)> =
            sqlx::query_as("SELECT id::text AS id, sealed FROM uc_credentials")
                .fetch_all(&self.pool)
                .await
                .map_err(db)?;
        let mut rewrapped = 0;
        for (id, sealed) in rows {
            let Some(fresh) = rewrap(&self.encryptor, &sealed).await? else {
                continue;
            };
            // Only replace the blob read above: a concurrent update has already
            // sealed its secrets under the active key.
            let result = sqlx::query(
                "UPDATE uc_credentials SET sealed = $1 WHERE id = $2::uuid AND sealed = $3",
            )
            .bind(&fresh)
            .bind(&id)
            .bind(&sealed)
            .execute(&self.pool)
            .await
            .map_err(db)?;
            rewrapped += result.rows_affected() as usize;
        }
        Ok(rewrapped)
    }
}
