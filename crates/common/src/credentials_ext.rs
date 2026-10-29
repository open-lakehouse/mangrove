//! Conversions between the generated credential models and the
//! catalog-independent types of `olai-uc-credentials`.
//!
//! Vending and the object-store primitives speak
//! [`unitycatalog_credentials::model`]; the Unity Catalog REST surface speaks the
//! generated [`Credential`] / [`TemporaryCredential`]. A UC host converts at its
//! boundary with these impls.

use unitycatalog_credentials::model as native;

use crate::models::credentials::v1::{
    AwsIamRoleConfig, AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey, Credential,
    azure_service_principal::Credential as AzureSpCredential,
};
use crate::models::temporary_credentials::v1::{
    AwsTemporaryCredentials, AzureAad, AzureUserDelegationSas, GcpOauthToken,
    R2TemporaryCredentials, TemporaryCredential, temporary_credential::Credentials,
};
use crate::{Error, Result};

impl TryFrom<&Credential> for native::StorageCredential {
    type Error = Error;

    fn try_from(credential: &Credential) -> Result<Self> {
        if let Some(sp) = credential.azure_service_principal.as_option() {
            return Ok(Self::AzureServicePrincipal(sp.try_into()?));
        }
        if let Some(msi) = credential.azure_managed_identity.as_option() {
            return Ok(Self::AzureManagedIdentity(native::AzureManagedIdentity {
                access_connector_id: msi.access_connector_id.clone(),
                managed_identity_id: msi.managed_identity_id.clone(),
            }));
        }
        if let Some(key) = credential.azure_storage_key.as_option() {
            return Ok(Self::AzureStorageKey(native::AzureStorageKey {
                account_name: key.account_name.clone(),
                account_key: key.account_key.clone(),
            }));
        }
        if let Some(role) = credential.aws_iam_role.as_option() {
            return Ok(Self::AwsIamRole(native::AwsIamRole {
                role_arn: role.role_arn.clone(),
                region: role.region.clone(),
                access_key_id: role.access_key_id.clone(),
                secret_access_key: role.secret_access_key.clone(),
                session_token: role.session_token.clone(),
            }));
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
}

impl TryFrom<&AzureServicePrincipal> for native::AzureServicePrincipal {
    type Error = Error;

    fn try_from(sp: &AzureServicePrincipal) -> Result<Self> {
        let secret = match &sp.credential {
            Some(AzureSpCredential::ClientSecret(secret)) => {
                native::AzureClientSecret::ClientSecret(secret.clone())
            }
            Some(AzureSpCredential::FederatedTokenFile(file)) => {
                native::AzureClientSecret::FederatedTokenFile(file.clone())
            }
            None => {
                return Err(Error::invalid_argument(
                    "Azure service principal credential is missing client_secret or federated_token_file.",
                ));
            }
        };
        Ok(Self {
            directory_id: sp.directory_id.clone(),
            application_id: sp.application_id.clone(),
            secret,
        })
    }
}

/// Fill the credential kind of a [`Credential`] from a native storage credential.
/// Metadata (name, purpose, …) is left untouched.
pub fn set_storage_credential(credential: &mut Credential, storage: native::StorageCredential) {
    match storage {
        native::StorageCredential::AwsIamRole(role) => {
            credential.aws_iam_role = Some(AwsIamRoleConfig {
                role_arn: role.role_arn,
                region: role.region,
                access_key_id: role.access_key_id,
                secret_access_key: role.secret_access_key,
                session_token: role.session_token,
                ..Default::default()
            })
            .into();
        }
        native::StorageCredential::AzureServicePrincipal(sp) => {
            credential.azure_service_principal = Some(AzureServicePrincipal {
                directory_id: sp.directory_id,
                application_id: sp.application_id,
                credential: Some(match sp.secret {
                    native::AzureClientSecret::ClientSecret(s) => {
                        AzureSpCredential::ClientSecret(s)
                    }
                    native::AzureClientSecret::FederatedTokenFile(f) => {
                        AzureSpCredential::FederatedTokenFile(f)
                    }
                }),
                ..Default::default()
            })
            .into();
        }
        native::StorageCredential::AzureManagedIdentity(msi) => {
            credential.azure_managed_identity = Some(AzureManagedIdentity {
                access_connector_id: msi.access_connector_id,
                managed_identity_id: msi.managed_identity_id,
                ..Default::default()
            })
            .into();
        }
        native::StorageCredential::AzureStorageKey(key) => {
            credential.azure_storage_key = Some(AzureStorageKey {
                account_name: key.account_name,
                account_key: key.account_key,
                ..Default::default()
            })
            .into();
        }
    }
}

impl From<TemporaryCredential> for native::TemporaryCredential {
    fn from(credential: TemporaryCredential) -> Self {
        use native::VendedCredential as V;
        Self {
            expiration_time: credential.expiration_time,
            url: credential.url,
            credentials: credential.credentials.map(|c| match c {
                Credentials::AzureUserDelegationSas(sas) => {
                    V::AzureUserDelegationSas(native::AzureUserDelegationSas {
                        sas_token: sas.sas_token,
                    })
                }
                Credentials::AzureAad(aad) => V::AzureAad(native::AzureAad {
                    aad_token: aad.aad_token,
                }),
                Credentials::AwsTempCredentials(aws) => {
                    V::AwsTempCredentials(native::AwsTemporaryCredentials {
                        access_key_id: aws.access_key_id,
                        secret_access_key: aws.secret_access_key,
                        session_token: aws.session_token,
                        access_point: aws.access_point,
                    })
                }
                Credentials::GcpOauthToken(gcp) => V::GcpOauthToken(native::GcpOauthToken {
                    oauth_token: gcp.oauth_token,
                }),
                Credentials::R2TempCredentials(r2) => {
                    V::R2TempCredentials(native::R2TemporaryCredentials {
                        access_key_id: r2.access_key_id,
                        secret_access_key: r2.secret_access_key,
                        session_token: r2.session_token,
                    })
                }
            }),
        }
    }
}

impl From<native::TemporaryCredential> for TemporaryCredential {
    fn from(credential: native::TemporaryCredential) -> Self {
        use native::VendedCredential as V;
        Self {
            expiration_time: credential.expiration_time,
            url: credential.url,
            credentials: credential.credentials.map(|c| match c {
                V::AzureUserDelegationSas(sas) => {
                    Credentials::AzureUserDelegationSas(Box::new(AzureUserDelegationSas {
                        sas_token: sas.sas_token,
                        ..Default::default()
                    }))
                }
                V::AzureAad(aad) => Credentials::AzureAad(Box::new(AzureAad {
                    aad_token: aad.aad_token,
                    ..Default::default()
                })),
                V::AwsTempCredentials(aws) => {
                    Credentials::AwsTempCredentials(Box::new(AwsTemporaryCredentials {
                        access_key_id: aws.access_key_id,
                        secret_access_key: aws.secret_access_key,
                        session_token: aws.session_token,
                        access_point: aws.access_point,
                        ..Default::default()
                    }))
                }
                V::GcpOauthToken(gcp) => Credentials::GcpOauthToken(Box::new(GcpOauthToken {
                    oauth_token: gcp.oauth_token,
                    ..Default::default()
                })),
                V::R2TempCredentials(r2) => {
                    Credentials::R2TempCredentials(Box::new(R2TemporaryCredentials {
                        access_key_id: r2.access_key_id,
                        secret_access_key: r2.secret_access_key,
                        session_token: r2.session_token,
                        ..Default::default()
                    }))
                }
            }),
            ..Default::default()
        }
    }
}

impl From<unitycatalog_credentials::Error> for Error {
    fn from(e: unitycatalog_credentials::Error) -> Self {
        use unitycatalog_credentials::Error as E;
        match e {
            E::NotFound => Error::NotFound,
            E::InvalidArgument(msg) => Error::InvalidArgument(msg),
            E::Generic(msg) => Error::Generic(msg),
            E::Serde(source) => source.into(),
            #[allow(unreachable_patterns)]
            other => Error::Generic(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_credentials_convert_both_ways() {
        let storage =
            native::StorageCredential::AzureServicePrincipal(native::AzureServicePrincipal {
                directory_id: "tenant".into(),
                application_id: "app".into(),
                secret: native::AzureClientSecret::FederatedTokenFile("/token".into()),
            });
        let mut credential = Credential {
            name: "cred".into(),
            ..Default::default()
        };
        set_storage_credential(&mut credential, storage.clone());
        assert_eq!(
            native::StorageCredential::try_from(&credential).unwrap(),
            storage
        );
        // The JSON shapes agree, so values sealed in either form open as the other.
        let json = serde_json::to_value(&credential).unwrap();
        assert_eq!(
            serde_json::from_value::<native::StorageCredential>(json).unwrap(),
            storage
        );
        assert!(native::StorageCredential::try_from(&Credential::default()).is_err());
    }

    #[test]
    fn temporary_credentials_convert_both_ways() {
        let vended = native::TemporaryCredential {
            expiration_time: 42,
            url: "s3://bucket/prefix".into(),
            credentials: Some(native::VendedCredential::AwsTempCredentials(
                native::AwsTemporaryCredentials {
                    access_key_id: "id".into(),
                    secret_access_key: "secret".into(),
                    session_token: "token".into(),
                    access_point: String::new(),
                },
            )),
        };
        let uc = TemporaryCredential::from(vended.clone());
        // The native type reads the generated model's (proto) JSON.
        let json = serde_json::to_value(&uc).unwrap();
        assert_eq!(
            serde_json::from_value::<native::TemporaryCredential>(json).unwrap(),
            vended
        );
        assert_eq!(native::TemporaryCredential::from(uc), vended);
    }
}
