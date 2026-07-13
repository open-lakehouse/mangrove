use itertools::Itertools;

use unitycatalog_common::models::credentials::v1::{
    CreateCredentialRequest, Credential, DeleteCredentialRequest, GetCredentialRequest,
    ListCredentialsRequest, ListCredentialsResponse, UpdateCredentialRequest,
};
use unitycatalog_common::models::{ObjectLabel, ResourceIdent, ResourceName, ResourceRef};

use super::{RequestContext, SecuredAction};
pub use crate::codegen::credentials::CredentialHandler;
use crate::policy::{Permission, Policy, process_resources};
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

/// Whether a credential carries at least one secret configuration.
///
/// The secret material lives directly on the `Credential`'s sensitive fields
/// (sealed inline on the object row by the store's managed layer); a create/update
/// must supply exactly one.
fn has_secret(cred: &Credential) -> bool {
    cred.azure_service_principal.is_set()
        || cred.azure_managed_identity.is_set()
        || cred.azure_storage_key.is_set()
        || cred.aws_iam_role.is_set()
        || cred.databricks_gcp_service_account.is_set()
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> CredentialHandler<RequestContext> for T {
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
        // The secret configs ride the Credential itself; the store's managed layer
        // strips them out of the searchable properties and seals them into the
        // object's inline encrypted blob, atomically with the row.
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
            azure_managed_identity: request.azure_managed_identity,
            azure_service_principal: request.azure_service_principal,
            azure_storage_key: request.azure_storage_key,
            aws_iam_role: request.aws_iam_role,
            databricks_gcp_service_account: request.databricks_gcp_service_account,
            owner: None,
            created_by: None,
            updated_by: None,
            ..Default::default()
        };
        if !has_secret(&cred) {
            return Err(Error::invalid_argument("No credentials provided"));
        }
        // The create response is redacted (the store never returns sealed fields on
        // an ordinary read), which matches the public API contract.
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
        // `purpose` is immutable on update — carry it over from the stored row. A
        // plain (redacted) get suffices: purpose is a non-sensitive Data field.
        let ident = ResourceIdent::credential(ResourceName::new([request.name.as_str()]));
        let curr: Credential = self.get(&ident).await?.0.try_into()?;
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
            azure_managed_identity: request.azure_managed_identity,
            azure_service_principal: request.azure_service_principal,
            azure_storage_key: request.azure_storage_key,
            aws_iam_role: request.aws_iam_role,
            databricks_gcp_service_account: request.databricks_gcp_service_account,
            owner: None,
            created_by: None,
            updated_by: None,
            ..Default::default()
        };
        if !has_secret(&cred) {
            return Err(Error::invalid_argument("No credentials provided"));
        }
        Ok(self.update(&ident, cred.into()).await?.0.try_into()?)
    }

    #[tracing::instrument(skip(self, context), fields(resource_name))]
    async fn delete_credential(
        &self,
        request: DeleteCredentialRequest,
        context: RequestContext,
    ) -> Result<()> {
        tracing::Span::current().record("resource_name", &request.name);
        self.check_required(&request, &context).await?;
        // The sealed secret blob rides the object row, so deleting the credential
        // object drops it atomically — no separate secret cleanup.
        Ok(self.delete(&request.resource()).await?)
    }
}

#[async_trait::async_trait]
impl<T: ResourceStore + Policy<RequestContext>> CredentialHandlerExt for T {
    async fn get_credential_internal(&self, request: GetCredentialRequest) -> Result<Credential> {
        // `get_with_secrets` decrypts the inline sealed fields back into the object's
        // properties, so the Object -> Credential conversion hydrates the secret
        // config directly — no manual field routing.
        Ok(self
            .get_with_secrets(&request.resource())
            .await?
            .0
            .try_into()?)
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
