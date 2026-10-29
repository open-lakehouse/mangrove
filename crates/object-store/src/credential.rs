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
use unitycatalog_credentials::model::{TemporaryCredential, VendedCredential};

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
                .0
                .into()),
            SecurableRef::Volume(volume, op) => Ok(self
                .client
                .temporary_volume_credential(*volume, *op)
                .await
                .map_err(Error::from)?
                .0
                .into()),
            SecurableRef::Path(path, op, dry_run) => Ok(self
                .client
                .temporary_path_credential(path.clone(), *op, *dry_run)
                .await
                .map_err(Error::from)?
                .0
                .into()),
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
    use VendedCredential::*;

    let az_cred = match cred
        .credentials
        .as_ref()
        .ok_or(crate::Error::NoCredential)?
    {
        AzureAad(token) => AzureCredential::BearerToken(token.aad_token.clone()),
        AzureUserDelegationSas(sas) => AzureCredential::SASToken(split_sas(&sas.sas_token)?),
        _ => return Err(crate::Error::credential_mismatch("Expected Azure credential.").into()),
    };

    Ok(TemporaryToken {
        token: Arc::new(az_cred),
        expiry: get_expiry(cred)?,
    })
}

/// Split a SAS query string (`k1=v1&k2=v2&…`) into **decoded** `(key, value)`
/// pairs for [`AzureCredential::SASToken`].
///
/// Mirrors object_store's own `split_sas` (behind `AzureConfigKey::SasKey`).
/// Unity Catalog vends the token URL-encoded (`sig=…%2B…%3D`), and object_store
/// re-encodes `SASToken` pairs when appending them to each request — so they
/// must be decoded here, or the signature goes out double-encoded (`%252B`) and
/// Azure rejects it.
fn split_sas(sas: &str) -> Result<Vec<(String, String)>> {
    let sas = percent_encoding::percent_decode_str(sas)
        .decode_utf8()
        .map_err(|e| Error::credential_mismatch(format!("invalid SAS token encoding: {e}")))?;
    sas.trim_start_matches('?')
        .split('&')
        .filter(|pair| !pair.trim().is_empty())
        .map(|pair| {
            pair.trim()
                .split_once('=')
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .ok_or_else(|| Error::credential_mismatch("malformed SAS token component").into())
        })
        .collect()
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
    use VendedCredential::*;

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
    use VendedCredential::*;

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
        VendedCredential::AwsTempCredentials(aws) if !aws.access_point.is_empty() => {
            Some(aws.access_point.clone())
        }
        _ => None,
    }
}

