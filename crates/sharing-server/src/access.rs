//! [`SourceStorageAccess`]: storage access for shared assets, delegated to the
//! source catalog or served from the server's own credentials.
//!
//! Either way, access starts from one short-lived [`VendedCredential`] for the
//! asset's location, from which everything else is derived:
//!
//! - the server's own reads go through a store built from it ([`build_store`]);
//! - query responses carry URLs presigned with it ([`presign`]), which never
//!   outlive it;
//! - the recipient credential endpoints hand it on.
//!
//! In **delegated** mode the credential comes from the source catalog, so the
//! server holds no storage secrets. In **local** mode it is minted by
//! `olai-credential-vending` from a stored, sealed credential governing the
//! location's external location — the same minters the Unity Catalog server uses.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use axum::http::Method;
use chrono::{TimeDelta, Utc};
use olai_credential_vending::{
    CredentialMinter, CredentialResolver, LocalMinter, MintRequest, RegisteredCredential, Vendor,
};
use olai_storage_credentials::location::longest_match;
use olai_storage_credentials::wire::sharing as wire;
use olai_storage_credentials::{
    AccessLevel, StorageLocation, StorageProvider, StorageScope, VendedCredential,
};
use olai_vended_store::{CredentialSource, StoreOptions, build_store, presign};
use open_sharing_api::access::{FileSigner, SignedFile, StorageAccess, TableStorage};
use open_sharing_api::auth::Recipient;
use open_sharing_api::catalog::{AssetKind, SharedAsset};
use open_sharing_common::models::open_sharing::v1::{
    SharingAwsCredentials, SharingAzureUserDelegationSas, SharingGcpOauthToken,
    SharingR2Credentials, SharingTemporaryCredentials,
    sharing_temporary_credentials::Credentials as SharingCredentials,
};
use url::Url;

use crate::secrets::Secrets;
use crate::source::{AccessMode, SourceRegistry};
use crate::store::ShareStore;
use crate::{Error, Result};

/// Re-vend this long before a cached delegated credential expires.
const REFRESH_MARGIN: TimeDelta = TimeDelta::seconds(60);

