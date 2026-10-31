use async_trait::async_trait;
use olai_credential_vending::{CredentialResolver, Error, RegisteredCredential, Result};
use olai_storage_credentials::StorageLocation;

use crate::StorageAccessStore;

/// Resolves a location to the credential of the external location governing it.
///
/// Registered locations never overlap, so [`Error::Overlapping`] cannot arise.
#[async_trait]
impl CredentialResolver for dyn StorageAccessStore {
    async fn resolve(&self, location: &StorageLocation) -> Result<RegisteredCredential> {
        let governed = match self.governing_location(location).await {
            Ok(governed) => governed,
            Err(crate::Error::NotFound) => {
                return Err(Error::NotFound {
                    location: location.to_string(),
                });
            }
            Err(e) => return Err(Error::mint(e)),
        };
        RegisteredCredential::try_from(&governed.credential).map_err(|e| Error::Unsupported {
            location: location.to_string(),
            reason: e.to_string(),
        })
    }
}
