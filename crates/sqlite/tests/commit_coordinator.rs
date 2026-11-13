//! Integration tests for the SQLite-backed Delta commit coordinator.
//!
//! Each test runs against an isolated temp-file SQLite database that is cleaned
//! up on drop. The protocol rules are checked by the shared conformance suite;
//! the tests here cover what depends on SQLite itself: persistence across a
//! reopen, the `BEGIN IMMEDIATE` arbiter under racing writers, and the
//! pre-retention row layout.

use std::path::PathBuf;
use std::sync::Arc;

use unitycatalog_delta_api::coordinator::{CommitCoordinator, CommitError, CommitInfo};
use unitycatalog_delta_api::testing::coordinator_conformance::{self, commit_info};
use unitycatalog_sqlite::{SqliteCommitCoordinator, connect_pool, unified_migrator};

/// A temp-file SQLite path that cleans up its files on drop.
struct TempDb {
    path: PathBuf,
}

impl TempDb {
    fn new(tag: &str) -> Self {
        let mut path = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!(
            "uc-sqlite-cc-{tag}-{}-{nanos}.db",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        TempDb { path }
    }

    fn path(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }
}

impl Drop for TempDb {
    fn drop(&mut self) {
        for suffix in ["", "-wal", "-shm"] {
            let _ = std::fs::remove_file(format!("{}{suffix}", self.path.display()));
        }
    }
}

async fn pool(temp: &TempDb) -> sqlx::SqlitePool {
    let pool = connect_pool(temp.path()).await.expect("connect");
    unified_migrator().run(&pool).await.expect("migrate");
    pool
}

fn table_id() -> String {
    uuid::Uuid::now_v7().to_string()
}

#[tokio::test]
async fn conformance() {
    let temp = TempDb::new("conformance");
    let pool = pool(&temp).await;
    coordinator_conformance::run_all(|limits| {
        SqliteCommitCoordinator::new(pool.clone()).with_limits(limits)
    })
    .await;
}

#[tokio::test]
async fn commits_persist_across_reopen() {
    let temp = TempDb::new("persist");
    let t = table_id();
    {
        let cc = SqliteCommitCoordinator::new(pool(&temp).await);
        cc.commit(&t, Some(commit_info(1)), None).await.unwrap();
        cc.commit(&t, Some(commit_info(2)), None).await.unwrap();
    }
    // Reopen the same file: ratified commits survive the restart.
    let cc = SqliteCommitCoordinator::new(pool(&temp).await);
    let (commits, latest) = cc.get_commits(&t, 0, None).await.unwrap();
    assert_eq!(latest, 2);
    assert_eq!(
        commits.iter().map(|c| c.version).collect::<Vec<_>>(),
        vec![1, 2]
    );
}

/// Race `writers` commits at version 2 (distinct files when `distinct`, else the
/// same file) and count (wins, conflicts, replays).
async fn race(tag: &str, writers: usize, distinct: bool) -> (usize, usize, usize) {
    let temp = TempDb::new(tag);
    let cc = Arc::new(SqliteCommitCoordinator::new(pool(&temp).await));
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

#[tokio::test]
async fn first_writer_wins_under_concurrency() {
    assert_eq!(race("race-distinct", 8, true).await, (1, 7, 0));
}

#[tokio::test]
async fn concurrent_identical_retries_are_replays() {
    assert_eq!(race("race-identical", 8, false).await, (1, 0, 7));
}

/// Rows written before retention (backfilled rows deleted, only a fully
/// backfilled latest row flagged) read correctly under the new rule.
#[tokio::test]
async fn pre_retention_layout_reads_correctly() {
    let temp = TempDb::new("layout");
    let pool = pool(&temp).await;
    let cc = SqliteCommitCoordinator::new(pool.clone());
    let t = table_id();
    for v in 1..=3 {
        cc.commit(&t, Some(commit_info(v)), None).await.unwrap();
    }
    let id = uuid::Uuid::parse_str(&t).unwrap().as_bytes().to_vec();
    sqlx::query("DELETE FROM delta_commits WHERE table_id = ? AND commit_version < 3")
        .bind(&id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE delta_commits SET is_backfilled_latest = 1 \
         WHERE table_id = ? AND commit_version = 3",
    )
    .bind(&id)
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
