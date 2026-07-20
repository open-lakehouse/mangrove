//! Unity Catalog table resolution: interpret a `/delta/v1` `loadTable` response,
//! gate on the wasm engine's v1 support envelope, and discover the `_delta_log`
//! file set to prime.
//!
//! Plain HTTP has no listing, so the log manifest must be *derived*: the
//! `_last_checkpoint` hint names the checkpoint (classic naming only in v1) and
//! commit files follow the deterministic `{version:020}.json` scheme. When the
//! catalog reports the latest ratified version (managed tables) the commit range
//! is known outright; otherwise (external tables) versions are probed with HEAD
//! requests until the first miss.
//!
//! Everything here is transport-agnostic (`Arc<dyn ObjectStore>` + parsed JSON),
//! so it is tested natively. [`plan_table`] gates the canonical
//! `olai-uc-client` `delta_v1().load_table` response ([`DeltaLoadTableResponse`]);
//! [`discover_log`] then derives the `_delta_log` manifest through the resolved
//! object store.

use std::sync::Arc;

use futures::{StreamExt, TryStreamExt};
use object_store::path::Path;
use object_store::{ObjectMeta, ObjectStore, ObjectStoreExt as _};
use serde::Deserialize;
use unitycatalog_delta_api::models::DeltaLoadTableResponse;

use crate::error::{Error, Result};

/// How many HEAD probes run concurrently during log discovery.
const DISCOVER_CONCURRENCY: usize = 8;

/// Hard cap on the number of commit files a single preview may replay. A
/// preview reads 100 rows; a log this deep should long since have a checkpoint,
/// and probing further would hammer storage from the browser.
const MAX_COMMIT_FILES: u64 = 1024;

// =====================================================================
// loadTable gating
// =====================================================================

/// What log discovery needs to know from the catalog, after gating.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TablePlan {
    /// The table's storage location as reported by the catalog.
    pub location: String,
    /// The table's UUID (the `temporary-table-credentials` request key).
    pub table_uuid: String,
    /// Latest ratified version when the catalog tracks one (managed tables);
    /// pins the snapshot and bounds commit discovery.
    pub latest_version: Option<u64>,
}

/// Gate a `loadTable` response against the v1 support envelope and extract the
/// [`TablePlan`] for log discovery.
///
/// Unsupported shapes return [`Error::Unsupported`] naming the gate:
/// - **deletion vectors** enabled (`delta.enableDeletionVectors`) — the wasm
///   scan fails loud on DV tables, so gate before fetching anything;
/// - **unbackfilled CCv2 commits** — the wasm facade replays the filesystem
///   `_delta_log/` only; a non-empty staged-commit tail means the filesystem
///   view is behind the catalog's ratified state.
pub fn plan_table(loaded: &DeltaLoadTableResponse) -> Result<TablePlan> {
    let dv_enabled = loaded
        .metadata
        .properties
        .get("delta.enableDeletionVectors")
        .is_some_and(|v| v.eq_ignore_ascii_case("true"));
    if dv_enabled {
        return Err(Error::unsupported(
            "table has deletion vectors enabled (delta.enableDeletionVectors)".to_string(),
        ));
    }

    if let Some(commits) = &loaded.commits
        && !commits.is_empty()
    {
        return Err(Error::unsupported(format!(
            "catalog-managed table has {} unbackfilled commit(s); the in-browser \
             engine replays the backfilled _delta_log only",
            commits.len()
        )));
    }

    let latest_version = match loaded.latest_table_version {
        Some(v) if v < 0 => {
            return Err(Error::InvalidResponse(format!(
                "negative latest-table-version: {v}"
            )));
        }
        Some(v) => Some(v as u64),
        None => None,
    };

    Ok(TablePlan {
        location: loaded.metadata.location.clone(),
        table_uuid: loaded.metadata.table_uuid.clone(),
        latest_version,
    })
}

// =====================================================================
// Log discovery
// =====================================================================

/// Minimal projection of `_last_checkpoint` — classic checkpoints only.
#[derive(Debug, Clone, Deserialize)]
struct LastCheckpointHint {
    version: u64,
    /// Number of parts for a classic multi-part checkpoint; absent for a
    /// single-file checkpoint. UUID-named (v2) checkpoints carry no name hint
    /// here, which discovery detects as a missing classic file.
    #[serde(default)]
    parts: Option<u64>,
}

/// The discovered `_delta_log` tail: the manifest to prime plus the version the
/// snapshot should pin.
#[derive(Debug, Clone)]
pub struct DiscoveredLog {
    /// Log files to prime (absolute store paths, sizes from HEAD).
    pub manifest: Vec<ObjectMeta>,
    /// The newest commit version the manifest covers.
    pub version: u64,
}

