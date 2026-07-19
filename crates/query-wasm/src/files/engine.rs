//! Volume resolution + read-only file ops glue (wasm-only).
//!
//! Resolves a [`VolumePath`] through the canonical [`UnityObjectStoreFactory`]
//! (`for_volume` vends `READ_VOLUME` credentials and builds the real cloud
//! `object_store` — a `MicrosoftAzure` store on wasm — prefix-scoped to the
//! volume root), then serves the three read-only ops the files seam needs:
//! - `list_directory` — the store's native `list_with_delimiter` (delimiter
//!   rollup → directories vs files), normalized to canonical `/Volumes/…`
//!   entries and paged in-engine (see [`crate::files::page::paginate`]);
//! - `read_file` — a ranged GET through the store, streamed chunk-by-chunk;
//! - `stat` — a HEAD (`GetOptions { head: true }`) mapped to file metadata.
//!
//! Because the vended store is prefix-scoped to the volume root, every key is
//! **volume-relative** — the [`VolumePath`] helpers are called with an empty
//! `root_key`.
//!
//! Storage-IO failures are re-tagged via [`Error::from_object_store`] so a
//! blocked read/list still carries the `network/CORS:` marker the bindings'
//! `classify()` maps to the `NETWORK` fallback code, same as the query path.
//! (The credential-vend leg inherits that tag from the factory error `From`
//! impls in [`crate::error`].)
//!
//! On the wasm target the factory builds Azure / Azurite stores only; GCP and
//! AWS are gated out (they error during vending), so volume browsing in the
//! browser is Azure-first.

use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;
use object_store::path::Path as StorePath;
use object_store::{GetOptions, GetRange, ObjectStore};
use send_wrapper::SendWrapper;
use unitycatalog_client::VolumeOperation;
use unitycatalog_object_store::UnityObjectStoreFactory;

use crate::error::{Error, Result};
use crate::files::page::{DirectoryPage, FileEntry, FileMetadata, paginate};
use crate::files::path::VolumePath;

/// Vend `READ_VOLUME` credentials for `path`'s volume and return the real cloud
/// store, prefix-scoped to the volume root (so callers address it with
/// volume-relative keys).
pub async fn resolve_volume(
    factory: &UnityObjectStoreFactory,
    path: &VolumePath,
) -> Result<Arc<dyn ObjectStore>> {
    let uc_store = factory
        .for_volume(path.full_name(), VolumeOperation::Read)
        .await?;
    Ok(uc_store.as_dyn())
}

/// List one page of a directory's immediate children, returning a serializable
/// [`DirectoryPage`] with canonical `/Volumes/…` entry paths.
///
/// `object_store`'s `list_with_delimiter` collects the whole result set
/// internally (it drives the provider's continuation tokens to exhaustion and
/// exposes no cursor), so pagination is applied **in-engine** (see
/// [`paginate`]): the full listing is sorted by path, windowed by `max_results`,
/// and `next_page_token` is the stringified offset of the next window. A
/// follow-up call decodes that offset, re-lists (a consistent per-call snapshot;
/// cheap for a browser file browser), skips, and takes the next window.
pub async fn list_directory(
    factory: &UnityObjectStoreFactory,
    path: &VolumePath,
    max_results: Option<u32>,
    page_token: Option<String>,
) -> Result<DirectoryPage> {
    let store = resolve_volume(factory, path).await?;

    // The store is prefix-scoped to the volume root, so the listing prefix is
    // just the relative sub-path (empty `root_key`). An empty prefix lists the
    // volume root; object_store wants `None` there, and normalizes a trailing
    // slash away, so trim it.
    let prefix = path.list_prefix("");
    let trimmed = prefix.trim_end_matches('/');
    let prefix_arg = if trimmed.is_empty() {
        None
    } else {
        Some(StorePath::from(trimmed))
    };

    let listing = store
        .list_with_delimiter(prefix_arg.as_ref())
        .await
        .map_err(Error::from_object_store)?;

    // Directories are the delimiter-rolled common prefixes; files are the
    // objects. Re-attach the canonical `/Volumes/…` prefix to each store-relative
    // key (the PrefixStore already stripped the volume root on the way out).
    let mut entries: Vec<FileEntry> =
        Vec::with_capacity(listing.common_prefixes.len() + listing.objects.len());
    for dir in &listing.common_prefixes {
        entries.push(FileEntry {
            path: path.absolute("", dir.as_ref()),
            is_directory: true,
            file_size: 0,
            last_modified: 0,
        });
    }
    for obj in &listing.objects {
        entries.push(FileEntry {
            path: path.absolute("", obj.location.as_ref()),
            is_directory: false,
            file_size: obj.size,
            last_modified: obj.last_modified.timestamp_millis(),
        });
    }
    // Deterministic order so the synthetic offset token is stable across calls.
    entries.sort_by(|a, b| a.path.cmp(&b.path));

    let (entries, next_page_token) = paginate(entries, max_results, page_token)?;
    Ok(DirectoryPage {
        entries,
        next_page_token,
    })
}