/// Translate the vended `expiration_time` (epoch millis) into a cache expiry.
///
/// Servers that do not vend an expiry (OSS Unity Catalog returns `null`) surface
/// as the proto default `0`. Treat any non-positive value as "no expiry" so the
/// [`TokenCache`] keeps the credential — reading it as the Unix epoch would mark
/// it already expired and re-vend on every single request.
fn get_expiry(cred: &TemporaryCredential) -> Result<Option<Instant>> {
    if cred.expiration_time <= 0 {
        return Ok(None);
    }
    let expiry = DateTime::from_timestamp_millis(cred.expiration_time)
        .ok_or(Error::credential_mismatch("Invalid expiration time"))?;
    let ttl = (expiry - Utc::now()).to_std().unwrap_or_default();

    Ok(Some(Instant::now() + ttl))
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use object_store::ObjectStoreExt;
    use object_store::azure::MicrosoftAzureBuilder;
    use object_store::path::Path;
    use unitycatalog_credentials::model::AwsTemporaryCredentials;
    use url::Url;

    use super::*;

    fn now_epoch_millis() -> i64 {
        Utc::now().timestamp_millis()
    }

    fn aws_credential(expiration_time: i64) -> TemporaryCredential {
        TemporaryCredential {
            expiration_time,
            url: "s3://bucket/prefix/".to_string(),
            credentials: Some(
                AwsTemporaryCredentials {
                    access_key_id: "ak".to_string(),
                    secret_access_key: "sk".to_string(),
                    session_token: "st".to_string(),
                    ..Default::default()
                }
                .into(),
            ),
        }
    }

    /// A provider bound to a path securable whose refreshes go to `server`.
    async fn aws_provider(
        server: &mockito::Server,
        seed: &TemporaryCredential,
    ) -> UCCredentialProvider<AwsCredential> {
        let base = Url::parse(&format!("{}/api/2.1/unity-catalog/", server.url())).unwrap();
        let client = TemporaryCredentialClient::new_with_url(
            olai_http::CloudClient::new_unauthenticated(),
            base,
        );
        let securable = SecurableRef::Path(
            Url::parse("s3://bucket/prefix/").unwrap(),
            PathOperation::Read,
            Some(false),
        );
        new_aws(client, seed, securable).await.unwrap()
    }

    /// A credential vended without an expiry (OSS UC sends `null` → `0`) is
    /// served from the cache instead of being re-vended on every request.
    #[tokio::test]
    async fn missing_expiry_does_not_revend() {
        let mut server = mockito::Server::new_async().await;
        let vend = server
            .mock("POST", "/api/2.1/unity-catalog/temporary-path-credentials")
            .expect(0)
            .create_async()
            .await;

        let provider = aws_provider(&server, &aws_credential(0)).await;
        for _ in 0..3 {
            let cred = provider.get_credential().await.unwrap();
            assert_eq!(cred.key_id, "ak");
        }

        vend.assert_async().await;
    }

    /// An expired credential is re-vended for the same securable.
    #[tokio::test]
    async fn expired_credential_revends() {
        let mut server = mockito::Server::new_async().await;
        let fresh = format!(
            r#"{{"expiration_time":{},"url":"s3://bucket/prefix/","aws_temp_credentials":{{"access_key_id":"ak2","secret_access_key":"sk2","session_token":"st2"}}}}"#,
            now_epoch_millis() + 3_600_000
        );
        let vend = server
            .mock("POST", "/api/2.1/unity-catalog/temporary-path-credentials")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(fresh)
            .expect(1)
            .create_async()
            .await;

        let provider = aws_provider(&server, &aws_credential(now_epoch_millis() - 1_000)).await;
        // The first call refreshes; the second is served from the fresh cache entry.
        for _ in 0..2 {
            let cred = provider.get_credential().await.unwrap();
            assert_eq!(cred.key_id, "ak2");
        }

        vend.assert_async().await;
    }

    #[test]
    fn split_sas_decodes_pairs() {
        let pairs = split_sas("?sv=2021-08-06&sp=rl&&sig=ab%2Bcd%2Fef%3D").unwrap();
        assert_eq!(
            pairs,
            vec![
                ("sv".to_string(), "2021-08-06".to_string()),
                ("sp".to_string(), "rl".to_string()),
                ("sig".to_string(), "ab+cd/ef=".to_string()),
            ]
        );
    }

    #[test]
    fn split_sas_rejects_malformed_component() {
        assert!(split_sas("sv=2021-08-06&garbage").is_err());
    }

    /// The vended (URL-encoded) signature reaches Azure encoded exactly once.
    #[tokio::test]
    async fn azure_sas_signature_is_not_double_encoded() {
        use unitycatalog_credentials::model::AzureUserDelegationSas;

        let mut server = mockito::Server::new_async().await;
        // `UrlEncoded` matches the *decoded* value: a double-encoded `%252B`
        // would decode to `%2B` and miss.
        let get = server
            .mock("GET", mockito::Matcher::Any)
            .match_query(mockito::Matcher::UrlEncoded(
                "sig".to_string(),
                "ab+cd/ef=".to_string(),
            ))
            .with_status(404)
            .expect(1)
            .create_async()
            .await;

        let credential = TemporaryCredential {
            expiration_time: now_epoch_millis() + 3_600_000,
            url: "abfss://container@account.dfs.core.windows.net/prefix/".to_string(),
            credentials: Some(
                AzureUserDelegationSas {
                    sas_token: "sv=2021-08-06&sp=rl&sig=ab%2Bcd%2Fef%3D".to_string(),
                }
                .into(),
            ),
        };
        let token = Arc::try_unwrap(as_azure(&credential).unwrap().token).unwrap();
        let store = MicrosoftAzureBuilder::new()
            .with_account("account")
            .with_container_name("container")
            .with_endpoint(server.url())
            .with_allow_http(true)
            .with_retry(object_store::RetryConfig {
                max_retries: 0,
                ..Default::default()
            })
            .with_credentials(Arc::new(object_store::StaticCredentialProvider::new(token)))
            .build()
            .unwrap();

        let _ = store.get(&Path::from("prefix/file")).await;

        get.assert_async().await;
    }
}
