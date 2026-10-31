// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
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
    pub fn get(&self, py: Python<'_>) -> PyResult<PyProvider> {
        let request = self.client.get();
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyProvider::from(result))
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
        py: Python<'_>,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
        recipient_profile_str: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyResult<PyProvider> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        request = request.with_recipient_profile_str(recipient_profile_str);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyProvider::from(result))
    }
    pub fn delete(&self, py: Python<'_>) -> PyResult<()> {
        let request = self.client.delete();
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyProviderClient {
    pub fn new(client: ProviderClient) -> Self {
        Self { client }
    }
}
