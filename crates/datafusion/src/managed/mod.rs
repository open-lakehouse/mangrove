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
//! - **delta-rs read path**: upstream delta-rs `main` (2.0, delta-rs#4809) reads catalog-managed
//!   tables — `Snapshot::try_new_with_engine_and_catalog` takes our UC log tail as a
//!   `CatalogLogTail`, and `CatalogManaged` / `VacuumProtocolCheck` are supported reader
//!   features. The `roeap/delta-rs` `olai/catalog-managed` pin adds only a public
//!   `to_delta_predicate` (log explorer pushdown); drop the fork once 2.0 ships with it.
//! - **delta-rs write path**: delta-rs can't write catalog-managed tables yet (no ICT writes,
//!   `CatalogManaged` is not a writer feature, no catalog implementation of its
//!   `logstore::Committer` trait — delta-rs#4549). Until it does, creates and appends go
//!   through the kernel `Transaction` + [`UnityCatalogCommitter`] here.
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
