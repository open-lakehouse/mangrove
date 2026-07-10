//! Hand-written serde models for the UC Delta REST API (`/delta/v1/...`).
//!
//! These types moved to the standalone [`unitycatalog_delta_api`] crate (which
//! owns the portable Delta v1 API so it can be shared across server
//! implementations). They are re-exported here so existing
//! `unitycatalog_common::models::delta::v1::*` paths keep resolving unchanged.
pub use unitycatalog_delta_api::models::*;
