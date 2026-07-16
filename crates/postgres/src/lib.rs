//! PostgreSQL-backed resource and secret store for the Unity Catalog server.
//!
//! This crate is the production persistence backend for `unitycatalog-server`. The
//! composed [`PgGraphStore`] implements the generic
//! [`olai_store::ObjectStore`] / [`olai_store::AssociationStore`] traits (over the
//! project's `ObjectLabel`), which the blanket `ObjectStoreAdapter` in
//! `unitycatalog-common` lifts to the high-level `ResourceStore` API. Sensitive
//! fields (credentials, tokens) are sealed inline on the object rows by an
//! `olai_store::ManagedObjectStore` layer, and [`PgCommitCoordinator`] provides
//! durable Delta catalog-managed commits over the same connection pool.
//!
//! Its SQLite sibling is [`unitycatalog-sqlite`](../unitycatalog_sqlite); the two
//! share the same store contract and differ only in the embedded engine and a few
//! documented gaps (e.g. Unicode-aware case folding, which SQLite lacks).
//!
//! # Getting started
//!
//! Connect with [`connect_graph`] (or [`connect_pool`] for a bare pool) and run
//! [`unified_migrator`] to apply both the `olai_store` object-graph schema and the
//! crate-local `delta_commits` migrations in one ordered ledger.

pub use crate::commit_coordinator::PgCommitCoordinator;
pub use crate::error::{Error, Result};
pub use graph::*;

mod commit_coordinator;
mod error;
mod graph;
