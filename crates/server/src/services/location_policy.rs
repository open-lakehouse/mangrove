//! Server-side allowlist for local (`file://`) storage locations.
//!
//! Once the server can serve data from the host filesystem (see
//! `services::object_store`), an unrestricted caller could register a storage
//! location pointing at *any* path the server process can reach — `file:///etc`,
//! `file:///`, a co-tenant's data — by accident or design. [`LocalStoragePolicy`]
//! restricts which host paths may back a `file://` location.
//!
//! The policy is **deny-by-default**: an empty allowlist rejects every `file://`
//! location. A deployment opts in by configuring one or more allowed roots; only
//! paths at or beneath an allowed root are accepted. Cloud schemes (s3, abfss,
//! gs, …) are never inspected here — they are governed by external locations and
//! credential vending.

use std::path::{Component, Path, PathBuf};

use super::location::StorageLocationUrl;
use crate::{Error, Result};

/// An allowlist of host filesystem roots that may back `file://` storage
/// locations.
///
/// Construct via [`LocalStoragePolicy::new`]; an empty policy denies all local
/// storage. Held on the server handler and reached by request handlers through
/// [`ProvidesLocalStoragePolicy`](super::ProvidesLocalStoragePolicy).
#[derive(Debug, Clone, Default)]
pub struct LocalStoragePolicy {
    /// Canonicalized allowed roots. Empty ⇒ deny all `file://`.
    allowed_roots: Vec<PathBuf>,
}

