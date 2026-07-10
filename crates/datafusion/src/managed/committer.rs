//! A Unity Catalog catalog-managed [`Committer`] backed by our [`DeltaV1Client`].
//!
//! Ported from the buoyant kernel fork's experimental `delta-kernel-unity-catalog`
//! `UCCommitter` (`delta-kernel-unity-catalog/src/committer.rs`), but bound directly to
//! the unitycatalog-rs `DeltaV1Client` (`/delta/v1` `updateTable` `add-commit`) instead of
//! a separate `CommitClient` trait — so unitycatalog-rs stays the single UC client. Keep
//! this close to the upstream source so re-syncing (or depending on the crate once it
//! stabilizes) stays mechanical.
//!
//! For version 0 (table creation) the committer writes `00…0.json` directly to the
//! published commit path; the connector finalizes the table in UC via `createTable`
//! afterward (see [`crate::managed::create_managed_table`]). For version >= 1 it writes a
//! staged commit and calls the UC `add-commit` API to ratify it.
//!
//! NOTE: [`Committer::commit`] is synchronous but `DeltaV1Client` is async, so we bridge
//! via `tokio::task::block_in_place` + `Handle::block_on` — this committer therefore
//! requires a multi-threaded tokio runtime (the default engine's runtime is fine).

use std::sync::{Arc, Mutex};

use delta_kernel::committer::{
    CommitMetadata, CommitResponse, CommitType, Committer, PublishMetadata,
};
use delta_kernel::{
    DeltaResult, DeltaResultIterator, Engine, Error as DeltaError, FilteredEngineData,
};
use tracing::{debug, info};
use unitycatalog_client::DeltaV1Client;
use unitycatalog_delta_api::models::{
    DeltaCommit, DeltaTableRequirement, DeltaTableUpdate, DeltaUpdateTableRequest,
};

// UC catalog-managed contract identifiers (mirror the fork's `constants`).
const CATALOG_MANAGED_FEATURE: &str = "catalogManaged";
const VACUUM_PROTOCOL_CHECK_FEATURE: &str = "vacuumProtocolCheck";
const IN_COMMIT_TIMESTAMP_FEATURE: &str = "inCommitTimestamp";
const ENABLE_IN_COMMIT_TIMESTAMPS: &str = "delta.enableInCommitTimestamps";
const UC_TABLE_ID_KEY: &str = "io.unitycatalog.tableId";
const CLUSTERING_DOMAIN_NAME: &str = "delta.clustering";

macro_rules! require {
    ($cond:expr, $msg:expr) => {
        if !($cond) {
            return Err(DeltaError::generic($msg));
        }
    };
}

/// A Unity Catalog catalog-managed [`Committer`] for one table, addressed by its
/// three-level name (the key our `/delta/v1` `updateTable` endpoint uses) plus its UC
/// `table_id` (validated against the commit's `io.unitycatalog.tableId`).
#[derive(Clone)]
pub struct UnityCatalogCommitter {
    client: Arc<DeltaV1Client>,
    catalog: String,
    schema: String,
    table: String,
    table_id: String,
    /// The last non-conflict UC error from `update_table`, captured so the caller's
    /// retry loop can recover the *typed* `unitycatalog_client::Error` (429
    /// resource-exhausted, 500 commit-state-unknown, …). The `Committer::commit`
    /// signature only lets us return a kernel `DeltaError`, which erases the UC error
    /// type; this side channel preserves it. Set on every failed commit attempt and
    /// inspected by [`append_to_managed_table`](super::append::append_to_managed_table).
    last_error: Arc<Mutex<Option<unitycatalog_client::Error>>>,
    /// The `(version, staged_file_name)` of the most recent staged commit this committer
    /// proposed to UC. After a `CommitStateUnknownException` (500) the retry loop reloads
    /// the table and checks whether this exact file is present in the ratified tail: if so
    /// the commit actually succeeded (return success, never re-commit); if absent, retry.
    /// Matching on the full UUID file name — not just the version — avoids mistaking a
    /// concurrent writer's commit at the same version for ours.
    last_proposed: Arc<Mutex<Option<(i64, String)>>>,
}

