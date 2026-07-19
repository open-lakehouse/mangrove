//! Native cloud directory-listing: build the container/bucket-scoped list REST
//! request from a resolved storage location, and parse the Azure (XML) / GCS
//! (JSON) response bodies into a normalized [`ListPage`].
//!
//! [`crate::fetch_store::UcFetchStore`]'s `list` is `NotSupported` — plain
//! ranged GET/HEAD over `fetch` cannot enumerate a prefix. So directory browsing
//! issues the cloud provider's *native* list REST call directly (over the same
//! vended SAS query string / bearer header the store carries) and normalizes the
//! result here.
//!
//! The two provider bodies differ:
//! - **Azure Blob** list is **container-scoped**
//!   (`<account>.blob…/<container>?restype=container&comp=list&…`) and returns an
//!   `<EnumerationResults>` XML document. The resolved table URL points at the
//!   volume-root *object path* inside that container, so the list endpoint has to
//!   be re-derived to the container root while the object path becomes the list
//!   `prefix`.
//! - **GCS** list is served by the **JSON API host**
//!   (`storage.googleapis.com/storage/v1/b/<bucket>/o?…`), which differs from the
//!   download host (`storage.googleapis.com/<bucket>/…`) the query path uses; the
//!   same bearer authorizes both. Its body is JSON.
//!
//! The parsers are pure and byte-in / struct-out, so they are unit-tested
//! natively against committed fixture bodies. The endpoint mapping is likewise
//! derived from [`ResolvedStorage`] alone and native-testable. AWS / R2 hosts
//! are [`Error::Unsupported`] (SigV4 signing, deferred).

use crate::creds::ResolvedStorage;
use crate::error::{Error, Result};

/// A single normalized listing entry (a blob or a common-prefix directory).
/// Keys are **container/bucket-relative** — the caller re-attaches the Volumes
/// prefix via [`crate::files::path::VolumePath::absolute`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntry {
    /// Store key relative to the container/bucket (a blob name, or a common
    /// prefix ending in `/` for a directory).
    pub path: String,
    /// True for a common-prefix (subdirectory) entry.
    pub is_directory: bool,
    /// Size in bytes; `0` for directories.
    pub size: u64,
    /// Last-modified time in epoch milliseconds; `0` for directories.
    pub last_modified: i64,
}

/// One page of a normalized directory listing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListPage {
    /// Entries in this page (directories first is not guaranteed; the caller
    /// sorts).
    pub entries: Vec<RawEntry>,
    /// Provider continuation token; `Some` iff more pages remain.
    pub next_token: Option<String>,
}

/// Which cloud provider's native list REST call + body format applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListProvider {
    /// Azure Blob container list (XML `EnumerationResults`).
    Azure,
    /// GCS JSON-API object list.
    Gcs,
}

/// The container/bucket-scoped list endpoint derived from a [`ResolvedStorage`]:
/// a base URL to which per-request query params (`prefix`, `delimiter`, page
/// token) are appended, plus the object-key prefix of the volume root within
/// that container/bucket.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListEndpoint {
    /// The list URL *without* the per-listing query params — for Azure it
    /// carries the container path and (if present) the SAS query string; for GCS
    /// it is the JSON-API `…/b/<bucket>/o` URL. Bearer/SAS credentials are NOT
    /// in here beyond a SAS query string; header credentials come from
    /// [`ResolvedStorage::headers`].
    pub base: url::Url,
    /// The store key of the volume root within the container/bucket (the value
    /// the list `prefix` is built from). Empty when the volume sits at the
    /// container/bucket root.
    pub root_key: String,
}

