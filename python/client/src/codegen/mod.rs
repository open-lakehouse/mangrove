// @generated — do not edit by hand.
#![allow(dead_code, unused_imports, clippy::too_many_arguments)]

#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod agent_skills;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod agents;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod catalogs;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod credentials;
pub mod error;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod external_locations;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod functions;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod policies;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod providers;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod recipients;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod registered_models;
pub mod runtime;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod schemas;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod shares;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod staging_tables;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod tables;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod tag_policies;
#[allow(dead_code, unused_imports, clippy::too_many_arguments)]
pub mod volumes;
use crate::codegen::agent_skills::PyAgentSkillClient;
use crate::codegen::agents::PyAgentClient;
use crate::codegen::catalogs::PyCatalogClient;
use crate::codegen::credentials::PyCredentialClient;
use crate::codegen::error::{PyUnityCatalogError, PyUnityCatalogResult};
use crate::codegen::external_locations::PyExternalLocationClient;
use crate::codegen::functions::PyFunctionClient;
use crate::codegen::policies::PyPolicyClient;
use crate::codegen::providers::PyProviderClient;
use crate::codegen::recipients::PyRecipientClient;
use crate::codegen::registered_models::PyRegisteredModelClient;
use crate::codegen::schemas::PySchemaClient;
use crate::codegen::shares::PyShareClient;
use crate::codegen::staging_tables::PyStagingTableClient;
use crate::codegen::tables::PyTableClient;
use crate::codegen::tag_policies::PyTagPolicyClient;
use crate::codegen::volumes::PyVolumeClient;
use futures::stream::TryStreamExt;
use pyo3::prelude::*;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
use unitycatalog_common::models::agents::v0alpha1::*;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::model_versions::v1::*;
use unitycatalog_common::models::policies::v1::*;
use unitycatalog_common::models::providers::v1::*;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::registered_models::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::shares::v1::*;
use unitycatalog_common::models::staging_tables::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::*;
use unitycatalog_common::models::*;

/// A client of the API at one base URL.
#[pyclass(name = "UnityCatalogClient", module = "unitycatalog_client._client")]
pub struct PyUnityCatalogClient {
    pub(crate) client: UnityCatalogClient,
}

#[pymethods]
impl PyUnityCatalogClient {
    /// A client of the API at `base_url`. `token`, when set, goes in every
    /// request as the bearer token.
    #[new]
    #[pyo3(signature = (base_url, token = None))]
    pub fn new(base_url: String, token: Option<String>) -> PyResult<Self> {
        let base_url = base_url.parse().map_err(PyUnityCatalogError::from)?;
        let client = match token {
            Some(token) => UnityCatalogClient::new_with_token(base_url, token),
            None => UnityCatalogClient::new_unauthenticated(base_url),
        };
        Ok(Self { client })
    }