impl std::fmt::Debug for UnityCatalogCommitter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnityCatalogCommitter")
            .field(
                "table",
                &format_args!("{}.{}.{}", self.catalog, self.schema, self.table),
            )
            .field("table_id", &self.table_id)
            .finish()
    }
}

impl UnityCatalogCommitter {
    /// Create a committer for the table `catalog.schema.table` with UC id `table_id`.
    pub fn new(
        client: Arc<DeltaV1Client>,
        catalog: impl Into<String>,
        schema: impl Into<String>,
        table: impl Into<String>,
        table_id: impl Into<String>,
    ) -> Self {
        Self {
            client,
            catalog: catalog.into(),
            schema: schema.into(),
            table: table.into(),
            table_id: table_id.into(),
            last_error: Arc::new(Mutex::new(None)),
            last_proposed: Arc::new(Mutex::new(None)),
        }
    }

    /// The `(version, staged_file_name)` of the last staged commit proposed to UC, if any.
    /// Used by the retry loop's commit-state-unknown recovery to detect a commit that
    /// actually landed despite an ambiguous (500) response.
    pub(crate) fn last_proposed(&self) -> Option<(i64, String)> {
        self.last_proposed
            .lock()
            .expect("committer mutex poisoned")
            .clone()
    }

    /// Take the last non-conflict UC error recorded by a failed `update_table` (clearing
    /// it). The retry loop calls this after `Transaction::commit` returns a generic
    /// `Err`/`RetryableTransaction` to recover the typed `unitycatalog_client::Error` and
    /// dispatch on it (429 → backfill+retry, 500 commit-state-unknown → reload+check, …).
    pub(crate) fn take_last_error(&self) -> Option<unitycatalog_client::Error> {
        self.last_error
            .lock()
            .expect("committer mutex poisoned")
            .take()
    }

    fn record_error(&self, err: unitycatalog_client::Error) {
        *self.last_error.lock().expect("committer mutex poisoned") = Some(err);
    }

    fn has_catalog_managed_feature(commit_metadata: &CommitMetadata) -> bool {
        commit_metadata.has_writer_feature(CATALOG_MANAGED_FEATURE)
            && commit_metadata.has_reader_feature(CATALOG_MANAGED_FEATURE)
    }

    /// Validate protocol features + metadata properties for a UC catalog-managed table.
    fn validate_catalog_managed_state(&self, commit_metadata: &CommitMetadata) -> DeltaResult<()> {
        require!(
            commit_metadata.commit_type() != CommitType::UpgradeToCatalogManaged,
            "upgrade to catalog-managed is not supported"
        );
        require!(
            commit_metadata.commit_type() != CommitType::DowngradeToPathBased,
            "downgrade to path-based is not supported"
        );
        require!(
            Self::has_catalog_managed_feature(commit_metadata),
            "table is missing required feature: catalogManaged"
        );
        require!(
            commit_metadata.has_writer_feature(VACUUM_PROTOCOL_CHECK_FEATURE)
                && commit_metadata.has_reader_feature(VACUUM_PROTOCOL_CHECK_FEATURE),
            "table is missing required feature: vacuumProtocolCheck"
        );
        require!(
            commit_metadata.has_writer_feature(IN_COMMIT_TIMESTAMP_FEATURE),
            "table is missing required feature: inCommitTimestamp"
        );

        let config = commit_metadata
            .metadata_configuration()
            .ok_or_else(|| DeltaError::generic("commit metadata is missing table configuration"))?;
        let table_id = config.get(UC_TABLE_ID_KEY).ok_or_else(|| {
            DeltaError::generic(format!("table is missing property: {UC_TABLE_ID_KEY}"))
        })?;
        require!(
            table_id == &self.table_id,
            format!(
                "table ID mismatch: committer={}, metadata={table_id}",
                self.table_id
            )
        );
        require!(
            config.get(ENABLE_IN_COMMIT_TIMESTAMPS).map(String::as_str) == Some("true"),
            "in-commit timestamps must be enabled (delta.enableInCommitTimestamps=true)"
        );
        Ok(())
    }

