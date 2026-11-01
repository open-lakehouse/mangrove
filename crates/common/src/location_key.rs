use std::fmt;

use olai_storage_credentials::StorageLocation;
#[cfg(feature = "store")]
use olai_store::{Filter, PropertyIndex};

#[cfg(feature = "store")]
use crate::models::ObjectLabel;
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

/// The stored property holding a resource's [`LocationKey`].
///
/// The store writes it next to the payload of every table, volume, agent skill and
/// staging table whose location parses (see `TryFrom<Resource> for Object`), so
/// location lookups can be pushed into the store as [`Filter`]s. The API models
/// don't declare it and ignore it on read.
#[cfg(feature = "store")]
pub const LOCATION_KEY_PROPERTY: &str = "location_key";

/// The labels stored with a [`LOCATION_KEY_PROPERTY`].
#[cfg(feature = "store")]
pub const LOCATION_KEYED_LABELS: [ObjectLabel; 4] = [
    ObjectLabel::Table,
    ObjectLabel::Volume,
    ObjectLabel::AgentSkill,
    ObjectLabel::StagingTable,
];

/// The [`PropertyIndex`]es serving location searches: one per
/// [`LOCATION_KEYED_LABELS`] entry, on [`LOCATION_KEY_PROPERTY`].
///
/// Pass them to the SQL stores' `with_property_indexes`, and create them with
/// `ensure_property_indexes` alongside the migrations.
#[cfg(feature = "store")]
pub fn location_key_indexes() -> Vec<PropertyIndex> {
    LOCATION_KEYED_LABELS
        .into_iter()
        .map(|label| {
            PropertyIndex::new(label, LOCATION_KEY_PROPERTY)
                .expect("`location_key` is a valid property index path")
        })
        .collect()
}

#[cfg(feature = "store")]
impl LocationKey {
    /// Matches resources stored at exactly this location.
    pub fn at(&self) -> Filter {
        Filter::eq(LOCATION_KEY_PROPERTY, self.as_str())
    }

    /// Matches resources at or below this location.
    pub fn covered_by(&self) -> Filter {
        let (lower, upper) = self.descendants();
        Filter::all([
            Filter::ge(LOCATION_KEY_PROPERTY, lower),
            Filter::lt(LOCATION_KEY_PROPERTY, upper),
        ])
    }

    /// Matches resources at or above this location: an IN-list over its
    /// [`ancestors`](Self::ancestors).
    pub fn covering(&self) -> Filter {
        Filter::any(self.ancestors().iter().map(LocationKey::at))
    }

    /// The filters matching resources whose location overlaps this one, in either
    /// direction: [`covering`](Self::covering) and [`covered_by`](Self::covered_by).
    ///
    /// A resource overlaps when it matches either one. They stay two filters to
    /// search one after the other, because a store's property index serves an
    /// IN-list or a range, but not an `Or` of both.
    pub fn overlapping(&self) -> [Filter; 2] {
        [self.covering(), self.covered_by()]
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

    #[cfg(feature = "store")]
    #[test]
    fn filters_agree_with_covers() {
        let stored = |url: &str| serde_json::json!({ LOCATION_KEY_PROPERTY: key(url).as_str() });
        let k = key("s3://b/data/x");
        for url in ["s3://b/data/x", "s3://b/data/x/", "s3a://b/data/x"] {
            assert!(k.at().matches(&stored(url)), "{url}");
        }
        for url in ["s3://b/data/x/y", "s3://b/data/xy", "s3://b/data", "s3://b"] {
            assert_eq!(
                k.covered_by().matches(&stored(url)),
                k.covers(&key(url)),
                "{url}"
            );
            assert_eq!(
                k.overlapping().iter().any(|f| f.matches(&stored(url))),
                k.overlaps(&key(url)),
                "{url}"
            );
        }
        for url in [
            "s3://b/data/xy",
            "s3://b/dat",
            "s3://bb/data/x",
            "gs://b/data/x",
        ] {
            assert!(
                !k.overlapping().iter().any(|f| f.matches(&stored(url))),
                "{url}"
            );
        }
        // A resource without a key matches nothing.
        for filter in k.overlapping() {
            assert!(!filter.matches(&serde_json::json!({})));
        }
    }

    #[cfg(feature = "store")]
    #[test]
    fn indexes_cover_every_keyed_label() {
        let indexes = location_key_indexes();
        assert_eq!(indexes.len(), LOCATION_KEYED_LABELS.len());
        for (index, label) in indexes.iter().zip(LOCATION_KEYED_LABELS) {
            assert_eq!(index.label(), olai_store::Label::as_str(&label));
            assert_eq!(index.path().segments(), [LOCATION_KEY_PROPERTY]);
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
