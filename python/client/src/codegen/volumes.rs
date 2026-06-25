// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
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
    pub fn get(&self, py: Python, include_browse: Option<bool>) -> PyUnityCatalogResult<PyVolume> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyVolume::from(result))
        })
    }
    #[pyo3(signature = (new_name = None, comment = None, owner = None))]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyUnityCatalogResult<PyVolume> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_comment(comment);
        request = request.with_owner(owner);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyVolume::from(result))
        })
    }
    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PyVolumeClient {
    pub fn new(client: VolumeClient) -> Self {
        Self { client }
    }
}