    /// Reject ALTER-style changes (protocol / metadata / clustering) on a data commit —
    /// this committer supports append-style commits only.
    ///
    /// **Documented limitation (ManagedTablesSpec §"Write to the table", line 73).** The spec
    /// requires that schema/property/protocol evolution ride the *same* `updateTable` request as
    /// the `add-commit` action — i.e. a single commit carries `set-columns` /
    /// `set-properties` / `set-protocol` (and `set-domain-metadata` for clustering) alongside
    /// `add-commit`, and UC ratifies the whole bundle atomically. We do not yet build those
    /// update actions, so rather than write a staged commit that diverges from the catalog's
    /// metadata, we reject the commit outright (safer than silently desyncing UC).
    ///
    /// To lift this limitation, derive the metadata/protocol/domain update actions from
    /// `commit_metadata` and append them to the `DeltaUpdateTableRequest.updates` next to the
    /// `add-commit` in [`commit_version_non_zero`](Self::commit_version_non_zero).
    fn validate_no_alter_table_changes(commit_metadata: &CommitMetadata) -> DeltaResult<()> {
        require!(
            !commit_metadata.has_protocol_change(),
            "schema/protocol evolution on a catalog-managed table is not yet supported: this \
             commit changes the table protocol, which must be sent as a `set-protocol` action in \
             the same updateTable request as the commit (ManagedTablesSpec §Write to the table)"
        );
        require!(
            !commit_metadata.has_metadata_change(),
            "schema/property evolution on a catalog-managed table is not yet supported: this \
             commit changes table metadata, which must be sent as `set-columns`/`set-properties` \
             actions in the same updateTable request as the commit (ManagedTablesSpec §Write to \
             the table)"
        );
        require!(
            !commit_metadata.has_domain_metadata_change(CLUSTERING_DOMAIN_NAME),
            "changing clustering columns on a catalog-managed table is not yet supported: it must \
             be sent as a `set-domain-metadata` action in the same updateTable request as the \
             commit (ManagedTablesSpec §Write to the table)"
        );
        Ok(())
    }

    /// Version 0 (create): validate the contract, then write `00…0.json` directly to the
    /// published path. Conflict if it already exists.
    fn commit_version_0(
        &self,
        engine: &dyn Engine,
        actions: DeltaResultIterator<'_, FilteredEngineData>,
        commit_metadata: &CommitMetadata,
    ) -> DeltaResult<CommitResponse> {
        self.validate_catalog_managed_state(commit_metadata)?;
        let published = commit_metadata.published_commit_path()?;
        match engine
            .json_handler()
            .write_json_file(&published, Box::new(actions), false)
        {
            Ok(()) => {
                info!("wrote version 0 commit for UC managed table creation");
                let file_meta = engine.storage_handler().head(&published)?;
                Ok(CommitResponse::Committed { file_meta })
            }
            Err(DeltaError::FileAlreadyExists(_)) => Ok(CommitResponse::Conflict { version: 0 }),
            Err(e) => Err(e),
        }
    }

