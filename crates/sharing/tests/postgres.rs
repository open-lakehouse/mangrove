//! The Postgres store against a live database.
//!
//! Runs only when `SHARING_TEST_DATABASE_URL` holds the connection URL of an
//! empty, disposable Postgres database (it is migrated and written to), and is
//! skipped otherwise:
//!
//! ```sh
//! SHARING_TEST_DATABASE_URL=<connection url> \
//!   cargo test -p olai-uc-sharing --all-features --test postgres
//! ```

use sqlx::postgres::PgPoolOptions;
use unitycatalog_sharing::PgSharingStore;
use unitycatalog_sharing::postgres::MIGRATOR;
use unitycatalog_sharing::testing::round_trips;

#[tokio::test]
async fn postgres_store_round_trips() {
    let Ok(url) = std::env::var("SHARING_TEST_DATABASE_URL") else {
        eprintln!("SHARING_TEST_DATABASE_URL not set; skipping");
        return;
    };
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await
        .unwrap();
    MIGRATOR.run(&pool).await.unwrap();
    round_trips(&PgSharingStore::new(pool)).await;
}
