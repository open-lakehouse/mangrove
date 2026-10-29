//! The credential shapes this crate vends from and to.
//!
//! - [`StorageCredential`] is a registered, long-lived credential: what a host
//!   stores and hands to [`vend_credential`](crate::vending::vend_credential).
//! - [`TemporaryCredential`] is a short-lived credential downscoped to one
//!   storage location: what vending returns and what consumers build object
//!   stores or presigned URLs from.
//!
//! Both use the field names of the corresponding Unity Catalog REST models
//! (`credentials.v1.Credential` without its metadata fields, and
//! `temporary_credentials.v1.TemporaryCredential`) and deserialize their JSON,
//! so values persisted in that shape still load. The types themselves do not
//! depend on any catalog's models; hosts convert at their own boundary.

use serde::{Deserialize, Serialize};

/// A registered storage credential: the long-lived secret (or identity) that
/// temporary credentials are minted from.
///
/// Serialized as an object with exactly one of the keys `aws_iam_role`,
/// `azure_service_principal`, `azure_managed_identity` or `azure_storage_key`.
/// Other keys are ignored on input, so a full Unity Catalog `Credential` JSON
/// document deserializes too.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "StorageCredentialRepr", into = "StorageCredentialRepr")]
pub enum StorageCredential {
    /// An AWS IAM role, assumed via STS.
    AwsIamRole(AwsIamRole),
    /// An Azure service principal (client secret or federated token file).
    AzureServicePrincipal(AzureServicePrincipal),
    /// An Azure managed identity, available on Azure compute.
    AzureManagedIdentity(AzureManagedIdentity),
    /// An Azure storage account key.
    AzureStorageKey(AzureStorageKey),
}

/// An AWS IAM role to assume.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AwsIamRole {
    pub role_arn: String,
    /// The STS region (default `us-east-1`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    /// Static base credentials to assume the role with; the ambient AWS
    /// credentials are used when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_token: Option<String>,
}

/// An Azure service principal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AzureServicePrincipal {
    pub directory_id: String,
    pub application_id: String,
    #[serde(flatten)]
    pub secret: AzureClientSecret,
}

/// How an [`AzureServicePrincipal`] authenticates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AzureClientSecret {
    ClientSecret(String),
    FederatedTokenFile(String),
}

/// An Azure managed identity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AzureManagedIdentity {
    pub access_connector_id: String,
    /// The ARM resource id of a user-assigned identity; the system-assigned
    /// identity is used when unset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managed_identity_id: Option<String>,
}

/// An Azure storage account key.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AzureStorageKey {
    pub account_name: String,
    pub account_key: String,
}

/// The wire shape of [`StorageCredential`]: one optional field per kind.
#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
struct StorageCredentialRepr {
    #[serde(skip_serializing_if = "Option::is_none")]
    aws_iam_role: Option<AwsIamRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    azure_service_principal: Option<AzureServicePrincipal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    azure_managed_identity: Option<AzureManagedIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    azure_storage_key: Option<AzureStorageKey>,
}

impl TryFrom<StorageCredentialRepr> for StorageCredential {
    type Error = String;

    fn try_from(repr: StorageCredentialRepr) -> Result<Self, Self::Error> {
        let mut kinds = [
            repr.aws_iam_role.map(Self::AwsIamRole),
            repr.azure_service_principal
                .map(Self::AzureServicePrincipal),
            repr.azure_managed_identity.map(Self::AzureManagedIdentity),
            repr.azure_storage_key.map(Self::AzureStorageKey),
        ]
        .into_iter()
        .flatten();
        match (kinds.next(), kinds.next()) {
            (Some(credential), None) => Ok(credential),
            (None, _) => Err("no supported storage credential type is set".to_string()),
            (Some(_), Some(_)) => Err("more than one storage credential type is set".to_string()),
        }
    }
}

impl From<StorageCredential> for StorageCredentialRepr {
    fn from(credential: StorageCredential) -> Self {
        let mut repr = Self::default();
        match credential {
            StorageCredential::AwsIamRole(c) => repr.aws_iam_role = Some(c),
            StorageCredential::AzureServicePrincipal(c) => repr.azure_service_principal = Some(c),
            StorageCredential::AzureManagedIdentity(c) => repr.azure_managed_identity = Some(c),
            StorageCredential::AzureStorageKey(c) => repr.azure_storage_key = Some(c),
        }
        repr
    }
}

