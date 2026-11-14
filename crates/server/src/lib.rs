//! The Unity Catalog REST API server.
//!
//! This crate implements the Unity Catalog REST surface as a set of `axum`
//! routers over three pluggable dependencies, injected as trait objects:
//!
//! - a [`ResourceStore`](store::ResourceStore) — the persistence backend
//!   (in-memory, `unitycatalog-postgres`, or `unitycatalog-sqlite`);
//! - a [`Policy`](policy::Policy) — the authorization decision point; and
//! - a [`CommitCoordinator`](unitycatalog_delta_api::coordinator::CommitCoordinator)
//!   — the Delta catalog-managed commit backend.
//!
//! [`ServerHandler`](services::ServerHandler) composes those dependencies. It is
//! generic over an authorization context `Cx` (the identity or request state a
//! [`Policy`](policy::Policy) evaluates against), cheap to clone, and satisfies
//! the store and policy traits by delegation; the generated handler traits in
//! [`api`] are implemented on `ServerHandler<RequestContext>`. To swap a backend,
//! construct the handler with a different trait object; no handler code changes.
//!
//! # Layout
//!
//! - [`api`] — the per-resource handler traits (generated into
//!   `unitycatalog-handlers`) re-exported with their hand-written business logic,
//!   and the grants API.
//! - [`policy`] — the [`Policy`](policy::Policy) authorization trait over the
//!   generated [`Operation`](policy::Operation)s, [`Facts`](policy::Facts),
//!   [`Decision`](policy::Decision), and the allow-all
//!   [`ConstantPolicy`](policy::ConstantPolicy).
//! - [`grants`] — the grant store, the principal directory and ownership.
//! - [`store`] — storage-abstraction traits, re-exported from
//!   `unitycatalog-common`.
//! - [`rest`] — the `axum` routers that expose the handlers over HTTP.
//! - [`services`] — [`ServerHandler`](services::ServerHandler) and the
//!   `Provides*` dependency-injection traits.
//! - [`handlers`] — reusable handler patterns (e.g. the policy-checking
//!   decorator the upstream proxy is built on).
//!
//! # Feature flags
//!
//! `memory` enables the in-memory store; `proxy` pulls in the client handlers the
//! upstream proxy forwards through; `embed` assembles the server from a config
//! without binding a socket ([`embed::start`]), with `postgres` / `sqlite`
//! selecting the backends and `swagger`, `ui`, `storage-proxy` and `files` the
//! optional surfaces; `bin` turns all of them on and adds the CLI used by the
//! `uc-server` binary. See the crate README for the full table.

pub mod api;
pub mod error;
pub mod grants;
pub mod handlers;
#[cfg(feature = "memory")]
pub mod memory;
pub mod policy;
pub mod rest;
pub mod services;
pub mod store;
pub mod telemetry;

// Server assembly: config loading and the launch wiring shared by the
// `uc-server` binary and socket-less hosts (`embed`). Gated behind `embed` so a
// plain library build doesn't pull the config/serve/store stack.
#[cfg(feature = "embed")]
pub mod config;
#[cfg(feature = "embed")]
pub mod embed;
#[cfg(all(feature = "embed", feature = "proxy"))]
pub mod hybrid;
#[cfg(feature = "embed")]
pub mod run;
#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod schema;

// The CLI/subcommand surface of the `uc-server` binary (see `src/main.rs`).
#[cfg(feature = "bin")]
pub mod cli;

pub use crate::error::{Error, Result};