    /// Lists agent skills.
    ///
    /// Args:
    ///     catalog_name: The identifier of the catalog.
    ///     schema_name: The identifier of the schema.
    ///     max_results: The maximum number of results per page that should be returned.
    ///     include_browse: Whether to include agent skills in the response for which the principal can only
    ///                     access selective metadata for.
    ///
    /// Returns:
    ///     The agent skills returned.
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_agent_skills(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyAgentSkill>> {
        let request = self
            .client
            .list_agent_skills(catalog_name, schema_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyAgentSkill::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Args:
    ///     catalog_name: The identifier of the catalog.
    ///     schema_name: The identifier of the schema.
    ///     name: The identifier of the agent skill.
    ///     agent_skill_type: How the storage location is provisioned (external or managed).
    ///     storage_location: The storage location of the skill directory on the cloud. Required for
    ///                       EXTERNAL skills; ignored (server-derived) for MANAGED skills.
    ///     description: A human-readable description of what the skill does and when to use it.
    ///     license: SPDX license identifier or free-form license text for the skill.
    ///     allowed_tools: The tools the skill is permitted to use.
    ///     metadata: Arbitrary additional metadata declared by the skill.
    ///     comment: User-provided free-form text description.
    ///
    /// Returns:
    ///     An agent skill registered in Unity Catalog.
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            name,
            agent_skill_type,
            storage_location = None,
            description = None,
            license = None,
            allowed_tools = None,
            metadata = None,
            comment = None
        )
    )]
    pub fn create_agent_skill(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        name: String,
        agent_skill_type: PyAgentSkillType,
        storage_location: Option<String>,
        description: Option<String>,
        license: Option<String>,
        allowed_tools: Option<Vec<String>>,
        metadata: Option<HashMap<String, String>>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<PyAgentSkill> {
        let request = self
            .client
            .create_agent_skill(catalog_name, schema_name, name, agent_skill_type.into())
            .with_storage_location(storage_location)
            .with_description(description)
            .with_license(license)
            .with_allowed_tools(allowed_tools.unwrap_or_default())
            .with_metadata(metadata.unwrap_or_default())
            .with_comment(comment);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgentSkill::from(result))
        })
    }

    /// Lists agents.
    ///
    /// Args:
    ///     catalog_name: The identifier of the catalog.
    ///     schema_name: The identifier of the schema.
    ///     max_results: The maximum number of results per page that should be returned.
    ///     include_browse: Whether to include agents in the response for which the principal can only
    ///                     access selective metadata for.
    ///
    /// Returns:
    ///     The agents returned.
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_agents(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyAgent>> {
        let request = self
            .client
            .list_agents(catalog_name, schema_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyAgent::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Args:
    ///     catalog_name: The identifier of the catalog.
    ///     schema_name: The identifier of the schema.
    ///     name: The identifier of the agent.
    ///     invocation_protocol: The protocol a recipient uses to invoke the agent.
    ///     endpoint: The agent's invocation endpoint URL.
    ///     description: An LLM-readable description of what the agent does and the inputs it expects.
    ///     capabilities: Capability identifiers advertised by the agent.
    ///     input_schema: A JSON Schema (encoded as a JSON string) describing the expected input.
    ///     comment: User-provided free-form text description.
    ///
    /// Returns:
    ///     An agent registered in Unity Catalog.
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            name,
            invocation_protocol,
            endpoint,
            description = None,
            capabilities = None,
            input_schema = None,
            comment = None
        )
    )]
    pub fn create_agent(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        name: String,
        invocation_protocol: PyInvocationProtocol,
        endpoint: String,
        description: Option<String>,
        capabilities: Option<Vec<String>>,
        input_schema: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<PyAgent> {
        let request = self
            .client
            .create_agent(
                catalog_name,
                schema_name,
                name,
                invocation_protocol.into(),
                endpoint,
            )
            .with_description(description)
            .with_capabilities(capabilities.unwrap_or_default())
            .with_input_schema(input_schema)
            .with_comment(comment);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyAgent::from(result))
        })
    }

    /// List catalogs
    ///
    /// Gets an array of catalogs in the metastore. If the caller is the metastore admin, all catalogs
    /// will be retrieved. Otherwise, only catalogs owned by the caller (or for which the caller has the
    /// USE_CATALOG privilege) will be retrieved. There is no guarantee of a specific ordering of the
    /// elements in the array.
    ///
    /// Args:
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///
    /// Returns:
    ///     The catalogs returned.
    #[pyo3(signature = (max_results = None))]
    pub fn list_catalogs(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyCatalog>> {
        let request = self.client.list_catalogs().with_max_results(max_results);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyCatalog::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a new catalog
    ///
    /// Creates a new catalog instance in the parent metastore if the caller is a metastore admin or has the
    /// CREATE_CATALOG privilege.
    ///
    /// Args:
    ///     name: Name of catalog.
    ///     comment: User-provided free-form text description.
    ///     properties: A map of key-value properties attached to the securable.
    ///     storage_root: Storage root URL for managed tables within catalog.
    ///     provider_name: The name of delta sharing provider. A Delta Sharing catalog is a catalog that is
    ///                    based on a Delta share on a remote sharing server.
    ///     share_name: The name of the share under the share provider.
    ///
    /// Returns:
    ///     A catalog is a root-level namespace that contains schemas.
    #[pyo3(
        signature = (
            name,
            comment = None,
            properties = None,
            storage_root = None,
            provider_name = None,
            share_name = None
        )
    )]
    pub fn create_catalog(
        &self,
        py: Python,
        name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
        provider_name: Option<String>,
        share_name: Option<String>,
    ) -> PyUnityCatalogResult<PyCatalog> {
        let request = self
            .client
            .create_catalog(name)
            .with_comment(comment)
            .with_properties(properties.unwrap_or_default())
            .with_storage_root(storage_root)
            .with_provider_name(provider_name)
            .with_share_name(share_name);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyCatalog::from(result))
        })
    }

    /// Args:
    ///     purpose: Return only credentials for the specified purpose.
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///
    /// Returns:
    ///     The credentials returned.
    #[pyo3(signature = (purpose = None, max_results = None))]
    pub fn list_credentials(
        &self,
        py: Python,
        purpose: ::core::option::Option<PyPurpose>,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyCredential>> {
        let request = self
            .client
            .list_credentials()
            .with_purpose(purpose.map(::core::convert::Into::into))
            .with_max_results(max_results);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyCredential::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Args:
    ///     name: The credential name. The name must be unique among storage and service credentials within
    ///           the metastore.
    ///     purpose: Indicates the purpose of the credential.
    ///     comment: Comment associated with the credential.
    ///     read_only: Whether the credential is usable only for read operations. Only applicable when
    ///                purpose is STORAGE.
    ///     skip_validation: Supplying true to this argument skips validation of the created set of
    ///                      credentials. Defaults to false.
    ///     azure_service_principal: The Azure service principal configuration.
    ///     azure_managed_identity: The Azure managed identity configuration.
    ///     azure_storage_key: The Azure storage key configuration.
    ///     aws_iam_role: The AWS IAM role configuration.
    ///     databricks_gcp_service_account: The Databricks managed GCP service account configuration.
    ///
    /// Returns:
    ///     A credential used to access external data sources or services.
    #[pyo3(
        signature = (
            name,
            purpose,
            comment = None,
            read_only = None,
            skip_validation = None,
            azure_service_principal = None,
            azure_managed_identity = None,
            azure_storage_key = None,
            aws_iam_role = None,
            databricks_gcp_service_account = None
        )
    )]
    pub fn create_credential(
        &self,
        py: Python,
        name: String,
        purpose: PyPurpose,
        comment: Option<String>,
        read_only: Option<bool>,
        skip_validation: Option<bool>,
        azure_service_principal: ::core::option::Option<PyAzureServicePrincipal>,
        azure_managed_identity: ::core::option::Option<PyAzureManagedIdentity>,
        azure_storage_key: ::core::option::Option<PyAzureStorageKey>,
        aws_iam_role: ::core::option::Option<PyAwsIamRoleConfig>,
        databricks_gcp_service_account: ::core::option::Option<PyDatabricksGcpServiceAccount>,
    ) -> PyUnityCatalogResult<PyCredential> {
        let request = self
            .client
            .create_credential(name, purpose.into())
            .with_comment(comment)
            .with_read_only(read_only)
            .with_skip_validation(skip_validation)
            .with_azure_service_principal(azure_service_principal.map(::core::convert::Into::into))
            .with_azure_managed_identity(azure_managed_identity.map(::core::convert::Into::into))
            .with_azure_storage_key(azure_storage_key.map(::core::convert::Into::into))
            .with_aws_iam_role(aws_iam_role.map(::core::convert::Into::into))
            .with_databricks_gcp_service_account(
                databricks_gcp_service_account.map(::core::convert::Into::into),
            );
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyCredential::from(result))
        })
    }

    /// List entity tag assignments
    ///
    /// Gets the tag assignments for the specified entity.
    ///
    /// Args:
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///     page_token: Opaque pagination token to go to next page based on previous query.
    ///
    /// Returns:
    ///     List entity tag assignments response.
    #[pyo3(
        signature = (entity_type, entity_name, max_results = None, page_token = None)
    )]
    pub fn list_entity_tag_assignments(
        &self,
        py: Python,
        entity_type: String,
        entity_name: String,
        max_results: Option<i32>,
        page_token: Option<String>,
    ) -> PyUnityCatalogResult<PyListEntityTagAssignmentsResponse> {
        let request = self
            .client
            .list_entity_tag_assignments(entity_type, entity_name)
            .with_max_results(max_results)
            .with_page_token(page_token);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyListEntityTagAssignmentsResponse::from(result))
        })
    }

    /// Create an entity tag assignment
    ///
    /// Assigns a tag to a Unity Catalog entity.
    ///
    /// Args:
    ///     entity_type: The type of the entity to which the tag is assigned. Supported values: catalogs,
    ///                  schemas, tables, columns, volumes.
    ///     entity_name: The fully qualified name of the entity to which the tag is assigned.
    ///     tag_key: The key of the tag.
    ///     tag_value: The value of the tag.
    ///
    /// Returns:
    ///     The assignment of a tag to a Unity Catalog entity. Unlike a TagPolicy (a governed-tag
    ///     *definition*), an assignment is the application of a tag to a specific securable. It has no
    ///     identifier of its own — its identity is the composite of (entity_type, entity_name,
    ///     tag_key). It
    ///     is intentionally NOT a `google.api.resource`: assignments are stored as associations between
    ///     the
    ///     entity and its tag, not as standalone objects.
    #[pyo3(signature = (entity_type, entity_name, tag_key, tag_value = None))]
    pub fn create_entity_tag_assignment(
        &self,
        py: Python,
        entity_type: String,
        entity_name: String,
        tag_key: String,
        tag_value: Option<String>,
    ) -> PyUnityCatalogResult<PyEntityTagAssignment> {
        let request = self
            .client
            .create_entity_tag_assignment(entity_type, entity_name, tag_key)
            .with_tag_value(tag_value);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyEntityTagAssignment::from(result))
        })
    }

    /// Get an entity tag assignment
    ///
    /// Gets the tag assignment for the specified entity and tag key.
    ///
    /// Returns:
    ///     The assignment of a tag to a Unity Catalog entity. Unlike a TagPolicy (a governed-tag
    ///     *definition*), an assignment is the application of a tag to a specific securable. It has no
    ///     identifier of its own — its identity is the composite of (entity_type, entity_name,
    ///     tag_key). It
    ///     is intentionally NOT a `google.api.resource`: assignments are stored as associations between
    ///     the
    ///     entity and its tag, not as standalone objects.
    #[pyo3(signature = (entity_type, entity_name, tag_key))]
    pub fn get_entity_tag_assignment(
        &self,
        py: Python,
        entity_type: String,
        entity_name: String,
        tag_key: String,
    ) -> PyUnityCatalogResult<PyEntityTagAssignment> {
        let request = self
            .client
            .get_entity_tag_assignment(entity_type, entity_name, tag_key);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyEntityTagAssignment::from(result))
        })
    }

    /// Update an entity tag assignment
    ///
    /// Updates the tag assignment for the specified entity and tag key.
    ///
    /// Args:
    ///     tag_value: The value of the tag.
    ///     update_mask: The fields to update, as a comma-separated list of field names. If empty or "*",
    ///                  every field set in the request is updated. Otherwise only the listed fields are
    ///                  updated, and a listed field that is unset in the request is cleared.
    ///
    /// Returns:
    ///     The assignment of a tag to a Unity Catalog entity. Unlike a TagPolicy (a governed-tag
    ///     *definition*), an assignment is the application of a tag to a specific securable. It has no
    ///     identifier of its own — its identity is the composite of (entity_type, entity_name,
    ///     tag_key). It
    ///     is intentionally NOT a `google.api.resource`: assignments are stored as associations between
    ///     the
    ///     entity and its tag, not as standalone objects.
    #[pyo3(
        signature = (
            entity_type,
            entity_name,
            tag_key,
            tag_value = None,
            update_mask = None
        )
    )]
    pub fn update_entity_tag_assignment(
        &self,
        py: Python,
        entity_type: String,
        entity_name: String,
        tag_key: String,
        tag_value: Option<String>,
        update_mask: Option<String>,
    ) -> PyUnityCatalogResult<PyEntityTagAssignment> {
        let request = self
            .client
            .update_entity_tag_assignment(entity_type, entity_name, tag_key)
            .with_tag_value(tag_value)
            .with_update_mask(update_mask);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyEntityTagAssignment::from(result))
        })
    }

    /// Delete an entity tag assignment
    ///
    /// Deletes the tag assignment for the specified entity and tag key.
    #[pyo3(signature = (entity_type, entity_name, tag_key))]
    pub fn delete_entity_tag_assignment(
        &self,
        py: Python,
        entity_type: String,
        entity_name: String,
        tag_key: String,
    ) -> PyUnityCatalogResult<()> {
        let request = self
            .client
            .delete_entity_tag_assignment(entity_type, entity_name, tag_key);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }

    /// List external locations
    ///
    /// Args:
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///     include_browse: Whether to include schemas in the response for which the principal can only
    ///                     access selective metadata for
    ///
    /// Returns:
    ///     The external locations returned.
    #[pyo3(signature = (max_results = None, include_browse = None))]
    pub fn list_external_locations(
        &self,
        py: Python,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyExternalLocation>> {
        let request = self
            .client
            .list_external_locations()
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyExternalLocation::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a new external location
    ///
    /// Args:
    ///     name: Name of external location.
    ///     url: Path URL of the external location.
    ///     credential_name: Name of the storage credential used with this location.
    ///     read_only: Indicates whether the external location is read-only.
    ///     comment: User-provided free-form text description.
    ///     skip_validation: Skips validation of the storage credential associated with the external
    ///                      location.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(
        signature = (
            name,
            url,
            credential_name,
            read_only = None,
            comment = None,
            skip_validation = None
        )
    )]
    pub fn create_external_location(
        &self,
        py: Python,
        name: String,
        url: String,
        credential_name: String,
        read_only: Option<bool>,
        comment: Option<String>,
        skip_validation: Option<bool>,
    ) -> PyUnityCatalogResult<PyExternalLocation> {
        let request = self
            .client
            .create_external_location(name, url, credential_name)
            .with_read_only(read_only)
            .with_comment(comment)
            .with_skip_validation(skip_validation);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyExternalLocation::from(result))
        })
    }

    /// List functions
    ///
    /// List functions within the specified parent catalog and schema. If the caller is the metastore admin,
    /// all functions are returned in the response. Otherwise, the caller must have USE_CATALOG on the
    /// parent catalog and USE_SCHEMA on the parent schema, and the function must either be owned by the
    /// caller or have SELECT on the function.
    ///
    /// Args:
    ///     catalog_name: Name of parent catalog for functions of interest.
    ///     schema_name: Parent schema of functions.
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///     include_browse: Whether to include functions in the response for which the principal can only
    ///                     access selective metadata for.
    ///
    /// Returns:
    ///     The functions returned.
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_functions(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyFunction>> {
        let request = self
            .client
            .list_functions(catalog_name, schema_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyFunction::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a function
    ///
    /// Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION privilege
    /// on the parent catalog and schema.
    ///
    /// Args:
    ///     function_info: The function to create.
    ///
    /// Returns:
    ///     A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
    #[pyo3(signature = (function_info))]
    pub fn create_function(
        &self,
        py: Python,
        function_info: PyCreateFunction,
    ) -> PyUnityCatalogResult<PyFunction> {
        let request = self.client.create_function(function_info.into());
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyFunction::from(result))
        })
    }

    /// List model versions
    ///
    /// List the model versions of the specified registered model. If the caller is the metastore admin,
    /// all model versions are returned. Otherwise, the caller must have the appropriate privileges on the
    /// parent model.
    ///
    /// Args:
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///     include_browse: Whether to include model versions in the response for which the principal can
    ///                     only access selective metadata for.
    ///
    /// Returns:
    ///     The model versions returned.
    #[pyo3(signature = (full_name, max_results = None, include_browse = None))]
    pub fn list_model_versions(
        &self,
        py: Python,
        full_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyModelVersion>> {
        let request = self
            .client
            .list_model_versions(full_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyModelVersion::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a model version
    ///
    /// Creates a new model version in PENDING_REGISTRATION status. The server assigns the version number
    /// and a storage location for the artifacts. The caller must be a metastore admin or the owner of the
    /// parent registered model.
    ///
    /// Args:
    ///     model_name: Name of the parent registered model, relative to parent schema.
    ///     catalog_name: Name of parent catalog.
    ///     schema_name: Name of parent schema.
    ///     source: URI indicating the location of the source artifacts (e.g. an MLflow run artifact path)
    ///             used to create the model version.
    ///     run_id: The run id used by the ML package that generated this model.
    ///     comment: User-provided free-form text description.
    ///
    /// Returns:
    ///     A model version under a registered model. A version is identified by its parent
    ///     model's three-
    ///     level name plus an integer `version` number assigned by the server. It carries its own
    ///     artifact
    ///     storage location and moves through the PENDING_REGISTRATION -> READY lifecycle via the
    ///     finalize
    ///     operation.
    #[pyo3(
        signature = (
            model_name,
            catalog_name,
            schema_name,
            source,
            run_id = None,
            comment = None
        )
    )]
    pub fn create_model_version(
        &self,
        py: Python,
        model_name: String,
        catalog_name: String,
        schema_name: String,
        source: String,
        run_id: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<PyModelVersion> {
        let request = self
            .client
            .create_model_version(model_name, catalog_name, schema_name, source)
            .with_run_id(run_id)
            .with_comment(comment);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyModelVersion::from(result))
        })
    }

    /// Get a model version
    ///
    /// Gets a model version by its parent model name and version number.
    ///
    /// Args:
    ///     include_browse: Whether to include model versions in the response for which the principal can
    ///                     only access selective metadata for.
    ///
    /// Returns:
    ///     A model version under a registered model. A version is identified by its parent
    ///     model's three-
    ///     level name plus an integer `version` number assigned by the server. It carries its own
    ///     artifact
    ///     storage location and moves through the PENDING_REGISTRATION -> READY lifecycle via the
    ///     finalize
    ///     operation.
    #[pyo3(signature = (full_name, version, include_browse = None))]
    pub fn get_model_version(
        &self,
        py: Python,
        full_name: String,
        version: i64,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<PyModelVersion> {
        let request = self
            .client
            .get_model_version(full_name, version)
            .with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyModelVersion::from(result))
        })
    }

    /// Update a model version
    ///
    /// Updates the model version that matches the supplied name and version.
    ///
    /// Args:
    ///     comment: User-provided free-form text description.
    ///
    /// Returns:
    ///     A model version under a registered model. A version is identified by its parent
    ///     model's three-
    ///     level name plus an integer `version` number assigned by the server. It carries its own
    ///     artifact
    ///     storage location and moves through the PENDING_REGISTRATION -> READY lifecycle via the
    ///     finalize
    ///     operation.
    #[pyo3(signature = (full_name, version, comment = None))]
    pub fn update_model_version(
        &self,
        py: Python,
        full_name: String,
        version: i64,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<PyModelVersion> {
        let request = self
            .client
            .update_model_version(full_name, version)
            .with_comment(comment);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyModelVersion::from(result))
        })
    }

    /// Delete a model version
    ///
    /// Deletes the model version that matches the supplied name and version. For the deletion to succeed,
    /// the caller must be the owner of the parent registered model.
    #[pyo3(signature = (full_name, version))]
    pub fn delete_model_version(
        &self,
        py: Python,
        full_name: String,
        version: i64,
    ) -> PyUnityCatalogResult<()> {
        let request = self.client.delete_model_version(full_name, version);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(result)
        })
    }

    /// Finalize a model version
    ///
    /// Transitions a model version to READY once all artifacts have been written to its storage location.
    ///
    /// Returns:
    ///     A model version under a registered model. A version is identified by its parent
    ///     model's three-
    ///     level name plus an integer `version` number assigned by the server. It carries its own
    ///     artifact
    ///     storage location and moves through the PENDING_REGISTRATION -> READY lifecycle via the
    ///     finalize
    ///     operation.
    #[pyo3(signature = (full_name, version))]
    pub fn finalize_model_version(
        &self,
        py: Python,
        full_name: String,
        version: i64,
    ) -> PyUnityCatalogResult<PyModelVersion> {
        let request = self.client.finalize_model_version(full_name, version);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyModelVersion::from(result))
        })
    }

    /// Create a new policy
    ///
    /// Creates a new row-filter or column-mask policy on the specified securable.
    ///
    /// Args:
    ///     name: The name of the policy. Unique within the securable it is defined on.
    ///     on_securable_type: The type of securable the policy is defined on. Supported values: CATALOG,
    ///                        SCHEMA, TABLE.
    ///     on_securable_fullname: The fully qualified name of the securable the policy is defined on.
    ///     policy_type: The kind of enforcement this policy applies.
    ///     comment: User-provided free-form text description of the policy.
    ///     to_principals: The principals the policy applies to. Empty means "all principals".
    ///     except_principals: The principals explicitly excluded from the policy, even if matched by
    ///                        to_principals.
    ///     for_securable_type: The type of securables the policy takes effect on.
    ///     when_condition: An opaque condition expression evaluated by the query engine at enforcement
    ///                     time.
    ///     match_columns: The columns referenced by the policy function, and the aliases they are bound to.
    ///     row_filter: The row-filter function to invoke. Set only when policy_type is
    ///                 POLICY_TYPE_ROW_FILTER.
    ///     column_mask: The column-mask function to invoke. Set only when policy_type is
    ///                  POLICY_TYPE_COLUMN_MASK.
    ///     deny: The privileges to deny. Set only when policy_type is POLICY_TYPE_DENY.
    ///     grant: The privileges to grant. Set only when policy_type is POLICY_TYPE_GRANT.
    ///
    /// Returns:
    ///     A row-filter or column-mask policy bound to principals and scoped to a securable. Policies
    ///     are
    ///     read as whole documents; enforcement is performed by the query engine, not by this service.
    ///     `when_condition` is stored and returned opaque — this service does not parse or evaluate it.
    #[pyo3(
        signature = (
            name,
            on_securable_type,
            on_securable_fullname,
            policy_type,
            comment = None,
            to_principals = None,
            except_principals = None,
            for_securable_type = None,
            when_condition = None,
            match_columns = None,
            row_filter = None,
            column_mask = None,
            deny = None,
            grant = None
        )
    )]
    pub fn create_policy(
        &self,
        py: Python,
        name: String,
        on_securable_type: String,
        on_securable_fullname: String,
        policy_type: PyPolicyType,
        comment: Option<String>,
        to_principals: Option<Vec<String>>,
        except_principals: Option<Vec<String>>,
        for_securable_type: Option<String>,
        when_condition: Option<String>,
        match_columns: ::core::option::Option<::std::vec::Vec<PyMatchColumn>>,
        row_filter: ::core::option::Option<PyFunctionRef>,
        column_mask: ::core::option::Option<PyFunctionRef>,
        deny: ::core::option::Option<PyPrivilegeOptions>,
        grant: ::core::option::Option<PyPrivilegeOptions>,
    ) -> PyUnityCatalogResult<PyPolicyInfo> {
        let request = self
            .client
            .create_policy(
                name,
                on_securable_type,
                on_securable_fullname,
                policy_type.into(),
            )
            .with_comment(comment)
            .with_to_principals(to_principals.unwrap_or_default())
            .with_except_principals(except_principals.unwrap_or_default())
            .with_for_securable_type(for_securable_type)
            .with_when_condition(when_condition)
            .with_match_columns(
                match_columns
                    .unwrap_or_default()
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            )
            .with_row_filter(row_filter.map(::core::convert::Into::into))
            .with_column_mask(column_mask.map(::core::convert::Into::into))
            .with_deny(deny.map(::core::convert::Into::into))
            .with_grant(grant.map(::core::convert::Into::into));
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyPolicyInfo::from(result))
        })
    }

    /// List providers.
    ///
    /// Args:
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///
    /// Returns:
    ///     List of providers.
    #[pyo3(signature = (max_results = None))]
    pub fn list_providers(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyProvider>> {
        let request = self.client.list_providers().with_max_results(max_results);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyProvider::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a new provider.
    ///
    /// Args:
    ///     name: Name of the provider.
    ///     authentication_type: The delta sharing authentication type.
    ///     owner: Username of the provider owner.
    ///     comment: Description about the provider.
    ///     recipient_profile_str: The recipient profile (credential file contents) used to connect to
    ///                            the sharing server. Required when authentication_type is TOKEN or
    ///                            OAUTH_CLIENT_CREDENTIALS.
    ///     properties: Provider properties as map of string key-value pairs.
    ///
    /// Returns:
    ///     A provider represents an organization sharing data with this metastore. A provider is the
    ///     inbound counterpart of a recipient: it is registered from a share activation/ credential
    ///     file
    ///     and is used to access shares offered by an upstream Delta Sharing server.
    #[pyo3(
        signature = (
            name,
            authentication_type,
            owner = None,
            comment = None,
            recipient_profile_str = None,
            properties = None
        )
    )]
    pub fn create_provider(
        &self,
        py: Python,
        name: String,
        authentication_type: PyProviderAuthenticationType,
        owner: Option<String>,
        comment: Option<String>,
        recipient_profile_str: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyUnityCatalogResult<PyProvider> {
        let request = self
            .client
            .create_provider(name, authentication_type.into())
            .with_owner(owner)
            .with_comment(comment)
            .with_recipient_profile_str(recipient_profile_str)
            .with_properties(properties.unwrap_or_default());
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyProvider::from(result))
        })
    }

    /// List recipients.
    ///
    /// Args:
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///
    /// Returns:
    ///     List of recipients.
    #[pyo3(signature = (max_results = None))]
    pub fn list_recipients(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyRecipient>> {
        let request = self.client.list_recipients().with_max_results(max_results);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyRecipient::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a new recipient.
    ///
    /// Args:
    ///     name: Name of the recipient.
    ///     authentication_type: The delta sharing authentication type.
    ///     owner: Username of the recipient owner.
    ///     comment: Description about the recipient.
    ///     properties_kvpairs: Recipient properties as map of string key-value pairs. When provided
    ///                         in update request, the specified properties will override the existing
    ///                         properties. To add and remove properties, one would need to perform a read-
    ///                         modify-write.
    ///     expiration_time: Expiration timestamp of the token, in epoch milliseconds.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(
        signature = (
            name,
            authentication_type,
            owner,
            comment = None,
            properties_kvpairs = None,
            expiration_time = None
        )
    )]
    pub fn create_recipient(
        &self,
        py: Python,
        name: String,
        authentication_type: PyAuthenticationType,
        owner: String,
        comment: Option<String>,
        properties_kvpairs: ::core::option::Option<PySecurablePropertiesKvPairs>,
        expiration_time: Option<i64>,
    ) -> PyUnityCatalogResult<PyRecipient> {
        let request = self
            .client
            .create_recipient(name, authentication_type.into(), owner)
            .with_comment(comment)
            .with_properties_kvpairs(properties_kvpairs.map(::core::convert::Into::into))
            .with_expiration_time(expiration_time);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRecipient::from(result))
        })
    }

    /// List registered models
    ///
    /// List registered models within the specified parent catalog and schema. If the caller is the
    /// metastore admin, all registered models are returned in the response. Otherwise, the caller must have
    /// USE_CATALOG on the parent catalog and USE_SCHEMA on the parent schema, and the model must either be
    /// owned by the caller or the caller must have a privilege on the model.
    ///
    /// Args:
    ///     catalog_name: Name of parent catalog for models of interest.
    ///     schema_name: Name of parent schema for models of interest.
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///     include_browse: Whether to include registered models in the response for which the principal can
    ///                     only access selective metadata for.
    ///
    /// Returns:
    ///     The registered models returned.
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name = None,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_registered_models(
        &self,
        py: Python,
        catalog_name: Option<String>,
        schema_name: Option<String>,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyRegisteredModel>> {
        let request = self
            .client
            .list_registered_models()
            .with_catalog_name(catalog_name)
            .with_schema_name(schema_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyRegisteredModel::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a registered model
    ///
    /// Creates a new registered model. The caller must be a metastore admin or have the CREATE_MODEL
    /// privilege on the parent catalog and schema.
    ///
    /// Args:
    ///     name: Name of registered model, relative to parent schema.
    ///     catalog_name: Name of parent catalog.
    ///     schema_name: Name of parent schema.
    ///     comment: User-provided free-form text description.
    ///
    /// Returns:
    ///     A registered model within the three-level namespace (catalog.schema.model). Registered
    ///     models
    ///     group a collection of model versions. Individual versions carry their own artifact storage
    ///     location and are created, finalized, and governed independently.
    #[pyo3(signature = (name, catalog_name, schema_name, comment = None))]
    pub fn create_registered_model(
        &self,
        py: Python,
        name: String,
        catalog_name: String,
        schema_name: String,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<PyRegisteredModel> {
        let request = self
            .client
            .create_registered_model(name, catalog_name, schema_name)
            .with_comment(comment);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyRegisteredModel::from(result))
        })
    }

    /// Gets an array of schemas for a catalog in the metastore. If the caller is the metastore admin or the
    /// owner of the parent catalog, all schemas for the catalog will be retrieved. Otherwise, only schemas
    /// owned by the caller (or for which the caller has the USE_SCHEMA privilege) will be retrieved. There
    /// is no guarantee of a specific ordering of the elements in the array.
    ///
    /// Args:
    ///     catalog_name: Name of parent catalog.
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///     include_browse: Whether to include schemas in the response for which the principal can only
    ///                     access selective metadata for
    ///
    /// Returns:
    ///     The schemas returned.
    #[pyo3(signature = (catalog_name, max_results = None, include_browse = None))]
    pub fn list_schemas(
        &self,
        py: Python,
        catalog_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PySchema>> {
        let request = self
            .client
            .list_schemas(catalog_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PySchema::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Creates a new schema for catalog in the Metatastore. The caller must be a metastore admin, or have
    /// the CREATE_SCHEMA privilege in the parent catalog.
    ///
    /// Args:
    ///     name: Name of schema, relative to parent catalog.
    ///     catalog_name: Name of parent catalog.
    ///     comment: User-provided free-form text description.
    ///     properties: A map of key-value properties attached to the securable.
    ///     storage_root: Storage root URL for managed storage location of the schema. If not set, managed
    ///                   securables under this schema fall back to the parent catalog's storage location.
    ///                   Example: `s3://bucket/ucroot`.
    ///
    /// Returns:
    ///     A schema is a namespace within a catalog that contains tables.
    #[pyo3(
        signature = (
            name,
            catalog_name,
            comment = None,
            properties = None,
            storage_root = None
        )
    )]
    pub fn create_schema(
        &self,
        py: Python,
        name: String,
        catalog_name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
    ) -> PyUnityCatalogResult<PySchema> {
        let request = self
            .client
            .create_schema(name, catalog_name)
            .with_comment(comment)
            .with_properties(properties.unwrap_or_default())
            .with_storage_root(storage_root);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PySchema::from(result))
        })
    }

    /// List shares.
    ///
    /// Args:
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///
    /// Returns:
    ///     List of shares.
    #[pyo3(signature = (max_results = None))]
    pub fn list_shares(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyShare>> {
        let request = self.client.list_shares().with_max_results(max_results);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyShare::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a new share.
    ///
    /// Args:
    ///     name: Name of the share.
    ///     comment: User-provided free-form text description.
    ///     storage_root: Storage root URL for the share.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(signature = (name, comment = None, storage_root = None))]
    pub fn create_share(
        &self,
        py: Python,
        name: String,
        comment: Option<String>,
        storage_root: Option<String>,
    ) -> PyUnityCatalogResult<PyShare> {
        let request = self
            .client
            .create_share(name)
            .with_comment(comment)
            .with_storage_root(storage_root);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyShare::from(result))
        })
    }

    /// Creates a new staging table, allocating an immutable table id and a storage location under the
    /// parent schema/catalog managed storage root. The caller must have the CREATE privilege on the parent
    /// schema.
    ///
    /// Args:
    ///     name: Name of the staging table, relative to the parent schema.
    ///     catalog_name: Name of the parent catalog.
    ///     schema_name: Name of the parent schema relative to its parent catalog.
    ///
    /// Returns:
    ///     A staging table is a transient reservation created while provisioning a Unity Catalog
    ///     managed
    ///     table. On creation the server allocates an immutable table id and storage location under the
    ///     parent schema/catalog's managed storage root. The client then writes the initial Delta
    ///     commit
    ///     (`_delta_log/00...0.json`) at that location and finalizes the table via the CreateTable API,
    ///     which commits this staging row. A staging table is not yet a usable table and is not
    ///     returned by
    ///     the table APIs.
    #[pyo3(signature = (name, catalog_name, schema_name))]
    pub fn create_staging_table(
        &self,
        py: Python,
        name: String,
        catalog_name: String,
        schema_name: String,
    ) -> PyUnityCatalogResult<PyStagingTable> {
        let request = self
            .client
            .create_staging_table(name, catalog_name, schema_name);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyStagingTable::from(result))
        })
    }

    /// Gets an array of all tables for the current metastore under the parent catalog and schema.
    ///
    /// The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table. For
    /// the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the parent
    /// catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a specific
    /// ordering of the elements in the array.
    ///
    /// Args:
    ///     catalog_name: Name of parent catalog for tables of interest.
    ///     schema_name: Name of parent schema for tables of interest.
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///     include_delta_metadata: Whether delta metadata should be included in the response.
    ///     omit_columns: Whether to omit the columns of the table from the response or not.
    ///     omit_properties: Whether to omit the properties of the table from the response or not.
    ///     omit_username: Whether to omit the username of the table (e.g. owner, updated_by, created_by)
    ///                    from the response or not.
    ///     include_browse: Whether to include tables in the response for which the principal can only
    ///                     access selective metadata for
    ///     include_manifest_capabilities: Whether to include a manifest containing capabilities the table
    ///                                    has.
    ///
    /// Returns:
    ///     The tables returned.
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_delta_metadata = None,
            omit_columns = None,
            omit_properties = None,
            omit_username = None,
            include_browse = None,
            include_manifest_capabilities = None
        )
    )]
    pub fn list_tables(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_delta_metadata: Option<bool>,
        omit_columns: Option<bool>,
        omit_properties: Option<bool>,
        omit_username: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyTable>> {
        let request = self
            .client
            .list_tables(catalog_name, schema_name)
            .with_max_results(max_results)
            .with_include_delta_metadata(include_delta_metadata)
            .with_omit_columns(omit_columns)
            .with_omit_properties(omit_properties)
            .with_omit_username(omit_username)
            .with_include_browse(include_browse)
            .with_include_manifest_capabilities(include_manifest_capabilities);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyTable::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a table
    ///
    /// Args:
    ///     name: Name of table, relative to parent schema.
    ///     schema_name: Name of parent schema relative to its parent catalog.
    ///     catalog_name: Name of parent catalog.
    ///     columns: The array of Column definitions of the table's columns.
    ///     storage_location: Storage root URL for external table.
    ///     comment: User-provided free-form text description.
    ///     properties: A map of key-value properties attached to the securable.
    ///     view_definition: Definition text for view-like table types (VIEW, MATERIALIZED_VIEW,
    ///                      STREAMING_TABLE, METRIC_VIEW). The format depends on the table type: SQL for
    ///                      views, YAML for metric views. Required for METRIC_VIEW.
    ///     view_dependencies: Tables and functions the view-like table reads. For metric views the server
    ///                        derives this from view_definition and rejects a supplied list that diverges
    ///                        from the derived set (the definition is the single source of truth).
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(
        signature = (
            name,
            schema_name,
            catalog_name,
            table_type,
            data_source_format,
            columns = None,
            storage_location = None,
            comment = None,
            properties = None,
            view_definition = None,
            view_dependencies = None
        )
    )]
    pub fn create_table(
        &self,
        py: Python,
        name: String,
        schema_name: String,
        catalog_name: String,
        table_type: PyTableType,
        data_source_format: PyDataSourceFormat,
        columns: ::core::option::Option<::std::vec::Vec<PyColumn>>,
        storage_location: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        view_definition: Option<String>,
        view_dependencies: ::core::option::Option<PyDependencyList>,
    ) -> PyUnityCatalogResult<PyTable> {
        let request = self
            .client
            .create_table(
                name,
                schema_name,
                catalog_name,
                table_type.into(),
                data_source_format.into(),
            )
            .with_columns(
                columns
                    .unwrap_or_default()
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            )
            .with_storage_location(storage_location)
            .with_comment(comment)
            .with_properties(properties.unwrap_or_default())
            .with_view_definition(view_definition)
            .with_view_dependencies(view_dependencies.map(::core::convert::Into::into));
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTable::from(result))
        })
    }

    /// List tag policies
    ///
    /// Gets an array of tag policies. There is no guarantee of a specific ordering of the elements in the
    /// array.
    ///
    /// Args:
    ///     max_results: The maximum number of results to return in this request. Fewer results may be
    ///                  returned than requested. If unset or 0, this defaults to 1000; larger values are
    ///                  coerced down to 1000. Negative values are rejected.
    ///
    /// Returns:
    ///     The tag policies returned.
    #[pyo3(signature = (max_results = None))]
    pub fn list_tag_policies(
        &self,
        py: Python,
        max_results: Option<i32>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyTagPolicy>> {
        let request = self
            .client
            .list_tag_policies()
            .with_max_results(max_results);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyTagPolicy::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Create a new tag policy
    ///
    /// Creates a new governed tag definition.
    ///
    /// Args:
    ///     tag_key: The key of the governed tag.
    ///     description: User-provided free-form text description of the tag policy.
    ///     values: The set of allowed values for the governed tag. The list is unordered.
    ///
    /// Returns:
    ///     A governed tag definition (tag policy). A tag policy defines a tag key together with the
    ///     rules
    ///     that govern how it can be used, including the optional set of allowed values. Assigning a
    ///     governed tag to an entity is done through the Entity Tag Assignments API.
    #[pyo3(signature = (tag_key, description = None, values = None))]
    pub fn create_tag_policy(
        &self,
        py: Python,
        tag_key: String,
        description: Option<String>,
        values: ::core::option::Option<::std::vec::Vec<PyValue>>,
    ) -> PyUnityCatalogResult<PyTagPolicy> {
        let request = self
            .client
            .create_tag_policy(tag_key)
            .with_description(description)
            .with_values(
                values
                    .unwrap_or_default()
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            );
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTagPolicy::from(result))
        })
    }

    /// Generate a new set of credentials for a table.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true (default false). The caller
    /// must have the `EXTERNAL_USE_SCHEMA` privilege on the parent schema (granted by a catalog owner).
    ///
    /// Args:
    ///     table_id: UUID of the table to read or write.
    ///     operation: The operation performed against the table data, either READ or READ_WRITE. If
    ///                READ_WRITE is specified, the credentials returned will have write permissions,
    ///                otherwise, it will be read only.
    ///
    /// Returns:
    ///     The response to the GenerateTemporaryTableCredentialsRequest.
    #[pyo3(signature = (table_id, operation))]
    pub fn generate_temporary_table_credentials(
        &self,
        py: Python,
        table_id: String,
        operation: PyGenerateTemporaryTableCredentialsRequestOperation,
    ) -> PyUnityCatalogResult<PyTemporaryCredential> {
        let request = self
            .client
            .generate_temporary_table_credentials(table_id, operation.into());
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTemporaryCredential::from(result))
        })
    }

    /// Generate a new set of credentials for a path.
    ///
    /// Only external storage paths (external locations and external tables) are supported; managed tables
    /// are not. The metastore must have the `external_access_enabled` flag set to true (default false).
    /// The caller must have the `EXTERNAL_USE_LOCATION` privilege on the external location (granted by its
    /// owner) and, for existing external tables, the `EXTERNAL_USE_SCHEMA` privilege on the parent schema.
    ///
    /// Args:
    ///     url: URL for path-based access.
    ///     operation: The operation being performed on the path.
    ///     dry_run: When set to true, the service will not validate that the generated credentials can
    ///              perform write operations, therefore no new paths will be created and the response will
    ///              not contain valid credentials. Defaults to false.
    ///
    /// Returns:
    ///     The response to the GenerateTemporaryTableCredentialsRequest.
    #[pyo3(signature = (url, operation, dry_run = None))]
    pub fn generate_temporary_path_credentials(
        &self,
        py: Python,
        url: String,
        operation: PyGenerateTemporaryPathCredentialsRequestOperation,
        dry_run: Option<bool>,
    ) -> PyUnityCatalogResult<PyTemporaryCredential> {
        let request = self
            .client
            .generate_temporary_path_credentials(url, operation.into())
            .with_dry_run(dry_run);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTemporaryCredential::from(result))
        })
    }

    /// Generate a new set of credentials for a volume.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true (default false). The caller
    /// must have the `EXTERNAL_USE_SCHEMA` privilege on the parent schema (granted by a catalog owner).
    ///
    /// Args:
    ///     volume_id: UUID of the volume to read or write.
    ///     operation: The operation performed against the volume data, either READ_VOLUME or WRITE_VOLUME.
    ///                If WRITE_VOLUME is specified, the credentials returned will have write permissions,
    ///                otherwise, it will be read only.
    ///
    /// Returns:
    ///     The response to the GenerateTemporaryTableCredentialsRequest.
    #[pyo3(signature = (volume_id, operation))]
    pub fn generate_temporary_volume_credentials(
        &self,
        py: Python,
        volume_id: String,
        operation: PyGenerateTemporaryVolumeCredentialsRequestOperation,
    ) -> PyUnityCatalogResult<PyTemporaryCredential> {
        let request = self
            .client
            .generate_temporary_volume_credentials(volume_id, operation.into());
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTemporaryCredential::from(result))
        })
    }

    /// Generate a new set of credentials for a model version.
    ///
    /// The metastore must have the `external_access_enabled` flag set to true (default false). The caller
    /// must have the `EXTERNAL_USE_SCHEMA` privilege on the parent schema (granted by a catalog owner).
    ///
    /// Args:
    ///     catalog_name: Name of parent catalog of the model version.
    ///     schema_name: Name of parent schema of the model version.
    ///     model_name: Name of the parent registered model.
    ///     version: The integer version number of the model version.
    ///     operation: The operation performed against the model version data, either READ_MODEL_VERSION or
    ///                READ_WRITE_MODEL_VERSION. If READ_WRITE_MODEL_VERSION is specified, the credentials
    ///                returned will have write permissions, otherwise, it will be read only.
    ///
    /// Returns:
    ///     The response to the GenerateTemporaryTableCredentialsRequest.
    #[pyo3(signature = (catalog_name, schema_name, model_name, version, operation))]
    pub fn generate_temporary_model_version_credentials(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        model_name: String,
        version: i64,
        operation: PyGenerateTemporaryModelVersionCredentialsRequestOperation,
    ) -> PyUnityCatalogResult<PyTemporaryCredential> {
        let request = self.client.generate_temporary_model_version_credentials(
            catalog_name,
            schema_name,
            model_name,
            version,
            operation.into(),
        );
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyTemporaryCredential::from(result))
        })
    }

    /// Lists volumes.
    ///
    /// Args:
    ///     catalog_name: The identifier of the catalog
    ///     schema_name: The identifier of the schema
    ///     max_results: The maximum number of results per page that should be returned. If unset or 0, the
    ///                  server's default page size is used. Negative values are rejected.
    ///     include_browse: Whether to include volumes in the response for which the principal can only
    ///                     access selective metadata for
    ///
    /// Returns:
    ///     The volumes returned.
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            max_results = None,
            include_browse = None
        )
    )]
    pub fn list_volumes(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyUnityCatalogResult<::std::vec::Vec<PyVolume>> {
        let request = self
            .client
            .list_volumes(catalog_name, schema_name)
            .with_max_results(max_results)
            .with_include_browse(include_browse);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(async move {
                request
                    .into_stream()
                    .try_collect::<::std::vec::Vec<_>>()
                    .await
            })?;
            Ok::<_, PyUnityCatalogError>(
                result
                    .into_iter()
                    .map(PyVolume::from)
                    .collect::<::std::vec::Vec<_>>(),
            )
        })
    }

    /// Args:
    ///     catalog_name: The identifier of the catalog
    ///     schema_name: The identifier of the schema
    ///     name: The identifier of the volume
    ///     volume_type: The type of the volume. An external volume is located in the specified external
    ///                  location. A managed volume is located in the default location which is specified by
    ///                  the parent schema, or the parent catalog, or the Metastore.
    ///     storage_location: The storage location on the cloud
    ///     comment: The comment attached to the volume.
    ///
    /// Returns:
    ///     The requested resource
    #[pyo3(
        signature = (
            catalog_name,
            schema_name,
            name,
            volume_type,
            storage_location = None,
            comment = None
        )
    )]
    pub fn create_volume(
        &self,
        py: Python,
        catalog_name: String,
        schema_name: String,
        name: String,
        volume_type: PyVolumeType,
        storage_location: Option<String>,
        comment: Option<String>,
    ) -> PyUnityCatalogResult<PyVolume> {
        let request = self
            .client
            .create_volume(catalog_name, schema_name, name, volume_type.into())
            .with_storage_location(storage_location)
            .with_comment(comment);
        let runtime = crate::codegen::runtime::rt();
        py.detach(|| {
            #[allow(clippy::let_unit_value)]
            let result = runtime.block_on(request.into_future())?;
            Ok::<_, PyUnityCatalogError>(PyVolume::from(result))
        })
    }

    /// The requests on one `AgentSkill`. It sends no request itself.
    pub fn agent_skill(
        &self,
        catalog_name: String,
        schema_name: String,
        agent_skill_name: String,
    ) -> PyAgentSkillClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, agent_skill_name);
        PyAgentSkillClient {
            client: self.client.agent_skill_from_full_name(full_name),
        }
    }

    /// The requests on one `Agent`. It sends no request itself.
    pub fn agent(
        &self,
        catalog_name: String,
        schema_name: String,
        agent_name: String,
    ) -> PyAgentClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, agent_name);
        PyAgentClient {
            client: self.client.agent_from_full_name(full_name),
        }
    }

    /// The requests on one `Catalog`. It sends no request itself.
    pub fn catalog(&self, catalog_name: String) -> PyCatalogClient {
        PyCatalogClient {
            client: self.client.catalog(catalog_name),
        }
    }

    /// The requests on one `Credential`. It sends no request itself.
    pub fn credential(&self, credential_name: String) -> PyCredentialClient {
        PyCredentialClient {
            client: self.client.credential(credential_name),
        }
    }

    /// The requests on one `ExternalLocation`. It sends no request itself.
    pub fn external_location(&self, external_location_name: String) -> PyExternalLocationClient {
        PyExternalLocationClient {
            client: self.client.external_location(external_location_name),
        }
    }

    /// The requests on one `Function`. It sends no request itself.
    pub fn function(
        &self,
        catalog_name: String,
        schema_name: String,
        function_name: String,
    ) -> PyFunctionClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, function_name);
        PyFunctionClient {
            client: self.client.function_from_full_name(full_name),
        }
    }

    /// The requests on one `PolicyInfo`. It sends no request itself.
    pub fn policy(&self, policy_name: String) -> PyPolicyClient {
        PyPolicyClient {
            client: self.client.policy(policy_name),
        }
    }

    /// The requests on one `Provider`. It sends no request itself.
    pub fn provider(&self, provider_name: String) -> PyProviderClient {
        PyProviderClient {
            client: self.client.provider(provider_name),
        }
    }

    /// The requests on one `Recipient`. It sends no request itself.
    pub fn recipient(&self, recipient_name: String) -> PyRecipientClient {
        PyRecipientClient {
            client: self.client.recipient(recipient_name),
        }
    }

    /// The requests on one `RegisteredModel`. It sends no request itself.
    pub fn registered_model(
        &self,
        catalog_name: String,
        schema_name: String,
        registered_model_name: String,
    ) -> PyRegisteredModelClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, registered_model_name);
        PyRegisteredModelClient {
            client: self.client.registered_model_from_full_name(full_name),
        }
    }

    /// The requests on one `Schema`. It sends no request itself.
    pub fn schema(&self, catalog_name: String, schema_name: String) -> PySchemaClient {
        let full_name = format!("{}.{}", catalog_name, schema_name);
        PySchemaClient {
            client: self.client.schema_from_full_name(full_name),
        }
    }

    /// The requests on one `Share`. It sends no request itself.
    pub fn share(&self, share_name: String) -> PyShareClient {
        PyShareClient {
            client: self.client.share(share_name),
        }
    }

    /// The requests on one `StagingTable`. It sends no request itself.
    pub fn staging_table(&self, staging_table_name: String) -> PyStagingTableClient {
        PyStagingTableClient {
            client: self.client.staging_table(staging_table_name),
        }
    }

    /// The requests on one `Table`. It sends no request itself.
    pub fn table(
        &self,
        catalog_name: String,
        schema_name: String,
        table_name: String,
    ) -> PyTableClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, table_name);
        PyTableClient {
            client: self.client.table_from_full_name(full_name),
        }
    }

    /// The requests on one `TagPolicy`. It sends no request itself.
    pub fn tag_policy(&self, tag_policy_name: String) -> PyTagPolicyClient {
        PyTagPolicyClient {
            client: self.client.tag_policy(tag_policy_name),
        }
    }

    /// The requests on one `Volume`. It sends no request itself.
    pub fn volume(
        &self,
        catalog_name: String,
        schema_name: String,
        volume_name: String,
    ) -> PyVolumeClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, volume_name);
        PyVolumeClient {
            client: self.client.volume_from_full_name(full_name),
        }
    }
}

