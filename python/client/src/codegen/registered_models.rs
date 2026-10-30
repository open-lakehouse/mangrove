// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::RegisteredModelClient;
use unitycatalog_common::models::registered_models::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "RegisteredModelClient")]
pub struct PyRegisteredModelClient {
    pub(crate) client: RegisteredModelClient,
}
#[pymethods]
impl PyRegisteredModelClient {
    #[pyo3(signature = (include_browse = None))]
    pub fn get(&self, py: Python<'_>, include_browse: Option<bool>) -> PyResult<PyRegisteredModel> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyRegisteredModel::from(result))
    }
    #[pyo3(signature = (new_name = None, comment = None, owner = None))]
    pub fn update(
        &self,
        py: Python<'_>,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyResult<PyRegisteredModel> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_comment(comment);
        request = request.with_owner(owner);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyRegisteredModel::from(result))
    }
    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python<'_>, force: Option<bool>) -> PyResult<()> {
        let mut request = self.client.delete();
        request = request.with_force(force);
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyRegisteredModelClient {
    pub fn new(client: RegisteredModelClient) -> Self {
        Self { client }
    }
}