    /// Version >= 1: write a staged commit, then `updateTable add-commit` to ratify it.
    fn commit_version_non_zero(
        &self,
        engine: &dyn Engine,
        actions: DeltaResultIterator<'_, FilteredEngineData>,
        commit_metadata: CommitMetadata,
    ) -> DeltaResult<CommitResponse> {
        self.validate_catalog_managed_state(&commit_metadata)?;
        Self::validate_no_alter_table_changes(&commit_metadata)?;

        let staged = commit_metadata.staged_commit_path()?;
        engine
            .json_handler()
            .write_json_file(&staged, Box::new(actions), false)?;
        let committed = engine.storage_handler().head(&staged)?;
        debug!(path = %staged, "wrote staged commit file");

        let version: i64 = commit_metadata.version().try_into().map_err(|_| {
            DeltaError::generic("commit version does not fit into i64 for UC commit")
        })?;
        let file_name = staged
            .path_segments()
            .and_then(|mut s| s.next_back())
            .ok_or_else(|| DeltaError::generic("staged commit path had no file name"))?
            .to_string();
        let file_size: i64 = committed
            .size
            .try_into()
            .map_err(|_| DeltaError::generic("staged commit size does not fit into i64"))?;

        // Record what we're about to propose so the caller's commit-state-unknown recovery can
        // recognise this exact commit in the reloaded tail.
        *self.last_proposed.lock().expect("committer mutex poisoned") =
            Some((version, file_name.clone()));

        let request = DeltaUpdateTableRequest {
            requirements: vec![DeltaTableRequirement::AssertTableUuid {
                uuid: self.table_id.clone(),
            }],
            updates: vec![DeltaTableUpdate::AddCommit {
                commit: DeltaCommit {
                    version,
                    timestamp: commit_metadata.in_commit_timestamp(),
                    file_name,
                    file_size,
                    file_modification_timestamp: committed.last_modified,
                },
                uniform: None,
            }],
        };

        match self.block_on_update(request) {
            Ok(()) => Ok(CommitResponse::Committed {
                file_meta: committed,
            }),
            // A version conflict (UC returns 409) becomes a kernel Conflict so the caller's
            // commit loop can rebuild and retry, rather than a hard error.
            Err(e) if is_conflict(&e) => Ok(CommitResponse::Conflict {
                version: commit_metadata.version(),
            }),
            // Any other UC error (429 resource-exhausted, 500 commit-state-unknown, …) must
            // collapse to a kernel `DeltaError` here, but the retry loop needs the *typed*
            // error to decide whether/how to retry. Stash it in the side channel first, then
            // return a generic error carrying the same message for diagnostics.
            Err(e) => {
                let msg = format!("UC add-commit failed: {e}");
                self.record_error(e);
                Err(DeltaError::generic(msg))
            }
        }
    }

    /// Run an async `update_table` to completion from the synchronous `Committer::commit`.
    /// Requires a multi-threaded tokio runtime (matches the upstream committer).
    fn block_on_update(&self, request: DeltaUpdateTableRequest) -> unitycatalog_client::Result<()> {
        let handle = tokio::runtime::Handle::try_current().map_err(|_| {
            unitycatalog_client::Error::Generic(
                "UnityCatalogCommitter must be used within a tokio runtime".to_string(),
            )
        })?;
        let client = self.client.clone();
        let (catalog, schema, table) = (
            self.catalog.clone(),
            self.schema.clone(),
            self.table.clone(),
        );
        tokio::task::block_in_place(|| {
            handle.block_on(async move {
                client
                    .update_table(&catalog, &schema, &table, &request)
                    .await
                    .map(|_| ())
            })
        })
    }
}

impl Committer for UnityCatalogCommitter {
    fn commit(
        &self,
        engine: &dyn Engine,
        actions: DeltaResultIterator<'_, FilteredEngineData>,
        commit_metadata: CommitMetadata,
    ) -> DeltaResult<CommitResponse> {
        if commit_metadata.version() == 0 {
            return self.commit_version_0(engine, actions, &commit_metadata);
        }
        self.commit_version_non_zero(engine, actions, commit_metadata)
    }

    fn is_catalog_committer(&self) -> bool {
        true
    }

