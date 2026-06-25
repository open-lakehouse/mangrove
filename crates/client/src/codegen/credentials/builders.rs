// @generated — do not edit by hand.
#![allow(unused_mut)]
type BoxFut<'a, T> = ::futures::future::BoxFuture<'a, T>;
type BoxStr<'a, T> = ::futures::stream::BoxStream<'a, T>;
use super::super::stream_paginated;
use super::client::*;
use crate::Result;
use futures::{StreamExt, TryStreamExt};
use std::future::IntoFuture;
use unitycatalog_common::models::credentials::v1::*;
/// Builder for listing credentials
pub struct ListCredentialsBuilder {
    client: CredentialServiceClient,
    request: ListCredentialsRequest,
}
impl ListCredentialsBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CredentialServiceClient`.
    pub(crate) fn new(client: CredentialServiceClient) -> Self {
        let request = ListCredentialsRequest {
            ..Default::default()
        };
        Self { client, request }
    }
    /// Return only credentials for the specified purpose.
    pub fn with_purpose(mut self, purpose: impl Into<Option<Purpose>>) -> Self {
        self.request.purpose = purpose.into().map(buffa::EnumValue::Known);
        self
    }
    /// The maximum number of results per page that should be returned.
    pub fn with_max_results(mut self, max_results: impl Into<Option<i32>>) -> Self {
        self.request.max_results = max_results.into();
        self
    }
    /// Opaque pagination token to go to next page based on previous query.
    pub fn with_page_token(mut self, page_token: impl Into<Option<String>>) -> Self {
        self.request.page_token = page_token.into();
        self
    }
    /// Convert paginated request into stream of results
    pub fn into_stream(self) -> BoxStr<'static, Result<Credential>> {
        let remaining = self.request.max_results;
        let stream = stream_paginated(
            (self, remaining),
            move |(mut builder, mut remaining), page_token| async move {
                builder.request.page_token = page_token;
                let res = builder.client.list_credentials(&builder.request).await?;
                if let Some(ref mut rem) = remaining {
                    *rem -= res.credentials.len() as i32;
                }
                let next_page_token = if remaining.is_some_and(|r| r <= 0) {
                    None
                } else {
                    res.next_page_token.clone()
                };
                Ok((res, (builder, remaining), next_page_token))
            },
        )
        .map_ok(|resp| futures::stream::iter(resp.credentials.into_iter().map(Ok)))
        .try_flatten();
        stream.boxed()
    }
}
impl IntoFuture for ListCredentialsBuilder {
    type Output = Result<ListCredentialsResponse>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.list_credentials(&request).await })
    }
}
/// Builder for creating a credential
pub struct CreateCredentialBuilder {
    client: CredentialServiceClient,
    request: CreateCredentialRequest,
}
impl CreateCredentialBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CredentialServiceClient`.
    pub(crate) fn new(
        client: CredentialServiceClient,
        name: impl Into<String>,
        purpose: Purpose,
    ) -> Self {
        let request = CreateCredentialRequest {
            name: name.into(),
            purpose: buffa::EnumValue::Known(purpose),
            ..Default::default()
        };
        Self { client, request }
    }
    /// Comment associated with the credential.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// Whether the credential is usable only for read operations. Only applicable when purpose is STORAGE.
    pub fn with_read_only(mut self, read_only: impl Into<Option<bool>>) -> Self {
        self.request.read_only = read_only.into();
        self
    }
    /// Optional. Supplying true to this argument skips validation of the created set of credentials.
    pub fn with_skip_validation(mut self, skip_validation: impl Into<Option<bool>>) -> Self {
        self.request.skip_validation = skip_validation.into();
        self
    }
    /// The Azure service principal configuration.
    pub fn with_azure_service_principal(
        mut self,
        azure_service_principal: impl Into<Option<AzureServicePrincipal>>,
    ) -> Self {
        self.request.azure_service_principal = {
            let azure_service_principal: ::core::option::Option<_> = azure_service_principal.into();
            buffa::MessageField::from(azure_service_principal)
        };
        self
    }
    /// The Azure managed identity configuration.
    pub fn with_azure_managed_identity(
        mut self,
        azure_managed_identity: impl Into<Option<AzureManagedIdentity>>,
    ) -> Self {
        self.request.azure_managed_identity = {
            let azure_managed_identity: ::core::option::Option<_> = azure_managed_identity.into();
            buffa::MessageField::from(azure_managed_identity)
        };
        self
    }
    /// The Azure storage key configuration.
    pub fn with_azure_storage_key(
        mut self,
        azure_storage_key: impl Into<Option<AzureStorageKey>>,
    ) -> Self {
        self.request.azure_storage_key = {
            let azure_storage_key: ::core::option::Option<_> = azure_storage_key.into();
            buffa::MessageField::from(azure_storage_key)
        };
        self
    }
    /// The AWS IAM role configuration.
    pub fn with_aws_iam_role(mut self, aws_iam_role: impl Into<Option<AwsIamRoleConfig>>) -> Self {
        self.request.aws_iam_role = {
            let aws_iam_role: ::core::option::Option<_> = aws_iam_role.into();
            buffa::MessageField::from(aws_iam_role)
        };
        self
    }
    /// The Databricks managed GCP service account configuration.
    pub fn with_databricks_gcp_service_account(
        mut self,
        databricks_gcp_service_account: impl Into<Option<DatabricksGcpServiceAccount>>,
    ) -> Self {
        self.request.databricks_gcp_service_account = {
            let databricks_gcp_service_account: ::core::option::Option<_> =
                databricks_gcp_service_account.into();
            buffa::MessageField::from(databricks_gcp_service_account)
        };
        self
    }
}
impl IntoFuture for CreateCredentialBuilder {
    type Output = Result<Credential>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.create_credential(&request).await })
    }
}
/// Builder for getting a credential
pub struct GetCredentialBuilder {
    client: CredentialServiceClient,
    request: GetCredentialRequest,
}
impl GetCredentialBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CredentialServiceClient`.
    pub(crate) fn new(client: CredentialServiceClient, name: impl Into<String>) -> Self {
        let request = GetCredentialRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for GetCredentialBuilder {
    type Output = Result<Credential>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.get_credential(&request).await })
    }
}
/// Builder for updating a credential
pub struct UpdateCredentialBuilder {
    client: CredentialServiceClient,
    request: UpdateCredentialRequest,
}
impl UpdateCredentialBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CredentialServiceClient`.
    pub(crate) fn new(client: CredentialServiceClient, name: impl Into<String>) -> Self {
        let request = UpdateCredentialRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
    /// New name of the credential.
    pub fn with_new_name(mut self, new_name: impl Into<Option<String>>) -> Self {
        self.request.new_name = new_name.into();
        self
    }
    /// Comment associated with the credential.
    pub fn with_comment(mut self, comment: impl Into<Option<String>>) -> Self {
        self.request.comment = comment.into();
        self
    }
    /// Whether the credential is usable only for read operations. Only applicable when purpose is STORAGE.
    pub fn with_read_only(mut self, read_only: impl Into<Option<bool>>) -> Self {
        self.request.read_only = read_only.into();
        self
    }
    /// Username of current owner of credential.
    pub fn with_owner(mut self, owner: impl Into<Option<String>>) -> Self {
        self.request.owner = owner.into();
        self
    }
    /// Supply true to this argument to skip validation of the updated credential.
    pub fn with_skip_validation(mut self, skip_validation: impl Into<Option<bool>>) -> Self {
        self.request.skip_validation = skip_validation.into();
        self
    }
    /** Force an update even if there are dependent services (when purpose is SERVICE)
    or dependent external locations and external tables (when purpose is STORAGE).*/
    pub fn with_force(mut self, force: impl Into<Option<bool>>) -> Self {
        self.request.force = force.into();
        self
    }
    /// The Azure service principal configuration.
    pub fn with_azure_service_principal(
        mut self,
        azure_service_principal: impl Into<Option<AzureServicePrincipal>>,
    ) -> Self {
        self.request.azure_service_principal = {
            let azure_service_principal: ::core::option::Option<_> = azure_service_principal.into();
            buffa::MessageField::from(azure_service_principal)
        };
        self
    }
    /// The Azure managed identity configuration.
    pub fn with_azure_managed_identity(
        mut self,
        azure_managed_identity: impl Into<Option<AzureManagedIdentity>>,
    ) -> Self {
        self.request.azure_managed_identity = {
            let azure_managed_identity: ::core::option::Option<_> = azure_managed_identity.into();
            buffa::MessageField::from(azure_managed_identity)
        };
        self
    }
    /// The Azure storage key configuration.
    pub fn with_azure_storage_key(
        mut self,
        azure_storage_key: impl Into<Option<AzureStorageKey>>,
    ) -> Self {
        self.request.azure_storage_key = {
            let azure_storage_key: ::core::option::Option<_> = azure_storage_key.into();
            buffa::MessageField::from(azure_storage_key)
        };
        self
    }
    /// The AWS IAM role configuration.
    pub fn with_aws_iam_role(mut self, aws_iam_role: impl Into<Option<AwsIamRoleConfig>>) -> Self {
        self.request.aws_iam_role = {
            let aws_iam_role: ::core::option::Option<_> = aws_iam_role.into();
            buffa::MessageField::from(aws_iam_role)
        };
        self
    }
    /// The Databricks managed GCP service account configuration.
    pub fn with_databricks_gcp_service_account(
        mut self,
        databricks_gcp_service_account: impl Into<Option<DatabricksGcpServiceAccount>>,
    ) -> Self {
        self.request.databricks_gcp_service_account = {
            let databricks_gcp_service_account: ::core::option::Option<_> =
                databricks_gcp_service_account.into();
            buffa::MessageField::from(databricks_gcp_service_account)
        };
        self
    }
}
impl IntoFuture for UpdateCredentialBuilder {
    type Output = Result<Credential>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.update_credential(&request).await })
    }
}
/// Builder for deleting a credential
pub struct DeleteCredentialBuilder {
    client: CredentialServiceClient,
    request: DeleteCredentialRequest,
}
impl DeleteCredentialBuilder {
    /// Create a new builder instance.
    /// Obtain via the corresponding method on `CredentialServiceClient`.
    pub(crate) fn new(client: CredentialServiceClient, name: impl Into<String>) -> Self {
        let request = DeleteCredentialRequest {
            name: name.into(),
            ..Default::default()
        };
        Self { client, request }
    }
}
impl IntoFuture for DeleteCredentialBuilder {
    type Output = Result<()>;
    type IntoFuture = BoxFut<'static, Self::Output>;
    fn into_future(self) -> Self::IntoFuture {
        let client = self.client;
        let request = self.request;
        Box::pin(async move { client.delete_credential(&request).await })
    }
}
