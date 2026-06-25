// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::ExternalLocationClient;
use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "ExternalLocationClient")]
pub struct PyExternalLocationClient {
    pub(crate) client: ExternalLocationClient,
}
#[pymethods]
impl PyExternalLocationClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyExternalLocation> {
        let request = self.client.get();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyExternalLocation::from(result))
        })
    }
    #[pyo3(
        signature = (
            url = None,
            credential_name = None,
            read_only = None,
            owner = None,
            comment = None,
            new_name = None,
            force = None,
            skip_validation = None
        )
    )]
    pub fn update(
        &self,
        py: Python,
        url: Option<String>,
        credential_name: Option<String>,
        read_only: Option<bool>,
        owner: Option<String>,
        comment: Option<String>,
        new_name: Option<String>,
        force: Option<bool>,
        skip_validation: Option<bool>,
    ) -> PyUnityCatalogResult<PyExternalLocation> {
        let mut request = self.client.update();
        request = request.with_url(url);
        request = request.with_credential_name(credential_name);
        request = request.with_read_only(read_only);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        request = request.with_new_name(new_name);
        request = request.with_force(force);
        request = request.with_skip_validation(skip_validation);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyExternalLocation::from(result))
        })
    }
    #[pyo3(signature = (force = None))]
    pub fn delete(&self, py: Python, force: Option<bool>) -> PyUnityCatalogResult<()> {
        let mut request = self.client.delete();
        request = request.with_force(force);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PyExternalLocationClient {
    pub fn new(client: ExternalLocationClient) -> Self {
        Self { client }
    }
}
