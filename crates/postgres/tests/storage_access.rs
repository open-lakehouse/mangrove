//! The storage-access conformance suite against the Postgres store.
//!
//! Gated behind the `integration-pg` feature; `#[sqlx::test]` provisions a fresh
//! database. Run with `DATABASE_URL` pointing at a live Postgres:
//!
//! ```sh
//! cargo test -p olai-uc-postgres --features integration-pg --test storage_access
//! ```
#![cfg(feature = "integration-pg")]
use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_postgres::PgStorageAccess;

/// Each case gets the bundle's tables emptied.
#[sqlx::test(migrations = false)]
async fn conformance(pool: sqlx::PgPool) {
    unitycatalog_postgres::unified_migrator()
        .run(&pool)
        .await
        .unwrap();
    let encryptor =
        EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![7u8; 32]).unwrap());
    unitycatalog_storage_access::testing::conformance(|| {
        let pool = pool.clone();
        let encryptor = encryptor.clone();
        async move {
            sqlx::query("TRUNCATE uc_external_locations, uc_credentials CASCADE")
                .execute(&pool)
                .await
                .unwrap();
            PgStorageAccess::new(pool, encryptor)
        }
    })
    .await;
}

fn encryptor(active: &str, keys: &[&str]) -> EnvelopeEncryptor {
    let keys = keys
        .iter()
        .map(|k| ((*k).into(), vec![k.as_bytes()[0]; 32]));
    EnvelopeEncryptor::local(LocalKeyProvider::new(active, keys).unwrap())
}

/// Secrets sealed under a retired key are re-wrapped under the active one.
#[sqlx::test(migrations = false)]
async fn key_rotation(pool: sqlx::PgPool) {
    unitycatalog_postgres::unified_migrator()
        .run(&pool)
        .await
        .unwrap();
    let before = PgStorageAccess::new(pool.clone(), encryptor("a", &["a"]));
    let after = PgStorageAccess::new(pool, encryptor("b", &["a", "b"]));
    unitycatalog_storage_access::testing::key_rotation(&before, &after).await;
}
