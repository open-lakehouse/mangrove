// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::TagPolicyClient;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "TagPolicyClient")]
pub struct PyTagPolicyClient {
    pub(crate) client: TagPolicyClient,
}
#[pymethods]
impl PyTagPolicyClient {
    pub fn get(&self, py: Python<'_>) -> PyResult<PyTagPolicy> {
        let request = self.client.get();
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyTagPolicy::from(result))
    }
    #[pyo3(signature = (tag_policy, update_mask = None))]
    pub fn update(
        &self,
        py: Python<'_>,
        tag_policy: PyTagPolicy,
        update_mask: Option<String>,
    ) -> PyResult<PyTagPolicy> {
        let mut request = self.client.update(tag_policy.into());
        request = request.with_update_mask(update_mask);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyTagPolicy::from(result))
    }
    pub fn delete(&self, py: Python<'_>) -> PyResult<()> {
        let request = self.client.delete();
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyTagPolicyClient {
    pub fn new(client: TagPolicyClient) -> Self {
        Self { client }
    }
}
