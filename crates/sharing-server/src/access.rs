//! [`SourceStorageAccess`]: storage access for shared assets, delegated to the
//! source catalog or served from the server's own credentials.
//!
//! Either way, access starts from one short-lived [`TemporaryCredential`] for the
//! asset's location, from which everything else is derived:
//!
//! - the server's own reads go through a store built from it
//!   ([`store_from_vended_credential`]);
//! - query responses carry URLs presigned with it
//!   ([`presign_from_vended_credential`]), which never outlive it;
//! - the recipient credential endpoints hand it on.
//!
//! In **delegated** mode the credential comes from the source catalog, so the
//! server holds no storage secrets. In **local** mode it is minted by the shared
//! `olai-uc-credentials` vending code from a stored, sealed credential governing
//! the location's external location — the same implementation the Unity Catalog
//! server uses.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use open_sharing_api::access::{FileSigner, SignedFile, StorageAccess, TableStorage};
use open_sharing_api::auth::Recipient;
use open_sharing_api::catalog::{AssetKind, SharedAsset};
use open_sharing_common::models::open_sharing::v1::{
    SharingAwsCredentials, SharingAzureUserDelegationSas, SharingGcpOauthToken,
    SharingR2Credentials, SharingTemporaryCredentials,
    sharing_temporary_credentials::Credentials as SharingCredentials,
};
use unitycatalog_common::models::credentials::v1::Credential;
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;
use unitycatalog_common::models::temporary_credentials::v1::temporary_credential::Credentials;
use unitycatalog_credentials::location::{StorageLocationUrl, longest_prefix_match};
use unitycatalog_credentials::{CredentialResolver, VendOperation, local_path_credential};
use unitycatalog_object_store::{presign_from_vended_credential, store_from_vended_credential};
use url::Url;

use crate::secrets::Secrets;
use crate::source::{AccessMode, SourceRegistry};
use crate::store::ShareStore;
use crate::{Error, Result};

/// Re-vend this long before a cached credential expires.
const REFRESH_MARGIN_MILLIS: i64 = 60_000;

/// Storage access for shared assets. See the module docs.
pub struct SourceStorageAccess {
    sources: SourceRegistry,
    resolver: Arc<StoreCredentialResolver>,
    presign_ttl: Duration,
    aws_region: Option<String>,
    /// Delegated credentials, by asset kind and source name.
    cache: Mutex<HashMap<(AssetKindKey, String), TemporaryCredential>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AssetKindKey {
    Table,
    Volume,
}

impl SourceStorageAccess {
    pub fn new(
        sources: SourceRegistry,
        store: Arc<dyn ShareStore>,
        secrets: Secrets,
        presign_ttl: Duration,
        aws_region: Option<String>,
    ) -> Self {
        Self {
            sources,
            resolver: Arc::new(StoreCredentialResolver { store, secrets }),
            presign_ttl,
            aws_region,
            cache: Mutex::default(),
        }
    }

    /// A read credential for the asset's location.
    async fn credential(&self, asset: &SharedAsset) -> Result<TemporaryCredential> {
        let source_name = asset
            .source_name
            .as_deref()
            .ok_or_else(|| Error::generic("shared asset has no source catalog name"))?;
        let source = self.sources.for_name(source_name)?;
        match source.access {
            AccessMode::Local => {
                if asset.location.url.scheme() == "file" {
                    return Ok(local_path_credential(&asset.location.raw));
                }
                unitycatalog_credentials::vend_for_location(
                    self.resolver.as_ref(),
                    &asset.location.raw,
                    VendOperation::Read,
                )
                .await
                .map_err(credentials_error)
            }
            AccessMode::Delegated => {
                let kind = match asset.reference.kind {
                    AssetKind::Table => AssetKindKey::Table,
                    AssetKind::Volume | AssetKind::AgentSkill => AssetKindKey::Volume,
                };
                let key = (kind, source_name.to_string());
                let now = chrono::Utc::now().timestamp_millis();
                if let Some(cached) = self
                    .cache
                    .lock()
                    .expect("credential cache poisoned")
                    .get(&key)
                    .filter(|c| c.expiration_time - REFRESH_MARGIN_MILLIS > now)
                {
                    return Ok(cached.clone());
                }
                let credential = match kind {
                    AssetKindKey::Table => {
                        let table = source.source.get_table(source_name).await?;
                        source.source.vend_table(&table).await?
                    }
                    AssetKindKey::Volume => {
                        let volume = source.source.get_volume(source_name).await?;
                        source.source.vend_volume(&volume).await?
                    }
                };
                self.cache
                    .lock()
                    .expect("credential cache poisoned")
                    .insert(key, credential.clone());
                Ok(credential)
            }
        }
    }
}

#[async_trait]
impl StorageAccess<Recipient> for SourceStorageAccess {
    async fn table_storage(&self, asset: &SharedAsset, _cx: &Recipient) -> Result<TableStorage> {
        let credential = self.credential(asset).await?;
        let store = store_from_vended_credential(&credential, None, self.aws_region.clone())
            .map_err(|e| Error::generic(format!("building storage access: {e}")))?;
        // Address the table the way the store is rooted (e.g. a path-style
        // Azurite URL becomes `azurite://<container>/…`).
        let root = StorageLocationUrl::parse(&asset.location.raw)
            .map_err(|e| Error::invalid_argument(format!("invalid table location: {e}")))?
            .location()
            .clone();
        Ok(TableStorage {
            store: store.root(),
            root,
            signer: Arc::new(VendedSigner {
                credential,
                ttl: self.presign_ttl,
                aws_region: self.aws_region.clone(),
            }),
        })
    }

