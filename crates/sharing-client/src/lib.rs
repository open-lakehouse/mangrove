//! Delta Sharing protocol client and shared wire types for Unity Catalog.
//!
//! This crate provides a [`DeltaSharingClient`](client::DeltaSharingClient) for
//! consuming the Delta Sharing REST API — discovering shares, schemas, and
//! tables, and reading table metadata and data — along with the request/response
//! [`models`] that describe the protocol on the wire. Those models are shared:
//! the server-side `unitycatalog-sharing-api` crate serves the same types this
//! client sends.
//!
//! # Feature flags
//!
//! `axum` compiles the request extractors used by a server built on these types
//! (including the hand-written NDJSON query-path extractors); a plain client
//! build does not need it.

// The generated client/extractor code refers to this crate by its external name
// (`unitycatalog_sharing_client::...`); alias `self` so those paths resolve from
// within the crate.
extern crate self as unitycatalog_sharing_client;

pub use crate::error::{Error, Result};

pub mod client;
mod codegen;
pub mod error;
pub mod models;
mod utils;

// The generated axum request extractors are co-located in the models `_gen/`
// dir (re-declared by the generated models `mod.rs` and re-exported via
// `models::*`), so the `FromRequest`/`FromRequestParts` impls are in scope
// without a separate `extractors` module.

// Hand-written extractors for the NDJSON query path (not part of the generated
// service). See [`query_extractors`].
#[cfg(feature = "axum")]
mod query_extractors;
