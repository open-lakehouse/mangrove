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
//! [`get_router`] returns a fully-stated `Router<()>`, convenient for a host whose
//! whole router tree is `Router<()>`. To mount the surface **inside a host that
//! carries its own axum state** (its own middleware, its own `.with_state`), use
//! [`router_with_context`] (or [`router_from_extension`]): both return an
//! *unstated* `Router<S>` you compose with plain `.nest`/`.merge` and state
//! yourself. The context `Cx` is produced by a host-supplied closure rather than an
//! axum `FromRequestParts` impl, so no context glue is required:
//!
//! ```
//! # #[cfg(feature = "testing")] {
//! use std::sync::Arc;
//! use unitycatalog_delta_api::{ContextExtractor, router_with_context_at};
//! use unitycatalog_delta_api::testing::InMemoryDeltaBackend;
//!
//! // The host's own axum state (any `Clone + Send + Sync + 'static`).
//! #[derive(Clone)]
//! struct AppState;
//!
//! let extract_cx: ContextExtractor<()> = Arc::new(|_parts| Box::pin(async { Ok(()) }));
//! // `base = ""` yields relative routes; the host adds the `/delta/v1` prefix via
//! // `.nest`. (Pass `"/delta/v1"` instead to `.merge` the surface directly.)
//! let delta: axum::Router<AppState> =
//!     router_with_context_at("", Arc::new(InMemoryDeltaBackend::new()), extract_cx);
//!
//! // Compose into the host tree *before* `.with_state`.
//! let app: axum::Router = axum::Router::new()
//!     .nest("/delta/v1", delta)
//!     .with_state(AppState);
//! # let _ = app;
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
//! - [`router`] — the state-agnostic, host-composable axum router mounting all 12
//!   operations.

// `models` is the portable wire-DTO layer (no axum); it is the only part a wasm
// client needs, so it is always available. Everything else is the server-side
// surface (the `DeltaBackend` port, handler, axum router, error IntoResponse) and
// lives behind the default-on `server` feature.
pub mod models;

#[cfg(feature = "server")]
pub mod authz;
#[cfg(feature = "server")]
pub mod backend;
#[cfg(feature = "server")]
pub mod column;
#[cfg(feature = "server")]
pub mod config;
#[cfg(feature = "server")]
pub mod contract;
#[cfg(feature = "server")]
pub mod coordinator;
#[cfg(feature = "server")]
pub mod error;
#[cfg(feature = "server")]
pub mod handler;
#[cfg(feature = "server")]
pub mod router;
#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "server")]
pub use authz::DeltaAction;
#[cfg(feature = "server")]
pub use backend::{DeltaBackend, DeltaCapabilities};
#[cfg(feature = "server")]
pub use error::{DeltaApiError, DeltaApiResult, DeltaBackendError};
#[cfg(feature = "server")]
pub use handler::DeltaApiHandler;
#[cfg(feature = "server")]
pub use router::{
    ContextExtractor, get_router, router_from_extension, router_from_extension_at,
    router_with_context, router_with_context_at,
};
