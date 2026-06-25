// @generated — do not edit by hand.
use super::builders::*;
use super::client::AgentSkillServiceClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
/// A client scoped to a single `agent_skill`.
#[derive(Clone)]
pub struct AgentSkillClient {
    pub(crate) catalog_name: String,
    pub(crate) schema_name: String,
    pub(crate) agent_skill_name: String,
    pub(crate) client: AgentSkillServiceClient,
}
impl AgentSkillClient {
    /// Create a client bound to the resource's name components.
    pub fn new(
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        agent_skill_name: impl Into<String>,
        client: AgentSkillServiceClient,
    ) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            agent_skill_name: agent_skill_name.into(),
            client,
        }
    }
    /// Create a `agent_skill` client from its dot-joined full name (e.g. `"catalog_name.schema_name.agent_skill_name"`).
    pub fn from_full_name(full_name: impl Into<String>, client: AgentSkillServiceClient) -> Self {
        let full_name = full_name.into();
        let mut parts = full_name.splitn(3usize, '.');
        let catalog_name = parts.next().unwrap_or_default();
        let schema_name = parts.next().unwrap_or_default();
        let agent_skill_name = parts.next().unwrap_or_default();
        Self::new(catalog_name, schema_name, agent_skill_name, client)
    }
    /// The `catalog_name` component of this resource's name.
    pub fn catalog_name(&self) -> &str {
        &self.catalog_name
    }
    /// The `schema_name` component of this resource's name.
    pub fn schema_name(&self) -> &str {
        &self.schema_name
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.agent_skill_name
    }
    /// The fully-qualified name of this resource (its dot-joined name components).
    pub fn full_name(&self) -> String {
        format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.agent_skill_name
        )
    }
    pub fn get(&self) -> GetAgentSkillBuilder {
        GetAgentSkillBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.agent_skill_name
            ),
        )
    }
    pub fn update(&self) -> UpdateAgentSkillBuilder {
        UpdateAgentSkillBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.agent_skill_name
            ),
        )
    }
    pub fn delete(&self) -> DeleteAgentSkillBuilder {
        DeleteAgentSkillBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.agent_skill_name
            ),
        )
    }
}