/// A short-lived credential for one storage location.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TemporaryCredential {
    /// When the credential expires (ms since the epoch); non-positive when the
    /// issuer gave no expiry. Accepts a JSON number or, as proto JSON encodes
    /// `int64`, a string.
    #[serde(default, deserialize_with = "int64")]
    pub expiration_time: i64,
    /// The storage location the credential is scoped to.
    pub url: String,
    /// The credential material; `None` for locations that need none (`file://`).
    #[serde(flatten)]
    pub credentials: Option<VendedCredential>,
}

fn int64<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Int64 {
        Number(i64),
        String(String),
    }
    match Int64::deserialize(deserializer)? {
        Int64::Number(n) => Ok(n),
        Int64::String(s) => s.parse().map_err(serde::de::Error::custom),
    }
}

/// The material of a [`TemporaryCredential`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VendedCredential {
    AzureUserDelegationSas(AzureUserDelegationSas),
    AzureAad(AzureAad),
    AwsTempCredentials(AwsTemporaryCredentials),
    GcpOauthToken(GcpOauthToken),
    R2TempCredentials(R2TemporaryCredentials),
}

macro_rules! vended_from {
    ($($variant:ident($ty:ident)),* $(,)?) => {
        $(impl From<$ty> for VendedCredential {
            fn from(credential: $ty) -> Self {
                Self::$variant(credential)
            }
        })*
    };
}

vended_from!(
    AzureUserDelegationSas(AzureUserDelegationSas),
    AzureAad(AzureAad),
    AwsTempCredentials(AwsTemporaryCredentials),
    GcpOauthToken(GcpOauthToken),
    R2TempCredentials(R2TemporaryCredentials),
);

/// An Azure SAS token (user-delegation or service SAS).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AzureUserDelegationSas {
    pub sas_token: String,
}

/// An Azure AAD bearer token.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AzureAad {
    pub aad_token: String,
}

/// AWS STS session credentials.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AwsTemporaryCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
    pub access_point: String,
}

/// A GCP OAuth access token.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct GcpOauthToken {
    pub oauth_token: String,
}

/// Cloudflare R2 session credentials.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct R2TemporaryCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: String,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn storage_credential_reads_the_catalog_shape() {
        // A full catalog `Credential` document: metadata keys are ignored.
        let credential: StorageCredential = serde_json::from_value(json!({
            "name": "cred",
            "purpose": "STORAGE",
            "read_only": true,
            "azure_service_principal": {
                "directory_id": "tenant",
                "application_id": "app",
                "client_secret": "secret",
            },
        }))
        .unwrap();
        assert_eq!(
            credential,
            StorageCredential::AzureServicePrincipal(AzureServicePrincipal {
                directory_id: "tenant".into(),
                application_id: "app".into(),
                secret: AzureClientSecret::ClientSecret("secret".into()),
            })
        );
        let value = serde_json::to_value(&credential).unwrap();
        assert_eq!(
            value,
            json!({ "azure_service_principal": {
                "directory_id": "tenant",
                "application_id": "app",
                "client_secret": "secret",
            }})
        );
        assert_eq!(
            serde_json::from_value::<StorageCredential>(value).unwrap(),
            credential
        );
    }

    #[test]
    fn storage_credential_needs_exactly_one_kind() {
        assert!(serde_json::from_value::<StorageCredential>(json!({ "name": "x" })).is_err());
        assert!(
            serde_json::from_value::<StorageCredential>(json!({
                "aws_iam_role": { "role_arn": "arn" },
                "azure_storage_key": { "account_name": "a", "account_key": "k" },
            }))
            .is_err()
        );
    }

    #[test]
    fn temporary_credential_round_trips() {
        let value = json!({
            "expiration_time": 42,
            "url": "s3://bucket/prefix",
            "aws_temp_credentials": {
                "access_key_id": "id",
                "secret_access_key": "secret",
                "session_token": "token",
                "access_point": "",
            },
        });
        let credential: TemporaryCredential = serde_json::from_value(value.clone()).unwrap();
        assert!(matches!(
            credential.credentials,
            Some(VendedCredential::AwsTempCredentials(ref aws)) if aws.session_token == "token"
        ));
        assert_eq!(serde_json::to_value(&credential).unwrap(), value);

        // Proto JSON: `int64` as a string, empty strings omitted.
        let local: TemporaryCredential =
            serde_json::from_value(json!({ "url": "file:///tmp/t", "expiration_time": "1" }))
                .unwrap();
        assert_eq!(local.expiration_time, 1);
        assert_eq!(local.credentials, None);
    }
}
