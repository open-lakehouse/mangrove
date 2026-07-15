//! The conformance battery: a backend-agnostic set of API-coverage checks run
//! against a live Unity Catalog server.
//!
//! Each [`Check`] drives one API surface (a securable's lifecycle) through the
//! [`UnityCatalogClient`](unitycatalog_client::UnityCatalogClient) and asserts
//! observable behavior. Checks are gathered into two sets — [`baseline_checks`]
//! (portable to any UC implementation, including UC OSS) and [`extended_checks`]
//! (everything our Rust server adds on top) — and run against a target with
//! [`run`].
//!
//! ## Known-failing quarantine
//!
//! This suite deliberately *attempts* coverage of surfaces that may not work yet.
//! Rather than fail the whole run on the first broken surface, [`run`] executes
//! every check independently, captures each [`Outcome`], and consults a per-target
//! quarantine table ([`known_failing`]). A check the table marks as known-failing
//! for the active [`Target`] is expected to fail and does **not** break CI; a
//! check that fails while *not* quarantined is a hard regression. A quarantined
//! check that unexpectedly *passes* is surfaced loudly so the flag can be dropped.
//!
//! The quarantine list is the living inventory of "which API surfaces don't work
//! yet" — see `JOURNEY_CATALOG.md` for the fuller picture and follow-up issues.

use std::future::Future;
use std::pin::Pin;

use crate::JourneyContext;
use crate::checks;

/// The live server a battery is being run against.
///
/// Quarantine is resolved per target because a surface can work on our Rust
/// server yet be absent or broken on UC OSS or managed Databricks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Our own Rust `uc-server`.
    OssRust,
    /// The open-source Java Unity Catalog server.
    OssJava,
    /// The Databricks managed Unity Catalog service (reference implementation).
    ManagedDatabricks,
}

impl Target {
    fn as_str(self) -> &'static str {
        match self {
            Target::OssRust => "oss_rust",
            Target::OssJava => "oss_java",
            Target::ManagedDatabricks => "managed_databricks",
        }
    }
}

/// What happened when a check ran.
#[derive(Debug)]
pub enum Outcome {
    /// The check ran and all assertions held.
    Passed,
    /// The check declined to run — an expected-absent surface (no cloud identity
    /// configured, or a securable the target does not implement). Not a failure.
    Skipped(String),
    /// The check ran and failed (assertion or client/transport error).
    Failed(String),
}

type CheckFut<'a> = Pin<Box<dyn Future<Output = crate::AcceptanceResult<()>> + 'a>>;

/// A named coverage check plus the function that runs it.
pub struct Check {
    /// Stable identifier, used in the printed inventory and the quarantine table.
    pub name: &'static str,
    run: for<'a> fn(&'a JourneyContext) -> CheckFut<'a>,
}

impl Check {
    /// Build a check from its name and an async check function.
    pub fn new(name: &'static str, run: for<'a> fn(&'a JourneyContext) -> CheckFut<'a>) -> Self {
        Self { name, run }
    }
}

/// A check function may signal "expected-absent" by returning this sentinel error;
/// [`run`] maps it to [`Outcome::Skipped`] rather than [`Outcome::Failed`].
///
/// Checks construct it via [`skip`].
pub const SKIP_MARKER: &str = "__conformance_skip__:";

/// Return an error that [`run`] records as [`Outcome::Skipped`] with `reason`.
///
/// Use this inside a check when the target does not implement the surface, or a
/// prerequisite (cloud identity, external storage) is not configured.
pub fn skip(reason: impl std::fmt::Display) -> crate::AcceptanceError {
    crate::AcceptanceError::JourneyExecution(format!("{SKIP_MARKER}{reason}"))
}

/// Per-check outcome, paired with whether the check was quarantined for the target.
#[derive(Debug)]
pub struct CheckReport {
    pub name: &'static str,
    pub outcome: Outcome,
    /// `Some(reason)` when the target's quarantine table marks this check known-failing.
    pub quarantined: Option<&'static str>,
}

impl CheckReport {
    /// A quarantined check that failed as expected, or that was skipped, or that
    /// passed while not quarantined — none of these should fail the run.
    fn is_expected(&self) -> bool {
        match (&self.outcome, self.quarantined) {
            // Quarantined + failed = expected failure.
            (Outcome::Failed(_), Some(_)) => true,
            // Not-quarantined failure = regression.
            (Outcome::Failed(_), None) => false,
            // A skip is always fine.
            (Outcome::Skipped(_), _) => true,
            // A pass is fine; a quarantined pass is fine but flagged separately.
            (Outcome::Passed, _) => true,
        }
    }

    /// A quarantined check that unexpectedly passed — the flag should be removed.
    fn is_unexpected_pass(&self) -> bool {
        matches!(self.outcome, Outcome::Passed) && self.quarantined.is_some()
    }
}

/// The full result of running a battery against a target.
#[derive(Debug)]
pub struct Report {
    pub target: Target,
    pub checks: Vec<CheckReport>,
}

impl Report {
    /// Checks that failed without being quarantined — the CI-breaking set.
    pub fn unexpected_failures(&self) -> Vec<&CheckReport> {
        self.checks
            .iter()
            .filter(|c| matches!(c.outcome, Outcome::Failed(_)) && c.quarantined.is_none())
            .collect()
    }

