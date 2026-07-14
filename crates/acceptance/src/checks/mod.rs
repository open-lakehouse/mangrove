//! API-coverage checks, organized by securable.
//!
//! Each check is a free `pub async fn <name>(ctx: &JourneyContext) ->
//! AcceptanceResult<()>` that drives one API surface end-to-end against a live
//! server and asserts observable behavior. Checks are registered into the
//! [`baseline_checks`](crate::conformance::baseline_checks) /
//! [`extended_checks`](crate::conformance::extended_checks) batteries.
//!
//! Conventions every check follows:
//! - **Unique resource names** via a timestamp/uuid suffix, so checks are isolated
//!   from each other and from prior runs against a long-lived server (see [`unique`]).
//! - **Cleanup on every path** via [`with_cleanup`], so a failed check still tears
//!   down what it created and does not leak resources into later checks.
//! - **Self-skip** (via [`crate::conformance::skip`]) when a prerequisite is absent
//!   (no cloud identity/storage) or the target does not implement the surface.

use std::future::Future;

use crate::AcceptanceResult;

pub mod agents;
pub mod catalog;
pub mod credential;
pub mod cross;
pub mod function;
pub mod governance;
pub mod managed_delta;
pub mod sharing;
pub mod table;
pub mod temp_creds;
pub mod volume;

/// A collision-resistant suffix for resource names within a check run.
///
/// Combines a second-granularity timestamp (readable in server logs) with a short
/// random component (so two checks sharing a name prefix cannot collide even within
/// the same second).
pub fn unique(prefix: &str) -> String {
    let ts = chrono::Utc::now().timestamp();
    let rnd = uuid::Uuid::new_v4().simple().to_string();
    format!("{prefix}_{ts}_{}", &rnd[..8])
}

/// Run `body`, then always run `cleanup` (even if `body` errored), and return the
/// body's result. This replaces the old trait executor's guarantee that a journey's
/// `cleanup()` runs on the failure path.
///
/// `cleanup` is best-effort: its own errors are ignored (deletes of resources that
/// were never created, or already gone, are expected).
pub async fn with_cleanup<B, C, Fb, Fc>(body: B, cleanup: C) -> AcceptanceResult<()>
where
    B: FnOnce() -> Fb,
    C: FnOnce() -> Fc,
    Fb: Future<Output = AcceptanceResult<()>>,
    Fc: Future<Output = ()>,
{
    let result = body().await;
    cleanup().await;
    result
}
