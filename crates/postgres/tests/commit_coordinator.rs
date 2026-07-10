//! Integration tests for the Postgres-backed Delta commit coordinator.
//!
//! Gated behind the `integration-pg` feature; `#[sqlx::test]` provisions a fresh
//! database per test and runs migrations. Run with a live Postgres, e.g.:
//!
//! ```sh
//! DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres \
//!   cargo test -p unitycatalog-postgres --features integration-pg
//! ```
#![cfg(feature = "integration-pg")]

use std::sync::atomic::{AtomicU64, Ordering};

use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_delta_api::coordinator::CommitInfo;
use unitycatalog_delta_api::coordinator::{CommitCoordinator, CommitError};
use unitycatalog_postgres::GraphStore;
use uuid::Uuid;

fn store(pool: sqlx::PgPool) -> GraphStore {
    let encryptor =
        EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
    GraphStore::new(pool, encryptor)
}

/// A unique table id per call (the `uuid` crate's `v4` feature isn't enabled).
fn table_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    Uuid::from_u128(COUNTER.fetch_add(1, Ordering::Relaxed) as u128).to_string()
}

fn commit_info(version: i64) -> CommitInfo {
    CommitInfo {
        version,
        timestamp: 1000 + version,
        file_name: format!("{version:020}.uuid.json"),
        file_size: 128,
        file_modification_timestamp: 2000 + version,
    }
}

#[sqlx::test]
async fn onboarding_then_get_roundtrip(pool: sqlx::PgPool) {
    let cc = store(pool);
    let t = table_id();
    cc.commit(&t, Some(commit_info(1)), None).await.unwrap();
    cc.commit(&t, Some(commit_info(2)), None).await.unwrap();

    let (commits, latest) = cc.get_commits(&t, 0, None).await.unwrap();
    assert_eq!(latest, 2);
    assert_eq!(
        commits.iter().map(|c| c.version).collect::<Vec<_>>(),
        vec![1, 2]
    );
    // round-trip preserves the commit metadata
    assert_eq!(commits[0].file_name, commit_info(1).file_name);
    assert_eq!(commits[0].timestamp, commit_info(1).timestamp);
    assert_eq!(commits[0].file_size, 128);
}

#[sqlx::test]
async fn replay_and_gap(pool: sqlx::PgPool) {
    let cc = store(pool);
    let t = table_id();
    cc.commit(&t, Some(commit_info(1)), None).await.unwrap();
    assert!(matches!(
        cc.commit(&t, Some(commit_info(1)), None).await.unwrap_err(),
        CommitError::VersionConflict(_)
    ));
    assert!(matches!(
        cc.commit(&t, Some(commit_info(3)), None).await.unwrap_err(),
        CommitError::InvalidArgument(_)
    ));
}

#[sqlx::test]
async fn backfill_keeps_highest_as_marker(pool: sqlx::PgPool) {
    let cc = store(pool);
    let t = table_id();
    for v in 1..=4 {
        cc.commit(&t, Some(commit_info(v)), None).await.unwrap();
    }
    cc.commit(&t, None, Some(4)).await.unwrap();
    let (commits, latest) = cc.get_commits(&t, 0, None).await.unwrap();
    assert_eq!(latest, 4, "latest_table_version reported from marker row");
    assert!(commits.is_empty(), "marker row excluded from commits");

    cc.commit(&t, Some(commit_info(5)), None).await.unwrap();
    let (commits, latest) = cc.get_commits(&t, 0, None).await.unwrap();
    assert_eq!(latest, 5);
    assert_eq!(
        commits.iter().map(|c| c.version).collect::<Vec<_>>(),
        vec![5]
    );
}

#[sqlx::test]
async fn partial_backfill_prunes_below_watermark(pool: sqlx::PgPool) {
    let cc = store(pool);
    let t = table_id();
    for v in 1..=5 {
        cc.commit(&t, Some(commit_info(v)), None).await.unwrap();
    }
    cc.commit(&t, None, Some(3)).await.unwrap();
    let (commits, latest) = cc.get_commits(&t, 0, None).await.unwrap();
    assert_eq!(latest, 5);
    assert_eq!(
        commits.iter().map(|c| c.version).collect::<Vec<_>>(),
        vec![4, 5]
    );
}

#[sqlx::test]
async fn unbackfilled_cap_enforced(pool: sqlx::PgPool) {
    let cc = store(pool);
    let t = table_id();
    // Default cap is 10; 11th unbackfilled commit must be rejected.
    for v in 1..=10 {
        cc.commit(&t, Some(commit_info(v)), None).await.unwrap();
    }
    assert!(matches!(
        cc.commit(&t, Some(commit_info(11)), None)
            .await
            .unwrap_err(),
        CommitError::ResourceExhausted(_)
    ));
    // Backfilling re-opens room.
    cc.commit(&t, None, Some(5)).await.unwrap();
    cc.commit(&t, Some(commit_info(11)), None).await.unwrap();
    let (_, latest) = cc.get_commits(&t, 0, None).await.unwrap();
    assert_eq!(latest, 11);
}

#[sqlx::test]
async fn unknown_table_reports_zero(pool: sqlx::PgPool) {
    let cc = store(pool);
    let (commits, latest) = cc.get_commits(&table_id(), 0, None).await.unwrap();
    assert!(commits.is_empty());
    assert_eq!(latest, 0);
}

#[sqlx::test]
async fn first_writer_wins_under_concurrency(pool: sqlx::PgPool) {
    let cc = std::sync::Arc::new(store(pool));
    let t = table_id();
    cc.commit(&t, Some(commit_info(1)), None).await.unwrap();

    // Race many writers for version 2; the unique constraint must let exactly one win.
    let mut handles = Vec::new();
    for _ in 0..8 {
        let cc = cc.clone();
        let t = t.clone();
        handles.push(tokio::spawn(async move {
            cc.commit(&t, Some(commit_info(2)), None).await
        }));
    }
    let mut wins = 0;
    let mut conflicts = 0;
    for h in handles {
        match h.await.unwrap() {
            Ok(()) => wins += 1,
            Err(CommitError::VersionConflict(_)) => conflicts += 1,
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }
    assert_eq!(wins, 1, "exactly one writer wins version 2");
    assert_eq!(conflicts, 7);
}
