// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::CredentialClient;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::*;

/// The requests on one `Credential`.
#[pyclass(
    name = "CredentialClient",
    module = "unitycatalog_client._client.credentials"
)]
pub struct PyCredentialClient {
    pub(crate) client: CredentialClient,
}

#[pymethods]
impl PyCredentialClient {
    /// Returns:
    ///     A credential used to access external data sources or services.
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyCredential> {
        let request = self.client.get();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyCredential::from(result))
        })
    }

    /// Args:
    ///     new_name: New name of the credential.
    ///     comment: Comment associated with the credential.
    ///     read_only: Whether the credential is usable only for read operations. Only applicable when
    ///                purpose is STORAGE.
    ///     owner: Username of current owner of credential.
    ///     skip_validation: Supply true to this argument to skip validation of the updated credential.
    ///                      Defaults to false.
    ///     force: Force an update even if there are dependent services (when purpose is SERVICE) or
    ///            dependent external locations and external tables (when purpose is STORAGE).
    ///     azure_service_principal: The Azure service principal configuration.
    ///     azure_managed_identity: The Azure managed identity configuration.
    ///     azure_storage_key: The Azure storage key configuration.
    ///     aws_iam_role: The AWS IAM role configuration.
    ///     databricks_gcp_service_account: The Databricks managed GCP service account configuration.
    ///
    /// Returns:
    ///     A credential used to access external data sources or services.
    #[pyo3(
        signature = (
            new_name = None,
            comment = None,
            read_only = None,
            owner = None,
            skip_validation = None,
            force = None,
            azure_service_principal = None,
            azure_managed_identity = None,
            azure_storage_key = None,
            aws_iam_role = None,
            databricks_gcp_service_account = None
        )
    )]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        read_only: Option<bool>,
        owner: Option<String>,
        skip_validation: Option<bool>,
        force: Option<bool>,
        azure_service_principal: ::core::option::Option<PyAzureServicePrincipal>,
        azure_managed_identity: ::core::option::Option<PyAzureManagedIdentity>,
        azure_storage_key: ::core::option::Option<PyAzureStorageKey>,
        aws_iam_role: ::core::option::Option<PyAwsIamRoleConfig>,
        databricks_gcp_service_account: ::core::option::Option<PyDatabricksGcpServiceAccount>,
    ) -> PyUnityCatalogResult<PyCredential> {
        let request = self
            .client
            .update()
            .with_new_name(new_name)
            .with_comment(comment)
            .with_read_only(read_only)
            .with_owner(owner)
            .with_skip_validation(skip_validation)
            .with_force(force)
            .with_azure_service_principal(azure_service_principal.map(::core::convert::Into::into))
            .with_azure_managed_identity(azure_managed_identity.map(::core::convert::Into::into))
            .with_azure_storage_key(azure_storage_key.map(::core::convert::Into::into))
            .with_aws_iam_role(aws_iam_role.map(::core::convert::Into::into))
            .with_databricks_gcp_service_account(
                databricks_gcp_service_account.map(::core::convert::Into::into),
            );
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyCredential::from(result))
        })
    }

    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
}

impl PyCredentialClient {
    pub fn new(client: CredentialClient) -> Self {
        Self { client }
    }
}