/// Discover the `_delta_log` file set for the table rooted at `table_path`.
///
/// `latest_version` bounds the commit range when the catalog tracks it
/// (managed tables); otherwise versions are probed until the first missing
/// commit file. The returned manifest feeds `LogSource::Manifest` priming.
pub async fn discover_log(
    store: &Arc<dyn ObjectStore>,
    table_path: &Path,
    latest_version: Option<u64>,
) -> Result<DiscoveredLog> {
    let log_prefix = table_path.clone().join("_delta_log");
    let mut manifest: Vec<ObjectMeta> = Vec::new();

    // 1. `_last_checkpoint` names the checkpoint (when one exists).
    let last_checkpoint = log_prefix.clone().join("_last_checkpoint");
    let checkpoint = match store.get(&last_checkpoint).await {
        Ok(result) => {
            let bytes = result.bytes().await?;
            let hint: LastCheckpointHint = serde_json::from_slice(&bytes)
                .map_err(|e| Error::InvalidResponse(format!("unparsable _last_checkpoint: {e}")))?;
            Some(hint)
        }
        Err(object_store::Error::NotFound { .. }) => None,
        Err(err) => return Err(err.into()),
    };

    // 2. Checkpoint files under classic naming. A missing classic file means a
    //    UUID-named (v2) checkpoint, which discovery cannot derive — gate loud.
    if let Some(hint) = &checkpoint {
        let names: Vec<String> = match hint.parts {
            None => vec![format!("{:020}.checkpoint.parquet", hint.version)],
            Some(parts) => (1..=parts)
                .map(|i| {
                    format!(
                        "{:020}.checkpoint.{i:010}.{parts:010}.parquet",
                        hint.version
                    )
                })
                .collect(),
        };
        let heads: Vec<ObjectMeta> = futures::stream::iter(names)
            .map(|name| {
                let store = Arc::clone(store);
                let path = log_prefix.clone().join(name);
                async move { store.head(&path).await }
            })
            .buffered(DISCOVER_CONCURRENCY)
            .try_collect()
            .await
            .map_err(|err| match err {
                object_store::Error::NotFound { path, .. } => Error::unsupported(format!(
                    "checkpoint {path} is not classic-named (v2/UUID checkpoints are \
                     unsupported until Phase C)"
                )),
                other => Error::from(other),
            })?;
        manifest.extend(heads);
    }

    // 3. Commit files: deterministic range when the latest version is known,
    //    HEAD-probe forward otherwise.
    let start = checkpoint.as_ref().map(|c| c.version + 1).unwrap_or(0);
    let commit_path = |version: u64| log_prefix.clone().join(format!("{version:020}.json"));

    let newest = match latest_version {
        Some(latest) => {
            if latest.saturating_sub(start) >= MAX_COMMIT_FILES {
                return Err(Error::unsupported(format!(
                    "log tail spans {} commits (> {MAX_COMMIT_FILES}); table needs a \
                     checkpoint before in-browser replay is reasonable",
                    latest - start + 1
                )));
            }
            let heads: Vec<ObjectMeta> = futures::stream::iter(start..=latest)
                .map(|version| {
                    let store = Arc::clone(store);
                    let path = commit_path(version);
                    async move { store.head(&path).await }
                })
                .buffered(DISCOVER_CONCURRENCY)
                .try_collect()
                .await
                .map_err(|err| match err {
                    object_store::Error::NotFound { path, .. } => Error::unsupported(format!(
                        "commit {path} is not backfilled yet; the in-browser engine \
                         replays the backfilled _delta_log only"
                    )),
                    other => Error::from(other),
                })?;
            manifest.extend(heads);
            latest
        }
        None => {
            // Probe sequentially; commit versions are dense, so the first miss
            // is the end of the log.
            let mut version = start;
            loop {
                if version.saturating_sub(start) >= MAX_COMMIT_FILES {
                    return Err(Error::unsupported(format!(
                        "log tail exceeds {MAX_COMMIT_FILES} commits without a \
                         checkpoint; refusing to replay from the browser"
                    )));
                }
                match store.head(&commit_path(version)).await {
                    Ok(meta) => {
                        manifest.push(meta);
                        version += 1;
                    }
                    Err(object_store::Error::NotFound { .. }) => break,
                    Err(err) => return Err(err.into()),
                }
            }
            if version == start && checkpoint.is_none() {
                return Err(Error::InvalidResponse(format!(
                    "no _delta_log found under {log_prefix} (not a Delta table, or the \
                     storage credential does not reach it)"
                )));
            }
            version
                .saturating_sub(1)
                .max(checkpoint.as_ref().map(|c| c.version).unwrap_or(0))
        }
    };

    Ok(DiscoveredLog {
        manifest,
        version: newest,
    })
}

