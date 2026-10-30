// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
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
    pub fn get(&self, py: Python<'_>) -> PyResult<PyCredential> {
        let request = self.client.get();
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyCredential::from(result))
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
        py: Python<'_>,
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
    ) -> PyResult<PyCredential> {
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
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyCredential::from(result))
    }
    pub fn delete(&self, py: Python<'_>) -> PyResult<()> {
        let request = self.client.delete();
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyCredentialClient {
    pub fn new(client: CredentialClient) -> Self {
        Self { client }
    }
}
