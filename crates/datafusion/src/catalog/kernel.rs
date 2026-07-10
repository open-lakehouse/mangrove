//! Catalog-managed snapshot construction.
//!
//! Bridges the `/delta/v1` loadTable response (catalog-ratified commit tail +
//! latest version) to a deltalake [`Snapshot`] for a catalog-managed
//! (coordinated-commit) Delta table.
//!
//! The catalog — not the filesystem — is the source of truth for a managed
//! table's latest version: the newest commits are ratified but may not yet be
//! backfilled into `_delta_log/` (they live as staged commits under
//! `_delta_log/_staged_commits/`). We build a `delta_kernel` snapshot from the
//! supplied commit tail + catalog version and wrap it with [`Snapshot::new`],
//! mirroring the reference `delta-kernel-unity-catalog` read path.

use datafusion::common::DataFusionError;
use delta_kernel::snapshot::Snapshot as KernelSnapshot;
use delta_kernel::{Engine, LogPath, Version};
use deltalake_core::DeltaTableConfig;
use deltalake_core::kernel::Snapshot;
use unitycatalog_delta_api::models::{DeltaCommit, DeltaLoadTableResponse, DeltaTableType};
use url::Url;

use super::builder::TableProviderError;

/// How a `/delta/v1` loadTable response says a Delta table should be read.
///
/// Resolving this once — rather than re-deriving the latest version from
/// `metadata.last-commit-version` at each call site — is the fix for the stale-read
/// bug (review finding A5): `last-commit-version` only tracks *metadata-changing*
/// commits, so substituting it caps the snapshot below the latest ratified data
/// commit. No reference client ever makes that substitution.
#[derive(Debug)]
pub enum ManagedReadState {
    /// Catalog-managed (coordinated-commit): build the kernel snapshot from this
    /// ratified commit tail and latest version. `commits` is the unbackfilled CCv2
    /// tail sorted ascending; `latest` is the highest ratified version (may be `0`
    /// immediately after create, with an empty tail — the snapshot is then just the
    /// filesystem `0.json`).
    Managed {
        commits: Vec<DeltaCommit>,
        latest: Version,
    },
    /// Not catalog-managed — an `EXTERNAL` table, or the legacy `latest-table-version
    /// == -1` "Unity Catalog does not manage this table" signal. The caller routes
    /// to the plain filesystem snapshot (`_delta_log/` is authoritative).
    NotManaged,
}

/// Resolve how to read a Delta table from its loadTable response.
///
/// Resolution matrix (validated against the UC OSS Java server and the Delta
/// reference clients, 2026-06-13):
///
/// - `table-type == EXTERNAL` → [`ManagedReadState::NotManaged`] (the field set and
///   `commits` are omitted for external tables).
/// - `table-type == MANAGED`:
///   - `latest-table-version` **absent** → hard error. The server always sets it for
///     managed tables, so absence is a protocol violation; we never substitute
///     `metadata.last-commit-version`. The single tolerated fallback: when a non-empty
///     commit tail is present, use the tail's max version (so a server that ships the
///     tail but drops the scalar still reads correctly).
///   - `latest-table-version < 0` (the legacy `-1` signal) → [`ManagedReadState::NotManaged`].
///   - `latest-table-version >= 0` → [`ManagedReadState::Managed`] with the tail sorted
///     ascending.
///
/// Tail-completeness invariants the reference enforces are checked here so a partial
/// tail errors out rather than silently serving an older version: every commit's
/// version must be `<= latest`, and a non-empty tail's max version must equal `latest`
/// (an empty tail is valid only if the filesystem log reaches `latest`, which the
/// kernel enforces at snapshot build).
pub fn resolve_managed_read_state(
    loaded: &DeltaLoadTableResponse,
) -> Result<ManagedReadState, TableProviderError> {
    if loaded.metadata.table_type == DeltaTableType::External {
        return Ok(ManagedReadState::NotManaged);
    }

    // MANAGED from here on.
    let mut commits: Vec<DeltaCommit> = loaded.commits.clone().unwrap_or_default();
    // The tail arrives newest-first (descending) from loadTable/getCommits; sort
    // ascending defensively before any version reasoning or assembly.
    commits.sort_by_key(|c| c.version);

    let latest: i64 = match loaded.latest_table_version {
        Some(v) if v < 0 => return Ok(ManagedReadState::NotManaged),
        Some(v) => v,
        None => {
            // Tolerated fallback: a present tail's max version. Absent both → hard error.
            commits.last().map(|c| c.version).ok_or_else(|| {
                DataFusionError::Plan(format!(
                    "managed table '{}' loadTable response omitted latest-table-version \
                     and carried no commit tail; cannot resolve a latest version \
                     (refusing to substitute metadata.last-commit-version)",
                    loaded.metadata.location
                ))
            })?
        }
    };

    let latest: Version = latest.try_into().map_err(|_| {
        DataFusionError::Plan(format!(
            "negative resolved latest version {latest} for '{}'",
            loaded.metadata.location
        ))
    })?;

    // Tail-completeness: no commit may exceed the resolved latest, and a non-empty
    // tail must reach exactly `latest` (otherwise we'd silently serve an older snapshot).
    if let Some(max) = commits.last().map(|c| c.version) {
        let max: Version = max.try_into().map_err(|_| {
            DataFusionError::Plan(format!(
                "negative commit version in tail for '{}'",
                loaded.metadata.location
            ))
        })?;
        if max > latest {
            return Err(DataFusionError::Plan(format!(
                "managed table '{}' commit tail max version {max} exceeds latest ratified \
                 version {latest}",
                loaded.metadata.location
            )));
        }
        if max != latest {
            return Err(DataFusionError::Plan(format!(
                "managed table '{}' commit tail ends at version {max} but latest ratified \
                 version is {latest}; tail does not cover the latest version",
                loaded.metadata.location
            )));
        }
    }

    Ok(ManagedReadState::Managed { commits, latest })
}