/// The extension module, as Python imports it.
pub const CORE_MODULE: &str = "unitycatalog_client._client";
/// The submodules of the extension module.
const SUBMODULES: &[&str] = &[
    "agent_skills",
    "agents",
    "catalogs",
    "credentials",
    "external_locations",
    "functions",
    "model_versions",
    "policies",
    "providers",
    "recipients",
    "registered_models",
    "schemas",
    "shares",
    "staging_tables",
    "tables",
    "tags",
    "temporary_credentials",
    "volumes",
];
/// The extension module `unitycatalog_client._client`.
#[::pyo3::pymodule]
#[pyo3(name = "_client", module = "unitycatalog_client")]
pub mod extension_module {
    use ::pyo3::prelude::*;

    /// The version of the bindings crate.
    #[pymodule_export]
    #[allow(non_upper_case_globals)]
    const __version__: &str = env!("CARGO_PKG_VERSION");
    #[pymodule_export]
    use super::PyUnityCatalogClient;

    #[pymodule_export]
    use crate::codegen::error::{
        BadRequestError, ConflictError, NotFoundError, PermissionDeniedError, RateLimitError,
        ServerError, UnauthenticatedError, UnityCatalogError,
    };

    #[pymodule_export]
    use crate::client::PyTemporaryCredentialClient;

