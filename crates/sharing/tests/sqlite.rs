//! The SQLite store, on a fresh database file.

use std::str::FromStr;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use unitycatalog_sharing::sqlite::MIGRATOR;
use unitycatalog_sharing::testing::round_trips;
use unitycatalog_sharing::{ShareReader, ShareStore, SqliteSharingStore};

async fn connect(url: &str) -> SqlitePool {
    let options = SqliteConnectOptions::from_str(url)
        .unwrap()
        .create_if_missing(true)
        .foreign_keys(true);
    SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .unwrap()
}

#[tokio::test]
async fn sqlite_store_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let pool = connect(&format!(
        "sqlite://{}",
        dir.path().join("sharing.db").display()
    ))
    .await;
    MIGRATOR.run(&pool).await.unwrap();
    round_trips(&SqliteSharingStore::new(pool)).await;
}

#[tokio::test]
async fn sqlite_state_survives_reconnecting() {
    let dir = tempfile::tempdir().unwrap();
    let url = format!("sqlite://{}", dir.path().join("sharing.db").display());
    let pool = connect(&url).await;
    MIGRATOR.run(&pool).await.unwrap();
    SqliteSharingStore::new(pool.clone())
        .create_share("kept", None)
        .await
        .unwrap();
    pool.close().await;

    let pool = connect(&url).await;
    assert_eq!(
        SqliteSharingStore::new(pool)
            .get_share("kept")
            .await
            .unwrap()
            .name,
        "kept"
    );
}
