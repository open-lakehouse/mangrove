//! Cloud credential providers backed by Unity Catalog credential vending.
//!
//! Each [`UCCredentialProvider`] implements [`object_store::CredentialProvider`]
//! for a single cloud (AWS, Azure, or GCP). It wraps a Unity Catalog
//! [`TemporaryCredentialClient`] and an [`olai_http::TokenCache`] so that:
//!
//! 1. The first credential vended at construction time is reused until close
//!    to its `expiration_time`.
//! 2. When `object_store` requests a credential after expiry, the provider
//!    transparently calls back into Unity Catalog to mint a fresh one
//!    (without blocking concurrent callers thanks to the token cache's
//!    single-flight semantics).
//!
//! The cache key is the originating securable + operation, so refreshes
//! always hit the same vending endpoint with the same arguments — meaning
//! credentials can never silently widen privileges across renewals.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use object_store::aws::AwsCredential;
use object_store::azure::AzureCredential;
use object_store::gcp::GcpCredential;
use object_store::{CredentialProvider, Result};

// The token cache and its `Instant` are selected per target, mirroring the
// transport alias in `lib.rs`. `olai_http_wasm`'s cache is API-compatible with
// `olai_http`'s, but its `TemporaryToken.expiry` is a `web_time::Instant` —
// `std::time::Instant::now()` panics on `wasm32-unknown-unknown`.
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

#[cfg(target_arch = "wasm32")]
use web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use olai_http::{TemporaryToken, TokenCache};
#[cfg(target_arch = "wasm32")]
use olai_http_wasm::{TemporaryToken, TokenCache};
use unitycatalog_client::{
    PathOperation, TableOperation, TemporaryCredentialClient, VolumeOperation,
};
use unitycatalog_common::models::temporary_credentials::v1::{
    TemporaryCredential, temporary_credential::Credentials,
};

use crate::Error;

/// Identifies the securable that a credential was vended for so refreshes
/// hit the same endpoint with the same arguments.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum SecurableRef {
    Table(uuid::Uuid, TableOperation),
    Volume(uuid::Uuid, VolumeOperation),
    Path(url::Url, PathOperation, Option<bool>),
}

pub struct UCCredentialProvider<T> {
    client: TemporaryCredentialClient,
    cache: TokenCache<Arc<T>>,
    securable: SecurableRef,
}

pub async fn new_azure(
    client: TemporaryCredentialClient,
    cred: &TemporaryCredential,
    securable: SecurableRef,
) -> Result<UCCredentialProvider<AzureCredential>> {
    let cache = TokenCache::<Arc<AzureCredential>>::default();
    // Seed the cache with the credential we already have so the first
    // `get_credential()` call does not round-trip to Unity Catalog again.
    let _ = cache.get_or_insert_with(|| async { as_azure(cred) }).await;
    Ok(UCCredentialProvider::<AzureCredential> {
        client,
        cache,
        securable,
    })
}

