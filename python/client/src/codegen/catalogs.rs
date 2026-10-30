// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::CatalogClient;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "CatalogClient")]
pub struct PyCatalogClient {
    pub(crate) client: CatalogClient,
}
#[pymethods]
impl PyCatalogClient {
    #[pyo3(signature = (include_browse = None))]
    pub fn get(&self, py: Python<'_>, include_browse: Option<bool>) -> PyResult<PyCatalog> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyCatalog::from(result))
    }
    #[pyo3(
        signature = (owner = None, comment = None, properties = None, new_name = None)
    )]
    pub fn update(
        &self,
        py: Python<'_>,
        owner: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        new_name: Option<String>,
    ) -> PyResult<PyCatalog> {
        let mut request = self.client.update();
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_new_name(new_name);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyCatalog::from(result))
    }
    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python<'_>, force: Option<bool>) -> PyResult<()> {
        let mut request = self.client.delete();
        request = request.with_force(force);
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyCatalogClient {
    pub fn new(client: CatalogClient) -> Self {
        Self { client }
    }
}