    fn publish(&self, engine: &dyn Engine, publish_metadata: PublishMetadata) -> DeltaResult<()> {
        // Publish = copy each ratified staged commit to its published `_delta_log/<v>.json`
        // path (atomic / put-if-absent). An already-published version is not an error.
        for catalog_commit in publish_metadata.commits_to_publish() {
            match engine.storage_handler().copy_atomic(
                catalog_commit.location(),
                catalog_commit.published_location(),
            ) {
                Ok(()) | Err(DeltaError::FileAlreadyExists(_)) => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}

/// Whether a client error is a UC commit-version conflict (HTTP 409) that the
/// committer should surface as a retryable [`CommitResponse::Conflict`].
///
/// The `/delta/v1` server reports this through the typed Delta envelope as
/// `CommitVersionConflictException` (now parsed into `Error::Delta`, matched by
/// [`Error::is_commit_conflict`]) or, for a name collision, `AlreadyExistsException`
/// ([`Error::is_already_exists`]). The legacy `http_status() == 409` arm is kept as
/// a fallback for servers that still return an untyped 409 envelope.
fn is_conflict(err: &unitycatalog_client::Error) -> bool {
    err.is_commit_conflict()
        || err.is_already_exists()
        || matches!(err, unitycatalog_client::Error::Api(api) if api.http_status() == 409)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{UnityCatalogCommitter, is_conflict};
    use olai_http::CloudClient;
    use unitycatalog_client::{DeltaV1Client, Error, UcApiError};
    use unitycatalog_delta_api::models::{DeltaErrorModel, DeltaErrorType};
    use url::Url;

    fn test_committer() -> UnityCatalogCommitter {
        let client = DeltaV1Client::new(
            CloudClient::new_unauthenticated(),
            Url::parse("http://localhost/").unwrap(),
        );
        UnityCatalogCommitter::new(Arc::new(client), "c", "s", "t", "tid")
    }

    #[test]
    fn last_error_side_channel_round_trips_then_clears() {
        let committer = test_committer();
        assert!(committer.take_last_error().is_none());

        // A 429 the retry loop should recognise as resource-exhausted.
        committer.record_error(Error::Delta(DeltaErrorModel {
            message: "slow down".into(),
            error_type: DeltaErrorType::ResourceExhaustedException,
            code: 429,
            stack: None,
        }));
        let taken = committer.take_last_error().expect("error recorded");
        assert!(taken.is_resource_exhausted());
        // `take` clears the slot so a subsequent attempt starts clean.
        assert!(committer.take_last_error().is_none());
    }

    #[test]
    fn last_proposed_defaults_none() {
        let committer = test_committer();
        assert!(committer.last_proposed().is_none());
    }

    #[test]
    fn cloned_committer_shares_side_channels() {
        // The transaction takes a *clone* of the committer; both must observe the same cells so
        // the retry loop (holding the original) can read what the clone recorded.
        let committer = test_committer();
        let clone = committer.clone();
        clone.record_error(Error::Delta(DeltaErrorModel {
            message: "ambiguous".into(),
            error_type: DeltaErrorType::CommitStateUnknownException,
            code: 500,
            stack: None,
        }));
        let taken = committer
            .take_last_error()
            .expect("clone's error visible on original");
        assert!(taken.is_commit_state_unknown());
    }

    #[test]
    fn conflict_detects_already_exists_and_409_other() {
        // Typed AlreadyExists (409).
        assert!(is_conflict(&Error::Api(UcApiError::AlreadyExists {
            message: "commit 1 already exists".into(),
        })));
        // Untyped server error carrying status 409 (e.g. CommitVersionConflictException).
        assert!(is_conflict(&Error::Api(UcApiError::Other {
            status: 409,
            error_code: "CommitVersionConflictException".into(),
            message: "version conflict".into(),
        })));
    }

    #[test]
    fn conflict_detects_typed_delta_envelope() {
        // The /delta/v1 server now returns a typed Delta envelope, parsed into
        // Error::Delta — both the commit-version conflict and the name-collision
        // variants must be treated as retryable conflicts.
        assert!(is_conflict(&Error::Delta(DeltaErrorModel {
            message: "concurrent commit".into(),
            error_type: DeltaErrorType::CommitVersionConflictException,
            code: 409,
            stack: None,
        })));
        assert!(is_conflict(&Error::Delta(DeltaErrorModel {
            message: "already exists".into(),
            error_type: DeltaErrorType::AlreadyExistsException,
            code: 409,
            stack: None,
        })));
        // A 404 Delta envelope is not a conflict.
        assert!(!is_conflict(&Error::Delta(DeltaErrorModel {
            message: "no table".into(),
            error_type: DeltaErrorType::NoSuchTableException,
            code: 404,
            stack: None,
        })));
    }

    #[test]
    fn conflict_ignores_non_409() {
        assert!(!is_conflict(&Error::Api(UcApiError::NotFound {
            message: "no table".into(),
        })));
        assert!(!is_conflict(&Error::Api(UcApiError::Other {
            status: 400,
            error_code: "INVALID_PARAMETER_VALUE".into(),
            message: "bad".into(),
        })));
        assert!(!is_conflict(&Error::Generic("boom".into())));
    }
}