// AWS/GCP providers are never constructed on `wasm32` (the store is Azure-first
// there; `to_store` returns `unsupported` for them), so gate the constructors
// out of the wasm build to avoid dead-code warnings under `-D warnings`.
#[cfg(not(target_arch = "wasm32"))]
pub async fn new_aws(
    client: TemporaryCredentialClient,
    cred: &TemporaryCredential,
    securable: SecurableRef,
) -> Result<UCCredentialProvider<AwsCredential>> {
    let cache = TokenCache::<Arc<AwsCredential>>::default();
    let _ = cache.get_or_insert_with(|| async { as_aws(cred) }).await;
    Ok(UCCredentialProvider::<AwsCredential> {
        client,
        cache,
        securable,
    })
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn new_gcp(
    client: TemporaryCredentialClient,
    cred: &TemporaryCredential,
    securable: SecurableRef,
) -> Result<UCCredentialProvider<GcpCredential>> {
    let cache = TokenCache::<Arc<GcpCredential>>::default();
    let _ = cache.get_or_insert_with(|| async { as_gcp(cred) }).await;
    Ok(UCCredentialProvider::<GcpCredential> {
        client,
        cache,
        securable,
    })
}

impl<T> UCCredentialProvider<T> {
    /// Re-vend a credential for the securable this provider was bound to.
    ///
    /// Always hits the same endpoint with the same arguments to avoid
    /// silently widening privileges across renewals.
    async fn get_credential_inner(&self) -> Result<TemporaryCredential> {
        match &self.securable {
            SecurableRef::Table(table, op) => Ok(self
                .client
                .temporary_table_credential(*table, *op)
                .await
                .map_err(Error::from)?
                .0),
            SecurableRef::Volume(volume, op) => Ok(self
                .client
                .temporary_volume_credential(*volume, *op)
                .await
                .map_err(Error::from)?
                .0),
            SecurableRef::Path(path, op, dry_run) => Ok(self
                .client
                .temporary_path_credential(path.clone(), *op, *dry_run)
                .await
                .map_err(Error::from)?
                .0),
        }
    }
}

impl<T> std::fmt::Debug for UCCredentialProvider<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UCCredentialProvider")
            .field("securable", &self.securable)
            .finish()
    }
}

// The credential future on `wasm32` awaits the browser Fetch transport, which
// is `!Send`, yet `object_store::CredentialProvider` boxes a `Send` future.
// `send_wrapper::SendWrapper` makes it satisfy the bound: the wrapped future is
// only ever polled on its creating thread (true in the browser's single-thread
// model). The whole cache future is wrapped, not just the vending closure,
// because `TokenCache::get_or_insert_with`'s own future is `!Send` too.
#[async_trait::async_trait]
impl CredentialProvider for UCCredentialProvider<AzureCredential> {
    type Credential = AzureCredential;

    #[cfg(not(target_arch = "wasm32"))]
    async fn get_credential(&self) -> Result<Arc<AzureCredential>> {
        self.cache
            .get_or_insert_with(|| async { as_azure(&self.get_credential_inner().await?) })
            .await
    }

    #[cfg(target_arch = "wasm32")]
    async fn get_credential(&self) -> Result<Arc<AzureCredential>> {
        send_wrapper::SendWrapper::new(async move {
            self.cache
                .get_or_insert_with(|| async { as_azure(&self.get_credential_inner().await?) })
                .await
        })
        .await
    }
}

#[async_trait::async_trait]
impl CredentialProvider for UCCredentialProvider<AwsCredential> {
    type Credential = AwsCredential;

    #[cfg(not(target_arch = "wasm32"))]
    async fn get_credential(&self) -> Result<Arc<AwsCredential>> {
        self.cache
            .get_or_insert_with(|| async { as_aws(&self.get_credential_inner().await?) })
            .await
    }

    #[cfg(target_arch = "wasm32")]
    async fn get_credential(&self) -> Result<Arc<AwsCredential>> {
        send_wrapper::SendWrapper::new(async move {
            self.cache
                .get_or_insert_with(|| async { as_aws(&self.get_credential_inner().await?) })
                .await
        })
        .await
    }
}

#[async_trait::async_trait]
impl CredentialProvider for UCCredentialProvider<GcpCredential> {
    type Credential = GcpCredential;

    #[cfg(not(target_arch = "wasm32"))]
    async fn get_credential(&self) -> Result<Arc<GcpCredential>> {
        self.cache
            .get_or_insert_with(|| async { as_gcp(&self.get_credential_inner().await?) })
            .await
    }

    #[cfg(target_arch = "wasm32")]
    async fn get_credential(&self) -> Result<Arc<GcpCredential>> {
        send_wrapper::SendWrapper::new(async move {
            self.cache
                .get_or_insert_with(|| async { as_gcp(&self.get_credential_inner().await?) })
                .await
        })
        .await
    }
}

