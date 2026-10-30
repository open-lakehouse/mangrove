//! The browser (`wasm32`) credential source.
//!
//! `olai-vended-store`'s Unity Catalog client is native only (it speaks
//! `olai-http`), so on `wasm32` the factory vends through the generated
//! [`TemporaryCredentialClient`] over the browser Fetch transport instead. The
//! response is the same UC REST shape either way; it is read with
//! [`wire::unity`](olai_storage_credentials::wire::unity), and the stores, refresh
//! and routing all come from `olai-vended-store`.

use olai_storage_credentials::wire::{VendContext, unity};
use olai_storage_credentials::{
    AccessLevel, StorageLocation, StorageScope, StoreHints, VendedCredential,
};
use olai_vended_store::CredentialSource;
use unitycatalog_client::{
    PathOperation, TableOperation, TableReference, TemporaryCredentialClient, VolumeOperation,
    VolumeReference,
};
use url::Url;
use uuid::Uuid;

/// What a [`UcRestSource`] vends for.
#[derive(Debug, Clone)]
pub(crate) enum Securable {
    Table(Uuid),
    Volume(Uuid),
    Path(Url),
}

/// Vends for one securable at one access level through the generated client.
pub(crate) struct UcRestSource {
    client: TemporaryCredentialClient,
    securable: Securable,
    scope: StorageScope,
    hints: StoreHints,
    access: AccessLevel,
}

// Manual `Debug`: the generated client has none.
impl std::fmt::Debug for UcRestSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UcRestSource")
            .field("securable", &self.securable)
            .field("scope", &self.scope)
            .field("access", &self.access)
            .finish_non_exhaustive()
    }
}

impl UcRestSource {
    /// A source for `securable`, whose storage root is `location`.
    pub(crate) fn new(
        client: TemporaryCredentialClient,
        securable: Securable,
        location: &Url,
        access: AccessLevel,
    ) -> olai_vended_store::Result<Self> {
        let (location, hints) = StorageLocation::parse_with_hints(location)?;
        Ok(Self {
            client,
            securable,
            scope: StorageScope::new(location),
            hints,
            access,
        })
    }
}

#[async_trait::async_trait(?Send)]
impl CredentialSource for UcRestSource {
    async fn fetch(&self) -> olai_vended_store::Result<VendedCredential> {
        let read_write = self.access == AccessLevel::ReadWrite;
        let credential = match &self.securable {
            Securable::Table(id) => {
                let operation = if read_write {
                    TableOperation::ReadWrite
                } else {
                    TableOperation::Read
                };
                self.client
                    .temporary_table_credential(TableReference::Id(*id), operation)
                    .await
                    .map(|(credential, _)| credential)
            }
            Securable::Volume(id) => {
                let operation = if read_write {
                    VolumeOperation::ReadWrite
                } else {
                    VolumeOperation::Read
                };
                self.client
                    .temporary_volume_credential(VolumeReference::Id(*id), operation)
                    .await
                    .map(|(credential, _)| credential)
            }
            Securable::Path(url) => {
                let operation = if read_write {
                    PathOperation::ReadWrite
                } else {
                    PathOperation::Read
                };
                self.client
                    .temporary_path_credential(url.clone(), operation, false)
                    .await
                    .map(|(credential, _)| credential)
            }
        }
        .map_err(olai_vended_store::Error::vend)?;
        let wire = unity::TemporaryCredential::from(credential);
        let mut vended = wire.to_vended(&VendContext::new(self.scope.clone(), self.access))?;
        // What the storage URL itself implies (an Azurite endpoint) fills what the
        // response left out.
        vended.hints.fill_from(&self.hints);
        Ok(vended)
    }
}
