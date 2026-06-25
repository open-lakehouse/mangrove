// @generated — do not edit by hand.
use super::builders::*;
use super::client::AgentServiceClient;
use unitycatalog_common::models::agents::v0alpha1::*;
/// A client scoped to a single `agent`.
#[derive(Clone)]
pub struct AgentClient {
    pub(crate) catalog_name: String,
    pub(crate) schema_name: String,
    pub(crate) agent_name: String,
    pub(crate) client: AgentServiceClient,
}
impl AgentClient {
    /// Create a client bound to the resource's name components.
    pub fn new(
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        agent_name: impl Into<String>,
        client: AgentServiceClient,
    ) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            agent_name: agent_name.into(),
            client,
        }
    }
    /// Create a `agent` client from its dot-joined full name (e.g. `"catalog_name.schema_name.agent_name"`).
    pub fn from_full_name(full_name: impl Into<String>, client: AgentServiceClient) -> Self {
        let full_name = full_name.into();
        let mut parts = full_name.splitn(3usize, '.');
        let catalog_name = parts.next().unwrap_or_default();
        let schema_name = parts.next().unwrap_or_default();
        let agent_name = parts.next().unwrap_or_default();
        Self::new(catalog_name, schema_name, agent_name, client)
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
        &self.agent_name
    }
    /// The fully-qualified name of this resource (its dot-joined name components).
    pub fn full_name(&self) -> String {
        format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.agent_name
        )
    }
    pub fn get(&self) -> GetAgentBuilder {
        GetAgentBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.agent_name
            ),
        )
    }
    pub fn update(&self) -> UpdateAgentBuilder {
        UpdateAgentBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.agent_name
            ),
        )
    }
    pub fn delete(&self) -> DeleteAgentBuilder {
        DeleteAgentBuilder::new(
            self.client.clone(),
            format!(
                "{}.{}.{}",
                self.catalog_name, self.schema_name, self.agent_name
            ),
        )
    }
}