/// Build a catalog-managed [`Snapshot`] from a table location, the catalog-ratified
/// commit tail, and the latest ratified version.
///
/// `commits` is the unbackfilled CCv2 tail from the loadTable response; it is
/// sorted ascending here (the kernel requires a contiguous, ascending log tail).
/// `latest_table_version` is the highest version the catalog has ratified, used as
/// the catalog-version cap and — when `at_version` is `None` — as the effective
/// target version. By the time this is reached via [`resolve_managed_read_state`]
/// the version is guaranteed non-negative; the negative guard below is defensive.
///
/// When `at_version` is `Some(v)`, `v` must not exceed `latest_table_version` — a
/// reader must never read beyond the max ratified version (ManagedTablesSpec).
pub fn build_catalog_managed_snapshot(
    engine: &dyn Engine,
    location: &Url,
    commits: &[DeltaCommit],
    latest_table_version: i64,
    at_version: Option<Version>,
) -> Result<Snapshot, TableProviderError> {
    let latest_table_version: Version = latest_table_version.try_into().map_err(|_| {
        DataFusionError::Plan(format!(
            "negative latest-table-version {latest_table_version} for '{location}'"
        ))
    })?;

    if let Some(version) = at_version
        && version > latest_table_version
    {
        return Err(DataFusionError::Plan(format!(
            "requested version {version} for '{location}' exceeds latest ratified \
                 version {latest_table_version}"
        )));
    }

    // The kernel joins log paths onto the table root via `Url::join`, which drops the
    // final segment unless the root ends in a slash; guarantee the trailing slash.
    let mut table_root = location.clone();
    if !table_root.path().ends_with('/') {
        table_root.set_path(&format!("{}/", table_root.path()));
    }

    let log_tail = to_log_tail(&table_root, commits)?;

    let mut builder = KernelSnapshot::builder_for(table_root)
        .with_max_catalog_version(latest_table_version)
        .with_log_tail(log_tail);
    if let Some(version) = at_version {
        builder = builder.at_version(version);
    }

    let inner = builder
        .build(engine)
        .map_err(|e| DataFusionError::External(Box::new(e)))?;

    Ok(Snapshot::new(inner, DeltaTableConfig::default()))
}

/// The staged-commit file name to hand the kernel, tolerating both forms the spec
/// permits: the canonical `<20-digit-version>.<uuid>.json` (ManagedTablesSpec §
/// Terminology) and the bare `<uuid>.json` shown in the getCommits example. The
/// buoyant kernel's log-path parser only accepts the canonical form, so when the
/// 20-digit version prefix is absent we synthesize it from `commit.version`.
///
/// This is deliberately more tolerant than the kernel reference (whose `FileNames`
/// regex rejects the spec's bare-uuid example): the server stores and echoes
/// `file_name` verbatim, so identity/ordering is keyed on `commit.version`.
fn staged_file_name(commit: &DeltaCommit) -> String {
    if has_version_prefix(&commit.file_name) {
        commit.file_name.clone()
    } else {
        format!("{:020}.{}", commit.version, commit.file_name)
    }
}

