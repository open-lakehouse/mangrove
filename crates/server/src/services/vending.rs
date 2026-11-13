//! Minting temporary credentials from registered catalog credentials.
//!
//! Vending goes through one trestle [`Vendor`] per handler, over a
//! [`CatalogResolver`]: a location resolves to the credential of the external
//! location governing it, or, for paths under the metastore's managed storage
//! root, to the credential configured for that root. The vendor keeps one cached
//! minter per registered credential and is cleared when credentials change (see
//! [`ServerHandler::forget_vended_credentials`](super::ServerHandler)), so rotated
//! or deleted secrets do not linger. Local (`file://`) storage, which no
//! credential governs, is minted by the [`LocalMinter`].
//!
//! Authorization is not done here: the callers authorize against the concrete
//! securable first.

use std::sync::Arc;

use async_trait::async_trait;
use olai_credential_vending::{
    CredentialMinter, CredentialResolver, LocalMinter, MintRequest, RegisteredCredential, Vendor,
};
use olai_storage_credentials::{AccessLevel, StorageLocation, StorageScope, VendedCredential};
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;
use unitycatalog_storage_access::StorageAccessStore;

use super::location::StorageLocationUrl;
use crate::Result;

/// The metastore's managed storage root and the registered credential governing
/// it, for managed storage no external location covers.
#[derive(Debug, Clone)]
pub(crate) struct ManagedRootCredential {
    pub(crate) scope: StorageScope,
    pub(crate) credential_name: String,
}

/// Resolves a location to the registered credential allowed to mint for it: the
/// governing external location's credential, else the managed root's.
pub(crate) struct CatalogResolver {
    store: Arc<dyn StorageAccessStore>,
    managed_root: Option<ManagedRootCredential>,
}

impl CatalogResolver {
    pub(crate) fn new(
        store: Arc<dyn StorageAccessStore>,
        managed_root: Option<ManagedRootCredential>,
    ) -> Self {
        Self {
            store,
            managed_root,
        }
    }
}

#[async_trait]
impl CredentialResolver for CatalogResolver {
    async fn resolve(
        &self,
        location: &StorageLocation,
    ) -> olai_credential_vending::Result<RegisteredCredential> {
        let not_found = || olai_credential_vending::Error::NotFound {
            location: location.to_string(),
        };
        match self.store.as_ref().resolve(location).await {
            Err(olai_credential_vending::Error::NotFound { .. }) => {}
            other => return other,
        }
        let Some(root) = self
            .managed_root
            .as_ref()
            .filter(|root| root.scope.covers(location))
        else {
            return Err(not_found());
        };
        let credential = self
            .store
            .credential_with_secrets(&root.credential_name)
            .await
            .map_err(|e| match e {
                unitycatalog_common::Error::NotFound => not_found(),
                other => olai_credential_vending::Error::mint(other),
            })?;
        RegisteredCredential::try_from(&credential).map_err(|e| {
            olai_credential_vending::Error::Unsupported {
                location: location.to_string(),
                reason: e.to_string(),
            }
        })
    }
}

/// The vendor a handler mints through.
pub(crate) type CatalogVendor = Vendor<CatalogResolver>;

/// Build the vendor for a handler's stores and managed-root configuration.
pub(crate) fn catalog_vendor(
    store: Arc<dyn StorageAccessStore>,
    managed_root: Option<ManagedRootCredential>,
) -> Arc<CatalogVendor> {
    Arc::new(Vendor::new(CatalogResolver::new(store, managed_root)))
}

/// A [`CredentialMinter`] over a shared vendor, for callers (the storage proxy's
/// refreshing store) that re-mint the same request later.
#[derive(Clone)]
pub(crate) struct VendorMinter(pub(crate) Arc<CatalogVendor>);

impl std::fmt::Debug for VendorMinter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("VendorMinter").field(&self.0).finish()
    }
}

#[async_trait]
impl CredentialMinter for VendorMinter {
    async fn mint(
        &self,
        request: &MintRequest,
    ) -> olai_credential_vending::Result<VendedCredential> {
        self.0.vend(request).await
    }
}

/// The minter for `location`: the [`LocalMinter`] for local (`file://`) storage,
/// else the vendor.
pub(crate) fn minter_for_location(
    vendor: &Arc<CatalogVendor>,
    location: &StorageLocationUrl,
) -> Arc<dyn CredentialMinter> {
    if location.is_local() {
        Arc::new(LocalMinter)
    } else {
        Arc::new(VendorMinter(vendor.clone()))
    }
}

/// Mints a credential for `location` at `access`.
pub(crate) async fn mint(
    vendor: &Arc<CatalogVendor>,
    location: &StorageLocationUrl,
    access: AccessLevel,
) -> Result<VendedCredential> {
    let request = MintRequest::new(location.scope(), access);
    minter_for_location(vendor, location)
        .mint(&request)
        .await
        // Normalize like the store lookups do, so a location no credential
        // governs is a plain `NotFound`.
        .map_err(|e| crate::error::from_store(unitycatalog_common::Error::from(e)))
}

/// The UC REST response for `vended`, naming `location` as the client spelled it
/// (emulator and endpoint URLs do not survive the canonical form).
pub(crate) fn to_temporary_credential(
    vended: &VendedCredential,
    location: &StorageLocationUrl,
) -> TemporaryCredential {
    TemporaryCredential {
        url: location.raw().to_string(),
        ..TemporaryCredential::from(vended)
    }
}
