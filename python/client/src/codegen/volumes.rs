// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::VolumeClient;
use unitycatalog_common::models::volumes::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "VolumeClient")]
pub struct PyVolumeClient {
    pub(crate) client: VolumeClient,
}
#[pymethods]
impl PyVolumeClient {
    #[pyo3(signature = (include_browse = None))]
    pub fn get(&self, py: Python<'_>, include_browse: Option<bool>) -> PyResult<PyVolume> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyVolume::from(result))
    }
    #[pyo3(signature = (new_name = None, comment = None, owner = None))]
    pub fn update(
        &self,
        py: Python<'_>,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyResult<PyVolume> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_comment(comment);
        request = request.with_owner(owner);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyVolume::from(result))
    }
    pub fn delete(&self, py: Python<'_>) -> PyResult<()> {
        let request = self.client.delete();
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyVolumeClient {
    pub fn new(client: VolumeClient) -> Self {
        Self { client }
    }
}
