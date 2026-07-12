//! Embedded SQLite storage layer.
//!
//! The object/association graph is provided by the generic
//! [`olai_store::SqlStore`] (the SQLite-backed backend of `olai-store`), composed
//! with an [`olai_store::ManagedObjectStore`] so a resource's `FieldRole::Sensitive`
//! fields (credentials, tokens) are sealed inline on the object row. The composed
//! [`SqliteGraphStore`] implements the `olai_store` object/association traits
//! directly, so the blanket `ObjectStoreAdapter` in `unitycatalog-common` lifts it
//! to the high-level `ResourceStore` API — no local forwarding wrapper is needed.
//! Inverse edges are wired from [`AssociationLabel::inverse`].
//!
//! The one mangrove-specific concern that rides the same database — the
//! [`CommitCoordinator`](unitycatalog_delta_api::CommitCoordinator) over the
//! `delta_commits` table — lives in a separate
//! [`SqliteCommitCoordinator`](crate::SqliteCommitCoordinator) holding its own
//! clone of the same [`SqlitePool`].

use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use unitycatalog_common::services::encryption::EnvelopeEncryptor;
use unitycatalog_common::{AssociationLabel, ObjectLabel};

use olai_store::{ManagedObjectStore, ResourceRegistry, SqlStore};
use unitycatalog_common::models::labels::RESOURCE_DESCRIPTORS;

use crate::error::{Error, Result};

/// The composed SQLite object/association graph store: the generic [`SqlStore`]
/// wrapped in a registry-aware, sensitive-field-sealing [`ManagedObjectStore`].
/// `ObjectStoreAdapter` lifts this to `ResourceStore`.
pub type SqliteGraphStore = ManagedObjectStore<ObjectLabel, SqlStore<ObjectLabel>>;

/// The mangrove-local migrations (`delta_commits`) that ride the same database
/// as the generic object/association schema.
///
/// These are versioned in a high range (0100+) so they never collide with
/// `olai_store`'s object-graph migrations (0001+); [`unified_migrator`] merges
/// the two sets into one ordered ledger.
static LOCAL_MIGRATOR: Migrator = sqlx::migrate!();

/// One [`Migrator`] over `olai_store`'s object-graph schema **and** the
/// mangrove-local schema, sharing a single `_sqlx_migrations` ledger.
///
/// `olai_store::sql_migrator_with` merges its own migrations with the local set
/// so there is exactly one migrator per database — no second ledger, no
/// version-range/`ignore_missing` juggling between two independent migrators.
pub fn unified_migrator() -> Migrator {
    olai_store::sql_migrator_with(LOCAL_MIGRATOR.migrations.iter().cloned())
}

/// Map an [`AssociationLabel`] string to its inverse label string, for the
/// generic store's inverse-edge resolver.
fn inverse_resolver(label: &str) -> Option<String> {
    AssociationLabel::from_str(label)
        .ok()
        .and_then(|l| l.inverse())
        .map(|inv| inv.to_string())
}

/// Open (creating if necessary) a SQLite connection pool at `path`.
///
/// `path` is a filesystem path to the database file; the special value
/// `:memory:` opens an ephemeral in-memory database (useful for tests). The
/// database file and any missing schema are created on first use.
pub async fn connect_pool(path: impl AsRef<str>) -> Result<SqlitePool> {
    let path = path.as_ref();
    let options = if path == ":memory:" {
        SqliteConnectOptions::from_str("sqlite::memory:").map_err(Error::from)?
    } else {
        SqliteConnectOptions::from_str(&format!("sqlite://{path}"))
            .map_err(Error::from)?
            .create_if_missing(true)
            .journal_mode(SqliteJournalMode::Wal)
            .busy_timeout(std::time::Duration::from_secs(5))
            .foreign_keys(true)
    };
    // A single connection in WAL mode is the simplest correct setup for an
    // embedded single-writer store; an in-memory database must use one connection
    // so every caller sees the same database.
    let pool = SqlitePoolOptions::new()
        .max_connections(if path == ":memory:" { 1 } else { 16 })
        .connect_with(options)
        .await
        .map_err(Error::from)?;
    Ok(pool)
}

/// Compose the object/association graph store over a connection pool.
///
/// The managed layer strips + seals `FieldRole::Sensitive` fields (e.g. a
/// credential's secret material) into the object row's inline encrypted blob,
/// redacting them from ordinary reads. The registry is generated from the proto
/// `debug_redact` annotations (see `RESOURCE_DESCRIPTORS`).
pub fn connect_graph(pool: SqlitePool, encryptor: EnvelopeEncryptor) -> SqliteGraphStore {
    let inner = SqlStore::<ObjectLabel>::connect(pool).with_inverse(inverse_resolver);
    let registry = ResourceRegistry::from_static(RESOURCE_DESCRIPTORS);
    ManagedObjectStore::with_encryptor(inner, encryptor, registry)
}
