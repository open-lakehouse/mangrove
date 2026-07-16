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
//! the store and policy traits by delegation — so the generated handler traits in
//! [`api`] can be blanket-implemented over it. To swap a backend, construct the
//! handler with a different trait object; no handler code changes.
//!
//! # Layout
//!
//! - [`api`] — per-resource handler traits (generated) plus their hand-written
//!   business logic, and the [`SecuredAction`](api::SecuredAction) permission
//!   mapping.
//! - [`policy`] — the [`Policy`](policy::Policy) authorization trait,
//!   [`Permission`](policy::Permission)/[`Decision`](policy::Decision), and the
//!   allow-all [`ConstantPolicy`](policy::ConstantPolicy).
//! - [`store`] — storage-abstraction traits, re-exported from
//!   `unitycatalog-common`.
//! - [`rest`] — the `axum` routers that expose the handlers over HTTP.
//! - [`services`] — [`ServerHandler`](services::ServerHandler) and the
//!   `Provides*` dependency-injection traits.
//! - [`handlers`] — reusable handler patterns (e.g. proxy leaves that forward to
//!   an upstream catalog).
//!
//! # Feature flags
//!
//! `memory` enables the in-memory store; `proxy` pulls in the upstream-forwarding
//! handlers; `bin` adds the config/CLI/serve wiring used by the `uc-server`
//! binary. See the crate README for the full table.

pub mod api;
mod codegen;
pub mod error;
pub mod handlers;
#[cfg(feature = "memory")]
pub mod memory;
pub mod policy;
pub mod rest;
pub mod services;
pub mod store;
pub mod telemetry;

// Deployable-binary support: config loading, the CLI/subcommand surface, and the
// server-launch wiring used by the `uc-server` binary (see `src/main.rs`). Gated
// behind `bin` so a plain library build doesn't pull the CLI/serve/store stack.
#[cfg(feature = "bin")]
pub mod cli;
#[cfg(feature = "bin")]
pub mod config;
#[cfg(feature = "bin")]
pub mod hybrid;
#[cfg(feature = "bin")]
pub mod run;

pub use crate::error::{Error, Result};
