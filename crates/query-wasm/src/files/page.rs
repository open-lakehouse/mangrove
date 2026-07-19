//! Directory-listing DTOs + in-engine pagination (native-compilable + tested).
//!
//! `object_store`'s `list_with_delimiter` collects the whole result set
//! internally and exposes no continuation cursor, so the files engine pages the
//! fully-collected listing itself: sort deterministically, window by
//! `max_results`, and hand back the stringified offset of the next window as an
//! opaque `next_page_token`. This module holds that transport-agnostic logic so
//! it is unit-tested with ordinary native `cargo test`, independent of the
//! wasm-only [`engine`](super::engine) glue that drives the real store.

use crate::error::{Error, Result};

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

/// Window a fully-collected, sorted entry list by `max_results` starting at the
/// offset decoded from `page_token`, returning the window plus the next offset
/// token (when more entries remain).
///
/// `max_results` unset (or `0`) means "no window": everything from the offset is
/// returned with no continuation token. A `page_token` that isn't a
/// non-negative integer is rejected as [`Error::InvalidUrl`].
pub fn paginate(
    entries: Vec<FileEntry>,
    max_results: Option<u32>,
    page_token: Option<String>,
) -> Result<(Vec<FileEntry>, Option<String>)> {
    let offset = match page_token {
        Some(token) => token
            .parse::<usize>()
            .map_err(|_| Error::InvalidUrl(format!("invalid page token: {token:?}")))?,
        None => 0,
    };

    let total = entries.len();
    let start = offset.min(total);
    let limit = max_results.filter(|n| *n > 0).map(|n| n as usize);

    let mut iter = entries.into_iter().skip(start);
    match limit {
        Some(limit) => {
            let window: Vec<FileEntry> = iter.by_ref().take(limit).collect();
            let next = start + window.len();
            let next_token = if next < total {
                Some(next.to_string())
            } else {
                None
            };
            Ok((window, next_token))
        }
        None => Ok((iter.collect(), None)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str) -> FileEntry {
        FileEntry {
            path: path.to_string(),
            is_directory: false,
            file_size: 0,
            last_modified: 0,
        }
    }

    fn paths(entries: &[FileEntry]) -> Vec<&str> {
        entries.iter().map(|e| e.path.as_str()).collect()
    }

    #[test]
    fn windows_and_emits_offset_token() {
        let entries = vec![entry("a"), entry("b"), entry("c"), entry("d"), entry("e")];
        let (page, next) = paginate(entries, Some(2), None).unwrap();
        assert_eq!(paths(&page), ["a", "b"]);
        assert_eq!(next.as_deref(), Some("2"));
    }

    #[test]
    fn resumes_from_token_and_ends_cleanly() {
        let entries = vec![entry("a"), entry("b"), entry("c"), entry("d"), entry("e")];
        let (page, next) = paginate(entries, Some(2), Some("2".to_string())).unwrap();
        assert_eq!(paths(&page), ["c", "d"]);
        assert_eq!(next.as_deref(), Some("4"));

        let entries = vec![entry("a"), entry("b"), entry("c"), entry("d"), entry("e")];
        let (page, next) = paginate(entries, Some(2), Some("4".to_string())).unwrap();
        assert_eq!(paths(&page), ["e"]);
        assert_eq!(next, None, "last partial window has no continuation");
    }

    #[test]
    fn without_limit_returns_all() {
        let entries = vec![entry("a"), entry("b"), entry("c")];
        let (page, next) = paginate(entries, None, None).unwrap();
        assert_eq!(page.len(), 3);
        assert_eq!(next, None);
    }

    #[test]
    fn offset_past_end_is_empty() {
        let entries = vec![entry("a"), entry("b")];
        let (page, next) = paginate(entries, Some(2), Some("10".to_string())).unwrap();
        assert!(page.is_empty());
        assert_eq!(next, None);
    }

    #[test]
    fn rejects_a_garbage_token() {
        let entries = vec![entry("a")];
        assert!(paginate(entries, Some(2), Some("nope".to_string())).is_err());
    }
}
