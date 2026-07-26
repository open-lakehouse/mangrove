//! Unity Catalog volume-backed implementation of [`FileStore`].
//!
//! Files are addressed by Databricks Volumes paths
//! (`/Volumes/<catalog>/<schema>/<volume>/dir/file.txt`). Each operation parses
//! the path into a three-level volume name plus a relative sub-path, vends a
//! scoped credential for the **volume root** via [`UnityObjectStoreFactory`], and
//! runs the corresponding `object_store` operation against the relative path.
//! Vending at the volume root (rather than the full sub-path) keeps one credential
//! usable for every file/directory under the volume and is the correct
//! granularity for listing.
//!
//! Directories are modeled as path prefixes — there are no marker objects, so
//! empty directories do not persist (matching cloud-storage semantics).
//! Following the Databricks Files API, `delete_directory` only succeeds on an
//! empty directory: a non-empty one is rejected with `FailedPrecondition`.

use std::sync::Arc;

use futures::StreamExt;
use object_store::{
    Attribute, Attributes, GetOptions, GetRange, ObjectStore, ObjectStoreExt, PutMultipartOptions,
    PutOptions, WriteMultipart, path::Path as StorePath,
};
use unitycatalog_object_store::{UnityObjectStoreFactory, VolumeOperation};

use crate::error::{StoreError, StoreResult};
use crate::store::{ByteStream, EntryStream, FileStore, ListOpts, Page, paginate};
use unitycatalog_files_proto::buffa::portal::files::v1::{
    DirectoryEntry, DirectoryMetadata, FileMetadata,
};

/// A [`FileStore`] backed by Unity Catalog volumes.
pub struct UnityVolumeStore {
    factory: Arc<UnityObjectStoreFactory>,
}

impl UnityVolumeStore {
    pub fn new(factory: Arc<UnityObjectStoreFactory>) -> Self {
        Self { factory }
    }

    /// Resolve a Volumes path to a credential-scoped, volume-rooted store plus
    /// the relative path within it.
    async fn resolve(
        &self,
        path: &str,
        op: VolumeOperation,
    ) -> StoreResult<(Arc<dyn ObjectStore>, VolumePath)> {
        let parsed = parse_volume_path(path)?;
        let store = self
            .factory
            .for_volume(parsed.full_name.clone(), op)
            .await
            .map_err(|e| StoreError::Internal(format!("credential vending failed: {e}")))?;
        Ok((store.as_dyn(), parsed))
    }
}

#[async_trait::async_trait]
impl FileStore for UnityVolumeStore {
    async fn put_file(
        &self,
        path: &str,
        content_type: Option<String>,
        contents: Vec<u8>,
    ) -> StoreResult<FileMetadata> {
        let (store, parsed) = self.resolve(path, VolumeOperation::ReadWrite).await?;
        let location = parsed.store_path()?;

        // Persist the caller-supplied content type as an object attribute so it
        // round-trips on `stat_file` / `get_file_metadata`.
        let mut attributes = Attributes::new();
        if let Some(ct) = content_type.filter(|c| !c.is_empty()) {
            attributes.insert(Attribute::ContentType, ct.into());
        }
        let opts = PutOptions {
            attributes,
            ..Default::default()
        };
        store
            .put_opts(&location, contents.into(), opts)
            .await
            .map_err(map_store_err)?;

        // Re-stat for accurate size / mtime / etag / content type from the backend.
        self.stat_file(path).await
    }

