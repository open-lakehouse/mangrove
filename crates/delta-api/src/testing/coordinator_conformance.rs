//! A conformance suite for [`CommitCoordinator`] implementations.
//!
//! Every coordinator — the in-memory one, mangrove's Postgres and SQLite ones, or
//! a downstream backend's — must answer the commit protocol identically:
//! onboarding, the version-gap and field rules, retries recognized by file name
//! (before and after backfill), conflicts, the unbackfilled cap, backfill
//! flagging and retention, and the `get_commits` live window. Run
//! [`run_all`] from a test with a factory that builds a fresh coordinator for the
//! given [`CommitLimits`]:
//!
//! ```ignore
//! #[tokio::test]
//! async fn conformance() {
//!     unitycatalog_delta_api::testing::coordinator_conformance::run_all(|limits| {
//!         MyCoordinator::new(pool.clone()).with_limits(limits)
//!     })
//!     .await;
//! }
//! ```
//!
//! Each check uses its own freshly generated (UUID) table id, so the factory may
//! hand out coordinators sharing one database. Concurrency (first-writer-wins
//! under racing inserts) depends on the backend's arbiter and is left to each
//! backend's own tests.

use crate::coordinator::{
    CommitCoordinator, CommitError, CommitInfo, CommitLimits, DEFAULT_MAX_UNBACKFILLED_COMMITS,
};

/// A fresh table id.
fn table() -> String {
    uuid::Uuid::now_v7().to_string()
}

/// A valid commit at `version`, with a deterministic file name.
pub fn commit_info(version: i64) -> CommitInfo {
    CommitInfo {
        version,
        timestamp: 1_000 + version,
        file_name: format!("{version:020}.00000000-0000-0000-0000-00000000000a.json"),
        file_size: 128,
        file_modification_timestamp: 2_000 + version,
    }
}

/// A different commit (another writer's file) at `version`.
pub fn other_commit_info(version: i64) -> CommitInfo {
    CommitInfo {
        file_name: format!("{version:020}.00000000-0000-0000-0000-00000000000b.json"),
        ..commit_info(version)
    }
}

fn limits(max_unbackfilled_commits: i64, backfilled_commits_retained: i64) -> CommitLimits {
    CommitLimits {
        max_unbackfilled_commits,
        backfilled_commits_retained,
    }
}

async fn versions<C: CommitCoordinator>(cc: &C, t: &str) -> (Vec<i64>, i64) {
    let (commits, latest) = cc.get_commits(t, 0, None).await.expect("get_commits");
    (commits.iter().map(|c| c.version).collect(), latest)
}

async fn commit_all<C: CommitCoordinator>(
    cc: &C,
    t: &str,
    versions: std::ops::RangeInclusive<i64>,
) {
    for v in versions {
        cc.commit(t, Some(commit_info(v)), None)
            .await
            .unwrap_or_else(|e| panic!("commit {v}: {e}"));
    }
}