/// Whether `file_name` already begins with the canonical 20-digit version prefix
/// followed by `.` (e.g. `00000000000000000010.<uuid>.json`).
fn has_version_prefix(file_name: &str) -> bool {
    match file_name.split_once('.') {
        Some((prefix, _)) => prefix.len() == 20 && prefix.bytes().all(|b| b.is_ascii_digit()),
        None => false,
    }
}

/// Map the loadTable commit tail to kernel staged-commit [`LogPath`]s, sorted
/// ascending by version (the kernel rejects a non-contiguous / unsorted tail).
/// Rejects a negative `file-size`, which can't be a valid byte count. The single
/// canonical implementation — the managed write path consumes this too.
pub fn to_log_tail(
    table_root: &Url,
    commits: &[DeltaCommit],
) -> Result<Vec<LogPath>, TableProviderError> {
    let mut sorted: Vec<&DeltaCommit> = commits.iter().collect();
    sorted.sort_by_key(|c| c.version);

    sorted
        .into_iter()
        .map(|c| {
            let size: u64 = c.file_size.try_into().map_err(|_| {
                DataFusionError::Plan(format!(
                    "negative file-size {} for commit {} of '{table_root}'",
                    c.file_size, c.version
                ))
            })?;
            LogPath::staged_commit(
                table_root.clone(),
                &staged_file_name(c),
                c.file_modification_timestamp,
                size,
            )
            .map_err(|e| DataFusionError::External(Box::new(e)))
        })
        .collect()
}