    #[pymodule_export]
    use crate::reference::parse_uc_url;

    /// The classes of the `agent_skills` API.
    #[pymodule]
    pub mod agent_skills {
        #[pymodule_export]
        use super::super::{PyAgentSkill, PyAgentSkillClient, PyListAgentSkillsResponse};
    }

    /// The classes of the `agents` API.
    #[pymodule]
    pub mod agents {
        #[pymodule_export]
        use super::super::{PyAgent, PyAgentClient, PyListAgentsResponse};
    }

    /// The classes of the `catalogs` API.
    #[pymodule]
    pub mod catalogs {
        #[pymodule_export]
        use super::super::{PyCatalog, PyCatalogClient, PyListCatalogsResponse};
    }

    /// The classes of the `credentials` API.
    #[pymodule]
    pub mod credentials {
        #[pymodule_export]
        use super::super::{
            PyAwsIamRoleConfig, PyAzureManagedIdentity, PyAzureServicePrincipal, PyAzureStorageKey,
            PyCredential, PyCredentialClient, PyDatabricksGcpServiceAccount,
            PyListCredentialsResponse,
        };
    }

    /// The classes of the `external_locations` API.
    #[pymodule]
    pub mod external_locations {
        #[pymodule_export]
        use super::super::{
            PyExternalLocation, PyExternalLocationClient, PyListExternalLocationsResponse,
        };
    }

