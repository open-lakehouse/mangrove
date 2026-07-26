//! Volumes path model: parse and re-format canonical Databricks Volumes paths
//! (`/Volumes/<catalog>/<schema>/<volume>[/<rest…>]`).
//!
//! Files under a volume are addressed by a Volumes path. Each op parses the path
//! into a three-level volume name plus a relative sub-path, vends a credential
//! scoped to the **volume root**, and runs the storage op against the relative
//! path. Vending at the root keeps one credential usable for every file under the
//! volume and is the correct granularity for listing.
//!
//! Mirrors hydrofoil's `UnityVolumeStore::VolumePath` (and mangrove's
//! `UCReference`): the leading `Volumes` segment is matched case-insensitively,
//! all other segments are preserved verbatim. Everything here is transport-
//! agnostic, so it is unit-tested natively with ordinary `cargo test`.

use crate::error::{Error, Result};

/// A Volumes path split into its three-level volume name and the relative
/// sub-path inside the volume (empty when addressing the volume root).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VolumePath {
    /// The catalog (first level).
    pub catalog: String,
    /// The schema (second level).
    pub schema: String,
    /// The volume (third level).
    pub volume: String,
    /// The relative sub-path inside the volume, `/`-joined and empty when the
    /// path addresses the volume root.
    pub relative: String,
}

impl VolumePath {
    /// Parse a Databricks Volumes path into its components.
    ///
    /// Accepts `[/]Volumes/<catalog>/<schema>/<volume>[/<rest…>]`. The leading
    /// `Volumes` segment is matched case-insensitively (mirroring hydrofoil's
    /// `parse_volume_path` and mangrove's `UCReference`); all other segments are
    /// preserved verbatim. A path with fewer than the three name levels, or one
    /// whose first segment is not `Volumes`, is rejected as [`Error::InvalidUrl`].
    pub fn parse(path: &str) -> Result<Self> {
        let segments: Vec<&str> = path
            .trim_start_matches('/')
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        match segments.as_slice() {
            [kind, catalog, schema, volume, rest @ ..] if kind.eq_ignore_ascii_case("Volumes") => {
                Ok(VolumePath {
                    catalog: (*catalog).to_string(),
                    schema: (*schema).to_string(),
                    volume: (*volume).to_string(),
                    relative: rest.join("/"),
                })
            }
            _ => Err(Error::InvalidUrl(format!(
                "expected a Volumes path `/Volumes/<catalog>/<schema>/<volume>/…`, got {path:?}"
            ))),
        }
    }

    /// The dotted three-level volume name (`catalog.schema.volume`) — the key the
    /// `GetVolume` REST call takes as one path segment.
    pub fn full_name(&self) -> String {
        format!("{}.{}.{}", self.catalog, self.schema, self.volume)
    }

    /// True when the path addresses the volume root (no file/dir component).
    pub fn is_root(&self) -> bool {
        self.relative.is_empty()
    }

    /// Build the object key **under the volume root** for a file/dir op, joining
    /// the volume's own storage-location path prefix with this path's relative
    /// component. `root_key` is the store-relative path of the volume root
    /// (derived from the vended storage location; may be empty when the volume
    /// sits at the container/bucket root). The result never has a leading or
    /// trailing `/`.
    pub fn object_key(&self, root_key: &str) -> String {
        join_key(root_key, &self.relative)
    }

    /// The listing prefix under the volume root: the volume-root key with this
    /// path's relative component appended and, for a directory listing, a
    /// trailing `/` so a delimiter-scoped list returns the directory's immediate
    /// children. An empty relative at the container root yields `""` (no
    /// leading slash), so a caller must guard the empty-prefix case if the store
    /// requires one.
    pub fn list_prefix(&self, root_key: &str) -> String {
        let base = join_key(root_key, &self.relative);
        if base.is_empty() {
            String::new()
        } else {
            format!("{base}/")
        }
    }

    /// Re-attach `/Volumes/<catalog>/<schema>/<volume>/` to a store-relative key
    /// (a blob name / common prefix returned by the cloud list API, expressed
    /// relative to the container/bucket) to produce a canonical absolute Volumes
    /// path. `root_key` is the volume-root key that must be stripped from
    /// `store_key` first (the list API returns keys relative to the
    /// container/bucket, which includes the volume-root prefix).
    pub fn absolute(&self, root_key: &str, store_key: &str) -> String {
        let trimmed = strip_root(root_key, store_key);
        let rel = trimmed.trim_matches('/');
        if rel.is_empty() {
            format!("/Volumes/{}/{}/{}", self.catalog, self.schema, self.volume)
        } else {
            format!(
                "/Volumes/{}/{}/{}/{}",
                self.catalog, self.schema, self.volume, rel
            )
        }
    }

    /// The canonical absolute path for this exact `VolumePath` (round-trips
    /// [`parse`](Self::parse)). Used for a `stat` response path.
    pub fn to_canonical(&self) -> String {
        if self.relative.is_empty() {
            format!("/Volumes/{}/{}/{}", self.catalog, self.schema, self.volume)
        } else {
            format!(
                "/Volumes/{}/{}/{}/{}",
                self.catalog, self.schema, self.volume, self.relative
            )
        }
    }
}

