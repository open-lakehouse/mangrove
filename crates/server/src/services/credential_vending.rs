use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use olai_http::StaticCredentialProvider;
use olai_http::TemporaryToken;
use olai_http::aws::{AwsCredential, AwsCredentialProvider};
use olai_http::azure::AzureCredential;
use unitycatalog_common::models::credentials::v1::{
    Credential, azure_service_principal::Credential as AzureSpCredential,
};
use unitycatalog_common::models::temporary_credentials::v1::{
    AwsTemporaryCredentials, AzureUserDelegationSas, TemporaryCredential,
    temporary_credential::Credentials,
};

use crate::services::location::{StorageLocationScheme, StorageLocationUrl};
use crate::{Error, Result};

/// Default credential TTL when the cloud provider does not supply an expiry.
const DEFAULT_TTL_SECS: u64 = 3600;

/// The operation requested by the caller — used to determine the minimum permissions
/// that should be encoded in the vended credential.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum VendOperation {
    /// Read-only access (s3:GetObject, SAS sp=rl, …).
    Read,
    /// Read and write access (s3:PutObject + s3:DeleteObject, SAS sp=racwdl, …).
    ReadWrite,
}

/// Convert an optional `Instant` expiry to epoch milliseconds.
fn expiry_to_epoch_millis(expiry: Option<Instant>) -> i64 {
    let ttl = match expiry {
        Some(exp) => exp
            .checked_duration_since(Instant::now())
            .unwrap_or_default(),
        None => Duration::from_secs(DEFAULT_TTL_SECS),
    };
    let wall = SystemTime::now() + ttl;
    wall.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// A credential-less [`TemporaryCredential`] for a local (`file://`) location.
///
/// Local storage carries no secret: the client builds an unrooted
/// `LocalFileSystem` addressed by full path (mirroring the server's
/// `get_local_store`). The response still carries the `url` and an expiry so the
/// shape matches a cloud vend, but `credentials` is `None` — the signal the
/// client uses to take its local-store branch.
pub(crate) fn local_path_credential(url: &str) -> TemporaryCredential {
    TemporaryCredential {
        expiration_time: expiry_to_epoch_millis(None),
        url: url.to_owned(),
        credentials: None,
        ..Default::default()
    }
}

fn aws_token_to_temporary_credential(
    url: &str,
    token: TemporaryToken<Arc<AwsCredential>>,
) -> TemporaryCredential {
    let expiration_time = expiry_to_epoch_millis(token.expiry);
    let cred = token.token.as_ref();
    TemporaryCredential {
        expiration_time,
        url: url.to_owned(),
        credentials: Some(Credentials::AwsTempCredentials(Box::new(
            AwsTemporaryCredentials {
                access_key_id: cred.key_id.clone(),
                secret_access_key: cred.secret_key.clone(),
                session_token: cred.token.clone().unwrap_or_default(),
                access_point: String::new(),
                ..Default::default()
            },
        ))),
        ..Default::default()
    }
}

fn azure_sas_to_temporary_credential(url: &str, sas_token: String) -> TemporaryCredential {
    // Compute expiry from the `se=` parameter if present; fall back to 1 h.
    let expiration_time =
        parse_sas_expiry(&sas_token).unwrap_or_else(|| expiry_to_epoch_millis(None));
    TemporaryCredential {
        expiration_time,
        url: url.to_owned(),
        credentials: Some(Credentials::AzureUserDelegationSas(Box::new(
            AzureUserDelegationSas {
                sas_token,
                ..Default::default()
            },
        ))),
        ..Default::default()
    }
}

/// Parse the `se=` (signed-expiry) field from a SAS query string and return epoch millis.
fn parse_sas_expiry(sas: &str) -> Option<i64> {
    for part in sas.split('&') {
        if let Some(encoded) = part.strip_prefix("se=") {
            let decoded = percent_encoding::percent_decode_str(encoded)
                .decode_utf8()
                .ok()?;
            let dt = chrono::DateTime::parse_from_rfc3339(&decoded).ok()?;
            return Some(dt.timestamp_millis());
        }
    }
    None
}

/// Build an AWS inline session policy scoped to `bucket` / `prefix` for the given operation.
///
/// The returned JSON string can be passed as the `Policy` parameter to `STS:AssumeRole`.
/// It is intersected with the role's own policy, so it can only restrict, never expand.
fn build_s3_session_policy(bucket: &str, prefix: &str, operation: VendOperation) -> String {
    let object_arn = if prefix.is_empty() {
        format!("arn:aws:s3:::{bucket}/*")
    } else {
        format!("arn:aws:s3:::{bucket}/{prefix}/*")
    };
    let bucket_arn = format!("arn:aws:s3:::{bucket}");

    let actions: &[&str] = match operation {
        VendOperation::Read => &[
            "s3:GetObject",
            "s3:GetObjectVersion",
            "s3:ListBucket",
            "s3:GetBucketLocation",
        ],
        VendOperation::ReadWrite => &[
            "s3:GetObject",
            "s3:GetObjectVersion",
            "s3:PutObject",
            "s3:DeleteObject",
            "s3:ListBucket",
            "s3:GetBucketLocation",
        ],
    };

    // Build via `serde_json` so `bucket`/`prefix` (which derive from
    // caller-influenced input) are JSON-escaped. Hand-formatting this document
    // would let a crafted prefix break out of the `Resource` array and broaden
    // the session policy — defeating the downscoping this is meant to enforce.
    serde_json::json!({
        "Version": "2012-10-17",
        "Statement": [{
            "Effect": "Allow",
            "Action": actions,
            "Resource": [object_arn, bucket_arn],
        }],
    })
    .to_string()
}

/// Re-vend this many milliseconds before a cached credential actually expires, so callers always
/// receive a credential with usable lifetime left.
const CACHE_REFRESH_MARGIN_MILLIS: i64 = 60_000;

/// Process-wide cache of vended credentials, keyed by the vend inputs.
///
/// Vending makes a synchronous HTTPS round-trip to a cloud provider (Azure AAD/Storage, AWS STS) on
/// every call; the resulting SAS/STS tokens carry an expiry, so identical requests within that
/// window can reuse the previous result instead of re-hitting the provider.
static VENDED_CACHE: OnceLock<Mutex<HashMap<u64, TemporaryCredential>>> = OnceLock::new();

fn vended_cache() -> &'static Mutex<HashMap<u64, TemporaryCredential>> {
    VENDED_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Cache key for a vend request: the credential's secret material plus the storage scope and
/// operation. Hashing the credential content means a rotated credential produces a different key,
/// so a stale token is never served after rotation.
fn vend_cache_key(credential: &Credential, url: &str, operation: VendOperation) -> Result<u64> {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    // The secret-bearing credential variants serialize deterministically enough for cache keying.
    serde_json::to_vec(&credential.azure_service_principal)
        .and_then(|b| {
            hasher.write(&b);
            serde_json::to_vec(&credential.azure_managed_identity)
        })
        .and_then(|b| {
            hasher.write(&b);
            serde_json::to_vec(&credential.azure_storage_key)
        })
        .and_then(|b| {
            hasher.write(&b);
            serde_json::to_vec(&credential.aws_iam_role)
        })
        .map_err(Error::from)?;
    url.hash(&mut hasher);
    (operation as u8).hash(&mut hasher);
    Ok(hasher.finish())
}

/// Current wall-clock time in epoch milliseconds.
fn now_epoch_millis() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

/// Generate a temporary credential for the given `Credential` and storage `url`,
/// downscoped to the requested `operation`.
///
/// Results are cached by `(credential material, url, operation)` until shortly before the vended
/// token expires, so repeated requests for the same scope avoid redundant calls to the cloud
/// provider. See [`vend_credential_uncached`] for the dispatch logic.
pub(crate) async fn vend_credential(
    credential: &Credential,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let key = vend_cache_key(credential, url, operation)?;

    if let Some(cached) = vended_cache()
        .lock()
        .unwrap()
        .get(&key)
        .filter(|c| c.expiration_time - CACHE_REFRESH_MARGIN_MILLIS > now_epoch_millis())
        .cloned()
    {
        return Ok(cached);
    }

    let vended = vend_credential_uncached(credential, url, operation).await?;
    vended_cache().lock().unwrap().insert(key, vended.clone());
    Ok(vended)
}

/// Dispatch to the appropriate cloud provider based on which credential field is populated:
/// - `AzureServicePrincipal` → fetch AAD bearer token, then exchange for a User Delegation SAS
/// - `AzureManagedIdentity`  → fetch AAD bearer token via IMDS, then User Delegation SAS
/// - `AzureStorageKey`       → service SAS signed with the account key
/// - `AwsIamRoleConfig`      → AWS STS `AssumeRole` with inline session policy
/// - GCP                     → not yet implemented
async fn vend_credential_uncached(
    credential: &Credential,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    if let Some(sp) = credential.azure_service_principal.as_option() {
        return vend_azure_service_principal(sp, url, operation).await;
    }
    if let Some(msi) = credential.azure_managed_identity.as_option() {
        return vend_azure_managed_identity(msi, url, operation).await;
    }
    if let Some(key) = credential.azure_storage_key.as_option() {
        return vend_azure_storage_key(key, url, operation).await;
    }
    if let Some(role) = credential.aws_iam_role.as_option() {
        return vend_aws_iam_role(role, url, operation).await;
    }
    if credential.databricks_gcp_service_account.is_set() {
        return Err(Error::generic(
            "GCP service account credential vending is not yet implemented.",
        ));
    }
    Err(Error::invalid_argument(
        "No supported credential type found on this credential object.",
    ))
}

async fn vend_azure_service_principal(
    sp: &unitycatalog_common::models::credentials::v1::AzureServicePrincipal,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let bearer_token = match &sp.credential {
        Some(AzureSpCredential::ClientSecret(secret)) => {
            let token = olai_http::azure::fetch_client_secret_token(
                &sp.directory_id,
                sp.application_id.clone(),
                secret.clone(),
                None,
            )
            .await?;
            extract_bearer_token(token.token.as_ref())?
        }
        Some(AzureSpCredential::FederatedTokenFile(token_file)) => {
            let token = olai_http::azure::fetch_workload_identity_token(
                &sp.directory_id,
                sp.application_id.clone(),
                token_file.clone(),
                None,
            )
            .await?;
            extract_bearer_token(token.token.as_ref())?
        }
        None => {
            return Err(Error::invalid_argument(
                "Azure service principal credential is missing client_secret or federated_token_file.",
            ));
        }
    };
    vend_azure_sas_from_bearer(url, &bearer_token, operation).await
}

/// Vend credentials via Azure Managed Identity (IMDS).
///
/// Unlike service-principal or storage-key credentials, managed identities carry
/// no static secret — authentication relies on the Azure Instance Metadata Service
/// (`http://169.254.169.254/metadata/identity/…`) which is only reachable from
/// within Azure compute (VMs, AKS pods, App Service, etc.).
///
/// If a user registers an `AzureManagedIdentity` credential they are declaring
/// that the Unity Catalog server itself runs on Azure compute with that identity
/// assigned. The server cannot fall back to a stored key here.
///
/// When `managed_identity_id` is set, it is the ARM resource ID of the
/// user-assigned identity and is passed as `msi_res_id` to IMDS. When absent,
/// the system-assigned identity of the Access Connector is used automatically.
async fn vend_azure_managed_identity(
    msi: &unitycatalog_common::models::credentials::v1::AzureManagedIdentity,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let msi_res_id = msi.managed_identity_id.clone();
    let token = olai_http::azure::fetch_managed_identity_token(None, None, msi_res_id).await?;
    let bearer_token = extract_bearer_token(token.token.as_ref())?;
    vend_azure_sas_from_bearer(url, &bearer_token, operation).await
}

async fn vend_azure_storage_key(
    key: &unitycatalog_common::models::credentials::v1::AzureStorageKey,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let storage_url = StorageLocationUrl::parse(url)?;
    let account = storage_url
        .azure_account()
        .or_else(|| {
            // For Azurite the account is encoded in the path; fall back to the key's account field.
            Some(key.account_name.clone())
        })
        .ok_or_else(|| {
            Error::invalid_argument("Cannot determine Azure storage account from URL")
        })?;
    let (container, prefix) = storage_url.bucket_and_prefix()?;
    let read_only = operation == VendOperation::Read;
    // Azurite is served over http and uses a flat namespace; the SAS must permit
    // http and omit `sdd=` (see olai-http::generate_storage_key_sas).
    let emulator = matches!(storage_url.scheme(), StorageLocationScheme::Azurite);
    let sas_token = olai_http::azure::generate_storage_key_sas(
        &account,
        &container,
        &prefix,
        &key.account_key,
        read_only,
        DEFAULT_TTL_SECS,
        emulator,
    )?;
    Ok(azure_sas_to_temporary_credential(url, sas_token))
}

/// Given an AAD bearer token, fetch a User Delegation Key and build a scoped SAS.
async fn vend_azure_sas_from_bearer(
    url: &str,
    bearer_token: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let storage_url = StorageLocationUrl::parse(url)?;
    let account = storage_url.azure_account().ok_or_else(|| {
        Error::invalid_argument("Cannot determine Azure storage account from URL")
    })?;
    let (container, prefix) = storage_url.bucket_and_prefix()?;
    let read_only = operation == VendOperation::Read;
    let sas_token = olai_http::azure::generate_user_delegation_sas(
        &account,
        &container,
        &prefix,
        bearer_token,
        read_only,
        DEFAULT_TTL_SECS,
    )
    .await?;
    Ok(azure_sas_to_temporary_credential(url, sas_token))
}

fn extract_bearer_token(credential: &AzureCredential) -> Result<String> {
    match credential {
        AzureCredential::BearerToken(t) => Ok(t.clone()),
    }
}

async fn vend_aws_iam_role(
    role: &unitycatalog_common::models::credentials::v1::AwsIamRoleConfig,
    url: &str,
    operation: VendOperation,
) -> Result<TemporaryCredential> {
    let region = role.region.as_deref().unwrap_or("us-east-1");
    let storage_url = StorageLocationUrl::parse(url)?;
    let (bucket, prefix) = storage_url.bucket_and_prefix()?;
    let policy = build_s3_session_policy(&bucket, &prefix, operation);

    // Build base credentials from the registered access key when present.
    // Falls back to the server's ambient credentials (instance profile, ECS
    // task role, WebIdentity, etc.) when no static key is registered.
    let base_credentials: AwsCredentialProvider =
        if let (Some(key_id), Some(secret_key)) = (&role.access_key_id, &role.secret_access_key) {
            Arc::new(StaticCredentialProvider::new(AwsCredential {
                key_id: key_id.clone(),
                secret_key: secret_key.clone(),
                token: role.session_token.clone(),
            }))
        } else {
            olai_http::aws::AmazonBuilder::from_env()
                .with_region(region)
                .build(None)?
                .credentials
        };

    let token = olai_http::aws::assume_role_with_base(
        &role.role_arn,
        region,
        None,
        Some(policy),
        base_credentials,
    )
    .await?;
    Ok(aws_token_to_temporary_credential(url, token))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[test]
    fn test_expiry_to_epoch_millis_with_expiry() {
        let future_expiry = Instant::now() + Duration::from_secs(3600);
        let millis = expiry_to_epoch_millis(Some(future_expiry));
        let now_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert!(millis > now_millis + 3_590_000, "expiry too soon: {millis}");
        assert!(millis < now_millis + 3_610_000, "expiry too far: {millis}");
    }

    #[test]
    fn test_expiry_to_epoch_millis_none_defaults_to_one_hour() {
        let millis = expiry_to_epoch_millis(None);
        let now_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64;
        assert!(millis > now_millis + 3_590_000, "expiry too soon: {millis}");
        assert!(millis < now_millis + 3_610_000, "expiry too far: {millis}");
    }

    #[test]
    fn test_aws_token_to_temporary_credential() {
        let token = TemporaryToken {
            token: Arc::new(AwsCredential {
                key_id: "AKIAIOSFODNN7EXAMPLE".to_string(), // gitleaks:allow
                secret_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(), // gitleaks:allow
                token: Some("session-token".to_string()),
            }),
            expiry: Some(Instant::now() + Duration::from_secs(3600)),
        };
        let cred = aws_token_to_temporary_credential("s3://my-bucket/path", token);
        assert_eq!(cred.url, "s3://my-bucket/path");
        assert!(cred.expiration_time > 0);
        match cred.credentials {
            Some(Credentials::AwsTempCredentials(aws)) => {
                assert_eq!(aws.access_key_id, "AKIAIOSFODNN7EXAMPLE"); // gitleaks:allow
                assert_eq!(aws.session_token, "session-token");
            }
            _ => panic!("expected AwsTempCredentials credential"),
        }
    }

    #[test]
    fn test_s3_session_policy_read_only() {
        let policy = build_s3_session_policy("my-bucket", "some/prefix", VendOperation::Read);
        assert!(policy.contains("s3:GetObject"), "missing GetObject");
        assert!(policy.contains("s3:ListBucket"), "missing ListBucket");
        assert!(
            !policy.contains("s3:PutObject"),
            "should not allow PutObject for read"
        );
        assert!(
            !policy.contains("s3:DeleteObject"),
            "should not allow DeleteObject for read"
        );
        assert!(
            policy.contains("arn:aws:s3:::my-bucket/some/prefix/*"),
            "missing object ARN"
        );
        assert!(
            policy.contains("arn:aws:s3:::my-bucket\""),
            "missing bucket ARN"
        );
    }

    #[test]
    fn test_s3_session_policy_read_write() {
        let policy = build_s3_session_policy("my-bucket", "data/", VendOperation::ReadWrite);
        assert!(policy.contains("s3:PutObject"), "missing PutObject");
        assert!(policy.contains("s3:DeleteObject"), "missing DeleteObject");
        assert!(
            policy.contains("arn:aws:s3:::my-bucket/data//*"),
            "missing object ARN"
        );
    }

    #[test]
    fn test_s3_session_policy_empty_prefix() {
        let policy = build_s3_session_policy("my-bucket", "", VendOperation::Read);
        assert!(
            policy.contains("arn:aws:s3:::my-bucket/*"),
            "missing wildcard ARN"
        );
    }

    #[test]
    fn test_s3_session_policy_escapes_injection_in_prefix() {
        // A prefix containing JSON metacharacters must not break out of the
        // document. The result must remain valid JSON with exactly one
        // statement (no injected Allow/Action entries).
        let malicious =
            r#"x"],"Resource":["*"}],"Statement":[{"Effect":"Allow","Action":["*"],"Resource":["*"#;
        let policy = build_s3_session_policy("my-bucket", malicious, VendOperation::Read);

        let parsed: serde_json::Value =
            serde_json::from_str(&policy).expect("session policy must be valid JSON");
        let statements = parsed["Statement"]
            .as_array()
            .expect("Statement must be an array");
        assert_eq!(
            statements.len(),
            1,
            "injected prefix must not add statements"
        );
        // The whole malicious string is escaped inside the single Resource ARN.
        let resources = statements[0]["Resource"]
            .as_array()
            .expect("Resource must be an array");
        assert!(
            resources[0]
                .as_str()
                .unwrap()
                .contains(&format!("my-bucket/{malicious}/*")),
            "prefix must be contained verbatim within the object ARN"
        );
    }

    #[test]
    fn test_parse_sas_expiry() {
        let sas = "sv=2020-12-06&se=2024-01-01T00%3A00%3A00Z&sp=rl&sig=abc";
        let millis = parse_sas_expiry(sas);
        assert!(millis.is_some(), "expected to parse expiry");
        assert!(millis.unwrap() > 0);
    }

    fn empty_credential() -> Credential {
        Credential::default()
    }

    #[test]
    fn test_vend_cache_key_is_sensitive_to_inputs() {
        let cred = empty_credential();
        let base = vend_cache_key(&cred, "s3://bucket/a", VendOperation::Read).unwrap();
        // Same inputs → same key.
        assert_eq!(
            base,
            vend_cache_key(&cred, "s3://bucket/a", VendOperation::Read).unwrap()
        );
        // URL, operation each change the key.
        assert_ne!(
            base,
            vend_cache_key(&cred, "s3://bucket/b", VendOperation::Read).unwrap()
        );
        assert_ne!(
            base,
            vend_cache_key(&cred, "s3://bucket/a", VendOperation::ReadWrite).unwrap()
        );
        // A different credential (rotated secret) changes the key.
        let mut rotated = empty_credential();
        rotated.azure_storage_key = Some(
            unitycatalog_common::models::credentials::v1::AzureStorageKey {
                account_name: "acct".into(),
                account_key: "rotated-secret".into(),
                ..Default::default()
            },
        )
        .into();
        assert_ne!(
            base,
            vend_cache_key(&rotated, "s3://bucket/a", VendOperation::Read).unwrap()
        );
    }

    #[tokio::test]
    async fn test_vend_credential_serves_unexpired_cache_entry() {
        // A credential with no supported type would error if dispatched, so a successful
        // result proves the cached entry short-circuited the (network) dispatch.
        let cred = empty_credential();
        let url = "s3://cache-hit-bucket/unique-prefix";
        let key = vend_cache_key(&cred, url, VendOperation::Read).unwrap();
        let cached = TemporaryCredential {
            expiration_time: now_epoch_millis() + 3_600_000,
            url: url.to_string(),
            credentials: None,
            ..Default::default()
        };
        vended_cache().lock().unwrap().insert(key, cached.clone());

        let got = vend_credential(&cred, url, VendOperation::Read)
            .await
            .unwrap();
        assert_eq!(got.expiration_time, cached.expiration_time);
    }

    #[tokio::test]
    async fn test_vend_credential_ignores_expired_cache_entry() {
        // An entry inside the refresh margin must be treated as a miss, so dispatch runs (and here
        // errors, since the credential has no supported type).
        let cred = empty_credential();
        let url = "s3://cache-expired-bucket/unique-prefix";
        let key = vend_cache_key(&cred, url, VendOperation::Read).unwrap();
        let expired = TemporaryCredential {
            // Already within the refresh margin of "now".
            expiration_time: now_epoch_millis() + 1_000,
            url: url.to_string(),
            credentials: None,
            ..Default::default()
        };
        vended_cache().lock().unwrap().insert(key, expired);

        assert!(
            vend_credential(&cred, url, VendOperation::Read)
                .await
                .is_err(),
            "expired entry should fall through to dispatch"
        );
    }
}
