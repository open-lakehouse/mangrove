//! Resolving the storage credential that governs a location.

use unitycatalog_common::models::credentials::v1::Credential;
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;

use crate::Result;
use crate::location::StorageLocationUrl;
use crate::vending::{VendOperation, vend_credential};

/// Maps a storage location onto the [`Credential`] allowed to access it.
///
/// Unity Catalog resolves this through its external locations (the most specific
/// registered URL prefix, see [`longest_prefix_match`](crate::location::longest_prefix_match))
/// and the credential each one references. Other hosts — a standalone sharing
/// server, a test harness — plug in their own store of locations and credentials.
#[async_trait::async_trait]
pub trait CredentialResolver: Send + Sync {
    /// The credential governing `location`.
    ///
    /// # Errors
    ///
    /// Returns [`NotFound`](unitycatalog_common::Error::NotFound) when no
    /// registered location covers `location`.
    async fn resolve(&self, location: &StorageLocationUrl) -> Result<Credential>;
}

/// Resolve the credential governing `url` and vend a temporary credential for it,
/// downscoped to `operation`.
pub async fn vend_for_location(
    resolver: &(impl CredentialResolver + ?Sized),
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let location = StorageLocationUrl::parse(url)?;
    let credential = resolver.resolve(&location).await?;
    vend_credential(&credential, url, operation).await
}
