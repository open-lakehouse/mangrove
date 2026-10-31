use unitycatalog_common::models::credentials::v1::Credential;

use crate::{Error, Result};

/// The secret configuration of a credential, apart from the rest of it.
///
/// A [`Credential`] carries its secrets in one of five optional fields. Backends keep
/// them apart from the searchable columns, sealed at rest: [`take`](Self::take)
/// splits them off a credential, [`to_bytes`](Self::to_bytes) gives the plaintext to
/// seal, and [`apply`](Self::apply) puts them back on a credential read for a minter.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CredentialSecrets(Credential);

impl CredentialSecrets {
    /// Moves the secret fields out of `credential`, leaving it without secrets.
    pub fn take(credential: &mut Credential) -> Self {
        Self(Credential {
            azure_service_principal: std::mem::take(&mut credential.azure_service_principal),
            azure_managed_identity: std::mem::take(&mut credential.azure_managed_identity),
            azure_storage_key: std::mem::take(&mut credential.azure_storage_key),
            aws_iam_role: std::mem::take(&mut credential.aws_iam_role),
            databricks_gcp_service_account: std::mem::take(
                &mut credential.databricks_gcp_service_account,
            ),
            ..Default::default()
        })
    }

    /// Whether no secret configuration is set.
    pub fn is_empty(&self) -> bool {
        !(self.0.azure_service_principal.is_set()
            || self.0.azure_managed_identity.is_set()
            || self.0.azure_storage_key.is_set()
            || self.0.aws_iam_role.is_set()
            || self.0.databricks_gcp_service_account.is_set())
    }

    /// Sets the secret fields of `credential` to these.
    pub fn apply(self, credential: &mut Credential) {
        let Credential {
            azure_service_principal,
            azure_managed_identity,
            azure_storage_key,
            aws_iam_role,
            databricks_gcp_service_account,
            ..
        } = self.0;
        credential.azure_service_principal = azure_service_principal;
        credential.azure_managed_identity = azure_managed_identity;
        credential.azure_storage_key = azure_storage_key;
        credential.aws_iam_role = aws_iam_role;
        credential.databricks_gcp_service_account = databricks_gcp_service_account;
    }

    /// The plaintext to seal.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(&self.0)?)
    }

    /// Reads back plaintext written by [`to_bytes`](Self::to_bytes).
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut credential: Credential = serde_json::from_slice(bytes)
            .map_err(|e| Error::generic(format!("unreadable credential secrets: {e}")))?;
        Ok(Self::take(&mut credential))
    }
}

#[cfg(test)]
mod tests {
    use unitycatalog_common::models::credentials::v1::AwsIamRoleConfig;

    use super::*;

    #[test]
    fn take_apply_round_trips() {
        let mut credential = Credential {
            name: "c".into(),
            aws_iam_role: AwsIamRoleConfig {
                role_arn: "arn:aws:iam::1:role/r".into(),
                ..Default::default()
            }
            .into(),
            ..Default::default()
        };
        let original = credential.clone();
        let secrets = CredentialSecrets::take(&mut credential);
        assert!(!secrets.is_empty());
        assert!(credential.aws_iam_role.is_unset());

        let read = CredentialSecrets::from_bytes(&secrets.to_bytes().unwrap()).unwrap();
        assert_eq!(read, secrets);
        read.apply(&mut credential);
        assert_eq!(credential, original);
    }
}
