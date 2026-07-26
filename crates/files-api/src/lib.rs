//! The Files API — a ConnectRPC `portal.files.v1.FilesService` server over an
//! `object_store`-backed file store.
//!
//! Like [`olai-uc-storage-proxy`], the crate runs in two modes from one build:
//!
//! - **Embedded** in the Unity Catalog server: the [`server`](#feature-flags)
//!   feature exposes [`service::AppState`] and the generated `FilesService`
//!   implementation. The host builds a ConnectRPC router via
//!   [`AppState::register_all`](service::AppState::register_all), turns it into an
//!   axum fallback service, and nests it under a base path.
//! - **Standalone** against an upstream Unity Catalog service: the `bin` feature
//!   builds the `files-api` binary (CLI serve/healthcheck + self-contained auth
//!   layer + YAML config), vending volume credentials through
//!   [`store::UnityVolumeStore`].
//!
//! The `portal.files.v1` wire contract lives in the shared
//! [`unitycatalog_files_proto`] crate; this crate only implements it.
//!
//! # Feature flags
//!
//! - `server` (default): the service surface — `AppState`, the generated trait
//!   impl, and the [`StoreError`](error::StoreError) → `ConnectError` mapping.
//! - `client-arm`: the [`UnityVolumeStore`](store::UnityVolumeStore) backend.
//! - `testing`: the in-memory [`MemoryStore`](store::MemoryStore) + test helpers.
//! - `bin`: the deployable standalone `files-api` binary.

// The generated `FilesService` trait uses `impl Trait` in return position; the
// hand-written impl returns a more concrete (refined) type, which fires this
// lint. The refinement is deliberate and internal — the generated client is what
// callers use — so silence it crate-wide, matching the wasm engine's service impl.
#![allow(refining_impl_trait)]

#[cfg(feature = "bin")]
pub mod auth;
pub mod error;
#[cfg(feature = "server")]
pub mod service;
#[cfg(feature = "server")]
pub mod store;
#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "bin")]
pub mod cli;

pub use error::{StoreError, StoreResult};
