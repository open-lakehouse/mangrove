// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::PolicyClient;
use unitycatalog_common::models::policies::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "PolicyClient")]
pub struct PyPolicyClient {
    pub(crate) client: PolicyClient,
}
#[pymethods]
impl PyPolicyClient {
    #[pyo3(signature = (policy_info))]
    pub fn create_policy(
        &self,
        py: Python,
        policy_info: PyPolicyInfo,
    ) -> PyUnityCatalogResult<PyPolicyInfo> {
        let request = self.client.create_policy(policy_info.into());
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyPolicyInfo::from(result))
        })
    }
    pub fn get(&self, py: Python) -> PyUnityCatalogResult<PyPolicyInfo> {
        let request = self.client.get();
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyPolicyInfo::from(result))
        })
    }
    #[pyo3(signature = (policy_info, update_mask = None))]
    pub fn update(
        &self,
        py: Python,
        policy_info: PyPolicyInfo,
        update_mask: Option<String>,
    ) -> PyUnityCatalogResult<PyPolicyInfo> {
        let mut request = self.client.update(policy_info.into());
        request = request.with_update_mask(update_mask);
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyPolicyInfo::from(result))
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
impl PyPolicyClient {
    pub fn new(client: PolicyClient) -> Self {
        Self { client }
    }
}