/// Storage access for shared assets. See the module docs.
pub struct SourceStorageAccess {
    sources: SourceRegistry,
    /// Mints (and caches, per stored credential) for local access.
    vendor: Vendor<StoreCredentialResolver>,
    presign_ttl: Duration,
    options: StoreOptions,
    /// Delegated credentials, by asset kind and source name.
    cache: Mutex<HashMap<(AssetKindKey, String), VendedCredential>>,
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
        let mut options = StoreOptions::new();
        if let Some(region) = aws_region {
            options = options.with_aws_region(region);
        }
        Self {
            sources,
            vendor: Vendor::new(StoreCredentialResolver { store, secrets }),
            presign_ttl,
            options,
            cache: Mutex::default(),
        }
    }

    /// A read credential for the asset's location.
    async fn credential(&self, asset: &SharedAsset) -> Result<VendedCredential> {
        let source_name = asset
            .source_name
            .as_deref()
            .ok_or_else(|| Error::generic("shared asset has no source catalog name"))?;
        let source = self.sources.for_name(source_name)?;
        match source.access {
            AccessMode::Local => {
                let (location, hints) = StorageLocation::parse_with_hints(&asset.location.url)
                    .map_err(|e| Error::invalid_argument(format!("invalid asset location: {e}")))?;
                let request = MintRequest::new(StorageScope::new(location), AccessLevel::Read);
                let minted = if request.scope.location().provider() == StorageProvider::Local {
                    LocalMinter.mint(&request).await
                } else {
                    self.vendor.vend(&request).await
                };
                let mut credential = minted.map_err(vending_error)?;
                // What the location URL implies (an Azurite endpoint, a regional S3
                // host) fills what the minter left out.
                credential.hints.fill_from(&hints);
                Ok(credential)
            }
            AccessMode::Delegated => {
                let kind = match asset.reference.kind {
                    AssetKind::Table => AssetKindKey::Table,
                    AssetKind::Volume | AssetKind::AgentSkill => AssetKindKey::Volume,
                };
                let key = (kind, source_name.to_string());
                let fresh = |c: &&VendedCredential| {
                    c.expires_at
                        .is_none_or(|expires_at| expires_at - REFRESH_MARGIN > Utc::now())
                };
                if let Some(cached) = self
                    .cache
                    .lock()
                    .expect("credential cache poisoned")
                    .get(&key)
                    .filter(fresh)
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

/// The scope of the asset's location.
fn scope(asset: &SharedAsset) -> Result<StorageScope> {
    StorageLocation::parse(&asset.location.url)
        .map(StorageScope::new)
        .map_err(|e| Error::invalid_argument(format!("invalid asset location: {e}")))
}

#[async_trait]
impl StorageAccess<Recipient> for SourceStorageAccess {
    async fn table_storage(&self, asset: &SharedAsset, _cx: &Recipient) -> Result<TableStorage> {
        let credential = self.credential(asset).await?;
        // A store serves one query, which never outlives its credential: no re-vend.
        let store = build_store(
            &credential,
            Arc::new(Fixed(credential.clone())),
            &self.options,
        )
        .await
        .map_err(|e| Error::generic(format!("building storage access: {e}")))?;
        // Address the table the way the store is rooted: the canonical location,
        // whose path starts at the bucket or container (a path-style Azurite URL
        // also carries `/<account>/<container>`).
        let root = scope(asset)?.location().to_url();
        Ok(TableStorage {
            store: store.root(),
            root,
            signer: Arc::new(VendedSigner {
                credential,
                ttl: self.presign_ttl,
                options: self.options.clone(),
            }),
        })
    }

    async fn vend_read_credential(
        &self,
        asset: &SharedAsset,
        _cx: &Recipient,
    ) -> Result<SharingTemporaryCredentials> {
        let credential = self.credential(asset).await?;
        let mut credentials = to_sharing_credentials(&credential)?;
        // Name the location as the catalog spelled it: emulator and endpoint URLs do
        // not survive the canonical form.
        credentials.url = Some(asset.location.raw.clone());
        Ok(credentials)
    }
}

/// Hands out one credential, for stores that never outlive it.
#[derive(Debug)]
struct Fixed(VendedCredential);

#[async_trait]
impl CredentialSource for Fixed {
    async fn fetch(&self) -> olai_vended_store::Result<VendedCredential> {
        Ok(self.0.clone())
    }
}

/// Presigns files with a vended credential.
struct VendedSigner {
    credential: VendedCredential,
    ttl: Duration,
    options: StoreOptions,
}

#[async_trait]
impl FileSigner for VendedSigner {
    async fn sign(&self, files: &[Url]) -> Result<Vec<SignedFile>> {
        let locations = files
            .iter()
            .map(StorageLocation::parse)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| Error::invalid_argument(format!("invalid file location: {e}")))?;
        let signed = presign(
            &self.credential,
            Method::GET,
            &locations,
            self.ttl,
            &self.options,
        )
        .await
        .map_err(|e| Error::generic(format!("presigning files: {e}")))?;
        let expiration_timestamp = signed.expires_at.map(|t| t.timestamp_millis());
        Ok(signed
            .urls
            .into_iter()
            .map(|url| SignedFile {
                url: url.to_string(),
                expiration_timestamp,
            })
            .collect())
    }
}

/// Map a vended credential into the Open Sharing credential envelope.
pub(crate) fn to_sharing_credentials(
    credential: &VendedCredential,
) -> Result<SharingTemporaryCredentials> {
    let wire = wire::SharingTemporaryCredentials::try_from(credential).map_err(|_| {
        Error::invalid_argument("the storage credential type cannot be handed to recipients")
    })?;
    let credentials = if let Some(aws) = wire.aws_temp_credentials {
        Some(SharingCredentials::AwsTempCredentials(Box::new(
            SharingAwsCredentials {
                access_key_id: aws.access_key_id,
                secret_access_key: aws.secret_access_key,
                session_token: aws.session_token,
                ..Default::default()
            },
        )))
    } else if let Some(sas) = wire.azure_user_delegation_sas {
        Some(SharingCredentials::AzureUserDelegationSas(Box::new(
            SharingAzureUserDelegationSas {
                sas_token: sas.sas_token,
                ..Default::default()
            },
        )))
    } else if let Some(gcp) = wire.gcp_oauth_token {
        Some(SharingCredentials::GcpOauthToken(Box::new(
            SharingGcpOauthToken {
                oauth_token: gcp.oauth_token,
                ..Default::default()
            },
        )))
    } else {
        wire.r2_credentials.map(|r2| {
            SharingCredentials::R2Credentials(Box::new(SharingR2Credentials {
                access_key_id: r2.access_key_id,
                secret_access_key: r2.secret_access_key,
                session_token: r2.session_token,
                ..Default::default()
            }))
        })
    };
    Ok(SharingTemporaryCredentials {
        expiration_time: wire.expiration_time.unwrap_or_default(),
        url: wire.url,
        credentials,
        ..Default::default()
    })
}

pub(crate) fn vending_error(e: olai_credential_vending::Error) -> Error {
    use olai_credential_vending::Error as E;
    match e {
        E::NotFound { .. } => Error::invalid_argument(
            "no external location with a credential covers this storage location",
        ),
        E::Overlapping { .. } | E::Unsupported { .. } | E::Credentials(_) => {
            Error::InvalidArgument(e.to_string())
        }
        e => Error::generic(format!("vending storage credentials: {e}")),
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
    pub(crate) async fn open(&self, name: &str) -> Result<RegisteredCredential> {
        open_credential(self.store.as_ref(), &self.secrets, name).await
    }
}

/// Open the stored credential `name`, writing it back re-wrapped if it was sealed
/// under a retired key.
///
/// Credentials sealed as a full Unity Catalog `Credential` document (as the admin
/// API stored them before) open too: the metadata keys are ignored.
pub(crate) async fn open_credential(
    store: &dyn ShareStore,
    secrets: &Secrets,
    name: &str,
) -> Result<RegisteredCredential> {
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
        location: &StorageLocation,
    ) -> olai_credential_vending::Result<RegisteredCredential> {
        let to_vending = |e: Error| olai_credential_vending::Error::mint(e.to_string());
        let locations = self
            .store
            .list_external_locations()
            .await
            .map_err(to_vending)?;
        // Registrations whose URL no longer parses govern nothing.
        let scopes: Vec<_> = locations
            .iter()
            .filter_map(|l| Some((l.url.parse::<StorageScope>().ok()?, l)))
            .collect();
        let governing =
            longest_match(scopes.iter().map(|(s, l)| (s, *l)), location).ok_or_else(|| {
                olai_credential_vending::Error::NotFound {
                    location: location.to_string(),
                }
            })?;
        // A credential minted here covers everything below `location`, so refuse a
        // location containing one another credential governs.
        if let Some((nested, _)) = scopes.iter().find(|(scope, other)| {
            other.credential_name != governing.credential_name
                && scope.location().starts_with(location)
                && scope.location() != location
        }) {
            return Err(olai_credential_vending::Error::Overlapping {
                location: location.to_string(),
                nested: nested.to_string(),
            });
        }
        self.open(&governing.credential_name)
            .await
            .map_err(to_vending)
    }
}