    async fn put_file_stream(
        &self,
        path: &str,
        content_type: Option<String>,
        mut chunks: ByteStream,
    ) -> StoreResult<FileMetadata> {
        let (store, parsed) = self.resolve(path, VolumeOperation::ReadWrite).await?;
        let location = parsed.store_path()?;

        // Carry the caller-supplied content type through as an object attribute,
        // same as the buffered `put_file`, so it round-trips on `stat_file`.
        let mut attributes = Attributes::new();
        if let Some(ct) = content_type.filter(|c| !c.is_empty()) {
            attributes.insert(Attribute::ContentType, ct.into());
        }
        let opts = PutMultipartOptions {
            attributes,
            ..Default::default()
        };

        // Open a multipart upload and drive it with `WriteMultipart`, which
        // buffers into fixed-size parts (5 MiB) and uploads them as they fill —
        // the file is never fully materialized. Cap in-flight parts so a fast
        // producer can't queue unbounded uploads.
        let upload = store
            .put_multipart_opts(&location, opts)
            .await
            .map_err(map_store_err)?;
        let mut writer = WriteMultipart::new(upload);

        const MAX_CONCURRENCY: usize = 8;
        while let Some(chunk) = chunks.next().await {
            match chunk {
                Ok(bytes) => {
                    writer
                        .wait_for_capacity(MAX_CONCURRENCY)
                        .await
                        .map_err(map_store_err)?;
                    writer.put(bytes);
                }
                // The producing stream errored mid-upload — abort so no partial
                // object is committed, then surface the original error.
                Err(e) => {
                    let _ = writer.abort().await;
                    return Err(e);
                }
            }
        }

        writer.finish().await.map_err(map_store_err)?;

        // Re-stat for accurate size / mtime / etag / content type from the backend.
        self.stat_file(path).await
    }

    async fn read_file(
        &self,
        path: &str,
        offset: Option<i64>,
        length: Option<i64>,
    ) -> StoreResult<Vec<u8>> {
        let (store, parsed) = self.resolve(path, VolumeOperation::Read).await?;
        let location = parsed.store_path()?;

        let range = byte_range(offset, length)?;
        let bytes = match range {
            Some(range) => {
                let opts = GetOptions {
                    range: Some(range),
                    ..Default::default()
                };
                store
                    .get_opts(&location, opts)
                    .await
                    .map_err(map_store_err)?
                    .bytes()
                    .await
                    .map_err(map_store_err)?
            }
            None => store
                .get(&location)
                .await
                .map_err(map_store_err)?
                .bytes()
                .await
                .map_err(map_store_err)?,
        };
        Ok(bytes.to_vec())
    }

    async fn read_file_stream(
        &self,
        path: &str,
        offset: Option<i64>,
        length: Option<i64>,
    ) -> StoreResult<ByteStream> {
        let (store, parsed) = self.resolve(path, VolumeOperation::Read).await?;
        let location = parsed.store_path()?;

        // Open the GET (resolving not-found / bad-range up front), then hand back
        // the object store's own chunked byte stream — bytes flow straight from
        // storage to the caller without ever materializing the whole object.
        let opts = GetOptions {
            range: byte_range(offset, length)?,
            ..Default::default()
        };
        let result = store
            .get_opts(&location, opts)
            .await
            .map_err(map_store_err)?;
        Ok(result
            .into_stream()
            .map(|r| r.map_err(map_store_err))
            .boxed())
    }

    async fn stat_file(&self, path: &str) -> StoreResult<FileMetadata> {
        let (store, parsed) = self.resolve(path, VolumeOperation::Read).await?;
        let location = parsed.store_path()?;
        // A HEAD-style `get_opts` returns the object metadata *and* its
        // attributes (content type) without transferring the body — `head`
        // alone does not carry attributes.
        let opts = GetOptions {
            head: true,
            ..Default::default()
        };
        let result = store
            .get_opts(&location, opts)
            .await
            .map_err(map_store_err)?;
        Ok(file_metadata(path, &result.meta, &result.attributes))
    }

    async fn delete_file(&self, path: &str) -> StoreResult<()> {
        let (store, parsed) = self.resolve(path, VolumeOperation::ReadWrite).await?;
        let location = parsed.store_path()?;
        store.delete(&location).await.map_err(map_store_err)
    }

    async fn create_directory(&self, path: &str) -> StoreResult<DirectoryMetadata> {
        // Prefix-based model: directories exist implicitly via their contents.
        // Creating one is a no-op; empty directories do not persist.
        let parsed = parse_volume_path(path)?;
        if parsed.relative.is_empty() {
            return Err(StoreError::InvalidArgument(
                "cannot create the volume root as a directory".into(),
            ));
        }
        Ok(DirectoryMetadata {
            path: path.to_string(),
            last_modified: 0,
            ..Default::default()
        })
    }