/// Run every check. Panics (with the check's name) on the first violation.
pub async fn run_all<C, F>(make: F)
where
    C: CommitCoordinator,
    F: Fn(CommitLimits) -> C,
{
    let default = || make(CommitLimits::default());

    // --- onboarding, round-trip and empty tables ---
    {
        let (cc, t) = (default(), table());
        commit_all(&cc, &t, 1..=2).await;
        let (commits, latest) = cc.get_commits(&t, 0, None).await.unwrap();
        assert_eq!(latest, 2, "onboarding: latest");
        assert_eq!(
            commits,
            vec![commit_info(1), commit_info(2)],
            "onboarding: round-trip"
        );

        let (commits, latest) = cc.get_commits(&table(), 0, None).await.unwrap();
        assert!(
            commits.is_empty() && latest == 0,
            "unknown table reports (empty, 0)"
        );
    }

    // --- request validation ---
    {
        let (cc, t) = (default(), table());
        assert!(
            matches!(
                cc.commit(&t, None, None).await,
                Err(CommitError::InvalidArgument(_))
            ),
            "neither commit nor backfill is invalid"
        );
        assert!(
            matches!(
                cc.commit(&t, None, Some(1)).await,
                Err(CommitError::InvalidArgument(_))
            ),
            "backfill with no commits is invalid"
        );
        for bad in [
            CommitInfo {
                version: 0,
                ..commit_info(1)
            },
            CommitInfo {
                timestamp: 0,
                ..commit_info(1)
            },
            CommitInfo {
                file_name: String::new(),
                ..commit_info(1)
            },
            CommitInfo {
                file_size: 0,
                ..commit_info(1)
            },
            CommitInfo {
                file_modification_timestamp: 0,
                ..commit_info(1)
            },
        ] {
            assert!(
                matches!(
                    cc.commit(&t, Some(bad.clone()), None).await,
                    Err(CommitError::InvalidArgument(_))
                ),
                "invalid field rejected: {bad:?}"
            );
        }
        cc.commit(&t, Some(commit_info(1)), None).await.unwrap();
        assert!(
            matches!(
                cc.commit(&t, Some(commit_info(3)), None).await,
                Err(CommitError::InvalidArgument(_))
            ),
            "version gap is invalid"
        );
        assert!(
            matches!(
                cc.commit(&t, None, Some(2)).await,
                Err(CommitError::InvalidArgument(_))
            ),
            "backfill beyond latest is invalid"
        );
        assert!(
            matches!(
                cc.get_commits(&t, 5, Some(2)).await,
                Err(CommitError::InvalidArgument(_))
            ),
            "end before start is invalid"
        );
    }

    // --- retries and conflicts ---
    {
        let (cc, t) = (default(), table());
        commit_all(&cc, &t, 1..=2).await;
        for v in [1, 2] {
            assert!(
                matches!(
                    cc.commit(&t, Some(commit_info(v)), None).await,
                    Err(CommitError::AlreadyAccepted)
                ),
                "retry of v{v} is AlreadyAccepted"
            );
            assert!(
                matches!(
                    cc.commit(&t, Some(other_commit_info(v)), None).await,
                    Err(CommitError::VersionConflict(_))
                ),
                "another file at v{v} conflicts"
            );
        }
        assert_eq!(
            versions(&cc, &t).await,
            (vec![1, 2], 2),
            "retries change nothing"
        );
        assert_eq!(
            cc.find_commit(&t, 2).await.unwrap(),
            Some(commit_info(2)),
            "find_commit"
        );
        assert_eq!(
            cc.find_commit(&t, 3).await.unwrap(),
            None,
            "find_commit: absent"
        );
    }

    // --- backfill hides rows, keeps the latest, stays findable ---
    {
        let (cc, t) = (default(), table());
        commit_all(&cc, &t, 1..=5).await;
        cc.commit(&t, None, Some(3)).await.unwrap();
        assert_eq!(versions(&cc, &t).await, (vec![4, 5], 5), "partial backfill");
        cc.commit(&t, None, Some(1)).await.unwrap();
        assert_eq!(
            versions(&cc, &t).await,
            (vec![4, 5], 5),
            "backfill never moves back"
        );
        cc.commit(&t, None, Some(5)).await.unwrap();
        assert_eq!(
            versions(&cc, &t).await,
            (vec![], 5),
            "full backfill keeps latest version"
        );
        assert!(
            matches!(
                cc.commit(&t, Some(commit_info(2)), None).await,
                Err(CommitError::AlreadyAccepted)
            ),
            "retry recognized after backfill (within retention)"
        );
        assert_eq!(
            cc.find_commit(&t, 2).await.unwrap(),
            Some(commit_info(2)),
            "backfilled row findable"
        );
        cc.commit(&t, Some(commit_info(6)), Some(5)).await.unwrap();
        assert_eq!(
            versions(&cc, &t).await,
            (vec![6], 6),
            "commit with piggy-backed backfill"
        );
    }

    // --- retention ---
    {
        let (cc, t) = (make(limits(DEFAULT_MAX_UNBACKFILLED_COMMITS, 1)), table());
        commit_all(&cc, &t, 1..=4).await;
        cc.commit(&t, None, Some(4)).await.unwrap();
        assert_eq!(
            cc.find_commit(&t, 1).await.unwrap(),
            None,
            "pruned past retention"
        );
        assert!(
            cc.find_commit(&t, 3).await.unwrap().is_some(),
            "retained within window"
        );
        assert!(
            matches!(
                cc.commit(&t, Some(commit_info(1)), None).await,
                Err(CommitError::StateUnknown(_))
            ),
            "retry older than retention is StateUnknown"
        );
        assert_eq!(versions(&cc, &t).await.1, 4, "latest row never pruned");
    }

    // --- unbackfilled cap ---
    {
        let (cc, t) = (make(limits(3, 100)), table());
        commit_all(&cc, &t, 1..=3).await;
        assert!(
            matches!(
                cc.commit(&t, Some(commit_info(4)), None).await,
                Err(CommitError::ResourceExhausted(_))
            ),
            "cap enforced"
        );
        cc.commit(&t, Some(commit_info(4)), Some(1)).await.unwrap();
        cc.commit(&t, None, Some(4)).await.unwrap();
        // The fully backfilled latest row is backfilled: a whole window fits.
        commit_all(&cc, &t, 5..=7).await;
        assert!(
            matches!(
                cc.commit(&t, Some(commit_info(8)), None).await,
                Err(CommitError::ResourceExhausted(_))
            ),
            "cap counts only unbackfilled rows"
        );
    }

    // --- get_commits window ---
    {
        let (cc, t) = (make(limits(2, 100)), table());
        commit_all(&cc, &t, 1..=2).await;
        cc.commit(&t, None, Some(1)).await.unwrap();
        cc.commit(&t, Some(commit_info(3)), None).await.unwrap();
        assert_eq!(
            versions(&cc, &t).await,
            (vec![2, 3], 3),
            "window paginated to the cap"
        );
        let (commits, latest) = cc.get_commits(&t, 0, Some(2)).await.unwrap();
        assert_eq!(
            (
                commits.iter().map(|c| c.version).collect::<Vec<_>>(),
                latest
            ),
            (vec![2], 3),
            "end_version respected"
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::coordinator::InMemoryCommitCoordinator;

    #[tokio::test]
    async fn in_memory_coordinator_conforms() {
        super::run_all(InMemoryCommitCoordinator::with_limits).await;
    }
}
