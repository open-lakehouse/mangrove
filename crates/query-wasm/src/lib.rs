//! In-browser query engine for Unity Catalog table previews (Phase B of
//! `WASM_QUERY_PREVIEW.md`).
//!
//! Glue around [`deltalake-wasm`](deltalake_wasm): resolve a Unity Catalog table
//! over the REST API (`/delta/v1` `loadTable` + `temporary-table-credentials`),
//! turn the vended credential into a fetch-backed object store, prime and open
//! the Delta table with the delta-rs wasm facade, and execute preview SQL —
//! streaming results as the self-contained Arrow IPC chunks the
//! `open_lakehouse.query.v1` runner contract requires.
//!
//! Everything except the fetch store, the UC REST client, and the wasm-bindgen
//! surface compiles natively, so resolution, credential mapping, and query
//! execution are tested with ordinary `cargo test`.

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod bindings;
pub mod catalog;
pub mod engine;
pub mod error;
pub mod files;
pub mod log_udtf;
pub mod resolve;

pub use error::{Error, Result};