    async fn delete_directory(&self, path: &str) -> StoreResult<()> {
        let (store, parsed) = self.resolve(path, VolumeOperation::ReadWrite).await?;
        let prefix = parsed.list_prefix();

        // Databricks semantics: a directory delete only succeeds on an *empty*
        // directory; a non-empty directory is rejected (the caller must delete
        // its contents first). In the prefix-based model a directory exists only
        // by virtue of objects under its prefix, so any object found means the
        // directory is non-empty. An empty/absent directory deletes to a no-op
        // success — consistent with `create_directory` (empty dirs never
        // persist), so the two stay idempotent.
        let mut listing = store.list(prefix.as_ref());
        if listing
            .next()
            .await
            .transpose()
            .map_err(map_store_err)?
            .is_some()
        {
            return Err(StoreError::FailedPrecondition(format!(
                "directory {path:?} is not empty; delete its contents first"
            )));
        }
        Ok(())
    }

    async fn stat_directory(&self, path: &str) -> StoreResult<DirectoryMetadata> {
        let (store, parsed) = self.resolve(path, VolumeOperation::Read).await?;
        let prefix = parsed.list_prefix();
        let result = store
            .list_with_delimiter(prefix.as_ref())
            .await
            .map_err(map_store_err)?;
        if result.objects.is_empty() && result.common_prefixes.is_empty() {
            return Err(StoreError::NotFound(format!("directory {path:?}")));
        }
        Ok(DirectoryMetadata {
            path: path.to_string(),
            last_modified: 0,
            ..Default::default()
        })
    }

    async fn list_directory(
        &self,
        path: &str,
        page: Page,
    ) -> StoreResult<(Vec<DirectoryEntry>, Option<String>)> {
        let (store, parsed) = self.resolve(path, VolumeOperation::Read).await?;
        let prefix = parsed.list_prefix();
        let result = store
            .list_with_delimiter(prefix.as_ref())
            .await
            .map_err(map_store_err)?;

        let mut entries: Vec<DirectoryEntry> = Vec::new();
        // Common prefixes are sub-directories. Paths are store-relative (the
        // credential store is prefixed at the volume root), so re-attach the
        // `/Volumes/<full_name>/` portion to hand back absolute Volumes paths.
        for dir in result.common_prefixes {
            entries.push(DirectoryEntry {
                path: parsed.absolute(dir.as_ref()),
                is_directory: true,
                ..Default::default()
            });
        }
        for obj in result.objects {
            entries.push(DirectoryEntry {
                path: parsed.absolute(obj.location.as_ref()),
                is_directory: false,
                file_size: obj.size as i64,
                last_modified: obj.last_modified.timestamp_millis(),
                ..Default::default()
            });
        }
        entries.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(paginate(entries, &page))
    }

    async fn list_files_opts(&self, path: &str, opts: ListOpts) -> StoreResult<EntryStream> {
        let (store, parsed) = self.resolve(path, VolumeOperation::Read).await?;
        let prefix = parsed.list_prefix();

        // The object store's streaming list is recursive (no delimiter). The
        // non-recursive (delimited, hierarchical) view has no streaming form in
        // object_store, so fall back to the buffered delimiter listing and
        // replay it as a stream — same entry shape, bounded by directory width.
        if !opts.recursive {
            let (entries, _) = self
                .list_directory(
                    path,
                    Page {
                        max_results: opts.max_results,
                        page_token: None,
                    },
                )
                .await?;
            let filtered = filter_after(entries, opts.start_after.as_deref());
            return Ok(futures::stream::iter(filtered.into_iter().map(Ok)).boxed());
        }

        // Recursive: drive object_store's streaming list. `start_after` resumes
        // strictly after an absolute path; translate it to a store-relative
        // offset. Each yielded ObjectMeta becomes a (file) DirectoryEntry; the
        // `max_results` cap is applied by the consumer via `take`.
        let listing = match opts
            .start_after
            .as_deref()
            .map(|abs| parsed.relativize(abs))
        {
            Some(rel) => store.list_with_offset(prefix.as_ref(), &StorePath::from(rel)),
            None => store.list(prefix.as_ref()),
        };

        let parsed = Arc::new(parsed);
        let stream = listing.map(move |res| {
            res.map_err(map_store_err).map(|obj| DirectoryEntry {
                path: parsed.absolute(obj.location.as_ref()),
                is_directory: false,
                file_size: obj.size as i64,
                last_modified: obj.last_modified.timestamp_millis(),
                ..Default::default()
            })
        });
        let stream = match opts.max_results {
            Some(n) => stream.take(n).boxed(),
            None => stream.boxed(),
        };
        Ok(stream)
    }
}

