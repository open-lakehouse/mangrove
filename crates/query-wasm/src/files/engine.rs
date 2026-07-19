//! Volume resolution + read-only file ops glue (wasm-only).
//!
//! Resolves a [`VolumePath`] through Unity Catalog (`GetVolume` →
//! `temporary-volume-credentials` READ_VOLUME → [`crate::creds::resolve_storage`]),
//! then serves the three read-only ops the files seam needs:
//! - `list_directory` — the cloud provider's native list REST call (Azure Blob
//!   container XML / GCS JSON), issued over the vended SAS query / bearer header,
//!   normalized to canonical `/Volumes/…` entries;
//! - `read_file` — a ranged GET through [`UcFetchStore`], streamed chunk-by-chunk;
//! - `stat` — a HEAD through [`UcFetchStore`] mapped to file metadata.
//!
//! The list REST call reuses [`UcFetchStore`]'s CORS/network tagging convention
//! (`network/CORS: …`) so the bindings' `classify()` maps a blocked list to the
//! `NETWORK` fallback code, same as the query path.

use std::sync::Arc;

use bytes::Bytes;
use futures::StreamExt;
use object_store::path::Path as StorePath;
use object_store::{GetOptions, GetRange, ObjectStore};
use send_wrapper::SendWrapper;
use url::Url;

use crate::creds::{ResolvedStorage, resolve_storage};
use crate::error::{Error, Result};
use crate::fetch_store::UcFetchStore;
use crate::files::lister::{
    ListEndpoint, ListPage, ListProvider, parse_azure_list, parse_gcs_list,
    provider_and_list_endpoint,
};
use crate::files::path::VolumePath;
use crate::uc::UcClient;

/// The Blob REST API version the container-list call requires alongside a bearer
/// (matches `creds.rs`'s `AZURE_MS_VERSION`).
const AZURE_MS_VERSION: &str = "2021-08-06";

/// A fully-resolved volume: the credential-scoped storage plus the derived list
/// endpoint and a read/stat store over the same origin.
pub struct ResolvedVolume {
    /// The browser-fetchable storage (table_url carries a SAS query if any;
    /// headers carry bearer/AAD).
    pub storage: ResolvedStorage,
    /// Which provider lists this volume.
    pub provider: ListProvider,
    /// The container/bucket-scoped list endpoint + volume-root key.
    pub endpoint: ListEndpoint,
    /// A read-only store over the storage origin for ranged GET / HEAD.
    pub store: Arc<UcFetchStore>,
}

/// A normalized directory entry with a canonical absolute `/Volumes/…` path.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
    pub path: String,
    pub is_directory: bool,
    pub file_size: u64,
    /// Epoch millis.
    pub last_modified: i64,
}

/// One page of a directory listing (serializes to the TS `DirectoryPage`).
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirectoryPage {
    pub entries: Vec<FileEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_page_token: Option<String>,
}

/// File metadata (serializes to the TS `FileMetadata`).
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    pub path: String,
    pub file_size: u64,
    /// Epoch millis.
    pub last_modified: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub etag: Option<String>,
}

/// Resolve a volume path through Unity Catalog into a [`ResolvedVolume`].
pub async fn resolve_volume(uc: &UcClient, path: &VolumePath) -> Result<ResolvedVolume> {
    let volume = uc.get_volume(&path.full_name()).await?;
    let credential = uc.read_volume_credentials(&volume.volume_id).await?;
    let storage = resolve_storage(&volume.storage_location, &credential)?;
    let (provider, endpoint) = provider_and_list_endpoint(&storage)?;
    let store = Arc::new(UcFetchStore::try_new(
        storage.table_url.clone(),
        &storage.headers,
    )?);
    Ok(ResolvedVolume {
        storage,
        provider,
        endpoint,
        store,
    })
}

/// List one page of a directory's immediate children, returning a serializable
/// [`DirectoryPage`] with canonical `/Volumes/…` entry paths.
pub async fn list_directory(
    uc: &UcClient,
    path: &VolumePath,
    max_results: Option<u32>,
    page_token: Option<String>,
) -> Result<DirectoryPage> {
    let resolved = resolve_volume(uc, path).await?;

    // The list prefix is the volume-root key plus the relative sub-path, with a
    // trailing `/` so the delimiter rollup returns immediate children. The
    // volume-root key comes from the *endpoint* (derived from the resolved
    // storage), which is authoritative; `path.list_prefix` on the endpoint root
    // key gives the container-relative prefix.
    let prefix = path.list_prefix(&resolved.endpoint.root_key);

    let list_url = build_list_url(
        resolved.provider,
        &resolved.endpoint,
        &prefix,
        max_results,
        page_token.as_deref(),
    )?;

    let body = fetch_list_body(&resolved.storage, list_url).await?;
    let page: ListPage = match resolved.provider {
        ListProvider::Azure => parse_azure_list(&body)?,
        ListProvider::Gcs => parse_gcs_list(&body)?,
    };

    let entries = page
        .entries
        .into_iter()
        .map(|raw| FileEntry {
            path: path.absolute(&resolved.endpoint.root_key, &raw.path),
            is_directory: raw.is_directory,
            file_size: raw.size,
            last_modified: raw.last_modified,
        })
        .collect();

    Ok(DirectoryPage {
        entries,
        next_page_token: page.next_token,
    })
}

