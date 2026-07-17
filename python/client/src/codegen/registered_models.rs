// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
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
    pub fn get(
        &self,
        py: Python,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<PyRegisteredModel> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRegisteredModel::from(result))
        })
    }
    #[pyo3(signature = (new_name = None, comment = None, owner = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyUnityCatalogResult<PyRegisteredModel> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_comment(comment);
        request = request.with_owner(owner);
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRegisteredModel::from(result))
        })
    }
    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python, force: Option<bool>) -> PyUnityCatalogResult<()> {
        let mut request = self.client.delete();
        request = request.with_force(force);
        let runtime = get_runtime(py)?;
        py.detach(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PyRegisteredModelClient {
    pub fn new(client: RegisteredModelClient) -> Self {
        Self { client }
    }
}
