// @generated — do not edit by hand.
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::AgentSkillClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
use unitycatalog_common::models::*;

/// The requests on one `AgentSkill`.
#[pyclass(
    name = "AgentSkillClient",
    module = "unitycatalog_client._client.agent_skills"
)]
pub struct PyAgentSkillClient {
    pub(crate) client: AgentSkillClient,
}

#[pymethods]
impl PyAgentSkillClient {
    /// Args:
    ///     include_browse: Whether to include agent skills in the response for which the principal can only
    ///                     access selective metadata for.
    ///
    /// Returns:
    ///     An agent skill registered in Unity Catalog.
    #[pyo3(signature = (include_browse = None))]
    pub fn get(
        &self,
        py: Python,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<PyAgentSkill> {
        let request = self.client.get().with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgentSkill::from(result))
        })
    }

    /// Args:
    ///     new_name: New name for the agent skill.
    ///     description: Updated description of what the skill does and when to use it.
    ///     allowed_tools: Updated tools the skill is permitted to use.
    ///     comment: The comment attached to the agent skill.
    ///     owner: The identifier of the user who owns the agent skill.
    ///
    /// Returns:
    ///     An agent skill registered in Unity Catalog.
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
        let request = self
            .client
            .update()
            .with_new_name(new_name)
            .with_description(description)
            .with_allowed_tools(allowed_tools.unwrap_or_default())
            .with_comment(comment)
            .with_owner(owner);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgentSkill::from(result))
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

impl PyAgentSkillClient {
    pub fn new(client: AgentSkillClient) -> Self {
        Self { client }
    }
}