/// Inspect a resolved storage location and decide which provider lists it and
/// how the container/bucket-scoped list endpoint is addressed.
///
/// - Azure (`*.blob.core.windows.net` or the Azurite emulator path host): the
///   first path segment is the container; the list endpoint is the container
///   root with `restype=container&comp=list`, and the remaining path is the
///   volume-root key.
/// - GCS (`storage.googleapis.com/<bucket>/…`): the first path segment is the
///   bucket; the list endpoint moves to the JSON-API path
///   `/storage/v1/b/<bucket>/o` on the same host, and the remaining path is the
///   volume-root key.
/// - Any other host is [`Error::Unsupported`] (AWS / R2 need SigV4).
pub fn provider_and_list_endpoint(
    storage: &ResolvedStorage,
) -> Result<(ListProvider, ListEndpoint)> {
    let url = &storage.table_url;
    let host = url
        .host_str()
        .ok_or_else(|| Error::InvalidUrl(format!("storage url has no host: {url}")))?;

    let segments: Vec<String> = url
        .path_segments()
        .map(|s| s.filter(|p| !p.is_empty()).map(str::to_owned).collect())
        .unwrap_or_default();

    if host.contains(".blob.") || is_azurite_host(host) {
        azure_endpoint(url, host, &segments)
    } else if host == "storage.googleapis.com" {
        gcs_endpoint(url, &segments)
    } else if host.contains("amazonaws.com")
        || host.contains("r2.cloudflarestorage.com")
        || host.contains("s3.")
    {
        Err(Error::unsupported(format!(
            "listing on `{host}` needs SigV4 request signing, which the in-browser \
             files engine does not do yet"
        )))
    } else {
        Err(Error::unsupported(format!(
            "unrecognized storage host for directory listing: `{host}`"
        )))
    }
}

/// The default Azurite emulator host uses path-style addressing with the account
/// as the first path segment (`/devstoreaccount1/<container>/…`).
fn is_azurite_host(host: &str) -> bool {
    host == "127.0.0.1" || host == "localhost"
}

fn azure_endpoint(
    url: &url::Url,
    host: &str,
    segments: &[String],
) -> Result<(ListProvider, ListEndpoint)> {
    // Path-style Azurite carries the account as the first segment; the true
    // container is the next one. The blob-endpoint form has the container as the
    // very first segment.
    let (container_idx, container) = if is_azurite_host(host) {
        // /devstoreaccount1/<container>/<key…>
        let c = segments.get(1).ok_or_else(|| {
            Error::InvalidUrl(format!("azurite list url has no container: {url}"))
        })?;
        (2usize, c.clone())
    } else {
        let c = segments
            .first()
            .ok_or_else(|| Error::InvalidUrl(format!("azure list url has no container: {url}")))?;
        (1usize, c.clone())
    };

    // The list endpoint is the container root; the key prefix is everything past
    // the container.
    let root_key = segments[container_idx..].join("/");

    let mut base = url.clone();
    {
        let account_prefix: Vec<&str> = if is_azurite_host(host) {
            // Keep the account path segment for path-style hosts.
            vec![&segments[0], &container]
        } else {
            vec![&container]
        };
        let mut seg = base
            .path_segments_mut()
            .map_err(|_| Error::InvalidUrl(format!("cannot-be-a-base azure url: {url}")))?;
        seg.clear();
        for s in account_prefix {
            seg.push(s);
        }
    }
    // The SAS query string (if any) is preserved on `base` via clone; the list
    // params are layered on at request time by the caller.

    Ok((ListProvider::Azure, ListEndpoint { base, root_key }))
}

fn gcs_endpoint(url: &url::Url, segments: &[String]) -> Result<(ListProvider, ListEndpoint)> {
    // Download host path is `/<bucket>/<key…>`; the JSON list API is
    // `/storage/v1/b/<bucket>/o` on the same host.
    let bucket = segments
        .first()
        .ok_or_else(|| Error::InvalidUrl(format!("gcs list url has no bucket: {url}")))?;
    let root_key = segments[1..].join("/");

    let mut base = url.clone();
    base.set_query(None);
    {
        let mut seg = base
            .path_segments_mut()
            .map_err(|_| Error::InvalidUrl(format!("cannot-be-a-base gcs url: {url}")))?;
        seg.clear();
        seg.push("storage");
        seg.push("v1");
        seg.push("b");
        seg.push(bucket);
        seg.push("o");
    }

    Ok((ListProvider::Gcs, ListEndpoint { base, root_key }))
}

// =====================================================================
// Azure XML `EnumerationResults` parser
// =====================================================================

