// @generated — do not edit by hand.
use crate::codegen::runtime::{ListIterator, block_on};
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
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
    pub fn get(&self, py: Python<'_>, include_browse: Option<bool>) -> PyResult<PyAgentSkill> {
        let mut request = self.client.get();
        request = request.with_include_browse(include_browse);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyAgentSkill::from(result))
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
        py: Python<'_>,
        new_name: Option<String>,
        description: Option<String>,
        allowed_tools: Option<Vec<String>>,
        comment: Option<String>,
        owner: Option<String>,
    ) -> PyResult<PyAgentSkill> {
        let mut request = self.client.update();
        request = request.with_new_name(new_name);
        request = request.with_description(description);
        if let Some(allowed_tools) = allowed_tools {
            request = request.with_allowed_tools(allowed_tools);
        }
        request = request.with_comment(comment);
        request = request.with_owner(owner);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyAgentSkill::from(result))
    }
    pub fn delete(&self, py: Python<'_>) -> PyResult<()> {
        let request = self.client.delete();
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
}
impl PyAgentSkillClient {
    pub fn new(client: AgentSkillClient) -> Self {
        Self { client }
    }
}
