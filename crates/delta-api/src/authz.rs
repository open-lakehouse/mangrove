//! The authorization vocabulary the handler speaks to the backend.
//!
//! Authorization is the one aspect of the Delta API that is genuinely
//! server-specific — mangrove has its `Policy`/securable model, lakekeeper its
//! `Authorizer`, and neither wants to import the other's. So this crate does not
//! *decide* anything; it names the operation it is about to perform as a
//! [`DeltaAction`] and asks the backend, via a single
//! [`DeltaBackend::authorize`](crate::backend::DeltaBackend::authorize) hook,
//! whether the caller may proceed.
//!
//! Two properties fall out of routing every operation through one action enum:
//!
//! - **The authz contract is uniform and type-enforced.** Every user-facing
//!   operation authorizes before it touches storage, and a new operation cannot
//!   be added without giving it a [`DeltaAction`] variant. There is no
//!   "some methods self-authorize, some don't" split for an implementor to trip
//!   over — the backend's data methods (`resolve_table`, `delete_table`,
//!   `vend_*`, …) are pure data access and must **not** re-authorize.
//! - **No server types leak into the crate.** Each variant carries only the
//!   crate's own coordinate types, so the backend pattern-matches a
//!   [`DeltaAction`] into its own permission/securable model without either side
//!   depending on the other.
//!
//! This is also the natural seam for a future Databricks-style GRANTS check: a
//! variant already carries the securable coordinate and the operation, which is
//! exactly what a `(securable_type, securable_fullname, privilege)` lookup needs.
//! Growing that store is a backend concern — the crate is unaffected.

use crate::backend::{CredentialAccess, SchemaRef, StagingReservation, TableRef};
use crate::models::DeltaTableType;

/// An operation the handler is about to perform, in the crate's own vocabulary.
///
/// Passed to [`DeltaBackend::authorize`](crate::backend::DeltaBackend::authorize)
/// before the operation runs. Each variant borrows the coordinate it targets;
/// the backend maps it onto its own authorization model.
#[derive(Debug)]
#[non_exhaustive]
pub enum DeltaAction<'a> {
    /// `createTable` — create a table under a schema.
    CreateTable {
        at: &'a SchemaRef,
        name: &'a str,
        table_type: DeltaTableType,
    },
    /// `loadTable` / `tableExists` — read a table's metadata.
    ReadTable { table: &'a TableRef },
    /// `updateTable` — mutate an existing table (metadata and/or commit).
    WriteTable {
        table: &'a TableRef,
        table_id: &'a str,
    },
    /// `deleteTable` — drop a table.
    DeleteTable { table: &'a TableRef },
    /// `renameTable` — rename a table within its catalog + schema.
    RenameTable { from: &'a TableRef, to: &'a str },
    /// `getTableCredentials` — vend a credential for a resolved table's location.
    VendTableCredential {
        table_id: &'a str,
        access: CredentialAccess,
    },
    /// `getStagingTableCredentials` / `getTemporaryPathCredentials` — vend a
    /// credential for an arbitrary path.
    VendPathCredential {
        location: &'a str,
        access: CredentialAccess,
    },
    /// `createStagingTable` — reserve a staging table under a schema.
    CreateStaging { at: &'a SchemaRef, name: &'a str },
    /// The managed-`createTable` creator-match: may this caller adopt the
    /// staging reservation it is committing? The backend answers in its own
    /// identity terms (the crate no longer models principals). The crate still
    /// enforces the genuine Delta semantics around adoption (`stage_committed`,
    /// the `tableId` property match).
    AdoptStaging { reservation: &'a StagingReservation },
}
