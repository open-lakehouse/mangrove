// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::TagPolicyClient;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "TagPolicyClient")]
pub struct PyTagPolicyClient {
    pub(crate) client: TagPolicyClient,
}
#[pymethods]
impl PyTagPolicyClient {
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyTagPolicy> {
        let request = self.client.get();
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTagPolicy::from(result))
        })
    }
    #[pyo3(signature = (tag_policy, update_mask = None))]
    pub fn update(
        &self,
        py: Python,
        tag_policy: PyTagPolicy,
        update_mask: Option<String>,
    ) -> PyUnityCatalogResult<PyTagPolicy> {
        let mut request = self.client.update(tag_policy.into());
        request = request.with_update_mask(update_mask);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTagPolicy::from(result))
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
impl PyTagPolicyClient {
    pub fn new(client: TagPolicyClient) -> Self {
        Self { client }
    }
}