/// Read a file (or byte range), invoking `on_chunk(&Bytes)` for each body chunk
/// in file order. Returns the total number of bytes read.
pub async fn read_file<F>(
    factory: &UnityObjectStoreFactory,
    path: &VolumePath,
    offset: Option<u64>,
    length: Option<u64>,
    mut on_chunk: F,
) -> Result<u64>
where
    F: FnMut(&Bytes) -> Result<()>,
{
    if path.is_root() {
        return Err(Error::InvalidUrl(
            "a file path is required (got a volume root)".to_string(),
        ));
    }
    let store = resolve_volume(factory, path).await?;
    // Volume-relative key: the store is already prefix-scoped to the volume root.
    let location = StorePath::from(path.object_key("").as_str());

    let options = GetOptions {
        range: byte_range(offset, length),
        ..Default::default()
    };
    let result = store
        .get_opts(&location, options)
        .await
        .map_err(Error::from_object_store)?;

    // Drive the object store's own chunked byte stream — bytes flow straight
    // from the store to the callback without materializing the whole object.
    // Wrap in SendWrapper: the underlying fetch stream is `!Send`, and
    // `into_stream` requires `Send` on its bound even on single-threaded wasm.
    let mut total: u64 = 0;
    let mut stream = SendWrapper::new(result.into_stream());
    while let Some(chunk) = stream.next().await {
        let bytes = chunk.map_err(Error::from_object_store)?;
        total += bytes.len() as u64;
        on_chunk(&bytes)?;
    }
    Ok(total)
}

/// HEAD a file and map the metadata to a serializable [`FileMetadata`] with a
/// canonical `/Volumes/…` path.
pub async fn stat(factory: &UnityObjectStoreFactory, path: &VolumePath) -> Result<FileMetadata> {
    if path.is_root() {
        return Err(Error::InvalidUrl(
            "a file path is required (got a volume root)".to_string(),
        ));
    }
    let store = resolve_volume(factory, path).await?;
    let location = StorePath::from(path.object_key("").as_str());

    let options = GetOptions {
        head: true,
        ..Default::default()
    };
    let result = store
        .get_opts(&location, options)
        .await
        .map_err(Error::from_object_store)?;

    Ok(FileMetadata {
        path: path.to_canonical(),
        file_size: result.meta.size,
        last_modified: result.meta.last_modified.timestamp_millis(),
        content_type: None,
        etag: result.meta.e_tag,
    })
}

/// Translate an optional offset/length into a [`GetRange`]. Both unset reads the
/// whole object.
fn byte_range(offset: Option<u64>, length: Option<u64>) -> Option<GetRange> {
    match (offset, length) {
        (None, None) => None,
        (off, Some(len)) => {
            let start = off.unwrap_or(0);
            Some(GetRange::Bounded(start..start + len))
        }
        (Some(off), None) => Some(GetRange::Offset(off)),
    }
}
