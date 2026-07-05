//! Vended-credential mapping: turn a Unity Catalog [`TemporaryCredential`] and a
//! cloud storage location into something the browser can fetch directly.
//!
//! The browser cannot run cloud SDK request signing; it can only attach what a
//! plain `fetch` carries — a query string or request headers. That restricts v1
//! to credential shapes that ARE a query string or header (matching the
//! signed-URL/proxy posture in `WASM_QUERY_PREVIEW.md`):
//!
//! - **Azure SAS** → the SAS token is appended as the query string of the
//!   mapped `https://…blob.core.windows.net/…` URL (a signed URL; the happy path).
//! - **Azure AAD** → `Authorization: Bearer` plus the `x-ms-version` header the
//!   Blob REST API requires for OAuth.
//! - **GCP OAuth** → `Authorization: Bearer` against
//!   `https://storage.googleapis.com/<bucket>/…`.
//! - **AWS / R2** need SigV4 request signing — unsupported until Phase C; the
//!   caller falls back to another runner.
//!
//! The wire DTOs are a minimal hand-rolled projection of
//! `unitycatalog.temporary_credentials.v1.TemporaryCredential` (the pbjson JSON
//! shape, camelCase with snake_case tolerated) — the generated prost models in
//! `unitycatalog-common` are not wasm-lean, and this crate is a standalone
//! workspace anyway.

use serde::Deserialize;
use url::Url;

use crate::error::{Error, Result};

/// Minimal projection of the `TemporaryCredential` response message.
#[derive(Debug, Clone, Deserialize)]
pub struct TemporaryCredential {
    /// The storage location this credential grants access to. Some servers omit
    /// it for table credentials; the caller then falls back to the table's
    /// storage location from `loadTable`.
    #[serde(default)]
    pub url: Option<String>,
    /// One-of credential payload; exactly one is set by a conforming server.
    #[serde(flatten)]
    pub credentials: CredentialPayload,
}

