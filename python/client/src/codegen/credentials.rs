// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::CredentialClient;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "CredentialClient")]
pub struct PyCredentialClient {
    pub(crate) client: CredentialClient,
}
#[pymethods]
impl PyCredentialClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyCredential> {
        let request = self.client.get();
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyCredential::from(result))
        })
    }
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
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_comment(comment);
        request = request.with_read_only(read_only);
        request = request.with_owner(owner);
        request = request.with_skip_validation(skip_validation);
        request = request.with_force(force);
        request = request
            .with_azure_service_principal(azure_service_principal.map(::core::convert::Into::into));
        request = request
            .with_azure_managed_identity(azure_managed_identity.map(::core::convert::Into::into));
        request =
            request.with_azure_storage_key(azure_storage_key.map(::core::convert::Into::into));
        request = request.with_aws_iam_role(aws_iam_role.map(::core::convert::Into::into));
        request = request.with_databricks_gcp_service_account(
            databricks_gcp_service_account.map(::core::convert::Into::into),
        );
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyCredential::from(result))
        })
    }
    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = get_runtime(py)?;
        py.detach(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PyCredentialClient {
    pub fn new(client: CredentialClient) -> Self {
        Self { client }
    }
}
