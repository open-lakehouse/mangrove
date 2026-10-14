//! Unity Catalog catalog-managed Delta table support: create + commit through the
//! kernel's [`Committer`](delta_kernel::committer::Committer) framework, backed by the
//! unitycatalog-rs [`DeltaV1Client`](unitycatalog_client::DeltaV1Client).
//!
//! - [`UnityCatalogCommitter`] — the catalog-managed committer (v0 publish, v≥1 stage + ratify).
//! - [`create_managed_table`] — staging → `kernel::create_table` (writes `0.json`) → `createTable`.
//! - [`append_to_managed_table`] — load snapshot → kernel write transaction → commit (v≥1)
//!   with bounded conflict/throttle/ambiguity retry, then best-effort publish + backfill +
//!   metrics (ManagedTablesSpec §"Write to the table").
//!
//! Design + rationale: see open-lakehouse `docs/adr/0010-catalog-managed-table-writes.md`.
//!
//! ## Upstream / follow-ups (tracked here so they travel with the code)
//!
//! - **delta-rs kernel-snapshot constructor**: the catalog-managed read path wraps a kernel
//!   snapshot built with the catalog log tail via `Snapshot::from_kernel_snapshot`, which only
//!   the `roeap/delta-rs` `olai/rust-v1.1.1` fork exposes (alongside a public
//!   `to_delta_predicate`), and reading needs `CatalogManaged` / `VacuumProtocolCheck` in
//!   delta-rs's `ProtocolChecker` reader allow-list (also fork-only; `v2Checkpoint` is upstream
//!   as of 1.1). Upstream all three, then drop the fork pin for the crates.io release.
//! - **Schema/property evolution on append**: ALTER-style commits (protocol/metadata/clustering
//!   changes) are rejected, not propagated — see `UnityCatalogCommitter`'s
//!   `validate_no_alter_table_changes`. Lifting this requires bundling
//!   `set-columns`/`set-properties`/`set-protocol` actions into the same `updateTable` call as
//!   `add-commit`.
//! - **Upstream home**: keep this glue dependency-light so it can move toward
//!   `delta-rs/crates/catalog-unity`, or adopt the buoyant `delta-kernel-unity-catalog` /
//!   `unity-catalog-delta-rest-client` crates once they stabilize.

mod append;
mod committer;
mod create;

pub use append::append_to_managed_table;
pub use committer::UnityCatalogCommitter;
pub use create::{
    CreateManagedTableError, ManagedTable, create_managed_table,
    get_final_required_properties_for_uc, get_required_properties_for_disk,
};
