//! Conformance entry points: run the API-coverage battery against a live server.
//!
//! Each test is gated on the target's URL env var and skips (returns early, prints
//! a note) when it is unset — so `cargo test -p unitycatalog-acceptance` is a no-op
//! without a server, and CI opts a target in by setting its variable. A run passes
//! iff there are no *unexpected* failures (quarantined surfaces may fail; absent
//! surfaces skip). See `crate::conformance` for the quarantine mechanism.

use unitycatalog_acceptance::JourneyContext;
use unitycatalog_acceptance::conformance::{Target, baseline_checks, extended_checks, run};

/// Default storage root for the OSS servers (local filesystem inside the container).
const DEFAULT_OSS_STORAGE_ROOT: &str = "file:///tmp/uc-test/";

fn storage_root(default: &str) -> String {
    std::env::var("UC_INTEGRATION_STORAGE_ROOT").unwrap_or_else(|_| default.to_string())
}

/// Full battery against our own Rust `uc-server`. The new default validation.
// Multi-thread flavor: the managed-table check reads back through delta-rs, whose
// kernel executor calls `block_on` internally — that panics on a current-thread
// runtime ("Cannot start a runtime from within a runtime") but is fine when a
// worker thread can block while others drive tasks.
#[tokio::test(flavor = "multi_thread")]
async fn conformance_oss_rust() {
    let Ok(url) = std::env::var("UC_RUST_URL") else {
        eprintln!("skipping conformance_oss_rust: UC_RUST_URL not set");
        return;
    };
    let ctx = JourneyContext::live(&url, storage_root(DEFAULT_OSS_STORAGE_ROOT))
        .expect("build oss_rust context");
    let report = run(Target::OssRust, &ctx, extended_checks()).await;
    assert!(
        report.no_unexpected_failures(),
        "unexpected conformance failures against oss_rust: {:?}",
        report
            .unexpected_failures()
            .iter()
            .map(|c| c.name)
            .collect::<Vec<_>>()
    );
}

/// Portable baseline against the open-source Java Unity Catalog server.
// Multi-thread flavor — see `conformance_oss_rust`.
#[tokio::test(flavor = "multi_thread")]
async fn conformance_oss_java() {
    let Ok(url) = std::env::var("UC_OSS_JAVA_URL") else {
        eprintln!("skipping conformance_oss_java: UC_OSS_JAVA_URL not set");
        return;
    };
    let ctx = JourneyContext::live(&url, storage_root(DEFAULT_OSS_STORAGE_ROOT))
        .expect("build oss_java context")
        .with_managed_volume_needs_catalog_storage_root(true);
    let report = run(Target::OssJava, &ctx, baseline_checks()).await;
    assert!(
        report.no_unexpected_failures(),
        "unexpected conformance failures against oss_java: {:?}",
        report
            .unexpected_failures()
            .iter()
            .map(|c| c.name)
            .collect::<Vec<_>>()
    );
}

/// On-demand full battery against managed Databricks (the reference implementation).
/// Never runs in CI — no fixtures, gated on real workspace credentials.
// Multi-thread flavor — see `conformance_oss_rust`.
#[tokio::test(flavor = "multi_thread")]
async fn conformance_managed_databricks() {
    let (Ok(url), Ok(token)) = (
        std::env::var("UC_DATABRICKS_URL"),
        std::env::var("UC_DATABRICKS_TOKEN"),
    ) else {
        eprintln!("skipping conformance_managed_databricks: UC_DATABRICKS_URL/TOKEN not set");
        return;
    };
    let root = std::env::var("UC_DATABRICKS_STORAGE_ROOT")
        .expect("UC_DATABRICKS_STORAGE_ROOT required for the managed_databricks target");
    let ctx = JourneyContext::live_with_token(&url, &token, root).expect("build managed context");
    let report = run(Target::ManagedDatabricks, &ctx, extended_checks()).await;
    assert!(
        report.no_unexpected_failures(),
        "unexpected conformance failures against managed_databricks: {:?}",
        report
            .unexpected_failures()
            .iter()
            .map(|c| c.name)
            .collect::<Vec<_>>()
    );
}
