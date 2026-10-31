// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::AgentClient;
use unitycatalog_common::models::agents::v0alpha1::*;
use unitycatalog_common::models::*;

/// The requests on one `Agent`.
#[pyclass(name = "AgentClient", module = "unitycatalog_client._client.agents")]
pub struct PyAgentClient {
    pub(crate) client: AgentClient,
}

#[pymethods]
impl PyAgentClient {
    /// Args:
    ///     include_browse: Whether to include agents in the response for which the principal can only
    ///                     access selective metadata for.
    ///
    /// Returns:
    ///     An agent registered in Unity Catalog.
    #[pyo3(signature = (include_browse = None))]
    pub fn get(&self, py: Python, include_browse: Option<bool>) -> PyUnityCatalogResult<PyAgent> {
        let request = self.client.get().with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgent::from(result))
        })
    }

    /// Args:
    ///     new_name: New name for the agent.
    ///     invocation_protocol: The protocol a recipient uses to invoke the agent.
    ///     endpoint: The agent's invocation endpoint URL.
    ///     description: Updated LLM-readable description.
    ///     capabilities: Updated capability identifiers advertised by the agent.
    ///     input_schema: Updated JSON Schema (encoded as a JSON string) describing the expected input.
    ///     comment: The comment attached to the agent.
    ///     owner: The identifier of the user who owns the agent.
    ///
    /// Returns:
    ///     An agent registered in Unity Catalog.
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
        let request = self
            .client
            .update()
            .with_new_name(new_name)
            .with_invocation_protocol(invocation_protocol.map(::core::convert::Into::into))
            .with_endpoint(endpoint)
            .with_description(description)
            .with_capabilities(capabilities.unwrap_or_default())
            .with_input_schema(input_schema)
            .with_comment(comment)
            .with_owner(owner);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgent::from(result))
        })
    }

    pub fn delete(&self, py: Python) -> PyUnityCatalogResult<()> {
        let request = self.client.delete();
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }
}

impl PyAgentClient {
    pub fn new(client: AgentClient) -> Self {
        Self { client }
    }
}
