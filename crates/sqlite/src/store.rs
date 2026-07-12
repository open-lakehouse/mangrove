//! Embedded SQLite storage layer.
//!
//! The object/association graph is now provided by the generic
//! [`olai_store::SqlStore`] (the SQLite-backed backend of `olai-store`); this
//! crate composes it with the one mangrove-specific concern that rides the same
//! database — the [`CommitCoordinator`](unitycatalog_delta_api::CommitCoordinator)
//! (`delta_commits` table) — sharing one [`SqlitePool`]. Sensitive fields
//! (credentials, tokens) are sealed inline on the object rows by the
//! [`ManagedObjectStore`] layer rather than in a separate secret store.
//!
//! [`SqliteStore`] forwards the `olai_store` object/association traits to the
//! inner `SqlStore`, so the blanket `ObjectStoreAdapter` in `unitycatalog-common`
//! still lifts it to the high-level `ResourceStore` API. Inverse edges are wired
//! from [`AssociationLabel::inverse`], reproducing the auto-inverse behavior the
//! previous hand-written backend maintained.

use std::str::FromStr;

use sqlx::SqlitePool;
use sqlx::migrate::Migrator;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use unitycatalog_common::services::encryption::EnvelopeEncryptor;
use unitycatalog_common::{AssociationLabel, Object, ObjectLabel};
use uuid::Uuid;

use bytes::Bytes;
use olai_store::filter::Filter;
use olai_store::name::ResourceName;
use olai_store::store::EdgeEndpoint;
use olai_store::store::{StoreExec, StoreTx, Transactional};
use olai_store::{
    Association, AssociationStore, AssociationStoreReader, EdgeQuery, ManagedObjectStore,
    ObjectStore, ObjectStoreReader, Precondition, ResourceRegistry, SecretObjectReader, SqlStore,
};
use unitycatalog_common::models::labels::RESOURCE_DESCRIPTORS;