/// Read a file (or byte range), invoking `on_chunk(&Bytes)` for each body chunk
/// in file order. Returns the total number of bytes read.
pub async fn read_file<F>(
    uc: &UcClient,
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
    let resolved = resolve_volume(uc, path).await?;
    let key = path.object_key(&resolved.endpoint.root_key);
    let location = StorePath::from(key.as_str());

    let options = GetOptions {
        range: byte_range(offset, length),
        ..Default::default()
    };
    let result = resolved.store.get_opts(&location, options).await?;

    // Drive the object store's own chunked byte stream — bytes flow straight
    // from `fetch` to the callback without materializing the whole object. Wrap
    // in SendWrapper: the underlying fetch stream is `!Send`, and `into_stream`
    // requires `Send` on its bound even on the single-threaded wasm target.
    let mut total: u64 = 0;
    let mut stream = SendWrapper::new(result.into_stream());
    while let Some(chunk) = stream.next().await {
        let bytes = chunk?;
        total += bytes.len() as u64;
        on_chunk(&bytes)?;
    }
    Ok(total)
}

/// HEAD a file and map the metadata to a serializable [`FileMetadata`] with a
/// canonical `/Volumes/…` path.
pub async fn stat(uc: &UcClient, path: &VolumePath) -> Result<FileMetadata> {
    if path.is_root() {
        return Err(Error::InvalidUrl(
            "a file path is required (got a volume root)".to_string(),
        ));
    }
    let resolved = resolve_volume(uc, path).await?;
    let key = path.object_key(&resolved.endpoint.root_key);
    let location = StorePath::from(key.as_str());

    let options = GetOptions {
        head: true,
        ..Default::default()
    };
    let result = resolved.store.get_opts(&location, options).await?;

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

/// Build the provider-specific list URL with the per-listing query params
/// layered onto the endpoint base (preserving an Azure SAS query already on it).
fn build_list_url(
    provider: ListProvider,
    endpoint: &ListEndpoint,
    prefix: &str,
    max_results: Option<u32>,
    page_token: Option<&str>,
) -> Result<Url> {
    let mut url = endpoint.base.clone();
    match provider {
        ListProvider::Azure => {
            // Azure Blob container list: append the well-known list params to any
            // SAS query already present.
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("restype", "container");
            pairs.append_pair("comp", "list");
            pairs.append_pair("delimiter", "/");
            if !prefix.is_empty() {
                pairs.append_pair("prefix", prefix);
            }
            if let Some(n) = max_results {
                pairs.append_pair("maxresults", &n.to_string());
            }
            if let Some(token) = page_token {
                pairs.append_pair("marker", token);
            }
        }
        ListProvider::Gcs => {
            let mut pairs = url.query_pairs_mut();
            pairs.append_pair("delimiter", "/");
            if !prefix.is_empty() {
                pairs.append_pair("prefix", prefix);
            }
            if let Some(n) = max_results {
                pairs.append_pair("maxResults", &n.to_string());
            }
            if let Some(token) = page_token {
                pairs.append_pair("pageToken", token);
            }
        }
    }
    Ok(url)
}

/// Issue the native cloud list GET carrying the vended credential headers, and
/// return the raw response body. Uses the same `network/CORS:` tagging as
/// `UcFetchStore` so `classify()` maps a blocked list to `NETWORK`.
async fn fetch_list_body(storage: &ResolvedStorage, url: Url) -> Result<Bytes> {
    let fut = SendWrapper::new(async move {
        let client = reqwest::Client::new();
        let mut request = client.get(url);
        // Attach the static headers (bearer / AAD). Azure OAuth also needs the
        // Blob REST version header; add it when an authorization header is present
        // and no `x-ms-version` was supplied.
        let mut has_ms_version = false;
        let mut has_authorization = false;
        for (name, value) in &storage.headers {
            if name.eq_ignore_ascii_case("x-ms-version") {
                has_ms_version = true;
            }
            if name.eq_ignore_ascii_case("authorization") {
                has_authorization = true;
            }
            request = request.header(name.as_str(), value.as_str());
        }
        if has_authorization && !has_ms_version {
            request = request.header("x-ms-version", AZURE_MS_VERSION);
        }

        let response = request
            .send()
            .await
            .map_err(|e| Error::UnityCatalog(format!("listDirectory: network/CORS: {e}")))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|e| Error::UnityCatalog(format!("listDirectory: network/CORS: {e}")))?;
        if !status.is_success() {
            return Err(Error::UnityCatalog(format!(
                "listDirectory: HTTP {status}: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }
        Ok(bytes)
    });
    fut.await
}