impl LocalStoragePolicy {
    /// Build a policy from configured root paths.
    ///
    /// Each root is canonicalized (resolving symlinks and `..`) so later prefix
    /// checks compare against a stable, real path. A root that does not exist or
    /// cannot be canonicalized is a configuration error — surfacing it at
    /// startup is preferable to silently denying every local location later.
    pub fn new<I, P>(roots: I) -> Result<Self>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        let allowed_roots = roots
            .into_iter()
            .map(|root| {
                let root = root.as_ref();
                root.canonicalize().map_err(|e| {
                    Error::invalid_argument(format!(
                        "allowed local storage root '{}' is not accessible: {e}",
                        root.display()
                    ))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self { allowed_roots })
    }

    /// A deny-all policy (no allowed roots). The default for a server with no
    /// `local_storage` configuration.
    pub fn deny_all() -> Self {
        Self::default()
    }

    /// Whether any local root is allowed. Used by managed-root resolution to
    /// decide whether a sole allowed root can serve as an implicit root.
    pub(crate) fn allowed_roots(&self) -> &[PathBuf] {
        &self.allowed_roots
    }

    /// Authorize a storage location.
    ///
    /// Cloud schemes pass through untouched. A `file://` location is accepted
    /// only when its host path — with `..` rejected and `.` collapsed — sits at
    /// or beneath an allowed root. With no allowed roots, every `file://`
    /// location is rejected.
    pub fn check(&self, location: &StorageLocationUrl) -> Result<()> {
        if !location.is_local() {
            return Ok(());
        }

        let path = location.raw().to_file_path().map_err(|_| {
            Error::invalid_argument(format!("not a valid local file path: {}", location.raw()))
        })?;

        // Reject any traversal component outright rather than trying to resolve
        // it: a `..` in a governed storage path is never legitimate and is the
        // classic way to escape an allowed root.
        if path.components().any(|c| matches!(c, Component::ParentDir)) {
            return Err(Error::invalid_argument(format!(
                "local storage path '{}' must not contain '..'",
                path.display()
            )));
        }
        let normalized = lexically_normalize(&path);

        if self.allowed_roots.is_empty() {
            return Err(Error::invalid_argument(format!(
                "local (file://) storage is not enabled on this server; \
                 path '{}' is not within any allowed root",
                normalized.display()
            )));
        }

        if self
            .allowed_roots
            .iter()
            .any(|root| is_within(&normalized, root))
        {
            Ok(())
        } else {
            Err(Error::invalid_argument(format!(
                "local storage path '{}' is not within any allowed root",
                normalized.display()
            )))
        }
    }
}

/// Collapse `.` components from an absolute path without touching the
/// filesystem. `..` is handled by the caller (rejected), so it is not resolved
/// here. Used because governed paths frequently do not exist yet (managed roots,
/// not-yet-created tables) and so cannot be canonicalized.
fn lexically_normalize(path: &Path) -> PathBuf {
    path.components()
        .filter(|c| !matches!(c, Component::CurDir))
        .collect()
}

/// Whether `path` is equal to, or a descendant of, `root`, comparing on whole
/// path components (so `/data` does not match `/data-secret`).
fn is_within(path: &Path, root: &Path) -> bool {
    let mut root_parts = root.components();
    let mut path_parts = path.components();
    loop {
        match (root_parts.next(), path_parts.next()) {
            // Consumed the whole root: `path` is at or below it.
            (None, _) => return true,
            // Root has more components than the path, or a component differs.
            (Some(_), None) => return false,
            (Some(a), Some(b)) if a != b => return false,
            (Some(_), Some(_)) => continue,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loc(url: &str) -> StorageLocationUrl {
        StorageLocationUrl::parse(url).unwrap()
    }

    #[test]
    fn deny_all_rejects_every_file_url() {
        let policy = LocalStoragePolicy::deny_all();
        assert!(policy.check(&loc("file:///tmp/anything")).is_err());
        assert!(policy.check(&loc("file:///")).is_err());
    }

    #[test]
    fn cloud_schemes_pass_through_even_with_empty_policy() {
        let policy = LocalStoragePolicy::deny_all();
        assert!(policy.check(&loc("s3://bucket/data")).is_ok());
        assert!(
            policy
                .check(&loc("abfss://c@acct.dfs.core.windows.net/x"))
                .is_ok()
        );
    }

    // The following tests construct real `file://` paths under a canonicalized
    // root and exercise prefix matching. They are POSIX-only: on Windows,
    // `canonicalize()` yields a `\\?\C:\…` extended-length prefix that does not
    // match the plain `C:\…` from a URL's `to_file_path()`, and local file://
    // storage is gated off on Windows anyway (see `local_store` /
    // `get_local_store`). Deny-by-default and cloud pass-through (tested above)
    // are the behaviors that still matter on Windows.
    #[cfg(not(windows))]
    #[test]
    fn allows_paths_within_a_configured_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let policy = LocalStoragePolicy::new([&root]).unwrap();

        let inside = url::Url::from_directory_path(root.join("catalog/table"))
            .unwrap()
            .to_string();
        assert!(policy.check(&loc(&inside)).is_ok());
        // The root itself is allowed.
        let at_root = url::Url::from_directory_path(&root).unwrap().to_string();
        assert!(policy.check(&loc(&at_root)).is_ok());
    }

    #[cfg(not(windows))]
    #[test]
    fn rejects_paths_outside_every_root() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let policy = LocalStoragePolicy::new([&root]).unwrap();
        assert!(policy.check(&loc("file:///etc/passwd")).is_err());
    }

    #[cfg(not(windows))]
    #[test]
    fn rejects_sibling_prefix() {
        // `<root>` must not match `<root>-secret`. Build the sibling as a native
        // path (its file name + "-secret") so the URL is well-formed on every OS.
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let policy = LocalStoragePolicy::new([&root]).unwrap();

        let mut sibling = root.clone();
        let name = format!("{}-secret", root.file_name().unwrap().to_string_lossy());
        sibling.set_file_name(name);
        sibling.push("data");
        let url = url::Url::from_directory_path(&sibling).unwrap().to_string();
        assert!(policy.check(&loc(&url)).is_err());
    }

    #[cfg(not(windows))]
    #[test]
    fn rejects_parent_dir_traversal() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        let policy = LocalStoragePolicy::new([&root]).unwrap();

        // A crafted `..` under the root: built as a native path so the file URL
        // is well-formed (drive letters/separators) on every OS. The `..`
        // component is rejected outright.
        let escape = url::Url::from_directory_path(root.join("..").join("escape"))
            .unwrap()
            .to_string();
        assert!(policy.check(&loc(&escape)).is_err());
    }

    #[test]
    fn new_errors_on_missing_root() {
        let result = LocalStoragePolicy::new(["/this/path/does/not/exist/uc-xyz"]);
        assert!(result.is_err());
    }

    #[test]
    fn is_within_matches_on_component_boundaries() {
        assert!(is_within(Path::new("/data/t1"), Path::new("/data")));
        assert!(is_within(Path::new("/data"), Path::new("/data")));
        assert!(!is_within(Path::new("/data-secret/t1"), Path::new("/data")));
        assert!(!is_within(Path::new("/other/t1"), Path::new("/data")));
    }
}
