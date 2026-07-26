//! Generated `portal.files.v1.FilesService` proto contract — buffa message types
//! and the `connect-rust` service trait/dispatcher — shared by every consumer of
//! the Files API.
//!
//! # Consumers
//!
//! - [`olai-uc-files-api`] — the native standalone / embeddable Files API server
//!   implements the [`connect::portal::files::v1::FilesService`] trait over an
//!   `object_store`-backed file store.
//! - [`olai-uc-query-wasm`] — the in-browser engine implements the same trait for
//!   the volume-files write path; it depends on this crate cross-workspace via
//!   `path = "../../crates/files-proto"`.
//!
//! # Why a shared crate
//!
//! The generated code names only `buffa`, `buffa-types`, `connectrpc`, and
//! `serde` — no `object_store`, arrow, or wasm-specific type — so it is
//! *patch-agnostic* and compiles identically in the native root workspace and
//! under the wasm workspace's arrow/`object_store` `[patch.crates-io]` fork. That
//! is what lets one crate back both consumers despite their independent
//! `Cargo.lock`s.
//!
//! # Layout
//!
//! The module tree mirrors the proto package path (`portal.files.v1`):
//!
//! - [`buffa`] — message types + zero-copy views + serde impls.
//! - [`connect`] — the `FilesService` trait, `FilesServiceServer<S>`,
//!   `FilesServiceExt::register`, and `FilesServiceClient<T>`.
//!
//! Do not hand-edit anything under `src/generated/`; regenerate with
//! `just generate-files-proto`.

// The `connect-rust` codegen references the buffa message types by the
// crate-relative path `crate::generated::buffa::…` (the `buffa_module` option in
// buf.gen.files.yaml). Keep the `generated` module private and re-export its
// public trees so that path resolves inside this crate while consumers see a flat
// `unitycatalog_files_proto::{buffa, connect}` surface.
mod generated;

pub use generated::buffa;
pub use generated::connect;

/// The proto package root (`portal.files.v1` lives at `portal::files::v1`),
/// re-exported for callers that reference message types by package path.
pub use generated::buffa::portal;
