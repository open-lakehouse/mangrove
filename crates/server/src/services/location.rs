//! Storage locations of governed securables.
//!
//! A [`StorageLocationUrl`] keeps the URL a client sent (for messages and local
//! paths) next to its canonical [`StorageLocation`] from `olai-storage-credentials`,
//! which every comparison uses: two spellings of one location (`abfss://` and
//! `https://…dfs.core.windows.net`, say) match, and prefixes match on whole path
//! segments only.

use object_store::path::Path;
use olai_storage_credentials::{StorageLocation, StorageProvider, StorageScope};
use url::Url;

use crate::{Error, Result};

/// A parsed storage location: the URL as given plus its canonical form.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StorageLocationUrl {
    raw: Url,
    location: StorageLocation,
}

impl StorageLocationUrl {
    /// Parses a storage URL.
    pub fn parse(url: impl AsRef<str>) -> Result<Self> {
        let url = url.as_ref();
        let raw = Url::parse(url)
            .map_err(|e| Error::invalid_argument(format!("invalid storage location: {e}")))?;
        let location =
            StorageLocation::parse(&raw).map_err(|e| Error::invalid_argument(e.to_string()))?;
        Ok(Self { raw, location })
    }

    /// The URL as given.
    pub fn raw(&self) -> &Url {
        &self.raw
    }

    /// The canonical location.
    pub fn location(&self) -> &StorageLocation {
        &self.location
    }

    /// The scope a credential for this location covers.
    pub fn scope(&self) -> StorageScope {
        StorageScope::new(self.location.clone())
    }

    /// The object path within the bucket.
    pub fn path(&self) -> &Path {
        self.location.path()
    }

    /// Whether this is a local (`file://`) location.
    pub fn is_local(&self) -> bool {
        self.location.provider() == StorageProvider::Local
    }

    /// Whether `self` equals `prefix` or lies below it, on whole path segments.
    pub fn starts_with(&self, prefix: &StorageLocationUrl) -> bool {
        self.location.starts_with(&prefix.location)
    }

    /// Whether either location equals or lies below the other.
    pub fn overlaps(&self, other: &StorageLocationUrl) -> bool {
        self.starts_with(other) || other.starts_with(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(url: &str) -> StorageLocationUrl {
        StorageLocationUrl::parse(url).unwrap()
    }

    #[test]
    fn overlap_is_segment_aware_and_symmetric() {
        assert!(parse("s3://b/data").overlaps(&parse("s3://b/data/sub")));
        assert!(parse("s3://b/data/sub").overlaps(&parse("s3://b/data")));
        assert!(parse("s3://b/data").overlaps(&parse("s3://b/data/")));
        assert!(!parse("s3://b/data").overlaps(&parse("s3://b/data-secret")));
        assert!(!parse("s3://b/data").overlaps(&parse("s3://c/data")));
    }

    #[test]
    fn azure_spellings_compare_canonically() {
        let abfss = parse("abfss://container@account.dfs.core.windows.net/data/tab");
        let https = parse("https://account.blob.core.windows.net/container/data");
        assert!(abfss.starts_with(&https));
        assert!(!https.starts_with(&abfss));
    }

    #[test]
    fn local_locations_are_detected() {
        assert!(parse("file:///tmp/data").is_local());
        assert!(!parse("s3://b/data").is_local());
        assert!(StorageLocationUrl::parse("not a url").is_err());
    }
}
