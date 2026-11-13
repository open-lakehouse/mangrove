//! Integration tests for the Postgres-backed Delta commit coordinator.
//!
//! Gated behind the `integration-pg` feature; `#[sqlx::test]` provisions a fresh
//! database per test and runs migrations. Run with a live Postgres, e.g.:
//!
//! ```sh
//! DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres \
//!   cargo test -p olai-uc-postgres --features integration-pg
//! ```
//!
//! The protocol rules are checked by the shared conformance suite; the tests
//! here cover what depends on Postgres itself: the unique-constraint arbiter
//! under racing writers, and the pre-retention row layout.
#![cfg(feature = "integration-pg")]

use std::sync::Arc;

use unitycatalog_delta_api::coordinator::{CommitCoordinator, CommitError, CommitInfo};
use unitycatalog_delta_api::testing::coordinator_conformance::{self, commit_info};
use unitycatalog_postgres::PgCommitCoordinator;

fn table_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

#[sqlx::test]
async fn conformance(pool: sqlx::PgPool) {
    coordinator_conformance::run_all(|limits| {
        PgCommitCoordinator::new(pool.clone()).with_limits(limits)
    })
    .await;
}

/// Race `writers` commits at version 2 (distinct files when `distinct`, else the
/// same file) and count the outcomes.
async fn race(pool: sqlx::PgPool, writers: usize, distinct: bool) -> (usize, usize, usize) {
    let cc = Arc::new(PgCommitCoordinator::new(pool));
    let t = table_id();
    cc.commit(&t, Some(commit_info(1)), None).await.unwrap();

    let mut handles = Vec::new();
    for i in 0..writers {
        let (cc, t) = (cc.clone(), t.clone());
        let info = CommitInfo {
            file_name: if distinct {
                format!("00000000000000000002.{i}.json")
            } else {
                commit_info(2).file_name
            },
            ..commit_info(2)
        };
        handles.push(tokio::spawn(async move {
            cc.commit(&t, Some(info), None).await
        }));
    }
    let (mut wins, mut conflicts, mut replays) = (0, 0, 0);
    for h in handles {
        match h.await.unwrap() {
            Ok(()) => wins += 1,
            Err(CommitError::VersionConflict(_)) => conflicts += 1,
            Err(CommitError::AlreadyAccepted) => replays += 1,
            Err(e) => panic!("unexpected error: {e:?}"),
        }
    }
    (wins, conflicts, replays)
}

#[sqlx::test]
async fn first_writer_wins_under_concurrency(pool: sqlx::PgPool) {
    assert_eq!(race(pool, 8, true).await, (1, 7, 0));
}

#[sqlx::test]
async fn concurrent_identical_retries_are_replays(pool: sqlx::PgPool) {
    // The losers of the insert race carry the winner's file name: replays.
    assert_eq!(race(pool, 8, false).await, (1, 0, 7));
}

/// Rows written before retention (backfilled rows deleted, only a fully
/// backfilled latest row flagged) read correctly under the new rule.
#[sqlx::test]
async fn pre_retention_layout_reads_correctly(pool: sqlx::PgPool) {
    let cc = PgCommitCoordinator::new(pool.clone());
    let t = table_id();
    for v in 1..=3 {
        cc.commit(&t, Some(commit_info(v)), None).await.unwrap();
    }
    // Simulate the old layout: versions 1-2 deleted, 3 flagged as the marker.
    sqlx::query("DELETE FROM delta_commits WHERE table_id = $1::uuid AND commit_version < 3")
        .bind(&t)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE delta_commits SET is_backfilled_latest = true \
         WHERE table_id = $1::uuid AND commit_version = 3",
    )
    .bind(&t)
    .execute(&pool)
    .await
    .unwrap();

    let (commits, latest) = cc.get_commits(&t, 0, None).await.unwrap();
    assert!(commits.is_empty());
    assert_eq!(latest, 3);
    cc.commit(&t, Some(commit_info(4)), None).await.unwrap();
    let (commits, latest) = cc.get_commits(&t, 0, None).await.unwrap();
    assert_eq!(
        commits.iter().map(|c| c.version).collect::<Vec<_>>(),
        vec![4]
    );
    assert_eq!(latest, 4);
}