pub(super) fn as_azure(cred: &TemporaryCredential) -> Result<TemporaryToken<Arc<AzureCredential>>> {
    use Credentials::*;

    let az_cred = match cred
        .credentials
        .as_ref()
        .ok_or(crate::Error::NoCredential)?
    {
        AzureAad(token) => AzureCredential::BearerToken(token.aad_token.clone()),
        AzureUserDelegationSas(sas) => {
            // SAS tokens are query-string fragments (`k1=v1&k2=v2&...`).
            // `object_store` wants them as a `Vec<(String, String)>`.
            let pairs = sas.sas_token.split('&');
            let mut map = Vec::new();
            for pair in pairs {
                let mut parts = pair.split('=');
                if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                    map.push((key.to_string(), value.to_string()));
                }
            }
            AzureCredential::SASToken(map)
        }
        _ => return Err(crate::Error::credential_mismatch("Expected Azure credential.").into()),
    };

    Ok(TemporaryToken {
        token: Arc::new(az_cred),
        expiry: get_expiry(cred)?,
    })
}

/// Map an empty string to `None`, otherwise `Some(clone)`. An empty AWS session
/// token must not be signed as `x-amz-security-token: ` — some stores reject the
/// empty header — so credentials carrying no token (e.g. long-lived IAM keys)
/// surface as `token: None`.
fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

pub(super) fn as_aws(cred: &TemporaryCredential) -> Result<TemporaryToken<Arc<AwsCredential>>> {
    use Credentials::*;

    let aws_cred = match cred
        .credentials
        .as_ref()
        .ok_or(crate::Error::NoCredential)?
    {
        AwsTempCredentials(aws) => AwsCredential {
            key_id: aws.access_key_id.clone(),
            secret_key: aws.secret_access_key.clone(),
            token: non_empty(&aws.session_token),
        },
        R2TempCredentials(r2) => AwsCredential {
            key_id: r2.access_key_id.clone(),
            secret_key: r2.secret_access_key.clone(),
            token: non_empty(&r2.session_token),
        },
        _ => return Err(crate::Error::credential_mismatch("Expected AWS credential.").into()),
    };

    Ok(TemporaryToken {
        token: Arc::new(aws_cred),
        expiry: get_expiry(cred)?,
    })
}

pub(super) fn as_gcp(cred: &TemporaryCredential) -> Result<TemporaryToken<Arc<GcpCredential>>> {
    use Credentials::*;

    let gcp_cred = match cred
        .credentials
        .as_ref()
        .ok_or(crate::Error::NoCredential)?
    {
        GcpOauthToken(gcp) => GcpCredential {
            bearer: gcp.oauth_token.clone(),
        },
        _ => return Err(crate::Error::credential_mismatch("Expected GCS credential.").into()),
    };

    Ok(TemporaryToken {
        token: Arc::new(gcp_cred),
        expiry: get_expiry(cred)?,
    })
}

/// Returns the [`AwsTemporaryCredentials::access_point`] if present, so the
/// factory can use it when constructing the [`AmazonS3Builder`] — STS-vended
/// credentials are often only valid against the S3 access-point URL, not the
/// raw `s3://bucket/...` path.
///
/// Only used by the native AWS store branch (Azure-first on wasm).
#[cfg(not(target_arch = "wasm32"))]
pub(super) fn aws_access_point(cred: &TemporaryCredential) -> Option<String> {
    match cred.credentials.as_ref()? {
        Credentials::AwsTempCredentials(aws) if !aws.access_point.is_empty() => {
            Some(aws.access_point.clone())
        }
        _ => None,
    }
}

fn get_expiry(cred: &TemporaryCredential) -> Result<Option<Instant>> {
    let expiry = DateTime::from_timestamp_millis(cred.expiration_time)
        .ok_or(Error::credential_mismatch("Invalid expiration time"))?;
    let ttl = (expiry - Utc::now()).to_std().unwrap_or_default();

    Ok(Some(Instant::now() + ttl))
}
