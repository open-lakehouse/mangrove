// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::TableClient;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "TableClient")]
pub struct PyTableClient {
    pub(crate) client: TableClient,
}
#[pymethods]
impl PyTableClient {
    #[pyo3(
        signature = (
            include_delta_metadata = None,
            include_browse = None,
            include_manifest_capabilities = None
        )
    )]
    pub fn get(
        &self,
        py: Python,
        include_delta_metadata: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> PyUnityCatalogResult<PyTable> {
        let mut request = self.client.get();
        request = request.with_include_delta_metadata(include_delta_metadata);
        request = request.with_include_browse(include_browse);
        request = request.with_include_manifest_capabilities(include_manifest_capabilities);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTable::from(result))
        })
    }
    pub fn get_table_exists(&self, py: Python) -> PyUnityCatalogResult<PyGetTableExistsResponse> {
        let request = self.client.get_table_exists();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyGetTableExistsResponse::from(result))
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
impl PyTableClient {
    pub fn new(client: TableClient) -> Self {
        Self { client }
    }
}