    async fn vend_read_credential(
        &self,
        asset: &SharedAsset,
        _cx: &Recipient,
    ) -> Result<SharingTemporaryCredentials> {
        to_sharing_credentials(self.credential(asset).await?)
    }
}

/// Presigns files with a vended credential.
struct VendedSigner {
    credential: TemporaryCredential,
    ttl: Duration,
    aws_region: Option<String>,
}

#[async_trait]
impl FileSigner for VendedSigner {
    async fn sign(&self, files: &[Url]) -> Result<Vec<SignedFile>> {
        let signed = presign_from_vended_credential(
            &self.credential,
            files,
            self.ttl,
            self.aws_region.clone(),
        )
        .await
        .map_err(|e| Error::generic(format!("presigning files: {e}")))?;
        Ok(signed
            .urls
            .into_iter()
            .map(|url| SignedFile {
                url: url.to_string(),
                expiration_timestamp: signed.expiration_time,
            })
            .collect())
    }
}

/// Map a vended UC credential into the Open Sharing credential envelope.
pub(crate) fn to_sharing_credentials(
    credential: TemporaryCredential,
) -> Result<SharingTemporaryCredentials> {
    let credentials = match credential.credentials {
        None => None,
        Some(Credentials::AwsTempCredentials(aws)) => Some(SharingCredentials::AwsTempCredentials(
            Box::new(SharingAwsCredentials {
                access_key_id: aws.access_key_id,
                secret_access_key: aws.secret_access_key,
                session_token: aws.session_token,
                ..Default::default()
            }),
        )),
        Some(Credentials::AzureUserDelegationSas(sas)) => Some(
            SharingCredentials::AzureUserDelegationSas(Box::new(SharingAzureUserDelegationSas {
                sas_token: sas.sas_token,
                ..Default::default()
            })),
        ),
        Some(Credentials::GcpOauthToken(gcp)) => Some(SharingCredentials::GcpOauthToken(Box::new(
            SharingGcpOauthToken {
                oauth_token: gcp.oauth_token,
                ..Default::default()
            },
        ))),
        Some(Credentials::R2TempCredentials(r2)) => Some(SharingCredentials::R2Credentials(
            Box::new(SharingR2Credentials {
                access_key_id: r2.access_key_id,
                secret_access_key: r2.secret_access_key,
                session_token: r2.session_token,
                ..Default::default()
            }),
        )),
        Some(_) => {
            return Err(Error::invalid_argument(
                "the storage credential type cannot be handed to recipients",
            ));
        }
    };
    Ok(SharingTemporaryCredentials {
        expiration_time: credential.expiration_time,
        url: Some(credential.url),
        credentials,
        ..Default::default()
    })
}

pub(crate) fn credentials_error(e: unitycatalog_credentials::Error) -> Error {
    match e {
        unitycatalog_credentials::Error::Common(source) => match source {
            unitycatalog_common::Error::NotFound => Error::invalid_argument(
                "no external location with a credential covers this storage location",
            ),
            source => Error::Common { source },
        },
        unitycatalog_credentials::Error::CloudCredential(e) => {
            Error::generic(format!("vending storage credentials: {e}"))
        }
    }
}

/// Resolves a location's credential from the store's external locations and
/// sealed credentials, re-wrapping credentials sealed under a retired key.
pub(crate) struct StoreCredentialResolver {
    store: Arc<dyn ShareStore>,
    secrets: Secrets,
}

impl StoreCredentialResolver {
    /// Open the stored credential `name`, migrating it to the active key.
    pub(crate) async fn open(&self, name: &str) -> Result<Credential> {
        open_credential(self.store.as_ref(), &self.secrets, name).await
    }
}

/// Open the stored credential `name`, writing it back re-wrapped if it was sealed
/// under a retired key.
pub(crate) async fn open_credential(
    store: &dyn ShareStore,
    secrets: &Secrets,
    name: &str,
) -> Result<Credential> {
    let sealed = store.get_credential(name).await?;
    let (plaintext, rewrapped) = secrets.open(&credential_aad(name), &sealed.sealed).await?;
    if let Some(sealed) = rewrapped {
        store
            .put_credential(crate::store::SealedCredential {
                name: name.to_string(),
                sealed,
            })
            .await?;
    }
    Ok(serde_json::from_slice(&plaintext)?)
}

/// The AAD a stored credential is sealed under.
pub(crate) fn credential_aad(name: &str) -> String {
    format!("credential:{name}")
}

#[async_trait]
impl CredentialResolver for StoreCredentialResolver {
    async fn resolve(
        &self,
        location: &StorageLocationUrl,
    ) -> unitycatalog_credentials::Result<Credential> {
        let to_common = |e: Error| unitycatalog_common::Error::generic(e.to_string());
        let locations = self
            .store
            .list_external_locations()
            .await
            .map_err(to_common)?;
        let location = longest_prefix_match(location, locations, |l| l.url.as_str())
            .ok_or(unitycatalog_common::Error::NotFound)?;
        Ok(self
            .open(&location.credential_name)
            .await
            .map_err(to_common)?)
    }
}