// Native-only: unit tests never run on wasm32 (no test runner without
// wasm-bindgen-test), and the async ones need tokio.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use object_store::ObjectStoreExt as _;
    use object_store::memory::InMemory;

    use super::*;

    fn loaded(json: serde_json::Value) -> DeltaLoadTableResponse {
        serde_json::from_value(json).unwrap()
    }

    #[test]
    fn plan_parses_kebab_case_and_extracts_plan() {
        let response = loaded(serde_json::json!({
            "metadata": {
                "etag": "e", "table-type": "MANAGED", "table-uuid": "u-1",
                "location": "abfss://c@a.dfs.core.windows.net/t",
                "created-time": 0, "updated-time": 0,
                "columns": {"type": "struct", "fields": []},
                "properties": {"delta.minReaderVersion": "1"}
            },
            "latest-table-version": 7
        }));
        let plan = plan_table(&response).unwrap();
        assert_eq!(
            plan,
            TablePlan {
                location: "abfss://c@a.dfs.core.windows.net/t".into(),
                table_uuid: "u-1".into(),
                latest_version: Some(7),
            }
        );
    }

    #[test]
    fn plan_gates_deletion_vectors_and_unbackfilled_commits() {
        let dv = loaded(serde_json::json!({
            "metadata": {
                "etag": "e", "table-type": "EXTERNAL", "table-uuid": "u",
                "location": "gs://b/t", "created-time": 0, "updated-time": 0,
                "columns": {"type": "struct", "fields": []},
                "properties": {"delta.enableDeletionVectors": "true"}
            }
        }));
        assert!(plan_table(&dv).unwrap_err().is_unsupported());

        let staged = loaded(serde_json::json!({
            "metadata": {
                "etag": "e", "table-type": "MANAGED", "table-uuid": "u",
                "location": "gs://b/t", "created-time": 0, "updated-time": 0,
                "columns": {"type": "struct", "fields": []},
                "properties": {}
            },
            "commits": [{"version": 9, "timestamp": 1, "file-name": "x.json",
                         "file-size": 10, "file-modification-timestamp": 1}],
            "latest-table-version": 9
        }));
        assert!(plan_table(&staged).unwrap_err().is_unsupported());
    }

    async fn put(store: &InMemory, path: &str, bytes: &[u8]) {
        store
            .put(&Path::from(path), bytes.to_vec().into())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn discovers_commit_only_log_by_probing() {
        let mem = InMemory::new();
        put(&mem, "tbl/_delta_log/00000000000000000000.json", b"{}").await;
        put(&mem, "tbl/_delta_log/00000000000000000001.json", b"{}").await;
        let store: Arc<dyn ObjectStore> = Arc::new(mem);

        let log = discover_log(&store, &Path::from("tbl"), None)
            .await
            .unwrap();
        assert_eq!(log.version, 1);
        let mut names: Vec<_> = log
            .manifest
            .iter()
            .map(|m| m.location.to_string())
            .collect();
        names.sort();
        assert_eq!(
            names,
            vec![
                "tbl/_delta_log/00000000000000000000.json",
                "tbl/_delta_log/00000000000000000001.json",
            ]
        );
    }

    #[tokio::test]
    async fn discovers_bounded_range_for_managed_tables() {
        let mem = InMemory::new();
        put(&mem, "tbl/_delta_log/00000000000000000000.json", b"{}").await;
        put(&mem, "tbl/_delta_log/00000000000000000001.json", b"{}").await;
        // A stray later file must NOT be picked up beyond the ratified version.
        put(&mem, "tbl/_delta_log/00000000000000000002.json", b"{}").await;
        let store: Arc<dyn ObjectStore> = Arc::new(mem);

        let log = discover_log(&store, &Path::from("tbl"), Some(1))
            .await
            .unwrap();
        assert_eq!(log.version, 1);
        assert_eq!(log.manifest.len(), 2);
    }

    #[tokio::test]
    async fn discovers_classic_checkpoint_and_tail() {
        let mem = InMemory::new();
        put(
            &mem,
            "tbl/_delta_log/_last_checkpoint",
            br#"{"version":2,"size":10}"#,
        )
        .await;
        put(
            &mem,
            "tbl/_delta_log/00000000000000000002.checkpoint.parquet",
            b"pq",
        )
        .await;
        put(&mem, "tbl/_delta_log/00000000000000000003.json", b"{}").await;
        let store: Arc<dyn ObjectStore> = Arc::new(mem);

        let log = discover_log(&store, &Path::from("tbl"), None)
            .await
            .unwrap();
        assert_eq!(log.version, 3);
        let names: Vec<_> = log
            .manifest
            .iter()
            .map(|m| m.location.to_string())
            .collect();
        assert!(
            names.contains(&"tbl/_delta_log/00000000000000000002.checkpoint.parquet".to_string())
        );
        assert!(names.contains(&"tbl/_delta_log/00000000000000000003.json".to_string()));
        // `_last_checkpoint` itself is always re-fetched by priming; discovery
        // must not duplicate it in the manifest.
        assert!(!names.iter().any(|n| n.ends_with("_last_checkpoint")));
    }

    #[tokio::test]
    async fn missing_classic_checkpoint_is_unsupported() {
        // `_last_checkpoint` exists but the classic-named file does not (a
        // UUID-named v2 checkpoint).
        let mem = InMemory::new();
        put(
            &mem,
            "tbl/_delta_log/_last_checkpoint",
            br#"{"version":6,"size":3}"#,
        )
        .await;
        let store: Arc<dyn ObjectStore> = Arc::new(mem);

        let err = discover_log(&store, &Path::from("tbl"), None)
            .await
            .unwrap_err();
        assert!(err.is_unsupported(), "{err}");
    }

    #[tokio::test]
    async fn missing_backfill_within_range_is_unsupported() {
        let mem = InMemory::new();
        put(&mem, "tbl/_delta_log/00000000000000000000.json", b"{}").await;
        // Version 1 ratified by the catalog but not present in _delta_log.
        let store: Arc<dyn ObjectStore> = Arc::new(mem);

        let err = discover_log(&store, &Path::from("tbl"), Some(1))
            .await
            .unwrap_err();
        assert!(err.is_unsupported(), "{err}");
    }

    #[tokio::test]
    async fn empty_log_is_a_real_error_not_unsupported() {
        let mem = InMemory::new();
        let store: Arc<dyn ObjectStore> = Arc::new(mem);
        let err = discover_log(&store, &Path::from("tbl"), None)
            .await
            .unwrap_err();
        assert!(!err.is_unsupported(), "{err}");
    }

    // Regression pin for the #152 double-prefix bug (fixed by taking
    // `UCStore::root()`, not `as_dyn()`, in the wasm resolver).
    //
    // `discover_log` joins the FULL table path onto `_delta_log/...`, so it must
    // be handed a bucket-rooted store. The wasm resolver used to hand it the
    // prefix-scoped `as_dyn()` store instead, which re-applies the table prefix
    // and doubles every key (e.g. `<table>/<table>/_delta_log/00...json`), 404s,
    // and gets misreported as "not backfilled". This test encodes the contract:
    // full path + bucket-rooted store succeeds; full path + prefix-scoped store
    // fails on the doubled key.
    #[tokio::test]
    async fn discover_log_requires_bucket_rooted_store_not_prefix_scoped() {
        use object_store::prefix::PrefixStore;

        const TABLE_PREFIX: &str = "__unitystorage/catalogs/cid/tables/tid";
        let table_path = Path::from(TABLE_PREFIX);

        // The commit lives at its real, single-prefix key.
        let mem = InMemory::new();
        put(
            &mem,
            &format!("{TABLE_PREFIX}/_delta_log/00000000000000000000.json"),
            b"{}",
        )
        .await;
        let mem: Arc<InMemory> = Arc::new(mem);

        // Bucket-rooted store (what `UCStore::root()` yields): full-path join is
        // correct — discovery finds the commit.
        let bucket_rooted: Arc<dyn ObjectStore> = mem.clone();
        let log = discover_log(&bucket_rooted, &table_path, Some(0))
            .await
            .expect("bucket-rooted store + full path must find the backfilled commit");
        assert_eq!(log.version, 0);
        assert_eq!(log.manifest.len(), 1);

        // Prefix-scoped store (what `UCStore::as_dyn()` yields, the old bug):
        // the same full-path join doubles the prefix, so the HEAD 404s and
        // surfaces as the misleading "not backfilled" unsupported error.
        let prefix_scoped: Arc<dyn ObjectStore> =
            Arc::new(PrefixStore::new(mem, Path::from(TABLE_PREFIX)));
        let err = discover_log(&prefix_scoped, &table_path, Some(0))
            .await
            .expect_err("prefix-scoped store + full path must double the key and miss");
        assert!(err.is_unsupported(), "{err}");
    }
}