/// Ensure a table location string ends with a trailing slash so it joins cleanly as
/// a prefix (the kernel's `Url::join` drops the final segment otherwise). The single
/// canonical implementation, shared by the managed create/append paths and the
/// open-lakehouse lineage writer.
pub fn ensure_trailing_slash(s: &str) -> String {
    if s.ends_with('/') {
        s.to_string()
    } else {
        format!("{s}/")
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use unitycatalog_delta_api::models::{DeltaStructType, DeltaTableMetadata, StructTypeTag};

    use super::*;

    fn commit(version: i64, file_name: &str, file_size: i64) -> DeltaCommit {
        DeltaCommit {
            version,
            timestamp: 0,
            file_name: file_name.to_string(),
            file_size,
            file_modification_timestamp: 0,
        }
    }

    /// A valid staged-commit file name: `<20-digit version>.<uuid>.json`, matching
    /// what the catalog returns and what the kernel's log-path parser accepts.
    fn staged_name(version: u64) -> String {
        format!("{version:020}.3a0d65cd-4056-49b8-937b-95f9e3ee90e5.json")
    }

    /// The bare `<uuid>.json` form the getCommits spec example uses.
    fn bare_name() -> String {
        "3a0d65cd-4056-49b8-937b-95f9e3ee90e5.json".to_string()
    }

    fn loaded(
        table_type: DeltaTableType,
        latest: Option<i64>,
        commits: Option<Vec<DeltaCommit>>,
    ) -> DeltaLoadTableResponse {
        DeltaLoadTableResponse {
            metadata: DeltaTableMetadata {
                etag: "e".into(),
                table_type,
                table_uuid: "u".into(),
                location: "s3://bucket/table".into(),
                created_time: 0,
                updated_time: 0,
                columns: DeltaStructType {
                    type_tag: StructTypeTag::Struct,
                    fields: vec![],
                },
                partition_columns: None,
                properties: BTreeMap::new(),
                last_commit_version: None,
                last_commit_timestamp_ms: None,
            },
            commits,
            uniform: None,
            latest_table_version: latest,
        }
    }

    #[test]
    fn maps_commit_tail_to_staged_log_paths() {
        let root = Url::parse("s3://bucket/table/").unwrap();
        // Deliberately out of order; sorting must yield a contiguous ascending tail
        // the kernel will accept. `LogPath::staged_commit` succeeding for each entry
        // also asserts our staged-commit file-name handling is valid.
        let commits = vec![
            commit(2, &staged_name(2), 20),
            commit(1, &staged_name(1), 10),
            commit(3, &staged_name(3), 30),
        ];

        let tail = to_log_tail(&root, &commits).unwrap();
        assert_eq!(tail.len(), 3);
    }

    #[test]
    fn rejects_negative_file_size() {
        let root = Url::parse("s3://bucket/table/").unwrap();
        let commits = vec![commit(1, &staged_name(1), -1)];

        let err = to_log_tail(&root, &commits).unwrap_err();
        assert!(
            err.to_string().contains("negative file-size"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn tolerates_both_staged_filename_forms() {
        let root = Url::parse("s3://bucket/table/").unwrap();
        // Canonical 20-digit-prefix form and the bare-uuid form must both produce a
        // valid staged-commit log path (the bare form gets a synthesized prefix).
        let commits = vec![commit(1, &staged_name(1), 10), commit(2, &bare_name(), 20)];
        let tail = to_log_tail(&root, &commits).unwrap();
        assert_eq!(tail.len(), 2);
    }

    #[test]
    fn staged_file_name_synthesizes_prefix_only_when_absent() {
        assert_eq!(
            staged_file_name(&commit(10, &staged_name(10), 1)),
            staged_name(10),
            "an already-prefixed name is passed through verbatim"
        );
        assert_eq!(
            staged_file_name(&commit(7, &bare_name(), 1)),
            format!("00000000000000000007.{}", bare_name()),
            "a bare-uuid name gets the 20-digit version prefix synthesized"
        );
    }

    #[test]
    fn resolve_external_is_not_managed() {
        let r =
            resolve_managed_read_state(&loaded(DeltaTableType::External, Some(5), None)).unwrap();
        assert!(matches!(r, ManagedReadState::NotManaged));
    }

    #[test]
    fn resolve_negative_latest_is_not_managed() {
        // The legacy `-1` "UC does not manage this table" signal routes to filesystem.
        let r =
            resolve_managed_read_state(&loaded(DeltaTableType::Managed, Some(-1), None)).unwrap();
        assert!(matches!(r, ManagedReadState::NotManaged));
    }

    #[test]
    fn resolve_post_create_latest_zero_empty_tail() {
        // Right after create: latest = 0, empty tail → managed, snapshot is fs 0.json.
        let r = resolve_managed_read_state(&loaded(DeltaTableType::Managed, Some(0), Some(vec![])))
            .unwrap();
        match r {
            ManagedReadState::Managed { commits, latest } => {
                assert_eq!(latest, 0);
                assert!(commits.is_empty());
            }
            other => panic!("expected Managed, got {other:?}"),
        }
    }

    #[test]
    fn resolve_present_latest_sorts_tail() {
        let r = resolve_managed_read_state(&loaded(
            DeltaTableType::Managed,
            Some(3),
            Some(vec![
                commit(3, &staged_name(3), 30),
                commit(2, &staged_name(2), 20),
            ]),
        ))
        .unwrap();
        match r {
            ManagedReadState::Managed { commits, latest } => {
                assert_eq!(latest, 3);
                assert_eq!(
                    commits.iter().map(|c| c.version).collect::<Vec<_>>(),
                    [2, 3]
                );
            }
            other => panic!("expected Managed, got {other:?}"),
        }
    }

    #[test]
    fn resolve_absent_latest_with_tail_uses_tail_max() {
        let r = resolve_managed_read_state(&loaded(
            DeltaTableType::Managed,
            None,
            Some(vec![
                commit(2, &staged_name(2), 20),
                commit(4, &staged_name(4), 40),
            ]),
        ))
        .unwrap();
        match r {
            ManagedReadState::Managed { latest, .. } => assert_eq!(latest, 4),
            other => panic!("expected Managed, got {other:?}"),
        }
    }

    #[test]
    fn resolve_absent_latest_without_tail_is_error() {
        let err =
            resolve_managed_read_state(&loaded(DeltaTableType::Managed, None, None)).unwrap_err();
        assert!(
            err.to_string().contains("omitted latest-table-version"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn resolve_tail_not_reaching_latest_is_error() {
        // A tail that ends below `latest` would silently serve an older version.
        let err = resolve_managed_read_state(&loaded(
            DeltaTableType::Managed,
            Some(5),
            Some(vec![commit(3, &staged_name(3), 30)]),
        ))
        .unwrap_err();
        assert!(
            err.to_string()
                .contains("does not cover the latest version"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn resolve_tail_exceeding_latest_is_error() {
        let err = resolve_managed_read_state(&loaded(
            DeltaTableType::Managed,
            Some(2),
            Some(vec![commit(3, &staged_name(3), 30)]),
        ))
        .unwrap_err();
        assert!(
            err.to_string().contains("exceeds latest ratified"),
            "unexpected error: {err}"
        );
    }
}
