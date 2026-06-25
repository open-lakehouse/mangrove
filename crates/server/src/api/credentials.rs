use itertools::Itertools;
use serde::{Deserialize, Serialize};

use unitycatalog_common::models::credentials::v1::{
    AwsIamRoleConfig, AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey,
    CreateCredentialRequest, Credential, DatabricksGcpServiceAccount, DeleteCredentialRequest,
    GetCredentialRequest, ListCredentialsRequest, ListCredentialsResponse, UpdateCredentialRequest,
};
use unitycatalog_common::models::{
    ObjectLabel, ResourceExt, ResourceIdent, ResourceName, ResourceRef,
};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::credentials::CredentialHandler;
use crate::policy::{Permission, Policy, process_resources};
use crate::services::secrets::SecretManager;
use crate::store::ResourceStore;
use crate::{Error, Result};

#[async_trait::async_trait]
pub trait CredentialHandlerExt: Send + Sync + 'static {
    /// Get a credential without checking permissions.
    ///
    /// This is used internally when access to a resource is already checked
    /// and we need to create internal stores or vended credentials for the resource.
    ///
    // TODO: this could also be done by a server recipient / context type
    async fn get_credential_internal(&self, request: GetCredentialRequest) -> Result<Credential>;
}

#[derive(Clone, Serialize, Deserialize, Default)]
struct CredentialContainer {
    pub azure_sp: Option<AzureServicePrincipal>,
    pub azure_msi: Option<AzureManagedIdentity>,
    pub azure_key: Option<AzureStorageKey>,
    pub aws_iam_role: Option<AwsIamRoleConfig>,
    pub gcp_service_account: Option<DatabricksGcpServiceAccount>,
}

impl CredentialContainer {
    fn validate(&self) -> Result<()> {
        if self.azure_sp.is_none()
            && self.azure_msi.is_none()
            && self.azure_key.is_none()
            && self.aws_iam_role.is_none()
            && self.gcp_service_account.is_none()
        {
            Err(Error::invalid_argument("No credentials provided"))
        } else {
            Ok(())
        }
    }

    fn to_vec(&self) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(self)?)
    }
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext> + SecretManager> CredentialHandler<RequestContext>
    for T
{
    #[tracing::instrument(skip(self, context))]
    async fn list_credentials(
        &self,
        request: ListCredentialsRequest,
        context: RequestContext,
    ) -> Result<ListCredentialsResponse> {
        self.check_required(&request, &context).await?;
        let (mut resources, next_page_token) = self
            .list(
                &ObjectLabel::Credential,
                None,
                request.max_results.map(|v| v as usize),
                request.page_token,
            )
            .await?;
        process_resources(self, &context, &Permission::Read, &mut resources).await?;
        Ok(ListCredentialsResponse {
            credentials: resources.into_iter().map(|r| r.try_into()).try_collect()?,
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
        let credential = CredentialContainer {
            azure_msi: request.azure_managed_identity.into_option(),
            azure_sp: request.azure_service_principal.into_option(),
            azure_key: request.azure_storage_key.into_option(),
            aws_iam_role: request.aws_iam_role.into_option(),
            gcp_service_account: request.databricks_gcp_service_account.into_option(),
        };
        credential.validate()?;
        self.put_secret(&request.name, credential.to_vec()?.into())
            .await?;
        let cred = Credential {
            name: request.name.clone(),
            full_name: Some(request.name),
            comment: request.comment,
            purpose: request.purpose,
            read_only: request.read_only.unwrap_or(false),
            used_for_managed_storage: false,
            id: None,
            created_at: None,
            updated_at: None,
            azure_managed_identity: ::buffa::MessageField::none(),
            azure_service_principal: ::buffa::MessageField::none(),
            azure_storage_key: ::buffa::MessageField::none(),
            aws_iam_role: ::buffa::MessageField::none(),
            databricks_gcp_service_account: ::buffa::MessageField::none(),
            owner: None,
            created_by: None,
            updated_by: None,
            ..Default::default()
        };
        Ok(self.create(cred.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn get_credential(
        &self,
        request: GetCredentialRequest,
        context: RequestContext,
    ) -> Result<Credential> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        self.get_credential_internal(request).await
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn update_credential(
        &self,
        request: UpdateCredentialRequest,
        context: RequestContext,
    ) -> Result<Credential> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        let credential = CredentialContainer {
            azure_msi: request.azure_managed_identity.into_option(),
            azure_sp: request.azure_service_principal.into_option(),
            azure_key: request.azure_storage_key.into_option(),
            aws_iam_role: request.aws_iam_role.into_option(),
            gcp_service_account: request.databricks_gcp_service_account.into_option(),
        };
        credential.validate()?;
        self.put_secret(&request.name, credential.to_vec()?.into())
            .await?;
        let curr = self
            .get_credential(
                GetCredentialRequest {
                    name: request.name.clone(),
                    ..Default::default()
                },
                context.clone(),
            )
            .await?;
        let cred = Credential {
            name: request.name.clone(),
            full_name: Some(request.name),
            comment: request.comment,
            purpose: curr.purpose,
            read_only: request.read_only.unwrap_or(false),
            used_for_managed_storage: false,
            id: None,
            created_at: None,
            updated_at: None,
            azure_managed_identity: ::buffa::MessageField::none(),
            azure_service_principal: ::buffa::MessageField::none(),
            azure_storage_key: ::buffa::MessageField::none(),
            aws_iam_role: ::buffa::MessageField::none(),
            databricks_gcp_service_account: ::buffa::MessageField::none(),
            owner: None,
            created_by: None,
            updated_by: None,
            ..Default::default()
        };
        Ok(self
            .update(&curr.resource_ident(), cred.into())
            .await?
            .0
            .try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_credential(
        &self,
        request: DeleteCredentialRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        match self.delete_secret(&request.name).await {
            // Delete the resource even if the secret is not found to allow cleanup
            // when the secret is deleted manually.
            Ok(_) | Err(unitycatalog_common::Error::NotFound) => {
                Ok(self.delete(&request.resource()).await?)
            }
            Err(e) => Err(e.into()),
        }
    }
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext> + SecretManager> CredentialHandlerExt for T {
    async fn get_credential_internal(&self, request: GetCredentialRequest) -> Result<Credential> {
        let mut cred: Credential = self.get(&request.resource()).await?.0.try_into()?;
        let secret_data = self.get_secret(&cred.name).await?;
        let secret: CredentialContainer = serde_json::from_slice(&secret_data)?;
        if secret.azure_msi.is_some() {
            cred.azure_managed_identity = secret.azure_msi.into();
        } else if secret.azure_sp.is_some() {
            cred.azure_service_principal = secret.azure_sp.into();
        } else if secret.azure_key.is_some() {
            cred.azure_storage_key = secret.azure_key.into();
        } else if secret.aws_iam_role.is_some() {
            cred.aws_iam_role = secret.aws_iam_role.into();
        } else if secret.gcp_service_account.is_some() {
            cred.databricks_gcp_service_account = secret.gcp_service_account.into();
        }
        Ok(cred)
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
        ResourceIdent::catalog(ResourceName::new([self.name.as_str()]))
    }

    fn permission(&self) -> &'static Permission {
        &Permission::Manage
    }
}
