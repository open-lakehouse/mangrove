use unitycatalog_common::credentials_ext::neon_arn;
use unitycatalog_common::models::credentials::v1::{
    CreateCredentialRequest, Credential, DeleteCredentialRequest, GetCredentialRequest,
    ListCredentialsRequest, ListCredentialsResponse, Purpose, UpdateCredentialRequest,
};
use unitycatalog_common::models::{ResourceIdent, ResourceName, ResourceRef};
use unitycatalog_storage_access::{
    CredentialSecrets, CredentialUpdate, Page, ProvidesStorageAccessStore,
};

use super::{RequestContext, SecuredAction};
use crate::error::from_store;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::ServerHandler;
use crate::{Error, Result};
pub use unitycatalog_handlers::credentials::CredentialHandler;

#[async_trait::async_trait]
impl CredentialHandler<RequestContext> for ServerHandler<RequestContext> {
    #[tracing::instrument(skip(self, context))]
    async fn list_credentials(
        &self,
        request: ListCredentialsRequest,
        context: RequestContext,
    ) -> Result<ListCredentialsResponse> {
        self.check_required(&request, &context).await?;
        let purpose = request
            .purpose
            .and_then(|p| p.as_known())
            .filter(|p| *p != Purpose::PURPOSE_UNSPECIFIED);
        let page = Page::new(super::page_size(request.max_results)?, request.page_token);
        let (mut credentials, next_page_token) = self
            .storage_access()
            .list_credentials(purpose, page)
            .await
            .map_err(from_store)?;
        process_resources(self, &context, &Permission::Read, &mut credentials).await?;
        Ok(ListCredentialsResponse {
            credentials,
            next_page_token,
            ..Default::default()
        })
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn create_credential(
        &self,
        request: CreateCredentialRequest,
        context: RequestContext,
    ) -> Result<Credential> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let credential = Credential {
            name: request.name,
            comment: request.comment,
            purpose: request.purpose,
            read_only: request.read_only.unwrap_or(false),
            azure_managed_identity: request.azure_managed_identity,
            azure_service_principal: request.azure_service_principal,
            azure_storage_key: request.azure_storage_key,
            aws_iam_role: request.aws_iam_role,
            databricks_gcp_service_account: request.databricks_gcp_service_account,
            ..Default::default()
        };
        if CredentialSecrets::take(&mut credential.clone()).is_empty() {
            return Err(Error::invalid_argument("No credentials provided"));
        }
        let mut credential = credential;
        self.complete_neon_arn(&mut credential)?;
        // The store seals the secrets apart from the row; the response is redacted,
        // which matches the public API contract.
        self.storage_access()
            .create_credential(credential)
            .await
            .map_err(from_store)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_credential(
        &self,
        request: GetCredentialRequest,
        context: RequestContext,
    ) -> Result<Credential> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        self.storage_access()
            .get_credential(&request.name)
            .await
            .map_err(from_store)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_credential(
        &self,
        request: UpdateCredentialRequest,
        context: RequestContext,
    ) -> Result<Credential> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        // `purpose` is immutable; secrets are replaced only when the request sets one.
        let mut replacement = Credential {
            azure_managed_identity: request.azure_managed_identity,
            azure_service_principal: request.azure_service_principal,
            azure_storage_key: request.azure_storage_key,
            aws_iam_role: request.aws_iam_role,
            databricks_gcp_service_account: request.databricks_gcp_service_account,
            ..Default::default()
        };
        self.complete_neon_arn(&mut replacement)?;
        let secrets = CredentialSecrets::take(&mut replacement);
        let update = CredentialUpdate {
            new_name: request.new_name,
            comment: request.comment,
            read_only: request.read_only,
            owner: request.owner,
            secrets: (!secrets.is_empty()).then_some(secrets),
        };
        let updated = self
            .storage_access()
            .update_credential(&request.name, update)
            .await
            .map_err(from_store)?;
        // Rotated secrets must not keep minting: drop every cached minter and
        // vended credential (rare admin operation, so a full flush is fine).
        self.forget_vended_credentials();
        Ok(updated)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_credential(
        &self,
        request: DeleteCredentialRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        // Refused while an external location uses the credential.
        self.storage_access()
            .delete_credential(&request.name)
            .await
            .map_err(from_store)?;
        // A deleted credential's secrets and vended credentials must not be reused.
        self.forget_vended_credentials();
        Ok(())
    }
}

impl ServerHandler<RequestContext> {
    /// Complete a Neon storage ARN (see [`neon_arn`]) missing its project or
    /// branch from the server's `vending.neon` defaults, so the stored credential
    /// names both. Ids in the ARN win; other credentials are left alone.
    fn complete_neon_arn(&self, credential: &mut Credential) -> Result<()> {
        let Some(role) = credential.aws_iam_role.as_option_mut() else {
            return Ok(());
        };
        let neon = self.neon_vending();
        if let Some(completed) =
            neon_arn::with_defaults(&role.role_arn, neon.project_id(), neon.branch_id())
        {
            role.role_arn = completed?;
            // Fail now, not at the first vend, on what a Neon credential can't carry
            // (static keys).
            olai_credential_vending::RegisteredCredential::try_from(&*credential)?;
        }
        Ok(())
    }
}

impl SecuredAction for CreateCredentialRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::credential(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Create
    }
}

impl SecuredAction for ListCredentialsRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::credential(ResourceRef::Undefined)
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for GetCredentialRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::credential(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Read
    }
}

impl SecuredAction for UpdateCredentialRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::credential(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}

impl SecuredAction for DeleteCredentialRequest {
    fn resource(&self) -> ResourceIdent {
        ResourceIdent::credential(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}
