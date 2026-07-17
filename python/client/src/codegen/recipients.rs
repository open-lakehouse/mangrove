// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::RecipientClient;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "RecipientClient")]
pub struct PyRecipientClient {
    pub(crate) client: RecipientClient,
}
#[pymethods]
impl PyRecipientClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyRecipient> {
        let request = self.client.get();
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRecipient::from(result))
        })
    }
    #[pyo3(
        signature = (
            new_name = None,
            owner = None,
            comment = None,
            properties = None,
            expiration_time = None
        )
    )]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        owner: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        expiration_time: Option<i64>,
    ) -> PyUnityCatalogResult<PyRecipient> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_expiration_time(expiration_time);
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRecipient::from(result))
        })
    }
    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = get_runtime(py)?;
        py.detach(|| {
            runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(())
        })
    }
}
impl PyRecipientClient {
    pub fn new(client: RecipientClient) -> Self {
        Self { client }
    }
}