/// Drop entries up to and including `start_after` (an absolute path). Entries are
/// assumed sorted; `None` returns them unchanged.
fn filter_after(entries: Vec<DirectoryEntry>, start_after: Option<&str>) -> Vec<DirectoryEntry> {
    match start_after {
        Some(after) => entries
            .into_iter()
            .filter(|e| e.path.as_str() > after)
            .collect(),
        None => entries,
    }
}

/// A Volumes path split into its three-level volume name and the relative
/// sub-path inside the volume (empty when addressing the volume root).
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct VolumePath {
    pub full_name: String,
    pub catalog: String,
    pub schema: String,
    pub volume: String,
    pub relative: String,
}

impl VolumePath {
    /// The relative path as an `object_store` [`StorePath`]. Errors if the path
    /// addresses the volume root (no file component).
    fn store_path(&self) -> StoreResult<StorePath> {
        if self.relative.is_empty() {
            return Err(StoreError::InvalidArgument(
                "a file path is required (got a volume root)".into(),
            ));
        }
        Ok(StorePath::from(self.relative.as_str()))
    }

    /// The listing prefix: the relative sub-path, or `None` for the volume root.
    fn list_prefix(&self) -> Option<StorePath> {
        if self.relative.is_empty() {
            None
        } else {
            Some(StorePath::from(self.relative.as_str()))
        }
    }

    /// Reattach the `/Volumes/<catalog>/<schema>/<volume>/` prefix to a
    /// store-relative path to produce an absolute Volumes path.
    fn absolute(&self, relative: &str) -> String {
        format!(
            "/Volumes/{}/{}/{}/{}",
            self.catalog,
            self.schema,
            self.volume,
            relative.trim_start_matches('/')
        )
    }

    /// Inverse of [`absolute`](Self::absolute): strip the
    /// `/Volumes/<catalog>/<schema>/<volume>/` prefix from an absolute Volumes
    /// path to get the store-relative path used as a listing offset. A path that
    /// doesn't carry this volume's prefix is returned trimmed of a leading `/`
    /// (best-effort — the offset just needs to be store-relative).
    fn relativize(&self, absolute: &str) -> String {
        let prefix = format!("/Volumes/{}/{}/{}/", self.catalog, self.schema, self.volume);
        absolute
            .strip_prefix(&prefix)
            .unwrap_or_else(|| absolute.trim_start_matches('/'))
            .to_string()
    }
}

/// Parse a Databricks Volumes path into its components.
///
/// Accepts `[/]Volumes/<catalog>/<schema>/<volume>[/<rest...>]`. The leading
/// `Volumes` segment is matched case-insensitively (mirroring `UCReference`); all
/// other segments are preserved verbatim.
pub(crate) fn parse_volume_path(path: &str) -> StoreResult<VolumePath> {
    let segments: Vec<&str> = path
        .trim_start_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    match segments.as_slice() {
        [kind, catalog, schema, volume, rest @ ..] if kind.eq_ignore_ascii_case("Volumes") => {
            Ok(VolumePath {
                full_name: format!("{catalog}.{schema}.{volume}"),
                catalog: (*catalog).to_string(),
                schema: (*schema).to_string(),
                volume: (*volume).to_string(),
                relative: rest.join("/"),
            })
        }
        _ => Err(StoreError::InvalidArgument(format!(
            "expected a Volumes path `/Volumes/<catalog>/<schema>/<volume>/...`, got {path:?}"
        ))),
    }
}

/// Translate a byte offset/length pair into an `object_store` [`GetRange`].
/// Returns `Ok(None)` when neither is set (read the whole object).
fn byte_range(offset: Option<i64>, length: Option<i64>) -> StoreResult<Option<GetRange>> {
    let offset = offset.filter(|&o| o != 0 || length.is_some());
    match (offset, length) {
        (None, None) => Ok(None),
        (off, len) => {
            let start = off.unwrap_or(0);
            if start < 0 {
                return Err(StoreError::InvalidArgument("offset must be >= 0".into()));
            }
            let start = start as u64;
            match len {
                Some(len) if len < 0 => {
                    Err(StoreError::InvalidArgument("length must be >= 0".into()))
                }
                Some(len) => Ok(Some(GetRange::Bounded(start..start + len as u64))),
                None => Ok(Some(GetRange::Offset(start))),
            }
        }
    }
}

