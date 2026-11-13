//! An in-memory [`DeltaBackend`] for exercising the Delta business logic without a
//! real backend.
//!
//! Enabled by the `testing` feature. The crate's own tests use it to drive the
//! handler end-to-end; downstream servers (e.g. lakekeeper) can enable it to test
//! their own port wiring against known-good Delta semantics.
//!
//! It is deliberately permissive: authorization allows everything except the
//! staging creator-match (which compares the configured principal against the
//! reservation's `created_by`, reproducing mangrove's behavior so denial can be
//! tested), credential vending returns a fixed fake S3 credential, and
//! external-location validation is a no-op. The point is to exercise the *Delta*
//! logic (contract, dispatcher, commit arbitration), not access control.

pub mod coordinator_conformance;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::authz::DeltaAction;
use crate::backend::{
    BackendResult, CreateTableSpec, CredentialAccess, DeltaBackend, DeltaCapabilities,
    ResolvedTable, SchemaRef, StagingReservation, TableRef, TableUpdate, UpdateTableSpec,
    VendedCredential, etag_of,
};
use crate::coordinator::{CommitCoordinator, InMemoryCommitCoordinator};
use crate::error::DeltaBackendError;
use crate::models::DeltaDataSourceFormat;

/// The stored state for one table.
#[derive(Debug, Clone)]
struct StoredTable {
    full_name: String,
    resolved: ResolvedTable,
}

#[derive(Debug, Default)]
struct State {
    /// Tables keyed by `catalog.schema.table`.
    tables: BTreeMap<String, StoredTable>,
    /// Staging reservations keyed by location.
    staging: BTreeMap<String, StagingReservation>,
    /// Registered catalog names.
    catalogs: Vec<String>,
    /// Monotonic clock for created/updated timestamps.
    clock: i64,
}

/// An in-memory [`DeltaBackend`] (see the module docs). Generic over the context
/// type `Cx`, which it ignores.
///
/// [`Clone`] is a shallow copy: clones share the same underlying state and commit
/// coordinator (both behind an [`Arc`]), so a clone handed to [`get_router`] sees
/// the same tables. This is what lets it drop straight into the axum router, whose
/// state must be `Clone`.
///
/// [`get_router`]: crate::get_router
#[derive(Clone)]
pub struct InMemoryDeltaBackend {
    state: Arc<Mutex<State>>,
    coordinator: Arc<InMemoryCommitCoordinator>,
    /// The caller's principal, stamped onto staging reservations at allocation
    /// time and compared against `created_by` in the `AdoptStaging` decision.
    principal: Option<String>,
}

impl Default for InMemoryDeltaBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryDeltaBackend {
    /// A fresh backend with one catalog `"catalog"` registered and an anonymous
    /// principal.
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(State {
                catalogs: vec!["catalog".to_string()],
                ..Default::default()
            })),
            coordinator: Arc::new(InMemoryCommitCoordinator::default()),
            principal: None,
        }
    }

    /// Register an additional catalog name.
    pub fn with_catalog(self, name: impl Into<String>) -> Self {
        self.state.lock().unwrap().catalogs.push(name.into());
        self
    }

    /// Set the caller's principal, used to stamp `created_by` on new staging
    /// reservations and to decide the `AdoptStaging` creator-match.
    pub fn with_principal(mut self, principal: impl Into<String>) -> Self {
        self.principal = Some(principal.into());
        self
    }

    fn next_ts(state: &mut State) -> i64 {
        state.clock += 1;
        state.clock
    }

    fn find_by_id<'s>(state: &'s State, table_id: &str) -> BackendResult<&'s StoredTable> {
        state
            .tables
            .values()
            .find(|t| t.resolved.table_id.as_deref() == Some(table_id))
            .ok_or_else(|| DeltaBackendError::NotFound(format!("table id '{table_id}' not found")))
    }

    /// The `assert-etag` compare-and-swap, checked under the state lock against
    /// the current row (closing the TOCTOU race a handler-side check leaves open).
    fn check_etag(state: &State, spec: &UpdateTableSpec) -> BackendResult<()> {
        let stored = Self::find_by_id(state, &spec.table_id)?;
        if let Some(expected) = &spec.expected_etag
            && expected != &etag_of(&stored.resolved)
        {
            return Err(DeltaBackendError::UpdateRequirementConflict(
                "assert-etag failed: table has been modified".to_string(),
            ));
        }
        Ok(())
    }

    /// Write a metadata change to the row and bump its version (the etag).
    fn write_row(state: &mut State, spec: UpdateTableSpec) -> BackendResult<ResolvedTable> {
        let ts = Self::next_ts(state);
        let stored = state
            .tables
            .values_mut()
            .find(|t| t.resolved.table_id.as_deref() == Some(spec.table_id.as_str()))
            .ok_or_else(|| {
                DeltaBackendError::NotFound(format!("table id '{}' not found", spec.table_id))
            })?;
        stored.resolved.columns = spec.columns;
        stored.resolved.properties = spec.properties;
        // `None` means "leave the stored comment unchanged" (see `UpdateTableSpec`).
        if let Some(comment) = spec.comment {
            stored.resolved.comment = Some(comment);
        }
        stored.resolved.updated_at_ms = Some(ts);
        // Bump the version so the etag advances — the in-memory equivalent of the
        // store's per-object version counter (what a durable backend's CAS keys on).
        stored.resolved.version += 1;
        Ok(stored.resolved.clone())
    }
}

