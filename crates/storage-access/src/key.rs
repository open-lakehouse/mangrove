use std::fmt;

use olai_storage_credentials::StorageLocation;

use crate::{Error, Result};

/// The canonical form of a storage location, used to match locations as strings.
///
/// The key is the location's canonical URL with a trailing `/`, so a location
/// covers another exactly when its key is a string prefix of the other's: whole path
/// segments only (`s3://b/tab/` is not a prefix of `s3://b/table2/`), and every
/// spelling of a URL has the same key. A backend can store keys and match them with
/// a prefix comparison. Both directions of that comparison are index lookups:
/// the locations covering a key are among its [`ancestors`](Self::ancestors), and the
/// locations a key covers sort within [`descendants`](Self::descendants) under a
/// bytewise collation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LocationKey(String);

impl LocationKey {
    /// The key of `location`.
    pub fn new(location: &StorageLocation) -> Self {
        let mut key = location.to_string();
        if !key.ends_with('/') {
            key.push('/');
        }
        Self(key)
    }

    /// The key of the storage URL `url`.
    ///
    /// Returns [`Error::InvalidArgument`] for a URL that names no storage location.
    pub fn parse(url: &str) -> Result<Self> {
        let location: StorageLocation = url.parse().map_err(|e| {
            Error::invalid_argument(format!("invalid storage location '{url}': {e}"))
        })?;
        Ok(Self::new(&location))
    }

    /// Wraps a key read back from storage, as written by [`as_str`](Self::as_str).
    pub fn from_stored(key: String) -> Self {
        Self(key)
    }

    /// The key as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Whether `other` is this location or lies below it.
    pub fn covers(&self, other: &LocationKey) -> bool {
        other.0.starts_with(&self.0)
    }

    /// Whether either location covers the other.
    pub fn overlaps(&self, other: &LocationKey) -> bool {
        self.covers(other) || other.covers(self)
    }

    /// Every key covering this one, this one included, from the bucket root down.
    ///
    /// A path segment never holds an unescaped `/`, so the ancestors are the prefixes
    /// ending at each `/` from the one closing the authority on.
    pub fn ancestors(&self) -> Vec<LocationKey> {
        let after_scheme = self.0.find("://").map_or(0, |i| i + 3);
        let root = self.0[after_scheme..]
            .find('/')
            .map_or(self.0.len(), |i| after_scheme + i + 1);
        self.0
            .char_indices()
            .filter(|&(i, c)| c == '/' && i + 1 >= root)
            .map(|(i, _)| Self(self.0[..=i].to_owned()))
            .collect()
    }

    /// The bounds of the keys this one covers: every such key `k` has
    /// `lower <= k < upper` in bytewise order, and no other key does.
    pub fn descendants(&self) -> (&str, String) {
        // Keys end in `/`; the covered ones sort before the same prefix ending in the
        // next byte, `0`.
        let mut upper = self.0[..self.0.len() - 1].to_owned();
        upper.push('0');
        (&self.0, upper)
    }
}

impl fmt::Display for LocationKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(url: &str) -> LocationKey {
        LocationKey::parse(url).unwrap()
    }

    #[test]
    fn covers_on_whole_segments() {
        assert!(key("s3://b/tab").covers(&key("s3://b/tab/x")));
        assert!(key("s3://b/tab").covers(&key("s3://b/tab")));
        assert!(!key("s3://b/tab").covers(&key("s3://b/table2")));
        assert!(!key("s3://b/tab/x").covers(&key("s3://b/tab")));
        assert!(key("s3://b").covers(&key("s3://b/tab")));
        assert!(!key("s3://b").covers(&key("s3://bb/tab")));
    }

    #[test]
    fn spellings_share_a_key() {
        assert_eq!(key("s3://b/tab/"), key("s3://b/tab"));
        assert_eq!(key("s3a://b/tab"), key("s3://b/tab"));
        assert_eq!(key("https://b.s3.amazonaws.com/tab"), key("s3://b/tab"));
        assert_eq!(
            key("https://acct.dfs.core.windows.net/ctr/data"),
            key("abfss://ctr@acct.dfs.core.windows.net/data"),
        );
    }

    #[test]
    fn overlap_is_symmetric() {
        assert!(key("s3://b/data").overlaps(&key("s3://b/data/sub")));
        assert!(key("s3://b/data/sub").overlaps(&key("s3://b/data")));
        assert!(!key("s3://b/data").overlaps(&key("s3://b/database")));
        assert!(!key("s3://b/data").overlaps(&key("gs://b/data")));
    }

    #[test]
    fn ancestors_run_from_the_root() {
        let ancestors: Vec<_> = key("s3://b/a/x").ancestors();
        let ancestors: Vec<_> = ancestors.iter().map(LocationKey::as_str).collect();
        assert_eq!(ancestors, ["s3://b/", "s3://b/a/", "s3://b/a/x/"]);

        let ancestors = key("abfss://ctr@acct.dfs.core.windows.net/d").ancestors();
        let ancestors: Vec<_> = ancestors.iter().map(LocationKey::as_str).collect();
        assert_eq!(
            ancestors,
            [
                "abfss://ctr@acct.dfs.core.windows.net/",
                "abfss://ctr@acct.dfs.core.windows.net/d/"
            ]
        );

        let ancestors = key("file:///tmp/x").ancestors();
        let ancestors: Vec<_> = ancestors.iter().map(LocationKey::as_str).collect();
        assert_eq!(ancestors, ["file:///", "file:///tmp/", "file:///tmp/x/"]);

        assert_eq!(key("s3://b").ancestors(), vec![key("s3://b")]);
    }

    #[test]
    fn ancestors_are_exactly_the_covering_keys() {
        let k = key("s3://b/a/x/y");
        for ancestor in k.ancestors() {
            assert!(ancestor.covers(&k));
        }
        for other in ["s3://b/a/xy", "s3://b/ab", "s3://bb/a", "s3://b/a/x/y/z"] {
            assert!(!k.ancestors().contains(&key(other)), "{other}");
        }
    }

    #[test]
    fn descendants_bound_the_covered_keys() {
        let k = key("s3://b/data");
        let (lower, upper) = k.descendants();
        let within = |url: &str| {
            let other = key(url);
            lower <= other.as_str() && other.as_str() < upper.as_str()
        };
        for url in ["s3://b/data", "s3://b/data/x", "s3://b/data/x/y"] {
            assert!(within(url), "{url}");
            assert!(k.covers(&key(url)));
        }
        for url in [
            "s3://b/database",
            "s3://b/dat",
            "s3://b/data-x",
            "s3://b",
            "s3://b/e",
        ] {
            assert!(!within(url), "{url}");
            assert!(!k.covers(&key(url)));
        }
    }

    #[test]
    fn rejects_unknown_hosts() {
        assert!(matches!(
            LocationKey::parse("https://example.com/x"),
            Err(Error::InvalidArgument(_))
        ));
    }
}
