// @generated — do not edit by hand.
use crate::codegen::agent_skills::*;
use crate::codegen::agents::*;
use crate::codegen::catalogs::*;
use crate::codegen::credentials::*;
use crate::codegen::delta_commits::*;
use crate::codegen::entity_tag_assignments::*;
use crate::codegen::external_locations::*;
use crate::codegen::functions::*;
use crate::codegen::providers::*;
use crate::codegen::recipients::*;
use crate::codegen::schemas::*;
use crate::codegen::shares::*;
use crate::codegen::staging_tables::*;
use crate::codegen::tables::*;
use crate::codegen::tag_policies::*;
use crate::codegen::temporary_credentials::*;
use crate::codegen::volumes::*;
use olai_http::CloudClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
use unitycatalog_common::models::agents::v0alpha1::*;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::delta_commits::v1::*;
use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::providers::v1::*;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::shares::v1::*;
use unitycatalog_common::models::staging_tables::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::*;
use url::Url;
#[derive(Clone)]
pub struct UnityCatalogClient {
    client: CloudClient,
    base_url: Url,
}
impl UnityCatalogClient {
    /// Create a new aggregate client from a cloud client and base URL.
    ///
    /// Per-service clients are constructed on demand (they only hold a cheaply-cloneable
    /// `CloudClient` + `Url`), so nothing is allocated per service here.
    pub fn new(client: CloudClient, mut base_url: Url) -> Self {
        if !base_url.path().ends_with('/') {
            base_url.set_path(&format!("{}/", base_url.path()));
        }
        Self { client, base_url }
    }
    /// Create a new aggregate client with no authentication.
    pub fn new_unauthenticated(base_url: Url) -> Self {
        Self::new(CloudClient::new_unauthenticated(), base_url)
    }
    /// Create a new aggregate client authenticating with a bearer token.
    pub fn new_with_token(base_url: Url, token: impl ToString) -> Self {
        Self::new(CloudClient::new_with_token(token), base_url)
    }
    ///Low-level `agent_skills` client exposing request/response passthrough methods.
    pub fn agent_skills_client(&self) -> crate::codegen::agent_skills::AgentSkillServiceClient {
        crate::codegen::agent_skills::AgentSkillServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `agents` client exposing request/response passthrough methods.
    pub fn agents_client(&self) -> crate::codegen::agents::AgentServiceClient {
        crate::codegen::agents::AgentServiceClient::new(self.client.clone(), self.base_url.clone())
    }
    ///Low-level `catalogs` client exposing request/response passthrough methods.
    pub fn catalogs_client(&self) -> crate::codegen::catalogs::CatalogServiceClient {
        crate::codegen::catalogs::CatalogServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `credentials` client exposing request/response passthrough methods.
    pub fn credentials_client(&self) -> crate::codegen::credentials::CredentialServiceClient {
        crate::codegen::credentials::CredentialServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `delta_commits` client exposing request/response passthrough methods.
    pub fn delta_commits_client(&self) -> crate::codegen::delta_commits::DeltaCommitClient {
        crate::codegen::delta_commits::DeltaCommitClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `entity_tag_assignments` client exposing request/response passthrough methods.
    pub fn entity_tag_assignments_client(
        &self,
    ) -> crate::codegen::entity_tag_assignments::EntityTagAssignmentClient {
        crate::codegen::entity_tag_assignments::EntityTagAssignmentClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `external_locations` client exposing request/response passthrough methods.
    pub fn external_locations_client(
        &self,
    ) -> crate::codegen::external_locations::ExternalLocationServiceClient {
        crate::codegen::external_locations::ExternalLocationServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `functions` client exposing request/response passthrough methods.
    pub fn functions_client(&self) -> crate::codegen::functions::FunctionServiceClient {
        crate::codegen::functions::FunctionServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `providers` client exposing request/response passthrough methods.
    pub fn providers_client(&self) -> crate::codegen::providers::ProviderServiceClient {
        crate::codegen::providers::ProviderServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `recipients` client exposing request/response passthrough methods.
    pub fn recipients_client(&self) -> crate::codegen::recipients::RecipientServiceClient {
        crate::codegen::recipients::RecipientServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `schemas` client exposing request/response passthrough methods.
    pub fn schemas_client(&self) -> crate::codegen::schemas::SchemaServiceClient {
        crate::codegen::schemas::SchemaServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `shares` client exposing request/response passthrough methods.
    pub fn shares_client(&self) -> crate::codegen::shares::ShareServiceClient {
        crate::codegen::shares::ShareServiceClient::new(self.client.clone(), self.base_url.clone())
    }
    ///Low-level `staging_tables` client exposing request/response passthrough methods.
    pub fn staging_tables_client(
        &self,
    ) -> crate::codegen::staging_tables::StagingTableServiceClient {
        crate::codegen::staging_tables::StagingTableServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `tables` client exposing request/response passthrough methods.
    pub fn tables_client(&self) -> crate::codegen::tables::TableServiceClient {
        crate::codegen::tables::TableServiceClient::new(self.client.clone(), self.base_url.clone())
    }
    ///Low-level `tag_policies` client exposing request/response passthrough methods.
    pub fn tag_policies_client(&self) -> crate::codegen::tag_policies::TagPolicyServiceClient {
        crate::codegen::tag_policies::TagPolicyServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `temporary_credentials` client exposing request/response passthrough methods.
    pub fn temporary_credentials_client(
        &self,
    ) -> crate::codegen::temporary_credentials::TemporaryCredentialClient {
        crate::codegen::temporary_credentials::TemporaryCredentialClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    ///Low-level `volumes` client exposing request/response passthrough methods.
    pub fn volumes_client(&self) -> crate::codegen::volumes::VolumeServiceClient {
        crate::codegen::volumes::VolumeServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        )
    }
    /// Lists agent skills.
    ///
    /// # Arguments
    ///
    /// * `catalog_name` - The identifier of the catalog.
    /// * `schema_name` - The identifier of the schema.
    pub fn list_agent_skills(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> ListAgentSkillsBuilder {
        ListAgentSkillsBuilder::new(
            crate::codegen::agent_skills::AgentSkillServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
            schema_name,
        )
    }
    /// # Arguments
    ///
    /// * `catalog_name` - The identifier of the catalog.
    /// * `schema_name` - The identifier of the schema.
    /// * `name` - The identifier of the agent skill.
    /// * `agent_skill_type` - How the storage location is provisioned (external or managed).
    pub fn create_agent_skill(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        agent_skill_type: AgentSkillType,
    ) -> CreateAgentSkillBuilder {
        CreateAgentSkillBuilder::new(
            crate::codegen::agent_skills::AgentSkillServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
            schema_name,
            name,
            agent_skill_type,
        )
    }
    /// Access the `agent_skill` resource scoped to the given name.
    pub fn agent_skill(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        agent_skill_name: impl Into<String>,
    ) -> AgentSkillClient {
        AgentSkillClient::new(
            catalog_name,
            schema_name,
            agent_skill_name,
            crate::codegen::agent_skills::AgentSkillServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Access the `agent_skill` resource from its dot-joined full name.
    pub fn agent_skill_from_full_name(&self, full_name: impl Into<String>) -> AgentSkillClient {
        AgentSkillClient::from_full_name(
            full_name,
            crate::codegen::agent_skills::AgentSkillServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Lists agents.
    ///
    /// # Arguments
    ///
    /// * `catalog_name` - The identifier of the catalog.
    /// * `schema_name` - The identifier of the schema.
    pub fn list_agents(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> ListAgentsBuilder {
        ListAgentsBuilder::new(
            crate::codegen::agents::AgentServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
            schema_name,
        )
    }
    /// # Arguments
    ///
    /// * `catalog_name` - The identifier of the catalog.
    /// * `schema_name` - The identifier of the schema.
    /// * `name` - The identifier of the agent.
    /// * `invocation_protocol` - The protocol a recipient uses to invoke the agent.
    /// * `endpoint` - The agent's invocation endpoint URL.
    pub fn create_agent(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        invocation_protocol: InvocationProtocol,
        endpoint: impl Into<String>,
    ) -> CreateAgentBuilder {
        CreateAgentBuilder::new(
            crate::codegen::agents::AgentServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
            schema_name,
            name,
            invocation_protocol,
            endpoint,
        )
    }
    /// Access the `agent` resource scoped to the given name.
    pub fn agent(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        agent_name: impl Into<String>,
    ) -> AgentClient {
        AgentClient::new(
            catalog_name,
            schema_name,
            agent_name,
            crate::codegen::agents::AgentServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Access the `agent` resource from its dot-joined full name.
    pub fn agent_from_full_name(&self, full_name: impl Into<String>) -> AgentClient {
        AgentClient::from_full_name(
            full_name,
            crate::codegen::agents::AgentServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// List catalogs
    ///
    /// Gets an array of catalogs in the metastore. If the caller is the metastore admin,
    /// all catalogs will be retrieved. Otherwise, only catalogs owned by the caller
    /// (or for which the caller has the USE_CATALOG privilege) will be retrieved.
    /// There is no guarantee of a specific ordering of the elements in the array.
    pub fn list_catalogs(&self) -> ListCatalogsBuilder {
        ListCatalogsBuilder::new(crate::codegen::catalogs::CatalogServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        ))
    }
    /// Create a new catalog
    ///
    /// Creates a new catalog instance in the parent metastore if the caller
    /// is a metastore admin or has the CREATE_CATALOG privilege.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of catalog.
    pub fn create_catalog(&self, name: impl Into<String>) -> CreateCatalogBuilder {
        CreateCatalogBuilder::new(
            crate::codegen::catalogs::CatalogServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
        )
    }
    /// Access the `catalog` resource scoped to the given name.
    pub fn catalog(&self, catalog_name: impl Into<String>) -> CatalogClient {
        CatalogClient::new(
            catalog_name,
            crate::codegen::catalogs::CatalogServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    pub fn list_credentials(&self) -> ListCredentialsBuilder {
        ListCredentialsBuilder::new(crate::codegen::credentials::CredentialServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        ))
    }
    /// # Arguments
    ///
    /// * `name` - The credential name. The name must be unique among storage and service credentials within the metastore.
    /// * `purpose` - Indicates the purpose of the credential.
    pub fn create_credential(
        &self,
        name: impl Into<String>,
        purpose: Purpose,
    ) -> CreateCredentialBuilder {
        CreateCredentialBuilder::new(
            crate::codegen::credentials::CredentialServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
            purpose,
        )
    }
    /// Access the `credential` resource scoped to the given name.
    pub fn credential(&self, credential_name: impl Into<String>) -> CredentialClient {
        CredentialClient::new(
            credential_name,
            crate::codegen::credentials::CredentialServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Ratify a staged commit at the requested version (first-writer-wins), and/or
    /// notify the catalog that commits have been backfilled to the Delta log.
    ///
    /// # Arguments
    ///
    /// * `table_id` - UUID of the catalog-managed table being committed to.
    /// * `table_uri` - The storage URI of the table. Must match the table's registered storage
    /// location (normalized) on the commit path.
    pub fn commit(
        &self,
        table_id: impl Into<String>,
        table_uri: impl Into<String>,
    ) -> CommitBuilder {
        CommitBuilder::new(
            crate::codegen::delta_commits::DeltaCommitClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            table_id,
            table_uri,
        )
    }
    /// Return ratified-but-unpublished commits for a table, plus the latest
    /// version the catalog tracks.
    ///
    /// # Arguments
    ///
    /// * `table_id` - UUID of the catalog-managed table.
    /// * `table_uri` - The storage URI of the table.
    /// * `start_version` - The lowest version to return (inclusive). Defaults to 0.
    pub fn get_commits(
        &self,
        table_id: impl Into<String>,
        table_uri: impl Into<String>,
        start_version: i64,
    ) -> GetCommitsBuilder {
        GetCommitsBuilder::new(
            crate::codegen::delta_commits::DeltaCommitClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            table_id,
            table_uri,
            start_version,
        )
    }
    /// List entity tag assignments
    ///
    /// Gets the tag assignments for the specified entity.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - The type of the entity whose tag assignments to list.
    /// * `entity_name` - The fully qualified name of the entity whose tag assignments to list.
    pub fn list_entity_tag_assignments(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
    ) -> ListEntityTagAssignmentsBuilder {
        ListEntityTagAssignmentsBuilder::new(
            crate::codegen::entity_tag_assignments::EntityTagAssignmentClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            entity_type,
            entity_name,
        )
    }
    /// Create an entity tag assignment
    ///
    /// Assigns a tag to a Unity Catalog entity.
    ///
    /// # Arguments
    ///
    /// * `tag_assignment` - The tag assignment to create.
    pub fn create_entity_tag_assignment(
        &self,
        tag_assignment: EntityTagAssignment,
    ) -> CreateEntityTagAssignmentBuilder {
        CreateEntityTagAssignmentBuilder::new(
            crate::codegen::entity_tag_assignments::EntityTagAssignmentClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            tag_assignment,
        )
    }
    /// Get an entity tag assignment
    ///
    /// Gets the tag assignment for the specified entity and tag key.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - The type of the entity to which the tag is assigned.
    /// * `entity_name` - The fully qualified name of the entity to which the tag is assigned.
    /// * `tag_key` - The key of the tag.
    pub fn get_entity_tag_assignment(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
    ) -> GetEntityTagAssignmentBuilder {
        GetEntityTagAssignmentBuilder::new(
            crate::codegen::entity_tag_assignments::EntityTagAssignmentClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            entity_type,
            entity_name,
            tag_key,
        )
    }
    /// Update an entity tag assignment
    ///
    /// Updates the tag assignment for the specified entity and tag key.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - The type of the entity to which the tag is assigned.
    /// * `entity_name` - The fully qualified name of the entity to which the tag is assigned.
    /// * `tag_key` - The key of the tag to update.
    /// * `tag_assignment` - The tag assignment with the updated fields.
    pub fn update_entity_tag_assignment(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
        tag_assignment: EntityTagAssignment,
    ) -> UpdateEntityTagAssignmentBuilder {
        UpdateEntityTagAssignmentBuilder::new(
            crate::codegen::entity_tag_assignments::EntityTagAssignmentClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            entity_type,
            entity_name,
            tag_key,
            tag_assignment,
        )
    }
    /// Delete an entity tag assignment
    ///
    /// Deletes the tag assignment for the specified entity and tag key.
    ///
    /// # Arguments
    ///
    /// * `entity_type` - The type of the entity to which the tag is assigned.
    /// * `entity_name` - The fully qualified name of the entity to which the tag is assigned.
    /// * `tag_key` - The key of the tag to delete.
    pub fn delete_entity_tag_assignment(
        &self,
        entity_type: impl Into<String>,
        entity_name: impl Into<String>,
        tag_key: impl Into<String>,
    ) -> DeleteEntityTagAssignmentBuilder {
        DeleteEntityTagAssignmentBuilder::new(
            crate::codegen::entity_tag_assignments::EntityTagAssignmentClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            entity_type,
            entity_name,
            tag_key,
        )
    }
    /// List external locations
    pub fn list_external_locations(&self) -> ListExternalLocationsBuilder {
        ListExternalLocationsBuilder::new(
            crate::codegen::external_locations::ExternalLocationServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Create a new external location
    ///
    /// # Arguments
    ///
    /// * `name` - Name of external location.
    /// * `url` - Path URL of the external location.
    /// * `credential_name` - Name of the storage credential used with this location.
    pub fn create_external_location(
        &self,
        name: impl Into<String>,
        url: impl Into<String>,
        credential_name: impl Into<String>,
    ) -> CreateExternalLocationBuilder {
        CreateExternalLocationBuilder::new(
            crate::codegen::external_locations::ExternalLocationServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
            url,
            credential_name,
        )
    }
    /// Access the `external_location` resource scoped to the given name.
    pub fn external_location(
        &self,
        external_location_name: impl Into<String>,
    ) -> ExternalLocationClient {
        ExternalLocationClient::new(
            external_location_name,
            crate::codegen::external_locations::ExternalLocationServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// List functions
    ///
    /// List functions within the specified parent catalog and schema. If the caller is the metastore
    /// admin, all functions are returned in the response. Otherwise, the caller must have USE_CATALOG
    /// on the parent catalog and USE_SCHEMA on the parent schema, and the function must either be
    /// owned by the caller or have SELECT on the function.
    ///
    /// # Arguments
    ///
    /// * `catalog_name` - Name of parent catalog for functions of interest.
    /// * `schema_name` - Parent schema of functions.
    pub fn list_functions(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> ListFunctionsBuilder {
        ListFunctionsBuilder::new(
            crate::codegen::functions::FunctionServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
            schema_name,
        )
    }
    /// Create a function
    ///
    /// Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION
    /// privilege on the parent catalog and schema.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of function, relative to parent schema.
    /// * `catalog_name` - Name of parent catalog.
    /// * `schema_name` - Name of parent schema.
    /// * `data_type` - Full data type specification of the return type of the function.
    /// * `full_data_type` - Full data type specification as SQL/catalogString text.
    /// * `parameter_style` - The parameter-passing style.
    /// * `is_deterministic` - Indicates whether the function is deterministic.
    /// * `sql_data_access` - SQL data access information.
    /// * `is_null_call` - Indicates whether the function is null-calling.
    /// * `security_type` - The security type of the function.
    /// * `routine_body` - The routine body.
    pub fn create_function(
        &self,
        name: impl Into<String>,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        data_type: impl Into<String>,
        full_data_type: impl Into<String>,
        parameter_style: ParameterStyle,
        is_deterministic: bool,
        sql_data_access: SqlDataAccess,
        is_null_call: bool,
        security_type: SecurityType,
        routine_body: RoutineBody,
    ) -> CreateFunctionBuilder {
        CreateFunctionBuilder::new(
            crate::codegen::functions::FunctionServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
            catalog_name,
            schema_name,
            data_type,
            full_data_type,
            parameter_style,
            is_deterministic,
            sql_data_access,
            is_null_call,
            security_type,
            routine_body,
        )
    }
    /// Access the `function` resource scoped to the given name.
    pub fn function(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        function_name: impl Into<String>,
    ) -> FunctionClient {
        FunctionClient::new(
            catalog_name,
            schema_name,
            function_name,
            crate::codegen::functions::FunctionServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Access the `function` resource from its dot-joined full name.
    pub fn function_from_full_name(&self, full_name: impl Into<String>) -> FunctionClient {
        FunctionClient::from_full_name(
            full_name,
            crate::codegen::functions::FunctionServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// List providers.
    pub fn list_providers(&self) -> ListProvidersBuilder {
        ListProvidersBuilder::new(crate::codegen::providers::ProviderServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        ))
    }
    /// Create a new provider.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the provider.
    /// * `authentication_type` - The delta sharing authentication type.
    pub fn create_provider(
        &self,
        name: impl Into<String>,
        authentication_type: ProviderAuthenticationType,
    ) -> CreateProviderBuilder {
        CreateProviderBuilder::new(
            crate::codegen::providers::ProviderServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
            authentication_type,
        )
    }
    /// Access the `provider` resource scoped to the given name.
    pub fn provider(&self, provider_name: impl Into<String>) -> ProviderClient {
        ProviderClient::new(
            provider_name,
            crate::codegen::providers::ProviderServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// List recipients.
    pub fn list_recipients(&self) -> ListRecipientsBuilder {
        ListRecipientsBuilder::new(crate::codegen::recipients::RecipientServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        ))
    }
    /// Create a new recipient.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the recipient.
    /// * `authentication_type` - The delta sharing authentication type.
    /// * `owner` - Username of the recipient owner.
    pub fn create_recipient(
        &self,
        name: impl Into<String>,
        authentication_type: AuthenticationType,
        owner: impl Into<String>,
    ) -> CreateRecipientBuilder {
        CreateRecipientBuilder::new(
            crate::codegen::recipients::RecipientServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
            authentication_type,
            owner,
        )
    }
    /// Access the `recipient` resource scoped to the given name.
    pub fn recipient(&self, recipient_name: impl Into<String>) -> RecipientClient {
        RecipientClient::new(
            recipient_name,
            crate::codegen::recipients::RecipientServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Gets an array of schemas for a catalog in the metastore. If the caller is the metastore
    /// admin or the owner of the parent catalog, all schemas for the catalog will be retrieved.
    /// Otherwise, only schemas owned by the caller (or for which the caller has the USE_SCHEMA privilege)
    /// will be retrieved. There is no guarantee of a specific ordering of the elements in the array.
    ///
    /// # Arguments
    ///
    /// * `catalog_name` - Name of parent catalog.
    pub fn list_schemas(&self, catalog_name: impl Into<String>) -> ListSchemasBuilder {
        ListSchemasBuilder::new(
            crate::codegen::schemas::SchemaServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
        )
    }
    /// Creates a new schema for catalog in the Metatastore. The caller must be a metastore admin,
    /// or have the CREATE_SCHEMA privilege in the parent catalog.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of schema, relative to parent catalog.
    /// * `catalog_name` - Name of parent catalog.
    pub fn create_schema(
        &self,
        name: impl Into<String>,
        catalog_name: impl Into<String>,
    ) -> CreateSchemaBuilder {
        CreateSchemaBuilder::new(
            crate::codegen::schemas::SchemaServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
            catalog_name,
        )
    }
    /// Access the `schema` resource scoped to the given name.
    pub fn schema(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> SchemaClient {
        SchemaClient::new(
            catalog_name,
            schema_name,
            crate::codegen::schemas::SchemaServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Access the `schema` resource from its dot-joined full name.
    pub fn schema_from_full_name(&self, full_name: impl Into<String>) -> SchemaClient {
        SchemaClient::from_full_name(
            full_name,
            crate::codegen::schemas::SchemaServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// List shares.
    pub fn list_shares(&self) -> ListSharesBuilder {
        ListSharesBuilder::new(crate::codegen::shares::ShareServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        ))
    }
    /// Create a new share.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the share.
    pub fn create_share(&self, name: impl Into<String>) -> CreateShareBuilder {
        CreateShareBuilder::new(
            crate::codegen::shares::ShareServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
        )
    }
    /// Access the `share` resource scoped to the given name.
    pub fn share(&self, share_name: impl Into<String>) -> ShareClient {
        ShareClient::new(
            share_name,
            crate::codegen::shares::ShareServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Creates a new staging table, allocating an immutable table id and a storage
    /// location under the parent schema/catalog managed storage root. The caller
    /// must have the CREATE privilege on the parent schema.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the staging table, relative to the parent schema.
    /// * `catalog_name` - Name of the parent catalog.
    /// * `schema_name` - Name of the parent schema relative to its parent catalog.
    pub fn create_staging_table(
        &self,
        name: impl Into<String>,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> CreateStagingTableBuilder {
        CreateStagingTableBuilder::new(
            crate::codegen::staging_tables::StagingTableServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
            catalog_name,
            schema_name,
        )
    }
    /// Access the `staging_table` resource scoped to the given name.
    pub fn staging_table(&self, staging_table_name: impl Into<String>) -> StagingTableClient {
        StagingTableClient::new(
            staging_table_name,
            crate::codegen::staging_tables::StagingTableServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Gets an array of summaries for tables for a schema and catalog within the metastore. The table summaries returned are either:
    /// - summaries for tables (within the current metastore and parent catalog and schema), when the user is a metastore admin, or:
    /// - summaries for tables and schemas (within the current metastore and parent catalog) for which the user has ownership or the
    /// SELECT privilege on the table and ownership or USE_SCHEMA privilege on the schema, provided that the user also has ownership
    /// or the USE_CATALOG privilege on the parent catalog.
    ///
    /// There is no guarantee of a specific ordering of the elements in the array.
    ///
    /// # Arguments
    ///
    /// * `catalog_name` - Name of parent catalog for tables of interest.
    pub fn list_table_summaries(
        &self,
        catalog_name: impl Into<String>,
    ) -> ListTableSummariesBuilder {
        ListTableSummariesBuilder::new(
            crate::codegen::tables::TableServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
        )
    }
    /// Gets an array of all tables for the current metastore under the parent catalog and schema.
    ///
    /// The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table.
    /// For the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the
    /// parent catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a
    /// specific ordering of the elements in the array.
    ///
    /// # Arguments
    ///
    /// * `catalog_name` - Name of parent catalog for tables of interest.
    /// * `schema_name` - Name of parent schema for tables of interest.
    pub fn list_tables(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> ListTablesBuilder {
        ListTablesBuilder::new(
            crate::codegen::tables::TableServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
            schema_name,
        )
    }
    /// Create a table
    ///
    /// # Arguments
    ///
    /// * `name` - Name of table, relative to parent schema.
    /// * `schema_name` - Name of parent schema relative to its parent catalog.
    /// * `catalog_name` - Name of parent catalog.
    pub fn create_table(
        &self,
        name: impl Into<String>,
        schema_name: impl Into<String>,
        catalog_name: impl Into<String>,
        table_type: TableType,
        data_source_format: DataSourceFormat,
    ) -> CreateTableBuilder {
        CreateTableBuilder::new(
            crate::codegen::tables::TableServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            name,
            schema_name,
            catalog_name,
            table_type,
            data_source_format,
        )
    }
    /// Access the `table` resource scoped to the given name.
    pub fn table(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        table_name: impl Into<String>,
    ) -> TableClient {
        TableClient::new(
            catalog_name,
            schema_name,
            table_name,
            crate::codegen::tables::TableServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Access the `table` resource from its dot-joined full name.
    pub fn table_from_full_name(&self, full_name: impl Into<String>) -> TableClient {
        TableClient::from_full_name(
            full_name,
            crate::codegen::tables::TableServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// List tag policies
    ///
    /// Gets an array of tag policies. There is no guarantee of a specific ordering
    /// of the elements in the array.
    pub fn list_tag_policies(&self) -> ListTagPoliciesBuilder {
        ListTagPoliciesBuilder::new(crate::codegen::tag_policies::TagPolicyServiceClient::new(
            self.client.clone(),
            self.base_url.clone(),
        ))
    }
    /// Create a new tag policy
    ///
    /// Creates a new governed tag definition.
    ///
    /// # Arguments
    ///
    /// * `tag_policy` - The tag policy to create.
    pub fn create_tag_policy(&self, tag_policy: TagPolicy) -> CreateTagPolicyBuilder {
        CreateTagPolicyBuilder::new(
            crate::codegen::tag_policies::TagPolicyServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            tag_policy,
        )
    }
    /// Access the `tag_policy` resource scoped to the given name.
    pub fn tag_policy(&self, tag_policy_name: impl Into<String>) -> TagPolicyClient {
        TagPolicyClient::new(
            tag_policy_name,
            crate::codegen::tag_policies::TagPolicyServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Generate a new set of credentials for a table.
    ///
    /// # Arguments
    ///
    /// * `table_id` - UUID of the table to read or write.
    /// * `operation` - The operation performed against the table data, either READ or READ_WRITE.
    /// If READ_WRITE is specified, the credentials returned will have write
    /// permissions, otherwise, it will be read only.
    pub fn generate_temporary_table_credentials(
        &self,
        table_id: impl Into<String>,
        operation: generate_temporary_table_credentials_request::Operation,
    ) -> GenerateTemporaryTableCredentialsBuilder {
        GenerateTemporaryTableCredentialsBuilder::new(
            crate::codegen::temporary_credentials::TemporaryCredentialClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            table_id,
            operation,
        )
    }
    /// Generate a new set of credentials for a path.
    ///
    /// # Arguments
    ///
    /// * `url` - URL for path-based access.
    /// * `operation` - The operation being performed on the path.
    pub fn generate_temporary_path_credentials(
        &self,
        url: impl Into<String>,
        operation: generate_temporary_path_credentials_request::Operation,
    ) -> GenerateTemporaryPathCredentialsBuilder {
        GenerateTemporaryPathCredentialsBuilder::new(
            crate::codegen::temporary_credentials::TemporaryCredentialClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            url,
            operation,
        )
    }
    /// Generate a new set of credentials for a volume.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true
    /// (default false). The caller must have the `EXTERNAL_USE_SCHEMA`
    /// privilege on the parent schema (granted by a catalog owner).
    ///
    /// # Arguments
    ///
    /// * `volume_id` - UUID of the volume to read or write.
    /// * `operation` - The operation performed against the volume data, either READ_VOLUME or
    /// WRITE_VOLUME. If WRITE_VOLUME is specified, the credentials returned will
    /// have write permissions, otherwise, it will be read only.
    pub fn generate_temporary_volume_credentials(
        &self,
        volume_id: impl Into<String>,
        operation: generate_temporary_volume_credentials_request::Operation,
    ) -> GenerateTemporaryVolumeCredentialsBuilder {
        GenerateTemporaryVolumeCredentialsBuilder::new(
            crate::codegen::temporary_credentials::TemporaryCredentialClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            volume_id,
            operation,
        )
    }
    /// Lists volumes.
    ///
    /// # Arguments
    ///
    /// * `catalog_name` - The identifier of the catalog
    /// * `schema_name` - The identifier of the schema
    pub fn list_volumes(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
    ) -> ListVolumesBuilder {
        ListVolumesBuilder::new(
            crate::codegen::volumes::VolumeServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
            schema_name,
        )
    }
    /// # Arguments
    ///
    /// * `catalog_name` - The identifier of the catalog
    /// * `schema_name` - The identifier of the schema
    /// * `name` - The identifier of the volume
    /// * `volume_type` - The type of the volume.
    ///
    /// An external volume is located in the specified external location.
    /// A managed volume is located in the default location which is specified
    /// by the parent schema, or the parent catalog, or the Metastore.
    pub fn create_volume(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        name: impl Into<String>,
        volume_type: VolumeType,
    ) -> CreateVolumeBuilder {
        CreateVolumeBuilder::new(
            crate::codegen::volumes::VolumeServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
            catalog_name,
            schema_name,
            name,
            volume_type,
        )
    }
    /// Access the `volume` resource scoped to the given name.
    pub fn volume(
        &self,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        volume_name: impl Into<String>,
    ) -> VolumeClient {
        VolumeClient::new(
            catalog_name,
            schema_name,
            volume_name,
            crate::codegen::volumes::VolumeServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
    /// Access the `volume` resource from its dot-joined full name.
    pub fn volume_from_full_name(&self, full_name: impl Into<String>) -> VolumeClient {
        VolumeClient::from_full_name(
            full_name,
            crate::codegen::volumes::VolumeServiceClient::new(
                self.client.clone(),
                self.base_url.clone(),
            ),
        )
    }
}