fn fake_credential(url: &str, access: CredentialAccess) -> BackendResult<VendedCredential> {
    use olai_storage_credentials::StorageCredentials;

    let scope = url.parse().map_err(|e| {
        DeltaBackendError::InvalidArgument(format!("invalid storage location '{url}': {e}"))
    })?;
    let expires_at = chrono::DateTime::from_timestamp_millis(4_102_444_800_000); // 2100-01-01
    Ok(VendedCredential {
        url: url.to_string(),
        credential: olai_storage_credentials::VendedCredential::new(
            StorageCredentials::aws("AKIAEXAMPLE", "secret", Some("token".to_string())),
            scope,
            access,
        )
        .with_expiry(expires_at),
    })
}

#[async_trait]
impl<Cx: Send + Sync + 'static> DeltaBackend<Cx> for InMemoryDeltaBackend {
    fn capabilities(&self) -> DeltaCapabilities {
        // This backend implements a working `rename_table`, so it advertises rename —
        // exercising the capability-on branch of `getConfig`'s endpoint list.
        DeltaCapabilities { rename: true }
    }

    async fn authorize(&self, action: DeltaAction<'_>, _cx: &Cx) -> BackendResult<()> {
        // Allow everything except the creator-match, which reproduces mangrove's
        // string-equality-of-principal check so tests can exercise denial.
        if let DeltaAction::AdoptStaging { reservation } = action
            && reservation.created_by.as_deref() != self.principal.as_deref()
        {
            return Err(DeltaBackendError::PermissionDenied(
                "caller is not the creator of the staging table".to_string(),
            ));
        }
        Ok(())
    }

    async fn catalog_exists(&self, catalog: &str, _cx: &Cx) -> BackendResult<()> {
        let state = self.state.lock().unwrap();
        if state.catalogs.iter().any(|c| c == catalog) {
            Ok(())
        } else {
            Err(DeltaBackendError::NotFound(format!(
                "catalog '{catalog}' not found"
            )))
        }
    }

    async fn resolve_table(&self, table: &TableRef, _cx: &Cx) -> BackendResult<ResolvedTable> {
        let state = self.state.lock().unwrap();
        state
            .tables
            .get(&table.full_name())
            .map(|t| t.resolved.clone())
            .ok_or_else(|| {
                DeltaBackendError::NotFound(format!("table '{}' not found", table.full_name()))
            })
    }

    async fn validate_external_location(&self, _location: &str, _cx: &Cx) -> BackendResult<()> {
        Ok(())
    }

    async fn create_table_row(
        &self,
        spec: CreateTableSpec,
        _cx: &Cx,
    ) -> BackendResult<ResolvedTable> {
        let mut state = self.state.lock().unwrap();
        let ts = Self::next_ts(&mut state);
        let full_name = format!("{}.{}.{}", spec.at.catalog, spec.at.schema, spec.name);
        if state.tables.contains_key(&full_name) {
            // The reservation (if any) is untouched: holding the lock across the
            // check and the consume-and-insert below makes this atomic, so a
            // failed create never orphans the staging reservation.
            return Err(DeltaBackendError::AlreadyExists(format!(
                "table '{full_name}' already exists"
            )));
        }
        // Atomically consume the adopted staging reservation, if any, in the same
        // critical section as the insert (a transactional backend does the same in
        // one transaction).
        if let Some(reservation) = &spec.adopt_staging {
            state.staging.remove(&reservation.location);
        }
        let table_id = spec
            .table_id
            .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
        let resolved = ResolvedTable {
            table_id: Some(table_id),
            location: spec.location,
            table_type: Some(spec.table_type),
            data_source_format: Some(DeltaDataSourceFormat::Delta),
            columns: spec.columns,
            properties: spec.properties,
            comment: spec.comment,
            created_at_ms: Some(ts),
            updated_at_ms: Some(ts),
            // Fresh row starts at version 0; each update bumps it (the etag).
            version: 0,
        };
        state.tables.insert(
            full_name.clone(),
            StoredTable {
                full_name,
                resolved: resolved.clone(),
            },
        );
        Ok(resolved)
    }

    async fn apply_update(&self, update: TableUpdate, _cx: &Cx) -> BackendResult<ResolvedTable> {
        // Atomic by construction: the state lock is held across the etag CAS, the
        // commit ratification (`commit_sync` never awaits) and the row write, and
        // nothing is written unless all of them succeed — the in-memory
        // equivalent of one database transaction.
        let mut state = self.state.lock().unwrap();
        let table_id = update
            .base
            .table_id
            .clone()
            .ok_or_else(|| DeltaBackendError::Internal("table has no id".into()))?;
        if let Some(spec) = &update.metadata {
            Self::check_etag(&state, spec)?;
        }
        if update.has_commit_log_change() {
            self.coordinator.commit_sync(
                &table_id,
                update.commit,
                update.latest_backfilled_version,
            )?;
        }
        match update.metadata {
            Some(spec) => Self::write_row(&mut state, spec),
            None => Self::find_by_id(&state, &table_id).map(|t| t.resolved.clone()),
        }
    }

    async fn update_table_row(
        &self,
        spec: UpdateTableSpec,
        _cx: &Cx,
    ) -> BackendResult<ResolvedTable> {
        let mut state = self.state.lock().unwrap();
        Self::check_etag(&state, &spec)?;
        Self::write_row(&mut state, spec)
    }

    async fn delete_table(&self, table: &TableRef, _cx: &Cx) -> BackendResult<()> {
        let mut state = self.state.lock().unwrap();
        state
            .tables
            .remove(&table.full_name())
            .map(|_| ())
            .ok_or_else(|| {
                DeltaBackendError::NotFound(format!("table '{}' not found", table.full_name()))
            })
    }

    async fn rename_table(&self, from: &TableRef, to_name: &str, _cx: &Cx) -> BackendResult<()> {
        let mut state = self.state.lock().unwrap();
        let Some(mut stored) = state.tables.remove(&from.full_name()) else {
            return Err(DeltaBackendError::NotFound(format!(
                "table '{}' not found",
                from.full_name()
            )));
        };
        let new_full = format!("{}.{}.{}", from.catalog, from.schema, to_name);
        stored.full_name = new_full.clone();
        state.tables.insert(new_full, stored);
        Ok(())
    }

    async fn allocate_staging(
        &self,
        _at: &SchemaRef,
        name: &str,
        _cx: &Cx,
    ) -> BackendResult<StagingReservation> {
        let mut state = self.state.lock().unwrap();
        let id = uuid::Uuid::now_v7().to_string();
        let location = format!("s3://bucket/staging/{id}");
        let reservation = StagingReservation {
            table_id: id,
            name: name.to_string(),
            location: location.clone(),
            created_by: self.principal.clone(),
            stage_committed: false,
        };
        state.staging.insert(location, reservation.clone());
        Ok(reservation)
    }

    async fn resolve_staging_by_location(
        &self,
        location: &str,
        _cx: &Cx,
    ) -> BackendResult<StagingReservation> {
        // Like a real backend's location index, ignore a trailing slash.
        let wanted = location.trim_end_matches('/');
        let state = self.state.lock().unwrap();
        state
            .staging
            .values()
            .find(|s| s.location.trim_end_matches('/') == wanted)
            .cloned()
            .ok_or_else(|| DeltaBackendError::NotFound(format!("no staging table at '{location}'")))
    }

    async fn resolve_staging_by_id(
        &self,
        table_id: &str,
        _cx: &Cx,
    ) -> BackendResult<StagingReservation> {
        let state = self.state.lock().unwrap();
        state
            .staging
            .values()
            .find(|s| s.table_id == table_id)
            .cloned()
            .ok_or_else(|| {
                DeltaBackendError::NotFound(format!("staging table '{table_id}' not found"))
            })
    }

    async fn vend_table_credential(
        &self,
        table_id: &str,
        access: CredentialAccess,
        _cx: &Cx,
    ) -> BackendResult<VendedCredential> {
        let state = self.state.lock().unwrap();
        let location = state
            .tables
            .values()
            .find(|t| t.resolved.table_id.as_deref() == Some(table_id))
            .map(|t| t.resolved.location.clone())
            .ok_or_else(|| {
                DeltaBackendError::NotFound(format!("table id '{table_id}' not found"))
            })?;
        fake_credential(&location, access)
    }

    async fn vend_path_credential(
        &self,
        location: &str,
        access: CredentialAccess,
        _cx: &Cx,
    ) -> BackendResult<VendedCredential> {
        fake_credential(location, access)
    }

    fn commit_coordinator(&self) -> &dyn CommitCoordinator {
        self.coordinator.as_ref()
    }
}
