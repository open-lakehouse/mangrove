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
use object_store::{
    Attribute, Attributes, GetOptions, GetRange, ObjectStore, ObjectStoreExt, PutMode, PutOptions,
    PutPayload, UpdateVersion,
};
use send_wrapper::SendWrapper;
use unitycatalog_client::VolumeOperation;
use unitycatalog_object_store::UnityObjectStoreFactory;

use crate::error::{Error, Result};
use crate::files::page::{DirectoryPage, FileEntry, paginate};
use crate::files::path::VolumePath;
use crate::generated::portal::files::v1::{DirectoryMetadata, FileMetadata};

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

/// Vend `WRITE_VOLUME` credentials for `path`'s volume and return the real cloud
/// store, prefix-scoped to the volume root (so callers address it with
/// volume-relative keys).
///
/// The read-only [`resolve_volume`] stays the vend path for list/read/stat so
/// browsing keeps the least-privilege `READ_VOLUME` credential; only the write
/// verbs (`write_file`/`delete_file`/`create_dir`) reach for `ReadWrite`.
pub async fn resolve_volume_rw(
    factory: &UnityObjectStoreFactory,
    path: &VolumePath,
) -> Result<Arc<dyn ObjectStore>> {
    let uc_store = factory
        .for_volume(path.full_name(), VolumeOperation::ReadWrite)
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
        file_size: result.meta.size as i64,
        last_modified: result.meta.last_modified.timestamp_millis(),
        // The store's HEAD doesn't surface a content type here; proto empty
        // string == absent (buffa skips empty on the wire).
        content_type: String::new(),
        etag: result.meta.e_tag.unwrap_or_default(),
        ..Default::default()
    })
}

/// Write (create or overwrite) a file, returning its post-write metadata.
///
/// `content_type` is recorded as the object's `Content-Type` attribute when
/// supplied. `if_match_etag` opts into a conditional write: when `Some`, the put
/// only succeeds if the current object still carries that etag (an optimistic
/// lock against a lost update), otherwise it is an unconditional overwrite.
///
/// A failed precondition (the object moved on under a conditional write) is
/// re-tagged with the `conflict:` marker so the bindings' `classify()` maps it
/// to the `CONFLICT` code, distinct from a hard `FAILED`. Every other storage
/// error flows through [`Error::from_object_store`] like the read path.
pub async fn write_file(
    factory: &UnityObjectStoreFactory,
    path: &VolumePath,
    bytes: Vec<u8>,
    content_type: Option<String>,
    if_match_etag: Option<String>,
) -> Result<FileMetadata> {
    if path.is_root() {
        return Err(Error::InvalidUrl(
            "a file path is required (got a volume root)".to_string(),
        ));
    }
    let store = resolve_volume_rw(factory, path).await?;
    write_file_on_store(&store, path, bytes, content_type, if_match_etag).await
}

/// The store-level half of [`write_file`]: everything after credential vending.
/// Split out so native `InMemory`-backed tests exercise the key construction,
/// `PutMode` selection, and conflict mapping without a live factory.
async fn write_file_on_store(
    store: &Arc<dyn ObjectStore>,
    path: &VolumePath,
    bytes: Vec<u8>,
    content_type: Option<String>,
    if_match_etag: Option<String>,
) -> Result<FileMetadata> {
    let location = StorePath::from(path.object_key("").as_str());

    let size = bytes.len() as u64;
    let payload = PutPayload::from(bytes);

    let mode = match &if_match_etag {
        Some(etag) => PutMode::Update(UpdateVersion {
            e_tag: Some(etag.clone()),
            version: None,
        }),
        None => PutMode::Overwrite,
    };
    let mut attributes = Attributes::new();
    if let Some(ct) = content_type.clone() {
        attributes.insert(Attribute::ContentType, ct.into());
    }
    let options = PutOptions {
        mode,
        attributes,
        ..Default::default()
    };

    let result = store
        .put_opts(&location, payload, options)
        .await
        .map_err(map_write_error)?;

    Ok(FileMetadata {
        path: path.to_canonical(),
        file_size: size as i64,
        last_modified: 0,
        content_type: content_type.unwrap_or_default(),
        etag: result.e_tag.unwrap_or_default(),
        ..Default::default()
    })
}

/// Delete a file. A missing file surfaces as the store's `NotFound` (classified
/// `FAILED`) rather than being swallowed.
pub async fn delete_file(factory: &UnityObjectStoreFactory, path: &VolumePath) -> Result<()> {
    if path.is_root() {
        return Err(Error::InvalidUrl(
            "a file path is required (got a volume root)".to_string(),
        ));
    }
    let store = resolve_volume_rw(factory, path).await?;
    let location = StorePath::from(path.object_key("").as_str());
    store
        .delete(&location)
        .await
        .map_err(Error::from_object_store)?;
    Ok(())
}

/// Create a directory by writing a zero-byte sentinel object keyed on the
/// directory path with a trailing `/`.
///
/// Object stores have no real directories — a directory only exists as the
/// common prefix of some object's key. The `<dir>/` sentinel makes an otherwise
/// empty folder show up in `list_directory` (which rolls up common prefixes,
/// [`list_directory`]) and is invisible to that rollup (unlike a visible
/// `.keep` file). This is a best-effort UX marker; some backends materialize
/// prefixes lazily.
pub async fn create_dir(
    factory: &UnityObjectStoreFactory,
    path: &VolumePath,
) -> Result<DirectoryMetadata> {
    if path.is_root() {
        return Err(Error::InvalidUrl(
            "a directory path is required (got a volume root)".to_string(),
        ));
    }
    let store = resolve_volume_rw(factory, path).await?;
    // The sentinel key is the directory key with a trailing slash.
    let marker = format!("{}/", path.object_key(""));
    let location = StorePath::from(marker.as_str());
    store
        .put_opts(
            &location,
            PutPayload::from_static(b""),
            PutOptions::default(),
        )
        .await
        .map_err(Error::from_object_store)?;

    Ok(DirectoryMetadata {
        path: path.to_canonical(),
        last_modified: 0,
        ..Default::default()
    })
}

/// Map a `put_opts` error, re-tagging a failed precondition (conditional write
/// lost the race) so the bindings surface it as the `CONFLICT` code; everything
/// else defers to [`Error::from_object_store`] (transport re-tagging + passthrough).
fn map_write_error(err: object_store::Error) -> Error {
    match err {
        object_store::Error::Precondition { path, source } => Error::UnityCatalog(format!(
            "conflict: precondition failed for {path}: {source}"
        )),
        other => Error::from_object_store(other),
    }
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
