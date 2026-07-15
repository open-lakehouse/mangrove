// @generated — do not edit by hand.
#![allow(unused_imports)]
use super::builders::*;
use super::client::SchemaServiceClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
use unitycatalog_common::models::agents::v0alpha1::*;
use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::registered_models::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::volumes::v1::*;
/// A client scoped to a single `schema`.
#[derive(Clone)]
pub struct SchemaClient {
    pub(crate) catalog_name: String,
    pub(crate) schema_name: String,
    pub(crate) client: SchemaServiceClient,
}
impl SchemaClient {
    /// Create a client bound to the resource's name components.
    pub fn new(
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        client: SchemaServiceClient,
    ) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            client,
        }
    }
    /// Create a `schema` client from its dot-joined full name (e.g. `"catalog_name.schema_name"`).
    pub fn from_full_name(full_name: impl Into<String>, client: SchemaServiceClient) -> Self {
        let full_name = full_name.into();
        let mut parts = full_name.splitn(2usize, '.');
        let catalog_name = parts.next().unwrap_or_default();
        let schema_name = parts.next().unwrap_or_default();
        Self::new(catalog_name, schema_name, client)
    }
    /// The `catalog_name` component of this resource's name.
    pub fn catalog_name(&self) -> &str {
        &self.catalog_name
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.schema_name
    }
    /// The fully-qualified name of this resource (its dot-joined name components).
    pub fn full_name(&self) -> String {
        format!("{}.{}", self.catalog_name, self.schema_name)
    }
    /// Gets the specified schema within the metastore.
    /// The caller must be a metastore admin, the owner of the schema,
    /// or a user that has the USE_SCHEMA privilege on the schema.
    pub fn get(&self) -> GetSchemaBuilder {
        GetSchemaBuilder::new(
            self.client.clone(),
            format!("{}.{}", self.catalog_name, self.schema_name),
        )
    }
    /// Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
    /// If the caller is a metastore admin, only the owner field can be changed in the update.
    /// If the name field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA
    /// privilege on the parent catalog.
    pub fn update(&self) -> UpdateSchemaBuilder {
        UpdateSchemaBuilder::new(
            self.client.clone(),
            format!("{}.{}", self.catalog_name, self.schema_name),
        )
    }
    /// Deletes the specified schema from the parent catalog. The caller must be the owner
    /// of the schema or an owner of the parent catalog.
    pub fn delete(&self) -> DeleteSchemaBuilder {
        DeleteSchemaBuilder::new(
            self.client.clone(),
            format!("{}.{}", self.catalog_name, self.schema_name),
        )
    }
    /// Access a `agent` within this resource.
    pub fn agent(&self, agent_name: impl Into<String>) -> crate::codegen::agents::AgentClient {
        crate::codegen::agents::AgentClient::new(
            &self.catalog_name,
            &self.schema_name,
            agent_name,
            crate::codegen::agents::AgentServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
        )
    }
    /// Create a `agent` within this resource.
    pub fn create_agent(
        &self,
        name: impl Into<String>,
        invocation_protocol: InvocationProtocol,
        endpoint: impl Into<String>,
    ) -> crate::codegen::agents::CreateAgentBuilder {
        crate::codegen::agents::CreateAgentBuilder::new(
            crate::codegen::agents::AgentServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            &self.catalog_name,
            &self.schema_name,
            name,
            invocation_protocol,
            endpoint,
        )
    }
    /// List `agent` resources within this resource.
    pub fn list_agents(&self) -> crate::codegen::agents::ListAgentsBuilder {
        crate::codegen::agents::ListAgentsBuilder::new(
            crate::codegen::agents::AgentServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            &self.catalog_name,
            &self.schema_name,
        )
    }
    /// Access a `agent_skill` within this resource.
    pub fn agent_skill(
        &self,
        agent_skill_name: impl Into<String>,
    ) -> crate::codegen::agent_skills::AgentSkillClient {
        crate::codegen::agent_skills::AgentSkillClient::new(
            &self.catalog_name,
            &self.schema_name,
            agent_skill_name,
            crate::codegen::agent_skills::AgentSkillServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
        )
    }
    /// Create a `agent_skill` within this resource.
    pub fn create_agent_skill(
        &self,
        name: impl Into<String>,
        agent_skill_type: AgentSkillType,
    ) -> crate::codegen::agent_skills::CreateAgentSkillBuilder {
        crate::codegen::agent_skills::CreateAgentSkillBuilder::new(
            crate::codegen::agent_skills::AgentSkillServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            &self.catalog_name,
            &self.schema_name,
            name,
            agent_skill_type,
        )
    }
    /// List `agent_skill` resources within this resource.
    pub fn list_agent_skills(&self) -> crate::codegen::agent_skills::ListAgentSkillsBuilder {
        crate::codegen::agent_skills::ListAgentSkillsBuilder::new(
            crate::codegen::agent_skills::AgentSkillServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            &self.catalog_name,
            &self.schema_name,
        )
    }
    /// Access a `function` within this resource.
    pub fn function(
        &self,
        function_name: impl Into<String>,
    ) -> crate::codegen::functions::FunctionClient {
        crate::codegen::functions::FunctionClient::new(
            &self.catalog_name,
            &self.schema_name,
            function_name,
            crate::codegen::functions::FunctionServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
        )
    }
    /// Create a `function` within this resource.
    pub fn create_function(
        &self,
        function_info: CreateFunction,
    ) -> crate::codegen::functions::CreateFunctionBuilder {
        crate::codegen::functions::CreateFunctionBuilder::new(
            crate::codegen::functions::FunctionServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            function_info,
        )
    }
    /// List `function` resources within this resource.
    pub fn list_functions(&self) -> crate::codegen::functions::ListFunctionsBuilder {
        crate::codegen::functions::ListFunctionsBuilder::new(
            crate::codegen::functions::FunctionServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            &self.catalog_name,
            &self.schema_name,
        )
    }
    /// Access a `registered_model` within this resource.
    pub fn registered_model(
        &self,
        registered_model_name: impl Into<String>,
    ) -> crate::codegen::registered_models::RegisteredModelClient {
        crate::codegen::registered_models::RegisteredModelClient::new(
            &self.catalog_name,
            &self.schema_name,
            registered_model_name,
            crate::codegen::registered_models::RegisteredModelServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
        )
    }
    /// Create a `registered_model` within this resource.
    pub fn create_registered_model(
        &self,
        name: impl Into<String>,
    ) -> crate::codegen::registered_models::CreateRegisteredModelBuilder {
        crate::codegen::registered_models::CreateRegisteredModelBuilder::new(
            crate::codegen::registered_models::RegisteredModelServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            name,
            &self.catalog_name,
            &self.schema_name,
        )
    }
    /// List `registered_model` resources within this resource.
    pub fn list_registered_models(
        &self,
    ) -> crate::codegen::registered_models::ListRegisteredModelsBuilder {
        crate::codegen::registered_models::ListRegisteredModelsBuilder::new(
            crate::codegen::registered_models::RegisteredModelServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
        )
    }
    /// Access a `table` within this resource.
    pub fn table(&self, table_name: impl Into<String>) -> crate::codegen::tables::TableClient {
        crate::codegen::tables::TableClient::new(
            &self.catalog_name,
            &self.schema_name,
            table_name,
            crate::codegen::tables::TableServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
        )
    }
    /// Create a `table` within this resource.
    pub fn create_table(
        &self,
        name: impl Into<String>,
        table_type: TableType,
        data_source_format: DataSourceFormat,
    ) -> crate::codegen::tables::CreateTableBuilder {
        crate::codegen::tables::CreateTableBuilder::new(
            crate::codegen::tables::TableServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            name,
            &self.schema_name,
            &self.catalog_name,
            table_type,
            data_source_format,
        )
    }
    /// List `table` resources within this resource.
    pub fn list_tables(&self) -> crate::codegen::tables::ListTablesBuilder {
        crate::codegen::tables::ListTablesBuilder::new(
            crate::codegen::tables::TableServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            &self.catalog_name,
            &self.schema_name,
        )
    }
    /// Access a `volume` within this resource.
    pub fn volume(&self, volume_name: impl Into<String>) -> crate::codegen::volumes::VolumeClient {
        crate::codegen::volumes::VolumeClient::new(
            &self.catalog_name,
            &self.schema_name,
            volume_name,
            crate::codegen::volumes::VolumeServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
        )
    }
    /// Create a `volume` within this resource.
    pub fn create_volume(
        &self,
        name: impl Into<String>,
        volume_type: VolumeType,
    ) -> crate::codegen::volumes::CreateVolumeBuilder {
        crate::codegen::volumes::CreateVolumeBuilder::new(
            crate::codegen::volumes::VolumeServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            &self.catalog_name,
            &self.schema_name,
            name,
            volume_type,
        )
    }
    /// List `volume` resources within this resource.
    pub fn list_volumes(&self) -> crate::codegen::volumes::ListVolumesBuilder {
        crate::codegen::volumes::ListVolumesBuilder::new(
            crate::codegen::volumes::VolumeServiceClient::new(
                self.client.client.clone(),
                self.client.base_url.clone(),
            ),
            &self.catalog_name,
            &self.schema_name,
        )
    }
}