/// The `credentials` oneof, flattened the way pbjson serializes it: the active
/// variant appears as a sibling key of `url`/`expirationTime`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CredentialPayload {
    #[serde(default, alias = "azure_user_delegation_sas")]
    pub azure_user_delegation_sas: Option<AzureUserDelegationSas>,
    #[serde(default, alias = "azure_aad")]
    pub azure_aad: Option<AzureAad>,
    #[serde(default, alias = "gcp_oauth_token")]
    pub gcp_oauth_token: Option<GcpOauthToken>,
    #[serde(default, alias = "aws_temp_credentials")]
    pub aws_temp_credentials: Option<serde_json::Value>,
    #[serde(default, alias = "r2_temp_credentials")]
    pub r2_temp_credentials: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureUserDelegationSas {
    #[serde(alias = "sas_token")]
    pub sas_token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AzureAad {
    #[serde(alias = "aad_token")]
    pub aad_token: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GcpOauthToken {
    #[serde(alias = "oauth_token")]
    pub oauth_token: String,
}

/// A storage location the browser can fetch: an `http(s)` table URL (query
/// string carries a SAS token when present) plus any static request headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedStorage {
    /// The table root as a directly fetchable URL. The query string (if any) is
    /// a credential and must be preserved on every request under this base.
    pub table_url: Url,
    /// Static headers to attach to every storage request (e.g. `authorization`).
    pub headers: Vec<(String, String)>,
}

/// The Blob REST API version sent alongside `Authorization: Bearer` — OAuth
/// requests require an `x-ms-version` of 2017-11-09 or newer.
const AZURE_MS_VERSION: &str = "2021-08-06";

/// Map a table storage `location` plus its vended `credential` to a
/// browser-fetchable [`ResolvedStorage`].
///
/// Unsupported cloud/credential combinations return [`Error::Unsupported`] with
/// the exact gate that fired, so the caller can fall back cleanly.
pub fn resolve_storage(
    location: &str,
    credential: &TemporaryCredential,
) -> Result<ResolvedStorage> {
    let location = credential
        .url
        .as_deref()
        .filter(|u| !u.is_empty())
        .unwrap_or(location);
    let url = Url::parse(location).map_err(|e| Error::InvalidUrl(format!("{location}: {e}")))?;

    match url.scheme() {
        "abfss" | "abfs" | "az" | "wasbs" | "wasb" => azure_storage(&url, credential),
        "gs" => gcp_storage(&url, credential),
        // Already-HTTP locations (e.g. Azurite path-style endpoints) pass
        // through; an Azure SAS still applies as the query string.
        "http" | "https" => http_storage(url, credential),
        "azurite" => azurite_storage(&url, credential),
        "s3" | "s3a" | "r2" => Err(Error::unsupported(format!(
            "{} storage needs SigV4 request signing, which the in-browser engine \
             does not do yet (Phase C)",
            url.scheme()
        ))),
        other => Err(Error::unsupported(format!(
            "unrecognized storage scheme `{other}`"
        ))),
    }
}

/// `abfss://<container>@<account>.dfs.core.windows.net/<path>` →
/// `https://<account>.blob.core.windows.net/<container>/<path>`.
///
/// The `dfs` host segment swaps to `blob` (works for sovereign-cloud suffixes
/// too), because SAS-authorized ranged GETs are a Blob-endpoint operation.
fn azure_storage(url: &Url, credential: &TemporaryCredential) -> Result<ResolvedStorage> {
    let container = if url.username().is_empty() {
        return Err(Error::InvalidUrl(format!(
            "azure location has no container (expected <container>@<account-host>): {url}"
        )));
    } else {
        url.username().to_owned()
    };
    let host = url
        .host_str()
        .ok_or_else(|| Error::InvalidUrl(format!("azure location has no host: {url}")))?;
    let blob_host = host.replacen(".dfs.", ".blob.", 1);

    let mut https = Url::parse(&format!("https://{blob_host}"))
        .map_err(|e| Error::InvalidUrl(format!("{blob_host}: {e}")))?;
    {
        let mut segments = https
            .path_segments_mut()
            .map_err(|_| Error::InvalidUrl(format!("cannot-be-a-base azure url: {url}")))?;
        segments.push(&container);
        segments.extend(url.path().split('/').filter(|s| !s.is_empty()));
    }
    apply_azure_credential(https, credential)
}

/// Attach an Azure credential to an already-HTTPS blob URL: SAS as the query
/// string, or AAD as a bearer header.
fn apply_azure_credential(
    mut url: Url,
    credential: &TemporaryCredential,
) -> Result<ResolvedStorage> {
    if let Some(sas) = &credential.credentials.azure_user_delegation_sas {
        url.set_query(Some(sas.sas_token.trim_start_matches('?')));
        return Ok(ResolvedStorage {
            table_url: url,
            headers: Vec::new(),
        });
    }
    if let Some(aad) = &credential.credentials.azure_aad {
        return Ok(ResolvedStorage {
            table_url: url,
            headers: vec![
                ("authorization".into(), format!("Bearer {}", aad.aad_token)),
                ("x-ms-version".into(), AZURE_MS_VERSION.into()),
            ],
        });
    }
    Err(Error::unsupported(
        "azure storage location vended a non-Azure credential (expected SAS or AAD)".to_string(),
    ))
}

/// `gs://<bucket>/<path>` → `https://storage.googleapis.com/<bucket>/<path>`
/// with the vended OAuth token as a bearer header.
fn gcp_storage(url: &Url, credential: &TemporaryCredential) -> Result<ResolvedStorage> {
    let token = credential
        .credentials
        .gcp_oauth_token
        .as_ref()
        .ok_or_else(|| {
            Error::unsupported("gcs storage location vended a non-GCP credential".to_string())
        })?;
    let bucket = url
        .host_str()
        .ok_or_else(|| Error::InvalidUrl(format!("gs location has no bucket: {url}")))?;

    let mut https = Url::parse("https://storage.googleapis.com").expect("static url parses");
    {
        let mut segments = https.path_segments_mut().expect("https is a base");
        segments.push(bucket);
        segments.extend(url.path().split('/').filter(|s| !s.is_empty()));
    }
    Ok(ResolvedStorage {
        table_url: https,
        headers: vec![(
            "authorization".into(),
            format!("Bearer {}", token.oauth_token),
        )],
    })
}

/// Pass an `http(s)` location through, attaching an Azure credential when one
/// was vended (the Azurite emulator vends SAS for path-style localhost URLs).
fn http_storage(url: Url, credential: &TemporaryCredential) -> Result<ResolvedStorage> {
    if credential.credentials.azure_user_delegation_sas.is_some()
        || credential.credentials.azure_aad.is_some()
    {
        return apply_azure_credential(url, credential);
    }
    if let Some(token) = &credential.credentials.gcp_oauth_token {
        return Ok(ResolvedStorage {
            headers: vec![(
                "authorization".into(),
                format!("Bearer {}", token.oauth_token),
            )],
            table_url: url,
        });
    }
    if credential.credentials.aws_temp_credentials.is_some()
        || credential.credentials.r2_temp_credentials.is_some()
    {
        return Err(Error::unsupported(
            "http(s) storage location vended AWS/R2 credentials, which need SigV4 (Phase C)"
                .to_string(),
        ));
    }
    // No credential payload at all: a publicly readable location.
    Ok(ResolvedStorage {
        table_url: url,
        headers: Vec::new(),
    })
}

/// `azurite://<container>/<path>` → the default local emulator endpoint
/// `http://127.0.0.1:10000/devstoreaccount1/<container>/<path>` (path-style;
/// mirrors `crates/object-store`'s `parse_azurite`).
fn azurite_storage(url: &Url, credential: &TemporaryCredential) -> Result<ResolvedStorage> {
    const EMULATOR_ACCOUNT: &str = "devstoreaccount1";
    let container = url
        .host_str()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| Error::InvalidUrl(format!("azurite location has no container: {url}")))?;

    let mut http = Url::parse("http://127.0.0.1:10000").expect("static url parses");
    {
        let mut segments = http.path_segments_mut().expect("http is a base");
        segments.push(EMULATOR_ACCOUNT);
        segments.push(container);
        segments.extend(url.path().split('/').filter(|s| !s.is_empty()));
    }
    apply_azure_credential(http, credential)
}

