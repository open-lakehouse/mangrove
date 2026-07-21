//! Read-only in-browser Unity Catalog **volume files** backend.
//!
//! Volume browsing is driven by the canonical `olai-uc-object-store`
//! [`UnityObjectStoreFactory`](unitycatalog_object_store::UnityObjectStoreFactory):
//! `for_volume` vends `READ_VOLUME` credentials and builds the real cloud
//! `object_store` (a `MicrosoftAzure` store on wasm), so listing / reading /
//! stat all go through one shared store impl — no hand-rolled fetch store or
//! cloud-list REST parsing.
//!
//! Layers, matching the crate's native-vs-wasm split:
//! - [`path`]: the [`VolumePath`](path::VolumePath) model — parse/format
//!   canonical `/Volumes/<c>/<s>/<v>/…` paths and map between store keys and
//!   Volumes paths. Native-compilable + tested.
//! - [`page`]: the directory-listing DTOs + in-engine offset pagination over a
//!   fully-collected `list_with_delimiter` result. Native-compilable + tested.
//! - [`engine`] (wasm-only): resolve a volume through the factory
//!   (`for_volume` → prefix-scoped `Arc<dyn ObjectStore>`), then list / read /
//!   stat over the vended store.

pub mod page;
pub mod path;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod engine;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod service;
