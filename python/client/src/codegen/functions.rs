// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::FunctionClient;
use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "FunctionClient")]
pub struct PyFunctionClient {
    pub(crate) client: FunctionClient,
}
#[pymethods]
impl PyFunctionClient {
    #[pyo3(signature = (include_browse = None))]
    pub fn get(&self, py: Python<'_>, include_browse: Option<bool>) -> PyResult<PyFunction> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyFunction::from(result))
    }
    #[pyo3(signature = (owner = None))]
    pub fn update(&self, py: Python<'_>, owner: Option<String>) -> PyResult<PyFunction> {
        let mut request = self.client.update();
        request = request.with_owner(owner);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyFunction::from(result))
    }
    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python<'_>, force: Option<bool>) -> PyResult<()> {
        let mut request = self.client.delete();
        request = request.with_force(force);
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyFunctionClient {
    pub fn new(client: FunctionClient) -> Self {
        Self { client }
    }
}
