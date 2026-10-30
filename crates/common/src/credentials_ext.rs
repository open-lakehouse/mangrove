//! Conversions between the generated credential models and the
//! catalog-independent credential vending types.
//!
//! Vending and the object-store primitives speak [`olai_storage_credentials`] (vended
//! credentials) and [`olai_credential_vending`] (registered credentials); the Unity
//! Catalog REST surface speaks the generated [`Credential`] / [`TemporaryCredential`].
//! A UC host converts at its boundary with these impls.
//!
//! The vended direction goes through the UC REST wire shape,
//! [`wire::unity::TemporaryCredential`], which has the same fields as the generated
//! model: `wire::unity::TemporaryCredential::from(&vended)` serializes a credential,
//! and `to_vended` reads one for a request.

#[cfg(feature = "credential-vending")]
use olai_credential_vending::registered as native;
use olai_storage_credentials::wire::{self, unity};

use crate::Error;
#[cfg(feature = "credential-vending")]
use crate::Result;
#[cfg(feature = "credential-vending")]
use crate::models::credentials::v1::{
    AwsIamRoleConfig, AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey, Credential,
    DatabricksGcpServiceAccount, azure_service_principal::Credential as AzureSpCredential,
};
use crate::models::temporary_credentials::v1::{
    AwsTemporaryCredentials, AzureAad, AzureUserDelegationSas, GcpOauthToken,
    R2TemporaryCredentials, TemporaryCredential, temporary_credential::Credentials,
};

#[cfg(feature = "credential-vending")]
impl TryFrom<&Credential> for native::RegisteredCredential {
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
                ..Default::default()
            }));
        }
        if let Some(gcp) = credential.databricks_gcp_service_account.as_option() {
            let email = gcp.email.clone().filter(|email| !email.is_empty());
            let email = email.ok_or_else(|| {
                Error::invalid_argument("GCP service account credential is missing its email.")
            })?;
            // The UC shape carries no key: the host impersonates the account.
            return Ok(Self::GcpServiceAccount(native::GcpServiceAccount {
                email,
                key: None,
            }));
        }
        Err(Error::invalid_argument(
            "No supported credential type found on this credential object.",
        ))
    }
}

#[cfg(feature = "credential-vending")]
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

