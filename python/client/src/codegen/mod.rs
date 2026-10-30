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
use crate::codegen::external_locations::PyExternalLocationClient;
use crate::codegen::functions::PyFunctionClient;
use crate::codegen::policies::PyPolicyClient;
use crate::codegen::providers::PyProviderClient;
use crate::codegen::recipients::PyRecipientClient;
use crate::codegen::registered_models::PyRegisteredModelClient;
use crate::codegen::runtime::{ListIterator, block_on};
use crate::codegen::schemas::PySchemaClient;
use crate::codegen::shares::PyShareClient;
use crate::codegen::staging_tables::PyStagingTableClient;
use crate::codegen::tables::PyTableClient;
use crate::codegen::tag_policies::PyTagPolicyClient;
use crate::codegen::volumes::PyVolumeClient;
use crate::error::PyUnityCatalogError;
use futures::stream::StreamExt;
use pyo3::IntoPyObjectExt;
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
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::*;
use unitycatalog_common::models::*;
#[pyclass(name = "UnityCatalogClient")]
pub struct PyUnityCatalogClient {
    client: UnityCatalogClient,
}
#[pymethods]
impl PyUnityCatalogClient {
    #[new]
    #[pyo3(signature = (base_url, token = None))]
    pub fn new(base_url: String, token: Option<String>) -> PyResult<Self> {
        let client = if let Some(token) = token {
            olai_http::CloudClient::new_with_token(token)
        } else {
            olai_http::CloudClient::new_unauthenticated()
        };
        let base_url = base_url.parse().map_err(PyUnityCatalogError::from)?;
        Ok(Self {
            client: UnityCatalogClient::new(client, base_url),
        })
    }
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
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_agent_skills(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyAgentSkill::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
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
    ) -> PyResult<PyAgentSkill> {
        let mut request = self.client.create_agent_skill(
            catalog_name,
            schema_name,
            name,
            agent_skill_type.into(),
        );
        request = request.with_storage_location(storage_location);
        request = request.with_description(description);
        request = request.with_license(license);
        if let Some(allowed_tools) = allowed_tools {
            request = request.with_allowed_tools(allowed_tools);
        }
        if let Some(metadata) = metadata {
            request = request.with_metadata(metadata);
        }
        request = request.with_comment(comment);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyAgentSkill::from(result))
    }
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
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_agents(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyAgent::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
        catalog_name: String,
        schema_name: String,
        name: String,
        invocation_protocol: PyInvocationProtocol,
        endpoint: String,
        description: Option<String>,
        capabilities: Option<Vec<String>>,
        input_schema: Option<String>,
        comment: Option<String>,
    ) -> PyResult<PyAgent> {
        let mut request = self.client.create_agent(
            catalog_name,
            schema_name,
            name,
            invocation_protocol.into(),
            endpoint,
        );
        request = request.with_description(description);
        if let Some(capabilities) = capabilities {
            request = request.with_capabilities(capabilities);
        }
        request = request.with_input_schema(input_schema);
        request = request.with_comment(comment);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyAgent::from(result))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_catalogs(&self, max_results: Option<i32>) -> PyResult<ListIterator> {
        let mut request = self.client.list_catalogs();
        request = request.with_max_results(max_results);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyCatalog::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
        name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
        provider_name: Option<String>,
        share_name: Option<String>,
    ) -> PyResult<PyCatalog> {
        let mut request = self.client.create_catalog(name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_storage_root(storage_root);
        request = request.with_provider_name(provider_name);
        request = request.with_share_name(share_name);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyCatalog::from(result))
    }
    #[pyo3(signature = (purpose = None, max_results = None))]
    pub fn list_credentials(
        &self,
        purpose: ::core::option::Option<PyPurpose>,
        max_results: Option<i32>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_credentials();
        request = request.with_purpose(purpose.map(::core::convert::Into::into));
        request = request.with_max_results(max_results);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyCredential::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
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
    ) -> PyResult<PyCredential> {
        let mut request = self.client.create_credential(name, purpose.into());
        request = request.with_comment(comment);
        request = request.with_read_only(read_only);
        request = request.with_skip_validation(skip_validation);
        request = request
            .with_azure_service_principal(azure_service_principal.map(::core::convert::Into::into));
        request = request
            .with_azure_managed_identity(azure_managed_identity.map(::core::convert::Into::into));
        request =
            request.with_azure_storage_key(azure_storage_key.map(::core::convert::Into::into));
        request = request.with_aws_iam_role(aws_iam_role.map(::core::convert::Into::into));
        request = request.with_databricks_gcp_service_account(
            databricks_gcp_service_account.map(::core::convert::Into::into),
        );
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyCredential::from(result))
    }
    #[pyo3(
        signature = (entity_type, entity_name, max_results = None, page_token = None)
    )]
    pub fn list_entity_tag_assignments(
        &self,
        py: Python<'_>,
        entity_type: String,
        entity_name: String,
        max_results: Option<i32>,
        page_token: Option<String>,
    ) -> PyResult<PyListEntityTagAssignmentsResponse> {
        let mut request = self
            .client
            .list_entity_tag_assignments(entity_type, entity_name);
        request = request.with_max_results(max_results);
        request = request.with_page_token(page_token);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyListEntityTagAssignmentsResponse::from(result))
    }
    #[pyo3(signature = (tag_assignment))]
    pub fn create_entity_tag_assignment(
        &self,
        py: Python<'_>,
        tag_assignment: PyEntityTagAssignment,
    ) -> PyResult<PyEntityTagAssignment> {
        let request = self
            .client
            .create_entity_tag_assignment(tag_assignment.into());
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyEntityTagAssignment::from(result))
    }
    #[pyo3(signature = (entity_type, entity_name, tag_key))]
    pub fn get_entity_tag_assignment(
        &self,
        py: Python<'_>,
        entity_type: String,
        entity_name: String,
        tag_key: String,
    ) -> PyResult<PyEntityTagAssignment> {
        let request = self
            .client
            .get_entity_tag_assignment(entity_type, entity_name, tag_key);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyEntityTagAssignment::from(result))
    }
    #[pyo3(
        signature = (
            entity_type,
            entity_name,
            tag_key,
            tag_assignment,
            update_mask = None
        )
    )]
    pub fn update_entity_tag_assignment(
        &self,
        py: Python<'_>,
        entity_type: String,
        entity_name: String,
        tag_key: String,
        tag_assignment: PyEntityTagAssignment,
        update_mask: Option<String>,
    ) -> PyResult<PyEntityTagAssignment> {
        let mut request = self.client.update_entity_tag_assignment(
            entity_type,
            entity_name,
            tag_key,
            tag_assignment.into(),
        );
        request = request.with_update_mask(update_mask);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyEntityTagAssignment::from(result))
    }
    #[pyo3(signature = (entity_type, entity_name, tag_key))]
    pub fn delete_entity_tag_assignment(
        &self,
        py: Python<'_>,
        entity_type: String,
        entity_name: String,
        tag_key: String,
    ) -> PyResult<()> {
        let request = self
            .client
            .delete_entity_tag_assignment(entity_type, entity_name, tag_key);
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
    #[pyo3(signature = (max_results = None, include_browse = None))]
    pub fn list_external_locations(
        &self,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_external_locations();
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyExternalLocation::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
        name: String,
        url: String,
        credential_name: String,
        read_only: Option<bool>,
        comment: Option<String>,
        skip_validation: Option<bool>,
    ) -> PyResult<PyExternalLocation> {
        let mut request = self
            .client
            .create_external_location(name, url, credential_name);
        request = request.with_read_only(read_only);
        request = request.with_comment(comment);
        request = request.with_skip_validation(skip_validation);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyExternalLocation::from(result))
    }
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
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_functions(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyFunction::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
    #[pyo3(signature = (function_info))]
    pub fn create_function(
        &self,
        py: Python<'_>,
        function_info: PyCreateFunction,
    ) -> PyResult<PyFunction> {
        let request = self.client.create_function(function_info.into());
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyFunction::from(result))
    }
    #[pyo3(signature = (full_name, max_results = None, include_browse = None))]
    pub fn list_model_versions(
        &self,
        full_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_model_versions(full_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyModelVersion::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
        model_name: String,
        catalog_name: String,
        schema_name: String,
        source: String,
        run_id: Option<String>,
        comment: Option<String>,
    ) -> PyResult<PyModelVersion> {
        let mut request =
            self.client
                .create_model_version(model_name, catalog_name, schema_name, source);
        request = request.with_run_id(run_id);
        request = request.with_comment(comment);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyModelVersion::from(result))
    }
    #[pyo3(signature = (full_name, version, include_browse = None))]
    pub fn get_model_version(
        &self,
        py: Python<'_>,
        full_name: String,
        version: i64,
        include_browse: Option<bool>,
    ) -> PyResult<PyModelVersion> {
        let mut request = self.client.get_model_version(full_name, version);
        request = request.with_include_browse(include_browse);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyModelVersion::from(result))
    }
    #[pyo3(signature = (full_name, version, comment = None))]
    pub fn update_model_version(
        &self,
        py: Python<'_>,
        full_name: String,
        version: i64,
        comment: Option<String>,
    ) -> PyResult<PyModelVersion> {
        let mut request = self.client.update_model_version(full_name, version);
        request = request.with_comment(comment);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyModelVersion::from(result))
    }
    #[pyo3(signature = (full_name, version))]
    pub fn delete_model_version(
        &self,
        py: Python<'_>,
        full_name: String,
        version: i64,
    ) -> PyResult<()> {
        let request = self.client.delete_model_version(full_name, version);
        block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(())
    }
    #[pyo3(signature = (full_name, version))]
    pub fn finalize_model_version(
        &self,
        py: Python<'_>,
        full_name: String,
        version: i64,
    ) -> PyResult<PyModelVersion> {
        let request = self.client.finalize_model_version(full_name, version);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyModelVersion::from(result))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_providers(&self, max_results: Option<i32>) -> PyResult<ListIterator> {
        let mut request = self.client.list_providers();
        request = request.with_max_results(max_results);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyProvider::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
        name: String,
        authentication_type: PyProviderAuthenticationType,
        owner: Option<String>,
        comment: Option<String>,
        recipient_profile_str: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> PyResult<PyProvider> {
        let mut request = self
            .client
            .create_provider(name, authentication_type.into());
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        request = request.with_recipient_profile_str(recipient_profile_str);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyProvider::from(result))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_recipients(&self, max_results: Option<i32>) -> PyResult<ListIterator> {
        let mut request = self.client.list_recipients();
        request = request.with_max_results(max_results);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyRecipient::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
    #[pyo3(
        signature = (
            name,
            authentication_type,
            owner,
            comment = None,
            properties = None,
            expiration_time = None
        )
    )]
    pub fn create_recipient(
        &self,
        py: Python<'_>,
        name: String,
        authentication_type: PyAuthenticationType,
        owner: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        expiration_time: Option<i64>,
    ) -> PyResult<PyRecipient> {
        let mut request = self
            .client
            .create_recipient(name, authentication_type.into(), owner);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_expiration_time(expiration_time);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyRecipient::from(result))
    }
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
        catalog_name: Option<String>,
        schema_name: Option<String>,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_registered_models();
        request = request.with_catalog_name(catalog_name);
        request = request.with_schema_name(schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyRegisteredModel::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
    #[pyo3(signature = (name, catalog_name, schema_name, comment = None))]
    pub fn create_registered_model(
        &self,
        py: Python<'_>,
        name: String,
        catalog_name: String,
        schema_name: String,
        comment: Option<String>,
    ) -> PyResult<PyRegisteredModel> {
        let mut request = self
            .client
            .create_registered_model(name, catalog_name, schema_name);
        request = request.with_comment(comment);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyRegisteredModel::from(result))
    }
    #[pyo3(signature = (catalog_name, max_results = None, include_browse = None))]
    pub fn list_schemas(
        &self,
        catalog_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_schemas(catalog_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PySchema::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
        name: String,
        catalog_name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
    ) -> PyResult<PySchema> {
        let mut request = self.client.create_schema(name, catalog_name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_storage_root(storage_root);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PySchema::from(result))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_shares(&self, max_results: Option<i32>) -> PyResult<ListIterator> {
        let mut request = self.client.list_shares();
        request = request.with_max_results(max_results);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyShare::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
    #[pyo3(signature = (name, comment = None))]
    pub fn create_share(
        &self,
        py: Python<'_>,
        name: String,
        comment: Option<String>,
    ) -> PyResult<PyShare> {
        let mut request = self.client.create_share(name);
        request = request.with_comment(comment);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyShare::from(result))
    }
    #[pyo3(signature = (name, catalog_name, schema_name))]
    pub fn create_staging_table(
        &self,
        py: Python<'_>,
        name: String,
        catalog_name: String,
        schema_name: String,
    ) -> PyResult<PyStagingTable> {
        let request = self
            .client
            .create_staging_table(name, catalog_name, schema_name);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyStagingTable::from(result))
    }
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
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_delta_metadata: Option<bool>,
        omit_columns: Option<bool>,
        omit_properties: Option<bool>,
        omit_username: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_tables(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_delta_metadata(include_delta_metadata);
        request = request.with_omit_columns(omit_columns);
        request = request.with_omit_properties(omit_properties);
        request = request.with_omit_username(omit_username);
        request = request.with_include_browse(include_browse);
        request = request.with_include_manifest_capabilities(include_manifest_capabilities);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyTable::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
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
    ) -> PyResult<PyTable> {
        let mut request = self.client.create_table(
            name,
            schema_name,
            catalog_name,
            table_type.into(),
            data_source_format.into(),
        );
        if let Some(columns) = columns {
            request = request.with_columns(
                columns
                    .into_iter()
                    .map(::core::convert::Into::into)
                    .collect::<::std::vec::Vec<_>>(),
            );
        }
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_view_definition(view_definition);
        request =
            request.with_view_dependencies(view_dependencies.map(::core::convert::Into::into));
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyTable::from(result))
    }
    #[pyo3(signature = (max_results = None))]
    pub fn list_tag_policies(&self, max_results: Option<i32>) -> PyResult<ListIterator> {
        let mut request = self.client.list_tag_policies();
        request = request.with_max_results(max_results);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyTagPolicy::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
    #[pyo3(signature = (tag_policy))]
    pub fn create_tag_policy(
        &self,
        py: Python<'_>,
        tag_policy: PyTagPolicy,
    ) -> PyResult<PyTagPolicy> {
        let request = self.client.create_tag_policy(tag_policy.into());
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyTagPolicy::from(result))
    }
    #[pyo3(signature = (table_id, operation))]
    pub fn generate_temporary_table_credentials(
        &self,
        py: Python<'_>,
        table_id: String,
        operation: PyGenerateTemporaryTableCredentialsRequestOperation,
    ) -> PyResult<PyTemporaryCredential> {
        let request = self
            .client
            .generate_temporary_table_credentials(table_id, operation.into());
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyTemporaryCredential::from(result))
    }
    #[pyo3(signature = (url, operation, dry_run = None))]
    pub fn generate_temporary_path_credentials(
        &self,
        py: Python<'_>,
        url: String,
        operation: PyGenerateTemporaryPathCredentialsRequestOperation,
        dry_run: Option<bool>,
    ) -> PyResult<PyTemporaryCredential> {
        let mut request = self
            .client
            .generate_temporary_path_credentials(url, operation.into());
        request = request.with_dry_run(dry_run);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyTemporaryCredential::from(result))
    }
    #[pyo3(signature = (volume_id, operation))]
    pub fn generate_temporary_volume_credentials(
        &self,
        py: Python<'_>,
        volume_id: String,
        operation: PyGenerateTemporaryVolumeCredentialsRequestOperation,
    ) -> PyResult<PyTemporaryCredential> {
        let request = self
            .client
            .generate_temporary_volume_credentials(volume_id, operation.into());
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyTemporaryCredential::from(result))
    }
    #[pyo3(signature = (catalog_name, schema_name, model_name, version, operation))]
    pub fn generate_temporary_model_version_credentials(
        &self,
        py: Python<'_>,
        catalog_name: String,
        schema_name: String,
        model_name: String,
        version: i64,
        operation: PyGenerateTemporaryModelVersionCredentialsRequestOperation,
    ) -> PyResult<PyTemporaryCredential> {
        let request = self.client.generate_temporary_model_version_credentials(
            catalog_name,
            schema_name,
            model_name,
            version,
            operation.into(),
        );
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyTemporaryCredential::from(result))
    }
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
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> PyResult<ListIterator> {
        let mut request = self.client.list_volumes(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        let items = request.into_stream().map(|item| {
            let item = item.map_err(PyUnityCatalogError::from)?;
            Python::attach(|py| PyVolume::from(item).into_py_any(py))
        });
        Ok(ListIterator::new(items))
    }
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
        py: Python<'_>,
        catalog_name: String,
        schema_name: String,
        name: String,
        volume_type: PyVolumeType,
        storage_location: Option<String>,
        comment: Option<String>,
    ) -> PyResult<PyVolume> {
        let mut request =
            self.client
                .create_volume(catalog_name, schema_name, name, volume_type.into());
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        let result = block_on(py, request.into_future())?.map_err(PyUnityCatalogError::from)?;
        Ok(PyVolume::from(result))
    }
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
    pub fn catalog(&self, catalog_name: String) -> PyCatalogClient {
        PyCatalogClient {
            client: self.client.catalog(catalog_name),
        }
    }
    pub fn credential(&self, credential_name: String) -> PyCredentialClient {
        PyCredentialClient {
            client: self.client.credential(credential_name),
        }
    }
    pub fn external_location(&self, external_location_name: String) -> PyExternalLocationClient {
        PyExternalLocationClient {
            client: self.client.external_location(external_location_name),
        }
    }
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
    pub fn policy(&self, policy_name: String) -> PyPolicyClient {
        PyPolicyClient {
            client: self.client.policy(policy_name),
        }
    }
    pub fn provider(&self, provider_name: String) -> PyProviderClient {
        PyProviderClient {
            client: self.client.provider(provider_name),
        }
    }
    pub fn recipient(&self, recipient_name: String) -> PyRecipientClient {
        PyRecipientClient {
            client: self.client.recipient(recipient_name),
        }
    }
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
    pub fn schema(&self, catalog_name: String, schema_name: String) -> PySchemaClient {
        let full_name = format!("{}.{}", catalog_name, schema_name);
        PySchemaClient {
            client: self.client.schema_from_full_name(full_name),
        }
    }
    pub fn share(&self, share_name: String) -> PyShareClient {
        PyShareClient {
            client: self.client.share(share_name),
        }
    }
    pub fn staging_table(&self, staging_table_name: String) -> PyStagingTableClient {
        PyStagingTableClient {
            client: self.client.staging_table(staging_table_name),
        }
    }
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
    pub fn tag_policy(&self, tag_policy_name: String) -> PyTagPolicyClient {
        PyTagPolicyClient {
            client: self.client.tag_policy(tag_policy_name),
        }
    }
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
/// Add every generated client class (and the list pagers) to the extension module `m`.
///
/// Call this from the crate's `#[pymodule]` so newly generated clients are exposed
/// without hand-maintaining an `add_class` list.
pub fn register_clients(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyUnityCatalogClient>()?;
    m.add_class::<PyAgentSkillClient>()?;
    m.add_class::<PyAgentClient>()?;
    m.add_class::<PyCatalogClient>()?;
    m.add_class::<PyCredentialClient>()?;
    m.add_class::<PyExternalLocationClient>()?;
    m.add_class::<PyFunctionClient>()?;
    m.add_class::<PyPolicyClient>()?;
    m.add_class::<PyProviderClient>()?;
    m.add_class::<PyRecipientClient>()?;
    m.add_class::<PyRegisteredModelClient>()?;
    m.add_class::<PySchemaClient>()?;
    m.add_class::<PyShareClient>()?;
    m.add_class::<PyStagingTableClient>()?;
    m.add_class::<PyTableClient>()?;
    m.add_class::<PyTagPolicyClient>()?;
    m.add_class::<PyVolumeClient>()?;
    m.add_class::<ListIterator>()?;
    Ok(())
}