// Native-only: unit tests never run on wasm32 (no test runner without
// wasm-bindgen-test), and the async ones need tokio.
#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    fn sas_credential(url: Option<&str>) -> TemporaryCredential {
        TemporaryCredential {
            url: url.map(str::to_owned),
            credentials: CredentialPayload {
                azure_user_delegation_sas: Some(AzureUserDelegationSas {
                    sas_token: "sv=2021-08-06&sp=rl&sig=AAAA".into(),
                }),
                ..CredentialPayload::default()
            },
        }
    }

    #[test]
    fn abfss_maps_to_blob_endpoint_with_sas_query() {
        let resolved = resolve_storage(
            "abfss://data@acct.dfs.core.windows.net/tables/t1",
            &sas_credential(None),
        )
        .unwrap();
        assert_eq!(
            resolved.table_url.as_str(),
            "https://acct.blob.core.windows.net/data/tables/t1?sv=2021-08-06&sp=rl&sig=AAAA"
        );
        assert!(resolved.headers.is_empty());
    }

    #[test]
    fn credential_url_overrides_location() {
        // The vended credential may carry a narrower (or corrected) URL; it wins.
        let resolved = resolve_storage(
            "abfss://data@acct.dfs.core.windows.net/tables/t1",
            &sas_credential(Some("abfss://other@acct.dfs.core.windows.net/x")),
        )
        .unwrap();
        assert_eq!(
            resolved.table_url.as_str(),
            "https://acct.blob.core.windows.net/other/x?sv=2021-08-06&sp=rl&sig=AAAA"
        );
    }

    #[test]
    fn azure_aad_maps_to_bearer_with_ms_version() {
        let credential = TemporaryCredential {
            url: None,
            credentials: CredentialPayload {
                azure_aad: Some(AzureAad {
                    aad_token: "tok".into(),
                }),
                ..CredentialPayload::default()
            },
        };
        let resolved =
            resolve_storage("az://data@acct.blob.core.windows.net/t", &credential).unwrap();
        assert_eq!(
            resolved.table_url.as_str(),
            "https://acct.blob.core.windows.net/data/t"
        );
        assert_eq!(resolved.headers[0].0, "authorization");
        assert_eq!(resolved.headers[0].1, "Bearer tok");
        assert_eq!(resolved.headers[1].0, "x-ms-version");
    }

    #[test]
    fn gs_maps_to_googleapis_with_bearer() {
        let credential = TemporaryCredential {
            url: None,
            credentials: CredentialPayload {
                gcp_oauth_token: Some(GcpOauthToken {
                    oauth_token: "gtok".into(),
                }),
                ..CredentialPayload::default()
            },
        };
        let resolved = resolve_storage("gs://bucket/tables/t1", &credential).unwrap();
        assert_eq!(
            resolved.table_url.as_str(),
            "https://storage.googleapis.com/bucket/tables/t1"
        );
        assert_eq!(resolved.headers[0].1, "Bearer gtok");
    }

    #[test]
    fn azurite_path_style_passthrough_keeps_sas() {
        let resolved = resolve_storage(
            "http://127.0.0.1:10000/devstoreaccount1/data/tables/t1",
            &sas_credential(None),
        )
        .unwrap();
        assert_eq!(
            resolved.table_url.as_str(),
            "http://127.0.0.1:10000/devstoreaccount1/data/tables/t1?sv=2021-08-06&sp=rl&sig=AAAA"
        );
    }

    #[test]
    fn azurite_scheme_maps_to_default_emulator_endpoint() {
        let resolved = resolve_storage("azurite://data/tables/t1", &sas_credential(None)).unwrap();
        assert_eq!(
            resolved.table_url.as_str(),
            "http://127.0.0.1:10000/devstoreaccount1/data/tables/t1?sv=2021-08-06&sp=rl&sig=AAAA"
        );
    }

    #[test]
    fn aws_is_unsupported_not_an_error() {
        let credential = TemporaryCredential {
            url: None,
            credentials: CredentialPayload::default(),
        };
        let err = resolve_storage("s3://bucket/t", &credential).unwrap_err();
        assert!(err.is_unsupported(), "{err}");
    }

    #[test]
    fn wire_json_parses_camel_and_snake() {
        // pbjson emits camelCase; some servers emit snake_case. Both parse.
        let camel: TemporaryCredential = serde_json::from_str(
            r#"{"url":"abfss://c@a.dfs.core.windows.net/t","expirationTime":1,
                "azureUserDelegationSas":{"sasToken":"sig=x"}}"#,
        )
        .unwrap();
        assert_eq!(
            camel
                .credentials
                .azure_user_delegation_sas
                .unwrap()
                .sas_token,
            "sig=x"
        );

        let snake: TemporaryCredential =
            serde_json::from_str(r#"{"url":"gs://b/t","gcp_oauth_token":{"oauth_token":"g"}}"#)
                .unwrap();
        assert_eq!(snake.credentials.gcp_oauth_token.unwrap().oauth_token, "g");
    }
}
