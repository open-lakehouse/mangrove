//! Read-only in-browser Unity Catalog **volume files** backend: the "dedicated
//! store" path (mirroring the table path's hand-rolled `uc.rs` / `fetch_store.rs`
//! / `creds.rs`, because `olai-uc-object-store` cannot compile for wasm32 without
//! a large cross-repo refactor).
//!
//! Layers, matching the crate's native-vs-wasm split:
//! - [`path`]: the [`VolumePath`](path::VolumePath) model — parse/format
//!   canonical `/Volumes/<c>/<s>/<v>/…` paths and map between store keys and
//!   Volumes paths. Native-compilable + tested.
//! - [`lister`]: build the container/bucket-scoped cloud list REST endpoint from
//!   a resolved storage location, and parse the Azure (XML) / GCS (JSON) list
//!   bodies into normalized entries. Native-compilable + tested.
//! - [`engine`] (wasm-only): resolve a volume through Unity Catalog
//!   (`GetVolume` → `temporary-volume-credentials` READ_VOLUME →
//!   [`crate::creds::resolve_storage`]), then list / read / stat over the vended
//!   credential. Uses [`crate::fetch_store::UcFetchStore`] for reads and stats and
//!   the browser's `fetch` for the list REST call.

pub mod lister;
pub mod path;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod engine;