/// Join two store-key fragments with a single `/`, trimming stray slashes and
/// dropping empty fragments (so an empty root or empty relative is harmless).
fn join_key(a: &str, b: &str) -> String {
    let a = a.trim_matches('/');
    let b = b.trim_matches('/');
    match (a.is_empty(), b.is_empty()) {
        (true, true) => String::new(),
        (false, true) => a.to_string(),
        (true, false) => b.to_string(),
        (false, false) => format!("{a}/{b}"),
    }
}

/// Strip the volume-root key prefix from a container/bucket-relative store key.
/// A key that doesn't carry the root prefix is returned trimmed (best-effort —
/// the result only needs to be volume-relative).
fn strip_root(root_key: &str, store_key: &str) -> String {
    let root = root_key.trim_matches('/');
    let key = store_key.trim_start_matches('/');
    if root.is_empty() {
        return key.to_string();
    }
    // Match the root prefix followed by a `/` (a child) or exactly the root.
    if let Some(rest) = key.strip_prefix(root) {
        rest.trim_start_matches('/').to_string()
    } else {
        key.to_string()
    }
}

// Native-only: unit tests never run on wasm32 (no test runner without
// wasm-bindgen-test).
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn parse_root() {
        let p = VolumePath::parse("/Volumes/main/default/vol").unwrap();
        assert_eq!(p.catalog, "main");
        assert_eq!(p.schema, "default");
        assert_eq!(p.volume, "vol");
        assert_eq!(p.relative, "");
        assert!(p.is_root());
        assert_eq!(p.full_name(), "main.default.vol");
    }

    #[test]
    fn parse_nested() {
        let p = VolumePath::parse("/Volumes/main/default/vol/a/b/c.txt").unwrap();
        assert_eq!(p.relative, "a/b/c.txt");
        assert!(!p.is_root());
        assert_eq!(p.to_canonical(), "/Volumes/main/default/vol/a/b/c.txt");
    }

    #[test]
    fn parse_no_leading_slash() {
        let p = VolumePath::parse("Volumes/c/s/v/f.txt").unwrap();
        assert_eq!(p.full_name(), "c.s.v");
        assert_eq!(p.relative, "f.txt");
    }

    #[test]
    fn parse_case_insensitive_kind() {
        assert!(VolumePath::parse("/volumes/c/s/v/f").is_ok());
        assert!(VolumePath::parse("/VOLUMES/c/s/v/f").is_ok());
    }

    #[test]
    fn parse_trailing_slash_is_root() {
        // A trailing slash on the volume root yields no relative component.
        let p = VolumePath::parse("/Volumes/c/s/v/").unwrap();
        assert!(p.is_root());
    }

    #[test]
    fn parse_rejects_short_or_wrong() {
        assert!(VolumePath::parse("/Volumes/c/s").is_err());
        assert!(VolumePath::parse("/Tables/c/s/t").is_err());
        assert!(VolumePath::parse("").is_err());
        assert!(VolumePath::parse("/").is_err());
    }

    #[test]
    fn object_key_joins_root_and_relative() {
        let p = VolumePath::parse("/Volumes/c/s/v/sub/f.txt").unwrap();
        // Volume root sits at `volumes/v-uuid` inside the container.
        assert_eq!(p.object_key("volumes/v-uuid"), "volumes/v-uuid/sub/f.txt");
        // Empty root (volume at container root).
        assert_eq!(p.object_key(""), "sub/f.txt");
        // Root with stray slashes normalizes.
        assert_eq!(p.object_key("/volumes/v-uuid/"), "volumes/v-uuid/sub/f.txt");
    }

    #[test]
    fn list_prefix_has_trailing_slash_for_directory() {
        let p = VolumePath::parse("/Volumes/c/s/v/dir").unwrap();
        assert_eq!(p.list_prefix("root/vroot"), "root/vroot/dir/");
        // Volume root listing: just the root key with a trailing slash.
        let root = VolumePath::parse("/Volumes/c/s/v").unwrap();
        assert_eq!(root.list_prefix("root/vroot"), "root/vroot/");
        // Volume root AND empty store root: no prefix at all.
        assert_eq!(root.list_prefix(""), "");
    }

    #[test]
    fn absolute_reattaches_volumes_prefix_stripping_root() {
        let p = VolumePath::parse("/Volumes/main/default/vol/sub").unwrap();
        // The list API returns container-relative keys that include the root.
        assert_eq!(
            p.absolute("volumes/v-uuid", "volumes/v-uuid/sub/file.txt"),
            "/Volumes/main/default/vol/sub/file.txt"
        );
        // A common-prefix (directory) key ends in `/`; trailing slash trimmed.
        assert_eq!(
            p.absolute("volumes/v-uuid", "volumes/v-uuid/sub/nested/"),
            "/Volumes/main/default/vol/sub/nested"
        );
        // Empty root: key is already volume-relative.
        assert_eq!(
            p.absolute("", "sub/file.txt"),
            "/Volumes/main/default/vol/sub/file.txt"
        );
        // A key exactly at the root maps back to the volume root path.
        assert_eq!(
            p.absolute("volumes/v-uuid", "volumes/v-uuid"),
            "/Volumes/main/default/vol"
        );
    }
}
