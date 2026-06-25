// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::ProviderClient;
use unitycatalog_common::models::providers::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "ProviderClient")]
pub struct PyProviderClient {
    pub(crate) client: ProviderClient,
}
#[pymethods]
impl PyProviderClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyProvider> {
        let request = self.client.get();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyProvider::from(result))
        })
    }
    #[pyo3(
        signature = (
            new_name = None,
            owner = None,
            comment = None,
            recipient_profile_str = None,
            properties = None
        )
    )]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
        recipient_profile_str: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<PyProvider> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        request = request.with_recipient_profile_str(recipient_profile_str);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyProvider::from(result))
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
impl PyProviderClient {
    pub fn new(client: ProviderClient) -> Self {
        Self { client }
    }
}
