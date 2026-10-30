// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::PolicyClient;
use unitycatalog_common::models::policies::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "PolicyClient")]
pub struct PyPolicyClient {
    pub(crate) client: PolicyClient,
}
#[pymethods]
impl PyPolicyClient {
    #[pyo3(signature = (policy_info))]
    pub fn create_policy(
        &self,
        py: Python<'_>,
        policy_info: PyPolicyInfo,
    ) -> PyResult<PyPolicyInfo> {
        let request = self.client.create_policy(policy_info.into());
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyPolicyInfo::from(result))
    }
    pub fn get(&self, py: Python<'_>) -> PyResult<PyPolicyInfo> {
        let request = self.client.get();
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyPolicyInfo::from(result))
    }
    #[pyo3(signature = (policy_info, update_mask = None))]
    pub fn update(
        &self,
        py: Python<'_>,
        policy_info: PyPolicyInfo,
        update_mask: Option<String>,
    ) -> PyResult<PyPolicyInfo> {
        let mut request = self.client.update(policy_info.into());
        request = request.with_update_mask(update_mask);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyPolicyInfo::from(result))
    }
    pub fn delete(&self, py: Python<'_>) -> PyResult<()> {
        let request = self.client.delete();
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyPolicyClient {
    pub fn new(client: PolicyClient) -> Self {
        Self { client }
    }
}
