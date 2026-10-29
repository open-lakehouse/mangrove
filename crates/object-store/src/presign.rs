//! Presigned read URLs derived from a vended credential.
//!
//! A sharing server hands recipients URLs they can read without any credential
//! of their own. Given a [`TemporaryCredential`] covering a location (vended by a
//! catalog, or minted locally), [`presign_from_vended_credential`] turns the
//! storage URLs of files under it into such URLs:
//!
//! - **S3 / S3-compatible** (incl. R2): SigV4 query-signed `GET` URLs, signed with
//!   the vended (session) credentials;
//! - **Azure / Azurite**: the vended SAS (already scoped to the location) appended
//!   to each file's blob endpoint URL;
//! - **local** (`file://`): returned unchanged.
//!
//! A presigned URL can never outlive the credential it is derived from, so the
//! returned expiry is the earlier of the requested lifetime and the credential's
//! own expiration. GCS OAuth tokens and Azure AAD tokens cannot sign URLs; for
//! those this returns an error, and callers should vend credentials instead.

use std::time::Duration;

use object_store::Result;
use object_store::path::Path;
use object_store::signer::Signer;
use unitycatalog_common::temporary_credentials::v1::TemporaryCredential;
use unitycatalog_common::temporary_credentials::v1::temporary_credential::Credentials;
use url::Url;

use crate::credential::{as_aws, aws_access_point};
use crate::{Error, parse_azurite, s3_builder};

/// Presigned URLs for a batch of files, all valid until the same instant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresignedUrls {
    /// One readable URL per input file, in input order.
    pub urls: Vec<Url>,
    /// When the URLs stop being valid, in milliseconds since the Unix epoch.
    /// `None` for URLs that do not expire (local files).
    pub expiration_time: Option<i64>,
}

/// Presign `files` (absolute storage URLs under `credential.url`) for reading,
/// valid for at most `ttl`. See the module docs.
///
/// `aws_region` overrides the S3 region used for signing (else `AWS_REGION`, else
/// the object_store default), as for [`store_from_vended_credential`](crate::store_from_vended_credential).
pub async fn presign_from_vended_credential(
    credential: &TemporaryCredential,
    files: &[Url],
    ttl: Duration,
    aws_region: Option<String>,
) -> Result<PresignedUrls> {
    let location = Url::parse(&credential.url).map_err(Error::from)?;
    if location.scheme() == "file" {
        return Ok(PresignedUrls {
            urls: files.to_vec(),
            expiration_time: None,
        });
    }

    let now = chrono::Utc::now().timestamp_millis();
    let requested = now.saturating_add(i64::try_from(ttl.as_millis()).unwrap_or(i64::MAX));
    // A non-positive expiry means the catalog vended no expiry (see `get_expiry`).
    let expiration_time = if credential.expiration_time > 0 {
        requested.min(credential.expiration_time)
    } else {
        requested
    };

    let urls = match credential.credentials.as_ref() {
        Some(Credentials::AwsTempCredentials(_)) => {
            let token = as_aws(credential)?.token;
            let mut builder = s3_builder(&location, aws_region, aws_access_point(credential))?
                .with_access_key_id(&token.key_id)
                .with_secret_access_key(&token.secret_key);
            if let Some(session) = &token.token {
                builder = builder.with_token(session);
            }
            let store = builder.build()?;
            let paths = files
                .iter()
                .map(|file| Path::from_url_path(file.path()))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let lifetime =
                Duration::from_millis(u64::try_from(expiration_time - now).unwrap_or_default());
            store
                .signed_urls(reqwest::Method::GET, &paths, lifetime)
                .await?
        }
        Some(Credentials::AzureUserDelegationSas(sas)) => files
            .iter()
            .map(|file| azure_sas_url(file, &sas.sas_token))
            .collect::<Result<Vec<_>>>()?,
        Some(_) => {
            return Err(Error::InvalidCredential(
                "this credential type cannot presign URLs; vend credentials instead".to_string(),
            )
            .into());
        }
        None => return Err(Error::NoCredential.into()),
    };

    Ok(PresignedUrls {
        urls,
        expiration_time: Some(expiration_time),
    })
}

/// The blob endpoint URL of an Azure file with `sas` appended.
///
/// - `abfs[s]://<container>@<account>.dfs.core.windows.net/<path>` and
///   `az://…` spellings map to `https://<account>.blob.core.windows.net/<container>/<path>`;
/// - `https://<account>.blob.core.windows.net/…` is used as-is;
/// - Azurite (`http://127.0.0.1:10000/<account>/<container>/…` or
///   `azurite://<container>/…`) maps to the local emulator endpoint.
fn azure_sas_url(file: &Url, sas: &str) -> Result<Url> {
    let mut url = if let Some(loc) = parse_azurite(file) {
        Url::parse(&format!(
            "http://127.0.0.1:10000/{}/{}/{}",
            loc.account, loc.container, loc.prefix
        ))
        .map_err(Error::from)?
    } else {
        match file.scheme() {
            "https" => file.clone(),
            "abfs" | "abfss" | "az" | "adl" => {
                let host = file.host_str().unwrap_or_default();
                let (container, account) = match (file.username(), host.split_once('.')) {
                    // abfss://<container>@<account>.dfs.core.windows.net/<path>
                    (container, Some((account, _))) if !container.is_empty() => {
                        (container.to_string(), account.to_string())
                    }
                    _ => {
                        return Err(Error::invalid_url(format!(
                            "expected abfss://<container>@<account>.dfs.core.windows.net/..., got {file}"
                        ))
                        .into());
                    }
                };
                Url::parse(&format!(
                    "https://{account}.blob.core.windows.net/{container}{}",
                    file.path()
                ))
                .map_err(Error::from)?
            }
            other => {
                return Err(Error::invalid_url(format!(
                    "not an Azure storage URL (scheme {other}): {file}"
                ))
                .into());
            }
        }
    };
    url.set_query(Some(sas.trim_start_matches('?')));
    Ok(url)
}

