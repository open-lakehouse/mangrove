// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::AgentClient;
use unitycatalog_common::models::agents::v0alpha1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "AgentClient")]
pub struct PyAgentClient {
    pub(crate) client: AgentClient,
}
#[pymethods]
impl PyAgentClient {
    #[pyo3(signature = (include_browse = None))]
    pub fn get(&self, py: Python, include_browse: Option<bool>) -> PyUnityCatalogResult<PyAgent> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgent::from(result))
        })
    }
    #[pyo3(
        signature = (
            new_name = None,
            invocation_protocol = None,
            endpoint = None,
            description = None,
            capabilities = None,
            input_schema = None,
            comment = None,
            owner = None
        )
    )]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        invocation_protocol: ::core::option::Option<PyInvocationProtocol>,
        endpoint: Option<String>,
        description: Option<String>,
        capabilities: Option<Vec<String>>,
        input_schema: Option<String>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyUnityCatalogResult<PyAgent> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request =
            request.with_invocation_protocol(invocation_protocol.map(::core::convert::Into::into));
        request = request.with_endpoint(endpoint);
        request = request.with_description(description);
        if let Some(capabilities) = capabilities {
            request = request.with_capabilities(capabilities);
        }
        request = request.with_input_schema(input_schema);
        request = request.with_comment(comment);
        request = request.with_owner(owner);
        let runtime = get_runtime(py)?;
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgent::from(result))
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
impl PyAgentClient {
    pub fn new(client: AgentClient) -> Self {
        Self { client }
    }
}