/// Parse an Azure Blob `list` (`comp=list`) XML `EnumerationResults` body.
///
/// Hand-rolled (no `quick-xml` dep — this crate is size-tuned) tag scan over the
/// well-known, flat shape the Blob REST API emits: `<Blob>` elements (with
/// `<Name>` and a `<Properties>` block carrying `<Content-Length>` and
/// `<Last-Modified>` in RFC 1123), `<BlobPrefix>` elements (`<Name>` only, the
/// common-prefix directories from `delimiter=/`), and a trailing `<NextMarker>`
/// continuation token.
pub fn parse_azure_list(body: &[u8]) -> Result<ListPage> {
    let text = std::str::from_utf8(body)
        .map_err(|e| Error::InvalidResponse(format!("azure list body not utf-8: {e}")))?;

    let mut entries = Vec::new();

    // BlobPrefix directories: <BlobPrefix><Name>foo/</Name></BlobPrefix>
    for block in iter_blocks(text, "<BlobPrefix>", "</BlobPrefix>") {
        if let Some(name) = inner(block, "<Name>", "</Name>") {
            entries.push(RawEntry {
                path: xml_unescape(name),
                is_directory: true,
                size: 0,
                last_modified: 0,
            });
        }
    }

    // Blob files: <Blob><Name>…</Name><Properties>…</Properties></Blob>
    for block in iter_blocks(text, "<Blob>", "</Blob>") {
        let Some(name) = inner(block, "<Name>", "</Name>") else {
            continue;
        };
        let size = inner(block, "<Content-Length>", "</Content-Length>")
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let last_modified = inner(block, "<Last-Modified>", "</Last-Modified>")
            .map(str::trim)
            .and_then(parse_rfc1123_millis)
            .unwrap_or(0);
        entries.push(RawEntry {
            path: xml_unescape(name),
            is_directory: false,
            size,
            last_modified,
        });
    }

    // Continuation: <NextMarker>token</NextMarker> (empty element = last page).
    let next_token = inner(text, "<NextMarker>", "</NextMarker>")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(xml_unescape);

    Ok(ListPage {
        entries,
        next_token,
    })
}

/// Yield each `open`…`close` block (exclusive of the delimiters), in document
/// order. Non-nesting elements only, which the Blob list shape guarantees.
fn iter_blocks<'a>(text: &'a str, open: &'a str, close: &'a str) -> impl Iterator<Item = &'a str> {
    let mut rest = text;
    std::iter::from_fn(move || {
        let start = rest.find(open)? + open.len();
        let after = &rest[start..];
        let end = after.find(close)?;
        let block = &after[..end];
        rest = &after[end + close.len()..];
        Some(block)
    })
}

/// The text between the first `open` and its matching `close` within `block`.
fn inner<'a>(block: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let start = block.find(open)? + open.len();
    let after = &block[start..];
    let end = after.find(close)?;
    Some(&after[..end])
}