#[cfg(test)]
mod tests {
    use unitycatalog_common::temporary_credentials::v1::{
        AwsTemporaryCredentials, AzureUserDelegationSas, GcpOauthToken,
    };

    use super::*;

    fn credential(
        url: &str,
        credentials: Option<Credentials>,
        expiration_time: i64,
    ) -> TemporaryCredential {
        TemporaryCredential {
            url: url.to_string(),
            credentials,
            expiration_time,
            ..Default::default()
        }
    }

    fn far_future() -> i64 {
        chrono::Utc::now().timestamp_millis() + 24 * 3600 * 1000
    }

    #[tokio::test]
    async fn local_files_are_returned_unchanged() {
        let cred = credential("file:///tmp/table/", None, 0);
        let files = vec![Url::parse("file:///tmp/table/part-0.parquet").unwrap()];
        let signed = presign_from_vended_credential(&cred, &files, Duration::from_secs(60), None)
            .await
            .unwrap();
        assert_eq!(signed.urls, files);
        assert_eq!(signed.expiration_time, None);
    }

    #[tokio::test]
    async fn s3_urls_are_sigv4_signed_and_clamped_to_the_credential() {
        let expires = chrono::Utc::now().timestamp_millis() + 30_000;
        let cred = credential(
            "s3://bucket/tables/t1",
            Some(Credentials::AwsTempCredentials(Box::new(
                AwsTemporaryCredentials {
                    access_key_id: "AKIAEXAMPLE".to_string(), // gitleaks:allow
                    secret_access_key: "secret".to_string(),
                    session_token: "session".to_string(),
                    ..Default::default()
                },
            ))),
            expires,
        );
        let files = vec![Url::parse("s3://bucket/tables/t1/letter=a/part-0.parquet").unwrap()];
        let signed = presign_from_vended_credential(
            &cred,
            &files,
            Duration::from_secs(3600),
            Some("us-east-1".to_string()),
        )
        .await
        .unwrap();
        let url = &signed.urls[0];
        // The key is percent-encoded in the URL (`=` → `%3D`); S3 decodes it back.
        let key = percent_encoding::percent_decode_str(url.path())
            .decode_utf8()
            .unwrap();
        assert!(key.ends_with("/tables/t1/letter=a/part-0.parquet"), "{url}");
        let query: std::collections::HashMap<_, _> = url.query_pairs().collect();
        assert_eq!(query["X-Amz-Algorithm"], "AWS4-HMAC-SHA256");
        assert_eq!(query["X-Amz-Security-Token"], "session");
        // The URL may not outlive the vended credential.
        assert_eq!(signed.expiration_time, Some(expires));
        assert!(query["X-Amz-Expires"].parse::<u64>().unwrap() <= 30);
    }

    #[tokio::test]
    async fn azure_urls_carry_the_vended_sas() {
        let sas = "sv=2022-11-02&sr=d&sp=rl&sig=abc";
        let cred = credential(
            "abfss://data@acct.dfs.core.windows.net/tables/t1",
            Some(Credentials::AzureUserDelegationSas(Box::new(
                AzureUserDelegationSas {
                    sas_token: sas.to_string(),
                    ..Default::default()
                },
            ))),
            far_future(),
        );
        let files = vec![
            Url::parse("abfss://data@acct.dfs.core.windows.net/tables/t1/part-0.parquet").unwrap(),
            Url::parse("http://127.0.0.1:10000/devstoreaccount1/data/tables/t1/part-1.parquet")
                .unwrap(),
        ];
        let signed = presign_from_vended_credential(&cred, &files, Duration::from_secs(60), None)
            .await
            .unwrap();
        assert_eq!(
            signed.urls[0].as_str(),
            format!("https://acct.blob.core.windows.net/data/tables/t1/part-0.parquet?{sas}")
        );
        assert_eq!(
            signed.urls[1].as_str(),
            format!("http://127.0.0.1:10000/devstoreaccount1/data/tables/t1/part-1.parquet?{sas}")
        );
    }

    #[tokio::test]
    async fn oauth_tokens_cannot_presign() {
        let cred = credential(
            "gs://bucket/t1",
            Some(Credentials::GcpOauthToken(Box::new(GcpOauthToken {
                oauth_token: "token".to_string(),
                ..Default::default()
            }))),
            far_future(),
        );
        let files = vec![Url::parse("gs://bucket/t1/part-0.parquet").unwrap()];
        assert!(
            presign_from_vended_credential(&cred, &files, Duration::from_secs(60), None)
                .await
                .is_err()
        );
    }
}