    /// The classes of the `functions` API.
    #[pymodule]
    pub mod functions {
        #[pymodule_export]
        use super::super::{
            PyCreateFunction, PyFunction, PyFunctionClient, PyFunctionParameterInfo,
            PyFunctionParameterInfos, PyListFunctionsResponse,
        };
    }

    /// The classes of the `model_versions` API.
    #[pymodule]
    pub mod model_versions {
        #[pymodule_export]
        use super::super::{PyListModelVersionsResponse, PyModelVersion};
    }

    /// The classes of the `policies` API.
    #[pymodule]
    pub mod policies {
        #[pymodule_export]
        use super::super::{
            PyFunctionArg, PyFunctionRef, PyListPoliciesResponse, PyMatchColumn, PyPolicyClient,
            PyPolicyInfo, PyPrivilegeOptions,
        };
    }

    /// The classes of the `providers` API.
    #[pymodule]
    pub mod providers {
        #[pymodule_export]
        use super::super::{PyListProvidersResponse, PyProvider, PyProviderClient};
    }

    /// The classes of the `recipients` API.
    #[pymodule]
    pub mod recipients {
        #[pymodule_export]
        use super::super::{
            PyListRecipientsResponse, PyRecipient, PyRecipientClient, PyRecipientToken,
            PySecurablePropertiesKvPairs,
        };
    }

