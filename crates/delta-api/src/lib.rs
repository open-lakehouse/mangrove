//! Portable Unity Catalog **Delta v1** REST API.
//!
//! This crate owns the Delta API *semantics* — the hand-written wire models, the
//! catalog-managed table contract, the commit coordinator, and the `updateTable`
//! action dispatcher — behind a narrow backend [port](backend::DeltaBackend). Any
//! server can serve the identical `/delta/v1` surface by implementing
//! [`DeltaBackend`](backend::DeltaBackend) over its own storage, credential
//! vending, and authorization; all the Delta business logic is shared here, so the
//! behavior is identical by construction.
//!
//! It depends only on `axum`, `async-trait`, `serde`, `uuid`, and `thiserror` — it
//! does **not** depend on any server crate, so a downstream server (mangrove,
//! lakekeeper, …) takes this single dependency and nothing else.
//!
//! # Layout
//! - [`models`] — hand-written serde wire DTOs (kebab-case JSON).
//! - [`error`] — the decoupled error contract ([`DeltaApiError`](error::DeltaApiError) +
//!   [`DeltaBackendError`](error::DeltaBackendError)).
//! - [`column`] — the portable UC column model used by the contract.
//! - [`contract`] — the managed-table contract + Delta↔UC column mapping.
//! - [`coordinator`] — the commit coordinator (arbitration + backfill).
//! - [`backend`] — the `DeltaBackend` port trait + its coordinate/request types.
//! - [`handler`] — the `DeltaApiHandler` trait + the generic blanket impl.
//! - [`router`] — the axum router mounting all 12 operations.

pub mod backend;
pub mod column;
pub mod contract;
pub mod coordinator;
pub mod error;
pub mod handler;
pub mod models;
pub mod router;
#[cfg(feature = "testing")]
pub mod testing;

pub use backend::DeltaBackend;
pub use error::{DeltaApiError, DeltaApiResult, DeltaBackendError};
pub use handler::DeltaApiHandler;
pub use router::get_router;
