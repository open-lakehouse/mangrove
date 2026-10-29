//! Open Sharing / Delta Sharing wire models shared by client and server.
//!
//! The [`models`] describe the protocol on the wire: the generated
//! request/response types of the discovery services, the hand-written
//! newline-delimited JSON lines of the table query path
//! ([`QueryResponseLine`](models::QueryResponseLine),
//! [`MetadataResponse`](models::MetadataResponse), …), and the
//! `delta-sharing-capabilities` header
//! ([`DeltaSharingCapabilities`](models::DeltaSharingCapabilities)).
//!
//! The `open-sharing-client` crate sends these types and `open-sharing-api`
//! serves them; keeping them here lets the server side build without the
//! client's HTTP and data-processing dependencies.
//!
//! # Feature flags
//!
//! `axum` compiles the request extractors used by a server built on these types
//! (including the hand-written NDJSON query-path extractors); a plain client
//! build does not need it.

pub use crate::error::{Error, Result};

pub mod error;
pub mod models;

// The generated axum request extractors are co-located in the models `_gen/`
// dir (re-declared by the generated models `mod.rs` and re-exported via
// `models::*`), so the `FromRequest`/`FromRequestParts` impls are in scope
// without a separate `extractors` module.

// Hand-written extractors for the NDJSON query path (not part of the generated
// service). See [`query_extractors`].
#[cfg(feature = "axum")]
mod query_extractors;