    /// The classes of the `registered_models` API.
    #[pymodule]
    pub mod registered_models {
        #[pymodule_export]
        use super::super::{
            PyListRegisteredModelsResponse, PyRegisteredModel, PyRegisteredModelClient,
        };
    }

    /// The classes of the `schemas` API.
    #[pymodule]
    pub mod schemas {
        #[pymodule_export]
        use super::super::{PyListSchemasResponse, PySchema, PySchemaClient};
    }

    /// The classes of the `shares` API.
    #[pymodule]
    pub mod shares {
        #[pymodule_export]
        use super::super::{
            PyDataObject, PyDataObjectUpdate, PyGetPermissionsResponse, PyListSharesResponse,
            PyPartition, PyPartitionValue, PyPermissionsChange, PyPrivilegeAssignment, PyShare,
            PyShareClient, PyUpdatePermissionsResponse,
        };
    }

    /// The classes of the `staging_tables` API.
    #[pymodule]
    pub mod staging_tables {
        #[pymodule_export]
        use super::super::{PyStagingTable, PyStagingTableClient};
    }

    /// The classes of the `tables` API.
    #[pymodule]
    pub mod tables {
        #[pymodule_export]
        use super::super::{
            PyColumn, PyDependency, PyDependencyList, PyFunctionDependency,
            PyGetTableExistsResponse, PyListTableSummariesResponse, PyListTablesResponse, PyTable,
            PyTableClient, PyTableDependency, PyTableSummary,
        };
    }

