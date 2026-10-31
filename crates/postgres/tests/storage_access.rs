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
