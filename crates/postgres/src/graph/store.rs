//! Postgres storage layer.
//!
//! The object/association graph is provided by the generic
//! [`olai_store::PgStore`] (the native-Postgres backend of `olai-store`), composed
//! with an [`olai_store::ManagedObjectStore`] so a resource's `FieldRole::Sensitive`
//! fields (e.g. credential secrets) are sealed inline on the object row. The
//! composed [`PgGraphStore`] implements the `olai_store` object/association traits
//! directly, so the blanket `ObjectStoreAdapter` in `unitycatalog-common` lifts it
//! to the high-level `ResourceStore` API — no local forwarding wrapper is needed.
//! Inverse edges are wired from [`AssociationLabel::inverse`].
//!
//! The one mangrove-specific concern that rides the same database — the
//! [`CommitCoordinator`](unitycatalog_delta_api::CommitCoordinator) over the
//! `delta_commits` table — lives in a separate [`PgCommitCoordinator`](crate::PgCommitCoordinator)
//! holding its own clone of the same [`PgPool`].

use std::str::FromStr;

use sqlx::PgPool;
use sqlx::migrate::Migrator;
use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use unitycatalog_common::services::encryption::EnvelopeEncryptor;
use unitycatalog_common::{AssociationLabel, ObjectLabel};

use olai_store::{ManagedObjectStore, PgStore, ResourceRegistry};
use unitycatalog_common::models::labels::RESOURCE_DESCRIPTORS;

use crate::error::Result;

/// The composed Postgres object/association graph store: the generic
/// [`PgStore`] wrapped in a registry-aware, sensitive-field-sealing
/// [`ManagedObjectStore`]. `ObjectStoreAdapter` lifts this to `ResourceStore`.
pub type PgGraphStore = ManagedObjectStore<ObjectLabel, PgStore<ObjectLabel>>;

/// The mangrove-local migrations (`delta_commits`) that ride the same database
/// as the generic object/association schema.
///
/// These are versioned in a high range (0100+) so they never collide with
/// `olai_store`'s object-graph migrations (0001+); [`unified_migrator`] merges
/// the two sets into one ordered ledger.
static LOCAL_MIGRATOR: Migrator = sqlx::migrate!();

/// One [`Migrator`] over `olai_store`'s object-graph schema **and** the
/// mangrove-local schema, sharing a single `_sqlx_migrations` ledger.
pub fn unified_migrator() -> Migrator {
    olai_store::pg_migrator_with(LOCAL_MIGRATOR.migrations.iter().cloned())
}

/// Map an [`AssociationLabel`] string to its inverse label string, for the
/// generic store's inverse-edge resolver.
fn inverse_resolver(label: &str) -> Option<String> {
    AssociationLabel::from_str(label)
        .ok()
        .and_then(|l| l.inverse())
        .map(|inv| inv.to_string())
}

/// Open a connection pool to the Postgres database at `url`.
pub async fn connect_pool(url: impl AsRef<str>) -> Result<PgPool> {
    let options: PgConnectOptions = url.as_ref().parse()?;
    let pool = PgPoolOptions::new()
        .max_connections(96)
        .connect_with(options)
        .await?;
    Ok(pool)
}

/// Compose the object/association graph store over a connection pool.
///
/// The managed layer strips + seals `FieldRole::Sensitive` fields (e.g. a
/// credential's secret material) into the object row's inline encrypted blob,
/// redacting them from ordinary reads. The registry is generated from the proto
/// `debug_redact` annotations (see `RESOURCE_DESCRIPTORS`).
pub fn connect_graph(pool: PgPool, encryptor: EnvelopeEncryptor) -> PgGraphStore {
    let inner = PgStore::<ObjectLabel>::connect(pool).with_inverse(inverse_resolver);
    let registry = ResourceRegistry::from_static(RESOURCE_DESCRIPTORS);
    ManagedObjectStore::with_encryptor(inner, encryptor, registry)
}