    /// The classes of the `tags` API.
    #[pymodule]
    pub mod tags {
        #[pymodule_export]
        use super::super::{
            PyEntityTagAssignment, PyListEntityTagAssignmentsResponse, PyListTagPoliciesResponse,
            PyTagPolicy, PyTagPolicyClient, PyValue,
        };
    }

    /// The classes of the `temporary_credentials` API.
    #[pymodule]
    pub mod temporary_credentials {
        #[pymodule_export]
        use super::super::{
            PyAwsTemporaryCredentials, PyAzureAad, PyAzureUserDelegationSas, PyGcpOauthToken,
            PyR2TemporaryCredentials, PyTemporaryCredential,
        };
    }

    /// The classes of the `volumes` API.
    #[pymodule]
    pub mod volumes {
        #[pymodule_export]
        use super::super::{PyListVolumesResponse, PyVolume, PyVolumeClient};
    }

    /// Put the submodules in `sys.modules`. The import system puts the module there
    /// before it runs this. Not handed the module, so the stubs still describe the
    /// module in full.
    #[pymodule_init]
    fn init(py: Python<'_>) -> PyResult<()> {
        let modules = py.import("sys")?.getattr("modules")?;
        match modules.get_item(super::CORE_MODULE) {
            Ok(module) => super::add_submodules_to_sys_modules(module.cast()?),
            Err(_) => Ok(()),
        }
    }
}

/// Put the submodules of the extension module `module` in `sys.modules`, so that
/// `import <module>.<submodule>` finds them. The module does this on import; call it
/// for a module made otherwise, such as with `wrap_pymodule!`.
pub fn add_submodules_to_sys_modules(
    module: &::pyo3::Bound<'_, ::pyo3::types::PyModule>,
) -> ::pyo3::PyResult<()> {
    let modules = module.py().import("sys")?.getattr("modules")?;
    let name = module.name()?;
    for submodule in SUBMODULES {
        modules.set_item(format!("{name}.{submodule}"), module.getattr(*submodule)?)?;
    }
    Ok(())
}
