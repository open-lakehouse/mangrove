//! Minting temporary credentials from registered catalog credentials.
//!
//! The handlers resolve and authorize the governing external location themselves
//! (authorization is against the concrete securable), so this module only mints:
//! through one [`CachingMinter`] per registered credential, shared process-wide, or
//! through the [`LocalMinter`] for `file://` storage, which no credential governs.

use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, PoisonError};

use olai_credential_vending::{
    CachingMinter, CredentialMinter, LocalMinter, MintRequest, RegisteredCredential, minter_for,
};
use olai_storage_credentials::{AccessLevel, VendedCredential};
use unitycatalog_common::models::credentials::v1::Credential;
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;

use super::location::StorageLocationUrl;
use crate::Result;

/// The minter of each registered credential seen, keyed by the credential itself
/// (secrets included), so a rotated credential gets a fresh minter and cache.
static MINTERS: LazyLock<Mutex<HashMap<RegisteredCredential, Arc<CachingMinter>>>> =
    LazyLock::new(Mutex::default);

fn minter(credential: &RegisteredCredential) -> Result<Arc<CachingMinter>> {
    let minters = || MINTERS.lock().unwrap_or_else(PoisonError::into_inner);
    if let Some(minter) = minters().get(credential) {
        return Ok(Arc::clone(minter));
    }
    // Built outside the lock: building may read files or fail.
    let minter = Arc::new(CachingMinter::new(
        minter_for(credential).map_err(unitycatalog_common::Error::from)?,
    ));
    Ok(Arc::clone(
        minters().entry(credential.clone()).or_insert(minter),
    ))
}

/// The minter for `location`: the cached minter of `credential`, or the
/// [`LocalMinter`] for local (`file://`) storage, which no credential governs.
pub(crate) fn minter_for_location(
    credential: Option<&Credential>,
    location: &StorageLocationUrl,
) -> Result<Arc<dyn CredentialMinter>> {
    if location.is_local() {
        return Ok(Arc::new(LocalMinter));
    }
    let credential = credential.ok_or(crate::Error::NotFound)?;
    let registered = RegisteredCredential::try_from(credential)?;
    Ok(minter(&registered)?)
}

/// Mints a credential for `location` at `access`: from `credential`, or without one
/// for local (`file://`) storage.
pub(crate) async fn mint(
    credential: Option<&Credential>,
    location: &StorageLocationUrl,
    access: AccessLevel,
) -> Result<VendedCredential> {
    let request = MintRequest::new(location.scope(), access);
    let minter = minter_for_location(credential, location)?;
    Ok(minter
        .mint(&request)
        .await
        .map_err(unitycatalog_common::Error::from)?)
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
