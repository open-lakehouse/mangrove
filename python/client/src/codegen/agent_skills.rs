// @generated — do not edit by hand.
use crate::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::runtime::get_runtime;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::AgentSkillClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "AgentSkillClient")]
pub struct PyAgentSkillClient {
    pub(crate) client: AgentSkillClient,
}
#[pymethods]
impl PyAgentSkillClient {
    #[pyo3(signature = (include_browse = None))]
    pub fn get(
        &self,
        py: Python,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<PyAgentSkill> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgentSkill::from(result))
        })
    }
    #[pyo3(
        signature = (
            new_name = None,
            description = None,
            allowed_tools = None,
            comment = None,
            owner = None
        )
    )]
    pub fn update(
        &self,
        py: Python,
        new_name: Option<String>,
        description: Option<String>,
        allowed_tools: Option<Vec<String>>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyUnityCatalogResult<PyAgentSkill> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_description(description);
        if let Some(allowed_tools) = allowed_tools {
            request = request.with_allowed_tools(allowed_tools);
        }
        request = request.with_comment(comment);
        request = request.with_owner(owner);
        let runtime = get_runtime(py)?;
        py.allow_threads(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgentSkill::from(result))
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
impl PyAgentSkillClient {
    pub fn new(client: AgentSkillClient) -> Self {
        Self { client }
    }
}