fn file_metadata(
    path: &str,
    meta: &object_store::ObjectMeta,
    attributes: &Attributes,
) -> FileMetadata {
    let content_type = attributes
        .get(&Attribute::ContentType)
        .map(|v| v.as_ref().to_string())
        .unwrap_or_default();
    FileMetadata {
        path: path.to_string(),
        file_size: meta.size as i64,
        last_modified: meta.last_modified.timestamp_millis(),
        content_type,
        etag: meta.e_tag.clone().unwrap_or_default(),
        ..Default::default()
    }
}

/// Map an `object_store` error onto the file store error taxonomy.
fn map_store_err(err: object_store::Error) -> StoreError {
    match err {
        object_store::Error::NotFound { path, .. } => StoreError::NotFound(path),
        object_store::Error::AlreadyExists { path, .. } => StoreError::AlreadyExists(path),
        other => StoreError::Internal(other.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_volume_path_root() {
        let p = parse_volume_path("/Volumes/main/default/vol").unwrap();
        assert_eq!(p.full_name, "main.default.vol");
        assert_eq!(p.relative, "");
        assert!(p.list_prefix().is_none());
    }

    #[test]
    fn parse_volume_path_nested() {
        let p = parse_volume_path("/Volumes/main/default/vol/a/b/c.txt").unwrap();
        assert_eq!(p.full_name, "main.default.vol");
        assert_eq!(p.relative, "a/b/c.txt");
        assert_eq!(p.store_path().unwrap().as_ref(), "a/b/c.txt");
    }

    #[test]
    fn parse_volume_path_no_leading_slash() {
        let p = parse_volume_path("Volumes/c/s/v/f.txt").unwrap();
        assert_eq!(p.full_name, "c.s.v");
        assert_eq!(p.relative, "f.txt");
    }

    #[test]
    fn parse_volume_path_case_insensitive_kind() {
        assert!(parse_volume_path("/volumes/c/s/v/f").is_ok());
        assert!(parse_volume_path("/VOLUMES/c/s/v/f").is_ok());
    }

    #[test]
    fn parse_volume_path_rejects_short_or_wrong() {
        assert!(matches!(
            parse_volume_path("/Volumes/c/s"),
            Err(StoreError::InvalidArgument(_))
        ));
        assert!(matches!(
            parse_volume_path("/Tables/c/s/t"),
            Err(StoreError::InvalidArgument(_))
        ));
        assert!(matches!(
            parse_volume_path(""),
            Err(StoreError::InvalidArgument(_))
        ));
    }

    #[test]
    fn absolute_reattaches_volumes_prefix() {
        let p = parse_volume_path("/Volumes/main/default/vol/sub").unwrap();
        assert_eq!(
            p.absolute("sub/file.txt"),
            "/Volumes/main/default/vol/sub/file.txt"
        );
    }

    #[test]
    fn store_path_rejects_volume_root() {
        let p = parse_volume_path("/Volumes/main/default/vol").unwrap();
        assert!(matches!(
            p.store_path(),
            Err(StoreError::InvalidArgument(_))
        ));
    }

    #[test]
    fn byte_range_variants() {
        assert!(byte_range(None, None).unwrap().is_none());
        assert!(matches!(
            byte_range(Some(5), None).unwrap(),
            Some(GetRange::Offset(5))
        ));
        assert!(matches!(
            byte_range(Some(5), Some(10)).unwrap(),
            Some(GetRange::Bounded(r)) if r == (5..15)
        ));
        assert!(matches!(
            byte_range(Some(0), Some(4)).unwrap(),
            Some(GetRange::Bounded(r)) if r == (0..4)
        ));
        assert!(byte_range(Some(-1), None).is_err());
        assert!(byte_range(Some(0), Some(-1)).is_err());
    }
}
