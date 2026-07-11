//! Shared types and utilities for the Unity Catalog server and client crates.
//!
//! Most of the crate is the generated Unity Catalog data model, produced from the
//! protobuf definitions in `proto/` and re-exported from [`models`]. On top of that
//! it collects the hand-written pieces both sides of the API depend on:
//!
//! - [`error`] — the crate-wide [`Error`] and [`Result`] types.
//! - [`reference`](mod@reference) — the `uc://` URL scheme for addressing catalog
//!   securables ([`UCReference`]).
//! - [`store`] — storage-abstraction traits ([`ResourceStore`](store::ResourceStore),
//!   `SecretManager`) implemented by backends (feature `store`).
//! - [`services`] — envelope encryption and secret management (feature `store`).
//! - [`metric_view`] — the single parser for Unity Catalog metric-view definitions
//!   (feature `metric-view`).
//!
//! # Feature flags
//!
//! The crate is feature-flag heavy so that downstream crates pull in only what they
//! need. `rest-client` is on by default; `grpc`, `axum`, `sqlx`, `store`,
//! `metric-view`, `python`, and `node` gate the corresponding integrations. See the
//! crate README for the full table.

pub use error::{Error, Result};
pub use models::*;
pub use reference::UCReference;

pub mod error;
#[cfg(feature = "metric-view")]
pub mod metric_view;
pub mod models;
#[cfg(feature = "python")]
pub mod python;
pub mod reference;
#[cfg(feature = "store")]
pub mod services;
#[cfg(feature = "store")]
pub mod store;