/// Minimal XML entity unescape for the five predefined entities (blob names may
/// contain `&`).
fn xml_unescape(s: &str) -> String {
    if !s.contains('&') {
        return s.to_string();
    }
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// RFC 1123 date (`Tue, 09 Aug 2022 12:00:00 GMT`) → epoch millis.
fn parse_rfc1123_millis(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc2822(s)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

// =====================================================================
// GCS JSON object-list parser
// =====================================================================

#[derive(serde::Deserialize)]
struct GcsListBody {
    #[serde(default)]
    items: Vec<GcsObject>,
    #[serde(default)]
    prefixes: Vec<String>,
    #[serde(default, rename = "nextPageToken")]
    next_page_token: Option<String>,
}

#[derive(serde::Deserialize)]
struct GcsObject {
    name: String,
    /// Size is a stringified int64 in the JSON API.
    #[serde(default)]
    size: Option<String>,
    #[serde(default)]
    updated: Option<String>,
}

/// Parse a GCS JSON-API object list (`.../o?delimiter=/`) body: `items` are
/// files, `prefixes` are common-prefix directories, `nextPageToken` continues.
pub fn parse_gcs_list(body: &[u8]) -> Result<ListPage> {
    let parsed: GcsListBody = serde_json::from_slice(body)
        .map_err(|e| Error::InvalidResponse(format!("gcs list body: {e}")))?;

    let mut entries = Vec::new();
    for prefix in parsed.prefixes {
        entries.push(RawEntry {
            path: prefix,
            is_directory: true,
            size: 0,
            last_modified: 0,
        });
    }
    for obj in parsed.items {
        let size = obj
            .size
            .as_deref()
            .and_then(|v| v.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let last_modified = obj
            .updated
            .as_deref()
            .and_then(parse_rfc3339_millis)
            .unwrap_or(0);
        entries.push(RawEntry {
            path: obj.name,
            is_directory: false,
            size,
            last_modified,
        });
    }

    Ok(ListPage {
        entries,
        next_token: parsed.next_page_token.filter(|s| !s.is_empty()),
    })
}

/// RFC 3339 timestamp (`2022-08-09T12:00:00.000Z`) → epoch millis.
fn parse_rfc3339_millis(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp_millis())
}

// Native-only: pure parsers + endpoint mapping, tested against fixtures.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use crate::creds::ResolvedStorage;
    use url::Url;

    const AZURE_XML: &[u8] = br#"<?xml version="1.0" encoding="utf-8"?>
<EnumerationResults ServiceEndpoint="https://acct.blob.core.windows.net/" ContainerName="data">
  <Prefix>volumes/v-uuid/</Prefix>
  <Delimiter>/</Delimiter>
  <Blobs>
    <Blob>
      <Name>volumes/v-uuid/a.parquet</Name>
      <Properties>
        <Last-Modified>Tue, 09 Aug 2022 12:00:00 GMT</Last-Modified>
        <Content-Length>1024</Content-Length>
        <Content-Type>application/octet-stream</Content-Type>
      </Properties>
    </Blob>
    <Blob>
      <Name>volumes/v-uuid/b&amp;c.txt</Name>
      <Properties>
        <Last-Modified>Wed, 10 Aug 2022 00:00:00 GMT</Last-Modified>
        <Content-Length>7</Content-Length>
      </Properties>
    </Blob>
    <BlobPrefix>
      <Name>volumes/v-uuid/sub/</Name>
    </BlobPrefix>
  </Blobs>
  <NextMarker>2!marker!MjAyMg==</NextMarker>
</EnumerationResults>"#;

    const AZURE_XML_LAST_PAGE: &[u8] = br#"<?xml version="1.0"?>
<EnumerationResults>
  <Blobs>
    <Blob><Name>only.bin</Name><Properties><Content-Length>3</Content-Length>
    <Last-Modified>Tue, 09 Aug 2022 12:00:00 GMT</Last-Modified></Properties></Blob>
  </Blobs>
  <NextMarker />
</EnumerationResults>"#;

    const GCS_JSON: &[u8] = br#"{
      "kind": "storage#objects",
      "prefixes": ["volumes/v-uuid/sub/"],
      "items": [
        {"name": "volumes/v-uuid/a.parquet", "size": "2048",
         "updated": "2022-08-09T12:00:00.000Z"},
        {"name": "volumes/v-uuid/b.txt", "size": "9",
         "updated": "2022-08-10T00:00:00Z"}
      ],
      "nextPageToken": "CjRu"
    }"#;

    #[test]
    fn azure_xml_parses_blobs_prefixes_size_and_millis() {
        let page = parse_azure_list(AZURE_XML).unwrap();
        // One directory (BlobPrefix) + two blobs.
        let dirs: Vec<_> = page.entries.iter().filter(|e| e.is_directory).collect();
        let files: Vec<_> = page.entries.iter().filter(|e| !e.is_directory).collect();
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].path, "volumes/v-uuid/sub/");
        assert_eq!(files.len(), 2);

        let a = files
            .iter()
            .find(|e| e.path.ends_with("a.parquet"))
            .unwrap();
        assert_eq!(a.size, 1024);
        // 2022-08-09T12:00:00Z in millis.
        assert_eq!(a.last_modified, 1660046400000);

        // XML entity unescaped in the name.
        assert!(files.iter().any(|e| e.path == "volumes/v-uuid/b&c.txt"));

        assert_eq!(page.next_token.as_deref(), Some("2!marker!MjAyMg=="));
    }

    #[test]
    fn azure_empty_next_marker_is_last_page() {
        let page = parse_azure_list(AZURE_XML_LAST_PAGE).unwrap();
        assert_eq!(page.entries.len(), 1);
        assert!(page.next_token.is_none());
    }

    #[test]
    fn gcs_json_parses_items_prefixes_size_and_millis() {
        let page = parse_gcs_list(GCS_JSON).unwrap();
        let dirs: Vec<_> = page.entries.iter().filter(|e| e.is_directory).collect();
        let files: Vec<_> = page.entries.iter().filter(|e| !e.is_directory).collect();
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].path, "volumes/v-uuid/sub/");
        assert_eq!(files.len(), 2);

        let a = files
            .iter()
            .find(|e| e.path.ends_with("a.parquet"))
            .unwrap();
        assert_eq!(a.size, 2048);
        assert_eq!(a.last_modified, 1660046400000);

        assert_eq!(page.next_token.as_deref(), Some("CjRu"));
    }

    #[test]
    fn gcs_missing_page_token_is_last_page() {
        let page = parse_gcs_list(br#"{"items":[],"prefixes":[]}"#).unwrap();
        assert!(page.entries.is_empty());
        assert!(page.next_token.is_none());
    }

    fn resolved(url: &str, headers: Vec<(&str, &str)>) -> ResolvedStorage {
        ResolvedStorage {
            table_url: Url::parse(url).unwrap(),
            headers: headers
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }

    #[test]
    fn azure_endpoint_maps_container_root_with_sas_and_prefix() {
        // A vended SAS-bearing blob URL for the volume root object path.
        let storage = resolved(
            "https://acct.blob.core.windows.net/data/volumes/v-uuid?sv=2021&sig=X",
            vec![],
        );
        let (provider, endpoint) = provider_and_list_endpoint(&storage).unwrap();
        assert_eq!(provider, ListProvider::Azure);
        // Base is the container root; SAS query preserved.
        assert_eq!(
            endpoint.base.as_str(),
            "https://acct.blob.core.windows.net/data?sv=2021&sig=X"
        );
        assert_eq!(endpoint.root_key, "volumes/v-uuid");
    }

    #[test]
    fn azurite_path_style_endpoint_keeps_account_segment() {
        let storage = resolved(
            "http://127.0.0.1:10000/devstoreaccount1/data/volumes/v-uuid?sig=X",
            vec![],
        );
        let (provider, endpoint) = provider_and_list_endpoint(&storage).unwrap();
        assert_eq!(provider, ListProvider::Azure);
        assert_eq!(
            endpoint.base.as_str(),
            "http://127.0.0.1:10000/devstoreaccount1/data?sig=X"
        );
        assert_eq!(endpoint.root_key, "volumes/v-uuid");
    }

    #[test]
    fn gcs_endpoint_moves_to_json_api_host_with_bearer() {
        let storage = resolved(
            "https://storage.googleapis.com/bucket/volumes/v-uuid",
            vec![("authorization", "Bearer gtok")],
        );
        let (provider, endpoint) = provider_and_list_endpoint(&storage).unwrap();
        assert_eq!(provider, ListProvider::Gcs);
        assert_eq!(
            endpoint.base.as_str(),
            "https://storage.googleapis.com/storage/v1/b/bucket/o"
        );
        assert_eq!(endpoint.root_key, "volumes/v-uuid");
        // Bearer authorizes both hosts; it rides on ResolvedStorage.headers.
        assert_eq!(storage.headers[0].1, "Bearer gtok");
    }

    #[test]
    fn aws_host_is_unsupported() {
        let storage = resolved("https://bucket.s3.amazonaws.com/volumes/v", vec![]);
        assert!(
            provider_and_list_endpoint(&storage)
                .unwrap_err()
                .is_unsupported()
        );
    }
}
