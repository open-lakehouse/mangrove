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