/// Fill the credential kind of a [`Credential`] from a registered credential.
/// Metadata (name, purpose, …) is left untouched.
///
/// Fails for what the UC shape cannot carry: an AWS `ExternalId` or a GCP
/// service-account key.
#[cfg(feature = "credential-vending")]
pub fn set_storage_credential(
    credential: &mut Credential,
    storage: native::RegisteredCredential,
) -> Result<()> {
    match storage {
        native::RegisteredCredential::AwsIamRole(role) => {
            if role.external_id.is_some() {
                return Err(Error::invalid_argument(
                    "AWS IAM role credentials with an external id are not supported.",
                ));
            }
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
        native::RegisteredCredential::AzureServicePrincipal(sp) => {
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
        native::RegisteredCredential::AzureManagedIdentity(msi) => {
            credential.azure_managed_identity = Some(AzureManagedIdentity {
                access_connector_id: msi.access_connector_id,
                managed_identity_id: msi.managed_identity_id,
                ..Default::default()
            })
            .into();
        }
        native::RegisteredCredential::AzureStorageKey(key) => {
            credential.azure_storage_key = Some(AzureStorageKey {
                account_name: key.account_name,
                account_key: key.account_key,
                ..Default::default()
            })
            .into();
        }
        native::RegisteredCredential::GcpServiceAccount(gcp) => {
            if gcp.key.is_some() {
                return Err(Error::invalid_argument(
                    "GCP service account credentials with a key are not supported.",
                ));
            }
            credential.databricks_gcp_service_account = Some(DatabricksGcpServiceAccount {
                email: Some(gcp.email),
                ..Default::default()
            })
            .into();
        }
        other => {
            return Err(Error::invalid_argument(format!(
                "Unsupported credential kind `{}`.",
                other.kind()
            )));
        }
    }
    Ok(())
}

impl From<TemporaryCredential> for unity::TemporaryCredential {
    fn from(credential: TemporaryCredential) -> Self {
        let mut wire = Self {
            // The generated model has no "unset": non-positive means "does not expire".
            expiration_time: Some(credential.expiration_time).filter(|t| *t > 0),
            url: Some(credential.url).filter(|url| !url.is_empty()),
            ..Default::default()
        };
        match credential.credentials {
            Some(Credentials::AzureUserDelegationSas(sas)) => {
                wire.azure_user_delegation_sas = Some(wire::common::AzureUserDelegationSas {
                    sas_token: sas.sas_token,
                });
            }
            Some(Credentials::AzureAad(aad)) => {
                wire.azure_aad = Some(wire::common::AzureAad {
                    aad_token: aad.aad_token,
                });
            }
            Some(Credentials::AwsTempCredentials(aws)) => {
                wire.aws_temp_credentials = Some(wire::common::AwsTempCredentials {
                    access_key_id: aws.access_key_id,
                    secret_access_key: aws.secret_access_key,
                    session_token: aws.session_token,
                    access_point: aws.access_point,
                });
            }
            Some(Credentials::GcpOauthToken(gcp)) => {
                wire.gcp_oauth_token = Some(wire::common::GcpOauthToken {
                    oauth_token: gcp.oauth_token,
                });
            }
            Some(Credentials::R2TempCredentials(r2)) => {
                wire.r2_temp_credentials = Some(wire::common::R2TempCredentials {
                    access_key_id: r2.access_key_id,
                    secret_access_key: r2.secret_access_key,
                    session_token: r2.session_token,
                });
            }
            None => {}
        }
        wire
    }
}

impl From<unity::TemporaryCredential> for TemporaryCredential {
    fn from(wire: unity::TemporaryCredential) -> Self {
        // The wire shape holds at most one credential when built from a vended one;
        // take the first set, in the order the UC model declares them.
        let credentials = if let Some(sas) = wire.azure_user_delegation_sas {
            Some(Credentials::AzureUserDelegationSas(Box::new(
                AzureUserDelegationSas {
                    sas_token: sas.sas_token,
                    ..Default::default()
                },
            )))
        } else if let Some(aad) = wire.azure_aad {
            Some(Credentials::AzureAad(Box::new(AzureAad {
                aad_token: aad.aad_token,
                ..Default::default()
            })))
        } else if let Some(aws) = wire.aws_temp_credentials {
            Some(Credentials::AwsTempCredentials(Box::new(
                AwsTemporaryCredentials {
                    access_key_id: aws.access_key_id,
                    secret_access_key: aws.secret_access_key,
                    session_token: aws.session_token,
                    access_point: aws.access_point,
                    ..Default::default()
                },
            )))
        } else if let Some(gcp) = wire.gcp_oauth_token {
            Some(Credentials::GcpOauthToken(Box::new(GcpOauthToken {
                oauth_token: gcp.oauth_token,
                ..Default::default()
            })))
        } else {
            wire.r2_temp_credentials.map(|r2| {
                Credentials::R2TempCredentials(Box::new(R2TemporaryCredentials {
                    access_key_id: r2.access_key_id,
                    secret_access_key: r2.secret_access_key,
                    session_token: r2.session_token,
                    ..Default::default()
                }))
            })
        };
        Self {
            expiration_time: wire.expiration_time.unwrap_or_default(),
            url: wire.url.unwrap_or_default(),
            credentials,
            ..Default::default()
        }
    }
}

impl From<&olai_storage_credentials::VendedCredential> for TemporaryCredential {
    fn from(vended: &olai_storage_credentials::VendedCredential) -> Self {
        unity::TemporaryCredential::from(vended).into()
    }
}

impl From<olai_storage_credentials::Error> for Error {
    fn from(e: olai_storage_credentials::Error) -> Self {
        use olai_storage_credentials::Error as E;
        match e {
            E::InvalidLocation { .. } | E::InvalidCredential(_) => {
                Error::InvalidArgument(e.to_string())
            }
            E::Json(source) => source.into(),
            other => Error::Generic(other.to_string()),
        }
    }
}

#[cfg(feature = "credential-vending")]
impl From<olai_credential_vending::Error> for Error {
    fn from(e: olai_credential_vending::Error) -> Self {
        use olai_credential_vending::Error as E;
        match e {
            E::Credentials(source) => source.into(),
            E::NotFound { .. } => Error::NotFound,
            E::Overlapping { .. } | E::Unsupported { .. } => Error::InvalidArgument(e.to_string()),
            other => Error::Generic(other.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use olai_storage_credentials::wire::VendContext;
    use olai_storage_credentials::{AccessLevel, StorageCredentials, StorageScope};

    use super::*;

    #[cfg(feature = "credential-vending")]
    #[test]
    fn storage_credentials_convert_both_ways() {
        let storage =
            native::RegisteredCredential::AzureServicePrincipal(native::AzureServicePrincipal {
                directory_id: "tenant".into(),
                application_id: "app".into(),
                secret: native::AzureClientSecret::FederatedTokenFile("/token".into()),
            });
        let mut credential = Credential {
            name: "cred".into(),
            ..Default::default()
        };
        set_storage_credential(&mut credential, storage.clone()).unwrap();
        assert_eq!(
            native::RegisteredCredential::try_from(&credential).unwrap(),
            storage
        );
        // The JSON shapes agree, so values sealed in either form open as the other.
        let json = serde_json::to_value(&credential).unwrap();
        assert_eq!(
            serde_json::from_value::<native::RegisteredCredential>(json).unwrap(),
            storage
        );
        assert!(native::RegisteredCredential::try_from(&Credential::default()).is_err());
    }

    #[cfg(feature = "credential-vending")]
    #[test]
    fn gcp_service_accounts_convert_both_ways() {
        let storage = native::RegisteredCredential::GcpServiceAccount(native::GcpServiceAccount {
            email: "sa@project.iam.gserviceaccount.com".into(),
            key: None,
        });
        let mut credential = Credential::default();
        set_storage_credential(&mut credential, storage.clone()).unwrap();
        assert_eq!(
            native::RegisteredCredential::try_from(&credential).unwrap(),
            storage
        );
        let json = serde_json::to_value(&credential).unwrap();
        assert_eq!(
            serde_json::from_value::<native::RegisteredCredential>(json).unwrap(),
            storage
        );
    }

    #[test]
    fn temporary_credentials_convert_both_ways() {
        let wire = unity::TemporaryCredential {
            expiration_time: Some(42),
            url: Some("s3://bucket/prefix".into()),
            aws_temp_credentials: Some(wire::common::AwsTempCredentials {
                access_key_id: "id".into(),
                secret_access_key: "secret".into(),
                session_token: "token".into(),
                access_point: String::new(),
            }),
            ..Default::default()
        };
        let uc = TemporaryCredential::from(wire.clone());
        // The wire type reads the generated model's (proto) JSON.
        let json = serde_json::to_value(&uc).unwrap();
        assert_eq!(
            serde_json::from_value::<unity::TemporaryCredential>(json).unwrap(),
            wire
        );
        assert_eq!(unity::TemporaryCredential::from(uc), wire);
    }

    #[test]
    fn local_credentials_do_not_expire() {
        let uc = TemporaryCredential {
            url: "file:///tmp/table".into(),
            ..Default::default()
        };
        let wire = unity::TemporaryCredential::from(uc);
        assert_eq!(wire.expiration_time, None);
        let scope: StorageScope = "file:///tmp/table".parse().unwrap();
        let vended = wire
            .to_vended(&VendContext::new(scope, AccessLevel::Read))
            .unwrap();
        assert!(matches!(vended.credentials, StorageCredentials::None));
        assert_eq!(vended.expires_at, None);
    }
}
