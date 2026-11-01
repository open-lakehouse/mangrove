//! Storage credentials and the external locations they govern.
//!
//! The catalog needs more from these two resources than get-or-list by name: every
//! credential vend resolves the external location governing a storage path, and
//! every external table or volume is checked against the registered locations.
//! [`StorageAccessStore`] is the port shaped by those needs. It stands on its own,
//! rather than on the generic resource store, so a backend can keep the pair in
//! tables of its own and answer [`governing_location`] from an index.
//!
//! The store owns the integrity rules of the pair:
//!
//! - an external location names an existing credential, by id, so renaming the
//!   credential carries over to the locations it governs;
//! - no two external locations overlap, in either direction, on whole path
//!   segments ([`LocationKey`]);
//! - a credential cannot be deleted while an external location uses it.
//!
//! Because locations never overlap, a path has at most one covering location, and
//! no registered location nests inside another one governed by a different
//! credential.
//!
//! Names are case-insensitive. Reads never carry secret material, except
//! [`Governed::credential`], which is what a minter needs.
//!
//! [`governing_location`]: StorageAccessStore::governing_location

use async_trait::async_trait;
use olai_storage_credentials::StorageLocation;
use unitycatalog_common::models::credentials::v1::{Credential, Purpose};
use unitycatalog_common::models::external_locations::v1::ExternalLocation;

pub use unitycatalog_common::{Error, Result};

pub use crate::key::LocationKey;
pub use crate::secrets::CredentialSecrets;

mod key;
pub mod rules;
mod secrets;

#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "memory")]
pub use crate::memory::InMemoryStorageAccess;

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
pub use crate::postgres::PgStorageAccess;

#[cfg(feature = "sqlite")]
pub mod sqlite;
#[cfg(feature = "sqlite")]
pub use crate::sqlite::SqliteStorageAccess;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod sql;

#[cfg(feature = "testing")]
pub mod testing;

#[cfg(feature = "vending")]
mod vending;

/// One page of a listing request.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Page {
    /// The most entries to return; `None` returns all of them.
    pub max_results: Option<usize>,
    /// The token returned with the previous page.
    pub token: Option<String>,
}

impl Page {
    /// A page of at most `max_results` entries, after `token`.
    pub fn new(max_results: Option<usize>, token: Option<String>) -> Self {
        Self { max_results, token }
    }
}

/// The external location governing a storage path, with its credential.
#[derive(Debug, Clone, PartialEq)]
pub struct Governed {
    /// The external location covering the path.
    pub location: ExternalLocation,
    /// The credential the location names, secrets included.
    pub credential: Credential,
}

/// Changes to a credential. `None` leaves a field as it is.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CredentialUpdate {
    /// A new name.
    pub new_name: Option<String>,
    /// A new comment.
    pub comment: Option<String>,
    /// Whether the credential is read-only.
    pub read_only: Option<bool>,
    /// A new owner.
    pub owner: Option<String>,
    /// Replacement secret configuration.
    pub secrets: Option<CredentialSecrets>,
}

/// Changes to an external location. `None` leaves a field as it is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExternalLocationUpdate {
    /// A new name.
    pub new_name: Option<String>,
    /// A new URL; it must not overlap another location.
    pub url: Option<String>,
    /// The name of the credential to use instead; it must exist.
    pub credential_name: Option<String>,
    /// Whether the location is read-only.
    pub read_only: Option<bool>,
    /// A new comment.
    pub comment: Option<String>,
    /// A new owner.
    pub owner: Option<String>,
}

/// Credentials and external locations, with the lookups the catalog runs on them.
///
/// Every error is a [`unitycatalog_common::Error`]: [`Error::NotFound`] for a missing
/// name, [`Error::AlreadyExists`] for a taken one, and [`Error::InvalidArgument`] for
/// a request breaking one of the [crate-level](crate) rules.
#[async_trait]
pub trait StorageAccessStore: Send + Sync + 'static {
    /// Registers `credential`, which carries its secret configuration.
    ///
    /// The store assigns the id and timestamps; `purpose` is fixed from here on.
    /// Returns the stored credential without secrets.
    async fn create_credential(&self, credential: Credential) -> Result<Credential>;

    /// The credential named `name`, without secrets.
    async fn get_credential(&self, name: &str) -> Result<Credential>;

    /// Credentials in name order, optionally only those with `purpose`.
    async fn list_credentials(
        &self,
        purpose: Option<Purpose>,
        page: Page,
    ) -> Result<(Vec<Credential>, Option<String>)>;

    /// Applies `update` to the credential named `name`.
    async fn update_credential(&self, name: &str, update: CredentialUpdate) -> Result<Credential>;

    /// Deletes the credential named `name`, refused while a location uses it.
    async fn delete_credential(&self, name: &str) -> Result<()>;

    /// Registers `location`, governed by the credential named in its `credential_name`.
    ///
    /// Refused when the URL overlaps a registered location. The store assigns the id
    /// and timestamps and fills in `credential_id`.
    async fn create_external_location(
        &self,
        location: ExternalLocation,
    ) -> Result<ExternalLocation>;

    /// The external location named `name`.
    async fn get_external_location(&self, name: &str) -> Result<ExternalLocation>;

    /// External locations in name order.
    async fn list_external_locations(
        &self,
        page: Page,
    ) -> Result<(Vec<ExternalLocation>, Option<String>)>;

    /// Applies `update` to the external location named `name`.
    async fn update_external_location(
        &self,
        name: &str,
        update: ExternalLocationUpdate,
    ) -> Result<ExternalLocation>;

    /// Deletes the external location named `name`.
    async fn delete_external_location(&self, name: &str) -> Result<()>;

    /// The external location covering `location`, with its credential and secrets.
    ///
    /// Returns [`Error::NotFound`] when no registered location covers it.
    async fn governing_location(&self, location: &StorageLocation) -> Result<Governed>;

    /// The external location covering `location`, if any.
    ///
    /// The same lookup as [`governing_location`](Self::governing_location), without
    /// reading the credential.
    async fn covering_location(
        &self,
        location: &StorageLocation,
    ) -> Result<Option<ExternalLocation>>;

    /// Re-wraps every sealed secret under the active key-encryption key, after a
    /// key rotation. Returns how many were re-wrapped.
    ///
    /// Secrets already under the active key are left alone, so running it again
    /// returns 0. Backends that keep no sealed secrets return 0.
    async fn rewrap_secrets(&self) -> Result<usize>;
}

/// Access to a [`StorageAccessStore`].
pub trait ProvidesStorageAccessStore {
    /// The store.
    fn storage_access(&self) -> &dyn StorageAccessStore;
}
