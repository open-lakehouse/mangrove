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
        if let Some(role) = credential.aws_iam_role.as_option()
            && let Some(neon) = neon_arn::parse(&role.role_arn)
        {
            let (project_id, branch_id) = neon?;
            if project_id.is_empty() || branch_id.is_empty() {
                return Err(Error::invalid_argument(format!(
                    "Neon storage credential `{}` must name its project and branch: `{}`.",
                    credential.name,
                    neon_arn::FORM
                )));
            }
            let has = |v: &Option<String>| v.as_deref().is_some_and(|v| !v.is_empty());
            if has(&role.access_key_id) || has(&role.secret_access_key) || has(&role.session_token)
            {
                return Err(Error::invalid_argument(
                    "Neon storage credentials carry no keys: the server mints them with its own \
                     Neon API key (`vending.neon.api-key`).",
                ));
            }
            return Ok(Self::NeonStorage(native::NeonStorage {
                project_id,
                branch_id,
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
        native::RegisteredCredential::NeonStorage(neon) => {
            credential.aws_iam_role = Some(AwsIamRoleConfig {
                role_arn: neon_arn::format(&neon.project_id, &neon.branch_id),
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

/// Neon object storage registered as an `aws_iam_role` credential.
///
/// Neon storage speaks S3 but has no IAM roles or STS: keys are minted per
/// branch through Neon's control plane (see `olai_credential_vending::neon`). The
/// UC credentials API has no Neon kind, so a Neon credential stays within the
/// contract as an `aws_iam_role` whose `role_arn` is a Neon ARN,
/// `arn:neon:storage::<project_id>:branch/<branch_id>`. It carries no keys: the
/// server mints with its own Neon API key.
///
/// Either id may be left empty on registration (`arn:neon:storage::` leaves both),
/// and the server fills it from its `vending.neon` configuration before storing.
pub mod neon_arn {
    use crate::{Error, Result};

    /// The `role_arn` prefix marking a Neon storage credential.
    pub const PREFIX: &str = "arn:neon:storage:";

    /// The expected form, for error messages.
    pub const FORM: &str = "arn:neon:storage::<project_id>:branch/<branch_id>";

    /// The Neon ARN for `project_id`'s branch `branch_id`.
    pub fn format(project_id: &str, branch_id: &str) -> String {
        format!("{PREFIX}:{project_id}:branch/{branch_id}")
    }

    /// The project and branch ids of a Neon ARN, either possibly empty.
    ///
    /// `None` when `arn` is not a Neon ARN (an ordinary AWS role); an error when
    /// it is one but malformed.
    pub fn parse(arn: &str) -> Option<Result<(String, String)>> {
        let rest = arn.strip_prefix(PREFIX)?;
        Some(parse_rest(rest).ok_or_else(|| {
            Error::invalid_argument(format!(
                "Malformed Neon storage ARN `{arn}`: expected `{FORM}`."
            ))
        }))
    }

    fn parse_rest(rest: &str) -> Option<(String, String)> {
        // The region field is empty: a branch's storage lives in its project's region.
        let rest = rest.strip_prefix(':')?;
        if rest.is_empty() {
            return Some((String::new(), String::new()));
        }
        let (project, resource) = rest.split_once(':')?;
        let branch = resource.strip_prefix("branch/")?;
        let valid = |id: &str| !id.contains([':', '/']) && !id.chars().any(char::is_whitespace);
        (valid(project) && valid(branch)).then(|| (project.to_string(), branch.to_string()))
    }

    /// `arn` with its empty ids filled from `project_id` / `branch_id`, or `None`
    /// when it is not a Neon ARN. Ids already in `arn` are kept.
    pub fn with_defaults(
        arn: &str,
        project_id: Option<&str>,
        branch_id: Option<&str>,
    ) -> Option<Result<String>> {
        let parsed = parse(arn)?;
        Some(parsed.and_then(|(project, branch)| {
            let fill = |own: String, default: Option<&str>, what: &str, key: &str| {
                if !own.is_empty() {
                    return Ok(own);
                }
                default
                    .filter(|d| !d.is_empty())
                    .map(str::to_string)
                    .ok_or_else(|| {
                        Error::invalid_argument(format!(
                            "Neon storage ARN `{arn}` names no {what}, and the server has no \
                             default (`vending.neon.{key}`): use `{FORM}`."
                        ))
                    })
            };
            let project = fill(project, project_id, "project", "project-id")?;
            let branch = fill(branch, branch_id, "branch", "branch-id")?;
            Ok(format(&project, &branch))
        }))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn round_trips() {
            let arn = format("proj-1", "br-2");
            assert_eq!(arn, "arn:neon:storage::proj-1:branch/br-2");
            assert_eq!(
                parse(&arn).unwrap().unwrap(),
                ("proj-1".into(), "br-2".into())
            );
        }

        #[test]
        fn empty_ids_parse() {
            assert_eq!(
                parse("arn:neon:storage::").unwrap().unwrap(),
                (String::new(), String::new())
            );
            assert_eq!(
                parse("arn:neon:storage::p:branch/").unwrap().unwrap(),
                ("p".into(), String::new())
            );
            assert_eq!(
                parse("arn:neon:storage:::branch/b").unwrap().unwrap(),
                (String::new(), "b".into())
            );
        }

        #[test]
        fn aws_arns_are_not_neon() {
            assert!(parse("arn:aws:iam::123456789012:role/test").is_none());
            assert!(parse("").is_none());
        }

        #[test]
        fn malformed_neon_arns_fail() {
            for arn in [
                "arn:neon:storage:",
                "arn:neon:storage:eu-central-1:p:branch/b",
                "arn:neon:storage::p",
                "arn:neon:storage::p:role/b",
                "arn:neon:storage::p:branch/b/extra",
                "arn:neon:storage::p:x:branch/b",
                "arn:neon:storage::p q:branch/b",
            ] {
                let err = parse(arn).unwrap().unwrap_err();
                assert!(err.to_string().contains(FORM), "{arn}: {err}");
            }
        }

        #[test]
        fn defaults_fill_only_empty_ids() {
            let filled = |arn, p, b| with_defaults(arn, p, b).unwrap();
            assert_eq!(
                filled("arn:neon:storage::", Some("p"), Some("b")).unwrap(),
                "arn:neon:storage::p:branch/b"
            );
            assert_eq!(
                filled("arn:neon:storage::own:branch/mine", Some("p"), Some("b")).unwrap(),
                "arn:neon:storage::own:branch/mine"
            );
            let err = filled("arn:neon:storage::p:branch/", Some("p"), None).unwrap_err();
            assert!(err.to_string().contains("vending.neon.branch-id"), "{err}");
            let err = filled("arn:neon:storage::", Some(""), Some("b")).unwrap_err();
            assert!(err.to_string().contains("vending.neon.project-id"), "{err}");
            assert!(with_defaults("arn:aws:iam::1:role/r", Some("p"), Some("b")).is_none());
        }
    }
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
            // The credential kind needs server setup this host lacks (a feature not
            // compiled in, or vending disabled in config).
            E::NotCompiledIn { .. } | E::NeedsHostConfig { .. } => {
                Error::FailedPrecondition(e.to_string())
            }
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
    fn neon_storage_converts_both_ways_as_an_aws_role() {
        let storage = native::RegisteredCredential::NeonStorage(native::NeonStorage {
            project_id: "proj-1".into(),
            branch_id: "br-2".into(),
        });
        let mut credential = Credential {
            name: "neon".into(),
            ..Default::default()
        };
        set_storage_credential(&mut credential, storage.clone()).unwrap();
        let role = credential.aws_iam_role.as_option().unwrap();
        assert_eq!(role.role_arn, "arn:neon:storage::proj-1:branch/br-2");
        assert_eq!(
            native::RegisteredCredential::try_from(&credential).unwrap(),
            storage
        );

        let neon_role = |arn: &str, key: Option<&str>| Credential {
            name: "neon".into(),
            aws_iam_role: Some(AwsIamRoleConfig {
                role_arn: arn.into(),
                access_key_id: key.map(Into::into),
                ..Default::default()
            })
            .into(),
            ..Default::default()
        };
        // Static keys have no meaning for Neon: the server mints with its own key.
        let err = native::RegisteredCredential::try_from(&neon_role(
            "arn:neon:storage::p:branch/b",
            Some("AKIA"),
        ))
        .unwrap_err();
        assert!(err.to_string().contains("vending.neon.api-key"), "{err}");
        // Registration fills empty ids; a stored ARN must be complete.
        let err = native::RegisteredCredential::try_from(&neon_role("arn:neon:storage::", None))
            .unwrap_err();
        assert!(err.to_string().contains(neon_arn::FORM), "{err}");
        // An ordinary role stays an AWS role.
        assert!(matches!(
            native::RegisteredCredential::try_from(&neon_role("arn:aws:iam::1:role/r", None)),
            Ok(native::RegisteredCredential::AwsIamRole(_))
        ));
    }

    #[cfg(feature = "credential-vending")]
    #[test]
    fn vending_setup_errors_are_failed_preconditions() {
        let err = Error::from(olai_credential_vending::Error::NotCompiledIn {
            kind: "neon_storage",
            feature: "neon-trusted-vending",
        });
        assert_eq!(err.error_code(), "FAILED_PRECONDITION");
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
