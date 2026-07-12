//! Embedded, file-based SQLite backend for the Unity Catalog server.
//!
//! This crate mirrors [`unitycatalog-postgres`](../unitycatalog_postgres) but
//! targets an in-process SQLite database, so a durable Unity Catalog server can
//! run from a single binary with zero external infrastructure.
//!
//! The composed [`SqliteGraphStore`] implements the generic
//! [`olai_store::ObjectStore`] / [`olai_store::AssociationStore`] traits (over the
//! project's `ObjectLabel`), which the blanket `ObjectStoreAdapter` in
//! `unitycatalog-common` lifts to the high-level `ResourceStore` API. Sensitive
//! fields (credentials, tokens) are sealed inline on the object rows by the
//! `ManagedObjectStore` layer, and [`SqliteCommitCoordinator`] provides durable
//! Delta catalog-managed commits.
//!
//! ## Known gaps relative to the Postgres backend
//!
//! - **ASCII-only case-insensitivity.** Object names use SQLite's built-in
//!   `NOCASE` collation, which folds case for ASCII only. Postgres uses an ICU
//!   `case_insensitive` collation. Unicode-cased duplicate names that Postgres
//!   would reject may both be accepted here.

pub use crate::commit_coordinator::SqliteCommitCoordinator;
pub use crate::error::{Error, Result};
pub use crate::store::{SqliteGraphStore, connect_graph, connect_pool, unified_migrator};

mod commit_coordinator;
mod error;
mod store;