use crate::error::{Error, Result};

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
fn unified_migrator() -> Migrator {
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

/// An embedded, file-based SQLite store for catalog metadata (with sensitive
/// fields sealed inline) and Delta catalog-managed commits.
#[derive(Clone)]
pub struct SqliteStore {
    /// Registry-aware object/association graph store that seals `Sensitive` fields
    /// inline, over the generic [`SqlStore`], sharing [`pool`](Self::pool).
    store: ManagedObjectStore<ObjectLabel, SqlStore<ObjectLabel>>,
    /// Shared connection pool; used directly by the commit-coordinator impl
    /// (see `commit_coordinator.rs`).
    pub(crate) pool: SqlitePool,
}

impl SqliteStore {
    /// Compose a store over an **already-migrated** pool.
    pub fn new(pool: SqlitePool, encryptor: EnvelopeEncryptor) -> Self {
        let inner = SqlStore::<ObjectLabel>::connect(pool.clone()).with_inverse(inverse_resolver);
        // The managed layer strips + seals `FieldRole::Sensitive` fields (e.g. a
        // credential's secret material) into the object row's inline encrypted blob,
        // redacting them from ordinary reads. The registry is generated from the proto
        // `debug_redact` annotations (see `RESOURCE_DESCRIPTORS`).
        let registry = ResourceRegistry::from_static(RESOURCE_DESCRIPTORS);
        let store = ManagedObjectStore::with_encryptor(inner, encryptor, registry);
        Self { store, pool }
    }

    /// Open (creating if necessary) a SQLite database at `path`.
    ///
    /// `path` is a filesystem path to the database file; the special value
    /// `:memory:` opens an ephemeral in-memory database (useful for tests).
    /// The database file and any missing schema are created on first use.
    pub async fn connect(path: impl AsRef<str>, encryptor: EnvelopeEncryptor) -> Result<Self> {
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
        // embedded single-writer store; an in-memory database must use one
        // connection so every caller sees the same database.
        let pool = SqlitePoolOptions::new()
            .max_connections(if path == ":memory:" { 1 } else { 16 })
            .connect_with(options)
            .await
            .map_err(Error::from)?;
        Ok(Self::new(pool, encryptor))
    }

    /// Apply both the generic object/association schema and the local
    /// `delta_commits` schema.
    pub async fn migrate(&self) -> Result<()> {
        unified_migrator()
            .run(&self.pool)
            .await
            .map_err(Error::from)?;
        Ok(())
    }
}

// --- olai_store trait forwarding -------------------------------------------
//
// SqliteStore delegates the generic object/association surface to the inner
// `SqlStore`, so `ObjectStoreAdapter` (in unitycatalog-common) treats it as a
// full backend. The inverse-edge resolver is configured on `self.store`.

#[async_trait::async_trait]
impl ObjectStoreReader<ObjectLabel> for SqliteStore {
    async fn get(&self, id: &Uuid) -> olai_store::Result<Object> {
        self.store.get(id).await
    }

    async fn get_by_name(
        &self,
        label: ObjectLabel,
        name: &ResourceName,
    ) -> olai_store::Result<Object> {
        self.store.get_by_name(label, name).await
    }

    async fn list(
        &self,
        label: ObjectLabel,
        namespace: Option<&ResourceName>,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> olai_store::Result<(Vec<Object>, Option<String>)> {
        self.store
            .list(label, namespace, max_results, page_token)
            .await
    }

    async fn search(
        &self,
        label: ObjectLabel,
        namespace: Option<&ResourceName>,
        filter: &Filter,
        max_results: Option<usize>,
        page_token: Option<String>,
    ) -> olai_store::Result<(Vec<Object>, Option<String>)> {
        self.store
            .search(label, namespace, filter, max_results, page_token)
            .await
    }

    async fn get_sensitive(&self, id: &Uuid) -> olai_store::Result<Option<Bytes>> {
        self.store.get_sensitive(id).await
    }
}

#[async_trait::async_trait]
impl SecretObjectReader<ObjectLabel> for SqliteStore {
    async fn get_with_secrets(&self, id: &Uuid) -> olai_store::Result<Object> {
        self.store.get_with_secrets(id).await
    }
}

#[async_trait::async_trait]
impl ObjectStore<ObjectLabel> for SqliteStore {
    async fn create(
        &self,
        label: ObjectLabel,
        name: &ResourceName,
        properties: Option<serde_json::Value>,
        id: Option<Uuid>,
        sensitive: Option<bytes::Bytes>,
    ) -> olai_store::Result<Object> {
        self.store
            .create(label, name, properties, id, sensitive)
            .await
    }

    async fn update(
        &self,
        id: &Uuid,
        properties: Option<serde_json::Value>,
        precondition: Precondition,
        sensitive: Option<bytes::Bytes>,
    ) -> olai_store::Result<Object> {
        self.store
            .update(id, properties, precondition, sensitive)
            .await
    }

    async fn rename(
        &self,
        id: &Uuid,
        new_name: &ResourceName,
        precondition: Precondition,
    ) -> olai_store::Result<Object> {
        self.store.rename(id, new_name, precondition).await
    }

    async fn delete(&self, id: &Uuid) -> olai_store::Result<()> {
        self.store.delete(id).await
    }

    async fn set_sensitive(&self, id: &Uuid, sensitive: Bytes) -> olai_store::Result<()> {
        self.store.set_sensitive(id, sensitive).await
    }
}

#[async_trait::async_trait]
impl AssociationStoreReader<ObjectLabel> for SqliteStore {
    async fn query_edges(
        &self,
        query: EdgeQuery<'_, ObjectLabel>,
    ) -> olai_store::Result<(Vec<Association<ObjectLabel>>, Option<String>)> {
        self.store.query_edges(query).await
    }

    async fn count_edges(
        &self,
        endpoint: EdgeEndpoint,
        label: &str,
        target_label: Option<ObjectLabel>,
    ) -> olai_store::Result<u64> {
        self.store.count_edges(endpoint, label, target_label).await
    }
}

#[async_trait::async_trait]
impl AssociationStore<ObjectLabel> for SqliteStore {
    async fn add(
        &self,
        from_id: Uuid,
        to_id: Uuid,
        label: &str,
        properties: Option<serde_json::Value>,
    ) -> olai_store::Result<()> {
        self.store.add(from_id, to_id, label, properties).await
    }

    async fn remove(&self, from_id: Uuid, to_id: Uuid, label: &str) -> olai_store::Result<()> {
        self.store.remove(from_id, to_id, label).await
    }
}

#[async_trait::async_trait]
impl Transactional<ObjectLabel> for SqliteStore {
    async fn transaction<'a, T>(
        &'a self,
        f: Box<
            dyn for<'t> FnOnce(
                    &'t dyn StoreExec<ObjectLabel>,
                )
                    -> futures::future::BoxFuture<'t, olai_store::Result<T>>
                + Send
                + 'a,
        >,
    ) -> olai_store::Result<T>
    where
        T: Send + 'a,
    {
        self.store.transaction(f).await
    }

    async fn begin(&self) -> olai_store::Result<Box<dyn StoreTx<ObjectLabel>>> {
        self.store.begin().await
    }
}
