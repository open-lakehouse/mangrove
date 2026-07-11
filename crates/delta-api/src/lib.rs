#![cfg_attr(docsrs, feature(doc_cfg))]
//! Portable Unity Catalog **Delta v1** REST API.
//!
//! This crate owns the Delta API *semantics* — the hand-written wire models, the
//! catalog-managed table contract, the commit coordinator, and the `updateTable`
//! action dispatcher — behind a narrow backend [port](DeltaBackend). Any
//! server can serve the identical `/delta/v1` surface by implementing
//! [`DeltaBackend`] over its own storage, credential vending, and authorization;
//! all the Delta business logic is shared here, so the behavior is identical by
//! construction.
//!
//! It depends only on `axum`, `async-trait`, `serde`, `uuid`, and `thiserror` — it
//! does **not** depend on any server crate, so a downstream server (mangrove,
//! lakekeeper, …) takes this single dependency and nothing else.
//!
//! # Examples
//!
//! Implement [`DeltaBackend`] over your own storage, then hand it to
//! [`get_router`] to serve the entire `/delta/v1` surface. Here the in-memory
//! backend from the `testing` feature stands in for a real one:
//!
//! ```
//! # #[cfg(feature = "testing")] {
//! use unitycatalog_delta_api::get_router;
//! use unitycatalog_delta_api::testing::InMemoryDeltaBackend;
//!
//! // `InMemoryDeltaBackend` implements `DeltaBackend<()>`, so the context type
//! // is `()`. A real server uses its own request-context type here.
//! let router: axum::Router = get_router::<InMemoryDeltaBackend, ()>(InMemoryDeltaBackend::new());
//! # let _ = router;
//! # }
//! ```
//!
//! # Layout
//! - [`models`] — hand-written serde wire DTOs (kebab-case JSON).
//! - [`error`] — the decoupled error contract ([`DeltaApiError`] +
//!   [`DeltaBackendError`]).
//! - [`mod@column`] — the portable UC column model used by the contract.
//! - [`config`] — `getConfig` support: capability-driven endpoint list + protocol
//!   version negotiation.
//! - [`contract`] — the managed-table contract + Delta↔UC column mapping.
//! - [`coordinator`] — the commit coordinator (arbitration + backfill).
//! - [`authz`] — the [`DeltaAction`] vocabulary the handler authorizes with.
//! - [`backend`] — the `DeltaBackend` port trait + its coordinate/request types.
//! - [`handler`] — the `DeltaApiHandler` trait + the generic blanket impl.
//! - [`router`] — the axum router mounting all 12 operations.

pub mod authz;
pub mod backend;
pub mod column;
pub mod config;
pub mod contract;
pub mod coordinator;
pub mod error;
pub mod handler;
pub mod models;
pub mod router;
#[cfg(feature = "testing")]
pub mod testing;

pub use authz::DeltaAction;
pub use backend::{DeltaBackend, DeltaCapabilities};
pub use error::{DeltaApiError, DeltaApiResult, DeltaBackendError};
pub use handler::DeltaApiHandler;
pub use router::get_router;
