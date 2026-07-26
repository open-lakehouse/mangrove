//! File stores backing the [`FilesService`](crate::service) handlers.
//!
//! The [`FileStore`] trait surface is intentionally small and purpose-built — the
//! generated proto messages are the stored/returned values directly. Two backends
//! ship today: the process-local [`MemoryStore`] (behind `testing`, for
//! dependency-free runs + tests) and the cloud-backed [`UnityVolumeStore`] (behind
//! `client-arm`, addressing Databricks Volumes over UC-vended credentials).

#[cfg(feature = "testing")]
pub mod memory;
#[cfg(feature = "client-arm")]
pub mod unity;

#[cfg(feature = "testing")]
pub use memory::MemoryStore;
#[cfg(feature = "client-arm")]
pub use unity::UnityVolumeStore;

use bytes::Bytes;
use futures::stream::BoxStream;

use crate::error::StoreResult;
use unitycatalog_files_proto::buffa::portal::files::v1::{
    DirectoryEntry, DirectoryMetadata, FileMetadata,
};

/// A single page request (max results + opaque token).
#[derive(Debug, Default, Clone)]
pub struct Page {
    pub max_results: Option<usize>,
    pub page_token: Option<String>,
}

/// A lazy stream of a file's bytes — chunks arrive in file order and are never
/// fully buffered (see [`FileStore::read_file_stream`]).
pub type ByteStream = BoxStream<'static, StoreResult<Bytes>>;

/// A lazy stream of directory entries (see [`FileStore::list_files_stream`]).
pub type EntryStream = BoxStream<'static, StoreResult<DirectoryEntry>>;

/// Fine-grained options for a streaming list (see [`FileStore::list_files_opts`]).
///
/// This is the low-level listing knob: unlike the unary, hierarchical
/// [`FileStore::list_directory`] (which rolls subdirectories up as entries and
/// returns one bounded page), this drives `object_store`'s streaming list
/// directly.
#[derive(Debug, Default, Clone)]
pub struct ListOpts {
    /// List recursively (flat: every object under the prefix). When `false`, a
    /// `/` delimiter groups immediate children and rolls subdirectories up as
    /// directory entries — the hierarchical view.
    pub recursive: bool,
    /// Resume listing strictly *after* this absolute path (exclusive). Maps to
    /// `object_store::ObjectStore::list_with_offset`; lets a caller page through
    /// a large listing without re-walking what it has already seen.
    pub start_after: Option<String>,
    /// Stop after yielding this many entries (applied by the stream consumer).
    pub max_results: Option<usize>,
}

/// Apply a simple paging window over an already-collected, sorted vec.
///
/// The page token is the index of the first un-returned element, encoded as a
/// decimal string — adequate for stores with stable ordering that materialize
/// the full listing before paging.
#[cfg_attr(not(feature = "client-arm"), allow(dead_code))]
pub(crate) fn paginate<T>(mut items: Vec<T>, page: &Page) -> (Vec<T>, Option<String>) {
    let start: usize = page
        .page_token
        .as_deref()
        .and_then(|t| t.parse().ok())
        .unwrap_or(0);
    if start >= items.len() {
        return (Vec::new(), None);
    }
    let mut rest = items.split_off(start);
    match page.max_results {
        Some(limit) if limit < rest.len() => {
            rest.truncate(limit);
            (rest, Some((start + limit).to_string()))
        }
        _ => (rest, None),
    }
}

/// Storage for files and directories, addressed by path.
#[async_trait::async_trait]
pub trait FileStore: Send + Sync + 'static {
    async fn put_file(
        &self,
        path: &str,
        content_type: Option<String>,
        contents: Vec<u8>,
    ) -> StoreResult<FileMetadata>;

    /// Stream a file's bytes to storage without buffering the whole payload.
    ///
    /// Chunks are consumed from `chunks` in arrival order and uploaded
    /// incrementally (a multipart upload on cloud backends), so an arbitrarily
    /// large file is never fully materialized in memory. The natural sink for a
    /// client-streaming `UploadFile`.
    ///
    /// A failed item in `chunks` aborts the upload (best-effort cleanup of any
    /// already-uploaded parts) and surfaces as the returned `Err`.
    async fn put_file_stream(
        &self,
        path: &str,
        content_type: Option<String>,
        chunks: ByteStream,
    ) -> StoreResult<FileMetadata>;

    /// Read a (possibly partial) range of a file's bytes into memory.
    ///
    /// Buffers the whole range; prefer [`read_file_stream`](Self::read_file_stream)
    /// for transfers (e.g. `DownloadFile`) so bytes are never fully materialized.
    async fn read_file(
        &self,
        path: &str,
        offset: Option<i64>,
        length: Option<i64>,
    ) -> StoreResult<Vec<u8>>;

    /// Stream a (possibly partial) range of a file's bytes as ordered chunks,
    /// without buffering the whole range. Backed by the object store's own
    /// chunked GET — the natural source for a server-streaming `DownloadFile`.
    ///
    /// Errors raised while *opening* the read (not found, bad range, credential
    /// failure) surface as the returned `Err`; errors mid-transfer surface as a
    /// failed item in the stream.
    async fn read_file_stream(
        &self,
        path: &str,
        offset: Option<i64>,
        length: Option<i64>,
    ) -> StoreResult<ByteStream>;

    async fn stat_file(&self, path: &str) -> StoreResult<FileMetadata>;
    async fn delete_file(&self, path: &str) -> StoreResult<()>;

    async fn create_directory(&self, path: &str) -> StoreResult<DirectoryMetadata>;
    async fn delete_directory(&self, path: &str) -> StoreResult<()>;
    async fn stat_directory(&self, path: &str) -> StoreResult<DirectoryMetadata>;

    /// Unary, hierarchical, paged listing of a directory's immediate children
    /// (subdirectories rolled up as entries). Backed by a single
    /// delimiter-listing call; suitable for a bounded UI page.
    async fn list_directory(
        &self,
        path: &str,
        page: Page,
    ) -> StoreResult<(Vec<DirectoryEntry>, Option<String>)>;

    /// Stream every file under a directory recursively, as a lazy stream of
    /// entries (no in-memory accumulation of the full listing). Convenience for
    /// the common "list everything under here" case — equivalent to
    /// [`list_files_opts`](Self::list_files_opts) with `recursive: true`.
    async fn list_files_stream(&self, path: &str) -> StoreResult<EntryStream> {
        self.list_files_opts(
            path,
            ListOpts {
                recursive: true,
                ..Default::default()
            },
        )
        .await
    }

    /// Stream directory entries with fine-grained [`ListOpts`] — recursive vs.
    /// delimited, resume-after-path (`start_after`), and a max-results cap.
    /// Backed by `object_store`'s streaming list (`list` / `list_with_offset`),
    /// so it scales to large directories without buffering the whole listing.
    async fn list_files_opts(&self, path: &str, opts: ListOpts) -> StoreResult<EntryStream>;
}