    /// Quarantined checks that unexpectedly passed — prompt to drop the flag.
    pub fn unexpected_passes(&self) -> Vec<&CheckReport> {
        self.checks
            .iter()
            .filter(|c| c.is_unexpected_pass())
            .collect()
    }

    /// True when nothing regressed (every check is expected). Unexpected passes do
    /// not fail the run — they are surfaced as a warning in the inventory.
    pub fn no_unexpected_failures(&self) -> bool {
        self.checks.iter().all(CheckReport::is_expected)
    }

    /// Print a per-check inventory to stdout: the session artifact enumerating
    /// which API surfaces work, are absent, or are broken.
    pub fn print_inventory(&self) {
        println!(
            "\n──────── conformance inventory [{}] ────────",
            self.target.as_str()
        );
        for c in &self.checks {
            let line = match (&c.outcome, c.quarantined) {
                (Outcome::Passed, None) => format!("  ✅ PASS      {}", c.name),
                (Outcome::Passed, Some(reason)) => {
                    format!(
                        "  ⚠️  PASS(!)   {} — quarantined but now passing; drop flag: {reason}",
                        c.name
                    )
                }
                (Outcome::Skipped(why), _) => format!("  ⏭️  SKIP      {} — {why}", c.name),
                (Outcome::Failed(err), Some(reason)) => {
                    format!(
                        "  🟡 XFAIL     {} — known-failing ({reason}): {err}",
                        c.name
                    )
                }
                (Outcome::Failed(err), None) => format!("  ❌ FAIL      {} — {err}", c.name),
            };
            println!("{line}");
        }
        let passed = self
            .checks
            .iter()
            .filter(|c| matches!(c.outcome, Outcome::Passed))
            .count();
        let skipped = self
            .checks
            .iter()
            .filter(|c| matches!(c.outcome, Outcome::Skipped(_)))
            .count();
        let xfail = self
            .checks
            .iter()
            .filter(|c| matches!(c.outcome, Outcome::Failed(_)) && c.quarantined.is_some())
            .count();
        let fail = self.unexpected_failures().len();
        println!(
            "  ── {passed} passed, {skipped} skipped, {xfail} expected-fail, {fail} unexpected-fail\n"
        );
    }
}

/// Run `checks` against the `target`, capturing each outcome and applying the
/// per-target quarantine table. Prints the inventory before returning.
pub async fn run(target: Target, ctx: &JourneyContext, checks: Vec<Check>) -> Report {
    let mut reports = Vec::with_capacity(checks.len());
    for check in checks {
        let outcome = match (check.run)(ctx).await {
            Ok(()) => Outcome::Passed,
            Err(e) => {
                let msg = e.to_string();
                // The skip marker may be nested inside an error's Display prefix
                // (e.g. "Journey execution failed: __conformance_skip__:..."), so
                // search for it rather than requiring it at the start.
                match msg.find(SKIP_MARKER) {
                    Some(idx) => {
                        let reason = &msg[idx + SKIP_MARKER.len()..];
                        Outcome::Skipped(reason.to_string())
                    }
                    None => Outcome::Failed(msg),
                }
            }
        };
        reports.push(CheckReport {
            name: check.name,
            outcome,
            quarantined: known_failing(target, check.name),
        });
    }
    let report = Report {
        target,
        checks: reports,
    };
    report.print_inventory();
    report
}

/// Convenience macro: register a check by name from a `checks::<module>::<fn>` path,
/// boxing the returned future to the erased `CheckFut` type.
macro_rules! check {
    ($name:literal, $path:path) => {
        Check::new($name, |ctx| Box::pin($path(ctx)))
    };
}

/// Checks portable to any UC implementation, including UC OSS Java v0.5.0.
/// This set is the CI baseline.
pub fn baseline_checks() -> Vec<Check> {
    vec![
        check!("catalog_crud", checks::catalog::catalog_crud),
        check!("catalog_hierarchy", checks::catalog::catalog_hierarchy),
        check!("schema_lifecycle", checks::catalog::schema_lifecycle),
        check!(
            "managed_table_lifecycle",
            checks::table::managed_table_lifecycle
        ),
        check!(
            "metric_view_lifecycle",
            checks::table::metric_view_lifecycle
        ),
        check!(
            "managed_volume_lifecycle",
            checks::volume::managed_volume_lifecycle
        ),
        check!("function_lifecycle", checks::function::function_lifecycle),
        check!("function_update", checks::function::function_update),
        check!(
            "registered_model_lifecycle",
            checks::registered_model::registered_model_lifecycle
        ),
        check!(
            "model_version_lifecycle",
            checks::registered_model::model_version_lifecycle
        ),
    ]
}

/// [`baseline_checks`] plus everything only our Rust server (and mostly managed
/// Databricks) supports.
pub fn extended_checks() -> Vec<Check> {
    let mut checks = baseline_checks();
    checks.extend([
        check!(
            "credential_lifecycle",
            checks::credential::credential_lifecycle
        ),
        check!(
            "external_location_lifecycle",
            checks::credential::external_location_lifecycle
        ),
        check!(
            "external_volume_lifecycle",
            checks::volume::external_volume_lifecycle
        ),
        check!(
            "external_table_lifecycle",
            checks::table::external_table_lifecycle
        ),
        check!("table_extended_reads", checks::table::table_extended_reads),
        check!(
            "temporary_table_credentials",
            checks::temp_creds::table_credentials
        ),
        check!(
            "temporary_path_credentials",
            checks::temp_creds::path_credentials
        ),
        check!(
            "temporary_volume_credentials",
            checks::temp_creds::volume_credentials
        ),
        check!("share_lifecycle", checks::sharing::share_lifecycle),
        check!("recipient_lifecycle", checks::sharing::recipient_lifecycle),
        check!("provider_lifecycle", checks::sharing::provider_lifecycle),
        check!("policy_lifecycle", checks::governance::policy_lifecycle),
        check!(
            "tag_policy_lifecycle",
            checks::governance::tag_policy_lifecycle
        ),
        check!(
            "entity_tag_assignment_lifecycle",
            checks::governance::entity_tag_assignment_lifecycle
        ),
        check!("agent_lifecycle", checks::agents::agent_lifecycle),
        check!(
            "agent_skill_lifecycle",
            checks::agents::agent_skill_lifecycle
        ),
        check!("lakehouse_hierarchy", checks::cross::lakehouse_hierarchy),
        check!("governance_setup", checks::cross::governance_setup),
    ]);
    checks
}

/// Per-`(target, check name)` quarantine table.
///
/// `Some(reason)` marks a check as known-failing for that target: it is still
/// attempted, but a failure is expected and does not break CI. Populate this from
/// real live runs; every entry is a follow-up worklist item (see `JOURNEY_CATALOG.md`).
/// Removing an entry is how a fixed surface re-enters the must-pass set.
fn known_failing(target: Target, check_name: &str) -> Option<&'static str> {
    match (target, check_name) {
        // --- OssRust (our server), seeded from live runs 2026-07-14 ---
        // `managed_table_lifecycle` now drives the real /delta/v1 staging flow
        // (createStagingTable → write log → createTable → commit → read back) via
        // `checks::managed_delta`, so it is no longer quarantined. The checks
        // below still create managed tables the *old* way (bare create_table) and
        // remain quarantined until they adopt the same staging helper (#62).
        (Target::OssRust, "share_lifecycle") => Some(
            "creates a managed table via bare create_table; adopt the staging helper (follow-up: #62)",
        ),
        (Target::OssRust, "lakehouse_hierarchy") => Some(
            "creates managed tables via bare create_table; adopt the staging helper (follow-up: #62)",
        ),
        // Temporary table credentials need a managed table created via the staging
        // flow — same follow-up as the checks above.
        (Target::OssRust, "temporary_table_credentials") => {
            Some("needs a managed table via the staging helper (follow-up: #62)")
        }
        // A row-filter policy requires a backing row_filter.function_name; wiring
        // a real function into the policy is deferred.
        (Target::OssRust, "policy_lifecycle") => {
            Some("row-filter policy requires a backing function reference (follow-up: #63)")
        }
        // Temporary volume credentials return 404 on our server today.
        (Target::OssRust, "temporary_volume_credentials") => {
            Some("temporary-volume-credentials returns 404 on our server (follow-up: #63)")
        }
        // Tag policy create returns 405 (method not allowed) on our server.
        (Target::OssRust, "tag_policy_lifecycle") => {
            Some("tag-policies create returns 405 on our server (follow-up: #63)")
        }
        // Entity tag assignment returns 404 on our server today.
        (Target::OssRust, "entity_tag_assignment_lifecycle") => {
            Some("entity-tag-assignments returns 404 on our server (follow-up: #63)")
        }

        // --- OssJava (unitycatalog/unitycatalog:v0.5.0), from live runs 2026-07-14 ---
        // Pre-existing baseline gaps against the Java reference server (see #65).
        // `managed_table_lifecycle` is NO LONGER quarantined here (#66): its `/delta/v1`
        // staging flow + read-back works against Java v0.5.0 on both Linux and macOS.
        // The prior 404 was `list_table_summaries` (GET /table-summaries), an endpoint
        // v0.5.0 does not implement — that call now lives in the Rust-only
        // `table_summaries` check, and the macOS read-back `/tmp` symlink artifact is
        // fixed by canonicalizing the local location in `checks::managed_delta`.
        // `function_lifecycle` is NO LONGER quarantined here (#70): POST /functions now
        // wraps the payload in a `function_info` envelope (proto `body: "function_info"`),
        // matching UC OSS Java v0.5.0 and the Databricks SDK.
        (Target::OssJava, "function_update") => Some(
            "UC OSS v0.5.0 does not implement function update — PATCH /functions/{name} \
             returns 405 (follow-up: #70)",
        ),
        _ => None,
    }
}
