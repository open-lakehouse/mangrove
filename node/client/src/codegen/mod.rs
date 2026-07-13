// @generated — do not edit by hand.
#![allow(unused_mut, unused_imports, dead_code, clippy::all)]
pub mod agent_skills;
pub mod agents;
pub mod catalogs;
pub mod credentials;
pub mod external_locations;
pub mod functions;
pub mod policies;
pub mod providers;
pub mod recipients;
pub mod schemas;
pub mod shares;
pub mod staging_tables;
pub mod tables;
pub mod tag_policies;
pub mod volumes;
use crate::codegen::agent_skills::NapiAgentSkillClient;
use crate::codegen::agents::NapiAgentClient;
use crate::codegen::catalogs::NapiCatalogClient;
use crate::codegen::credentials::NapiCredentialClient;
use crate::codegen::external_locations::NapiExternalLocationClient;
use crate::codegen::functions::NapiFunctionClient;
use crate::codegen::policies::NapiPolicyClient;
use crate::codegen::providers::NapiProviderClient;
use crate::codegen::recipients::NapiRecipientClient;
use crate::codegen::schemas::NapiSchemaClient;
use crate::codegen::shares::NapiShareClient;
use crate::codegen::staging_tables::NapiStagingTableClient;
use crate::codegen::tables::NapiTableClient;
use crate::codegen::tag_policies::NapiTagPolicyClient;
use crate::codegen::volumes::NapiVolumeClient;
use crate::error::NapiErrorExt;
use buffa::Message;
use futures::StreamExt;
use futures::stream::TryStreamExt;
use napi::Env;
use napi::bindgen_prelude::{Buffer, ReadableStream};
use napi_derive::napi;
use std::collections::HashMap;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::agent_skills::v0alpha1::*;
use unitycatalog_common::models::agents::v0alpha1::*;
use unitycatalog_common::models::catalogs::v1::*;
use unitycatalog_common::models::credentials::v1::*;
use unitycatalog_common::models::external_locations::v1::*;
use unitycatalog_common::models::functions::v1::*;
use unitycatalog_common::models::policies::v1::*;
use unitycatalog_common::models::providers::v1::*;
use unitycatalog_common::models::recipients::v1::*;
use unitycatalog_common::models::schemas::v1::*;
use unitycatalog_common::models::shares::v1::*;
use unitycatalog_common::models::staging_tables::v1::*;
use unitycatalog_common::models::tables::v1::*;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::tags::v1::*;
use unitycatalog_common::models::temporary_credentials::v1::*;
use unitycatalog_common::models::volumes::v1::*;
#[napi]
pub struct NapiUnityCatalogClient {
    client: UnityCatalogClient,
}
#[napi]
impl NapiUnityCatalogClient {
    #[napi(factory)]
    pub fn from_url(base_url: String, token: Option<String>) -> napi::Result<Self> {
        let client = if let Some(token) = token {
            olai_http::CloudClient::new_with_token(token)
        } else {
            olai_http::CloudClient::new_unauthenticated()
        };
        let base_url = base_url.parse().map_err(|e: url::ParseError| {
            napi::Error::new(napi::Status::GenericFailure, e.to_string())
        })?;
        Ok(Self {
            client: UnityCatalogClient::new(client, base_url),
        })
    }
    #[napi(catch_unwind)]
    pub async fn list_agent_skills(
        &self,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_agent_skills(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_agent_skills_stream(
        &self,
        env: Env,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_agent_skills(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_agent_skill(
        &self,
        catalog_name: String,
        schema_name: String,
        name: String,
        agent_skill_type: i32,
        storage_location: Option<String>,
        description: Option<String>,
        license: Option<String>,
        allowed_tools: Option<Vec<String>>,
        metadata: Option<HashMap<String, String>>,
        comment: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_agent_skill(
            catalog_name,
            schema_name,
            name,
            <AgentSkillType as buffa::Enumeration>::from_i32(agent_skill_type).ok_or_else(
                || napi::Error::new(napi::Status::GenericFailure, "invalid enum value"),
            )?,
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
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_agents(
        &self,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_agents(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_agents_stream(
        &self,
        env: Env,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_agents(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_agent(
        &self,
        catalog_name: String,
        schema_name: String,
        name: String,
        invocation_protocol: i32,
        endpoint: String,
        description: Option<String>,
        capabilities: Option<Vec<String>>,
        input_schema: Option<String>,
        comment: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_agent(
            catalog_name,
            schema_name,
            name,
            <InvocationProtocol as buffa::Enumeration>::from_i32(invocation_protocol).ok_or_else(
                || napi::Error::new(napi::Status::GenericFailure, "invalid enum value"),
            )?,
            endpoint,
        );
        request = request.with_description(description);
        if let Some(capabilities) = capabilities {
            request = request.with_capabilities(capabilities);
        }
        request = request.with_input_schema(input_schema);
        request = request.with_comment(comment);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_catalogs(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_catalogs();
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_catalogs_stream(
        &self,
        env: Env,
        max_results: Option<i32>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_catalogs();
        request = request.with_max_results(max_results);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_catalog(
        &self,
        name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
        provider_name: Option<String>,
        share_name: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_catalog(name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_storage_root(storage_root);
        request = request.with_provider_name(provider_name);
        request = request.with_share_name(share_name);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_credentials(
        &self,
        purpose: Option<i32>,
        max_results: Option<i32>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_credentials();
        request = request.with_purpose(purpose.and_then(<Purpose as buffa::Enumeration>::from_i32));
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_credentials_stream(
        &self,
        env: Env,
        purpose: Option<i32>,
        max_results: Option<i32>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_credentials();
        request = request.with_purpose(purpose.and_then(<Purpose as buffa::Enumeration>::from_i32));
        request = request.with_max_results(max_results);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_credential(
        &self,
        name: String,
        purpose: i32,
        comment: Option<String>,
        read_only: Option<bool>,
        skip_validation: Option<bool>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_credential(
            name,
            <Purpose as buffa::Enumeration>::from_i32(purpose).ok_or_else(|| {
                napi::Error::new(napi::Status::GenericFailure, "invalid enum value")
            })?,
        );
        request = request.with_comment(comment);
        request = request.with_read_only(read_only);
        request = request.with_skip_validation(skip_validation);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_entity_tag_assignments(
        &self,
        entity_type: String,
        entity_name: String,
        max_results: Option<i32>,
        page_token: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .client
            .list_entity_tag_assignments(entity_type, entity_name);
        request = request.with_max_results(max_results);
        request = request.with_page_token(page_token);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_entity_tag_assignment(
        &self,
        tag_assignment: napi::bindgen_prelude::Buffer,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_entity_tag_assignment(
            <EntityTagAssignment as buffa::Message>::decode_from_slice(tag_assignment.as_ref())
                .map_err(|e| {
                    napi::Error::new(
                        napi::Status::GenericFailure,
                        format!("invalid {} payload: {e}", stringify!(EntityTagAssignment)),
                    )
                })?,
        );
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn get_entity_tag_assignment(
        &self,
        entity_type: String,
        entity_name: String,
        tag_key: String,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .client
            .get_entity_tag_assignment(entity_type, entity_name, tag_key);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn update_entity_tag_assignment(
        &self,
        entity_type: String,
        entity_name: String,
        tag_key: String,
        tag_assignment: napi::bindgen_prelude::Buffer,
        update_mask: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.update_entity_tag_assignment(
            entity_type,
            entity_name,
            tag_key,
            <EntityTagAssignment as buffa::Message>::decode_from_slice(tag_assignment.as_ref())
                .map_err(|e| {
                    napi::Error::new(
                        napi::Status::GenericFailure,
                        format!("invalid {} payload: {e}", stringify!(EntityTagAssignment)),
                    )
                })?,
        );
        request = request.with_update_mask(update_mask);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn delete_entity_tag_assignment(
        &self,
        entity_type: String,
        entity_name: String,
        tag_key: String,
    ) -> napi::Result<()> {
        let mut request =
            self.client
                .delete_entity_tag_assignment(entity_type, entity_name, tag_key);
        request.await.default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_external_locations(
        &self,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_external_locations();
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_external_locations_stream(
        &self,
        env: Env,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_external_locations();
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_external_location(
        &self,
        name: String,
        url: String,
        credential_name: String,
        read_only: Option<bool>,
        comment: Option<String>,
        skip_validation: Option<bool>,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .client
            .create_external_location(name, url, credential_name);
        request = request.with_read_only(read_only);
        request = request.with_comment(comment);
        request = request.with_skip_validation(skip_validation);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_functions(
        &self,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_functions(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_functions_stream(
        &self,
        env: Env,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_functions(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_function(
        &self,
        name: String,
        catalog_name: String,
        schema_name: String,
        data_type: String,
        full_data_type: String,
        parameter_style: i32,
        is_deterministic: bool,
        sql_data_access: i32,
        is_null_call: bool,
        security_type: i32,
        routine_body: i32,
        routine_definition: Option<String>,
        routine_body_language: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_function(
            name,
            catalog_name,
            schema_name,
            data_type,
            full_data_type,
            <ParameterStyle as buffa::Enumeration>::from_i32(parameter_style).ok_or_else(|| {
                napi::Error::new(napi::Status::GenericFailure, "invalid enum value")
            })?,
            is_deterministic,
            <SqlDataAccess as buffa::Enumeration>::from_i32(sql_data_access).ok_or_else(|| {
                napi::Error::new(napi::Status::GenericFailure, "invalid enum value")
            })?,
            is_null_call,
            <SecurityType as buffa::Enumeration>::from_i32(security_type).ok_or_else(|| {
                napi::Error::new(napi::Status::GenericFailure, "invalid enum value")
            })?,
            <RoutineBody as buffa::Enumeration>::from_i32(routine_body).ok_or_else(|| {
                napi::Error::new(napi::Status::GenericFailure, "invalid enum value")
            })?,
        );
        request = request.with_routine_definition(routine_definition);
        request = request.with_routine_body_language(routine_body_language);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_providers(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_providers();
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_providers_stream(
        &self,
        env: Env,
        max_results: Option<i32>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_providers();
        request = request.with_max_results(max_results);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_provider(
        &self,
        name: String,
        authentication_type: i32,
        owner: Option<String>,
        comment: Option<String>,
        recipient_profile_str: Option<String>,
        properties: Option<HashMap<String, String>>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_provider(
            name,
            <ProviderAuthenticationType as buffa::Enumeration>::from_i32(authentication_type)
                .ok_or_else(|| {
                    napi::Error::new(napi::Status::GenericFailure, "invalid enum value")
                })?,
        );
        request = request.with_owner(owner);
        request = request.with_comment(comment);
        request = request.with_recipient_profile_str(recipient_profile_str);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_recipients(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_recipients();
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_recipients_stream(
        &self,
        env: Env,
        max_results: Option<i32>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_recipients();
        request = request.with_max_results(max_results);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_recipient(
        &self,
        name: String,
        authentication_type: i32,
        owner: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        expiration_time: Option<i64>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_recipient(
            name,
            <AuthenticationType as buffa::Enumeration>::from_i32(authentication_type).ok_or_else(
                || napi::Error::new(napi::Status::GenericFailure, "invalid enum value"),
            )?,
            owner,
        );
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_expiration_time(expiration_time);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_schemas(
        &self,
        catalog_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_schemas(catalog_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_schemas_stream(
        &self,
        env: Env,
        catalog_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_schemas(catalog_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_schema(
        &self,
        name: String,
        catalog_name: String,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        storage_root: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_schema(name, catalog_name);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_storage_root(storage_root);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_shares(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_shares();
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_shares_stream(
        &self,
        env: Env,
        max_results: Option<i32>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_shares();
        request = request.with_max_results(max_results);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_share(
        &self,
        name: String,
        comment: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_share(name);
        request = request.with_comment(comment);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn create_staging_table(
        &self,
        name: String,
        catalog_name: String,
        schema_name: String,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .client
            .create_staging_table(name, catalog_name, schema_name);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_tables(
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
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_tables(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_delta_metadata(include_delta_metadata);
        request = request.with_omit_columns(omit_columns);
        request = request.with_omit_properties(omit_properties);
        request = request.with_omit_username(omit_username);
        request = request.with_include_browse(include_browse);
        request = request.with_include_manifest_capabilities(include_manifest_capabilities);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_tables_stream(
        &self,
        env: Env,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_delta_metadata: Option<bool>,
        omit_columns: Option<bool>,
        omit_properties: Option<bool>,
        omit_username: Option<bool>,
        include_browse: Option<bool>,
        include_manifest_capabilities: Option<bool>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_tables(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_delta_metadata(include_delta_metadata);
        request = request.with_omit_columns(omit_columns);
        request = request.with_omit_properties(omit_properties);
        request = request.with_omit_username(omit_username);
        request = request.with_include_browse(include_browse);
        request = request.with_include_manifest_capabilities(include_manifest_capabilities);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_table(
        &self,
        name: String,
        schema_name: String,
        catalog_name: String,
        table_type: i32,
        data_source_format: i32,
        storage_location: Option<String>,
        comment: Option<String>,
        properties: Option<HashMap<String, String>>,
        view_definition: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_table(
            name,
            schema_name,
            catalog_name,
            <TableType as buffa::Enumeration>::from_i32(table_type).ok_or_else(|| {
                napi::Error::new(napi::Status::GenericFailure, "invalid enum value")
            })?,
            <DataSourceFormat as buffa::Enumeration>::from_i32(data_source_format).ok_or_else(
                || napi::Error::new(napi::Status::GenericFailure, "invalid enum value"),
            )?,
        );
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        if let Some(properties) = properties {
            request = request.with_properties(properties);
        }
        request = request.with_view_definition(view_definition);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_tag_policies(&self, max_results: Option<i32>) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_tag_policies();
        request = request.with_max_results(max_results);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_tag_policies_stream(
        &self,
        env: Env,
        max_results: Option<i32>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_tag_policies();
        request = request.with_max_results(max_results);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_tag_policy(
        &self,
        tag_policy: napi::bindgen_prelude::Buffer,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_tag_policy(
            <TagPolicy as buffa::Message>::decode_from_slice(tag_policy.as_ref()).map_err(|e| {
                napi::Error::new(
                    napi::Status::GenericFailure,
                    format!("invalid {} payload: {e}", stringify!(TagPolicy)),
                )
            })?,
        );
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn generate_temporary_table_credentials(
        &self,
        table_id: String,
        operation: i32,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .client
            .generate_temporary_table_credentials(
                table_id,
                <generate_temporary_table_credentials_request::Operation as buffa::Enumeration>::from_i32(
                        operation,
                    )
                    .ok_or_else(|| napi::Error::new(
                        napi::Status::GenericFailure,
                        "invalid enum value",
                    ))?,
            );
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn generate_temporary_path_credentials(
        &self,
        url: String,
        operation: i32,
        dry_run: Option<bool>,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .client
            .generate_temporary_path_credentials(
                url,
                <generate_temporary_path_credentials_request::Operation as buffa::Enumeration>::from_i32(
                        operation,
                    )
                    .ok_or_else(|| napi::Error::new(
                        napi::Status::GenericFailure,
                        "invalid enum value",
                    ))?,
            );
        request = request.with_dry_run(dry_run);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn generate_temporary_volume_credentials(
        &self,
        volume_id: String,
        operation: i32,
    ) -> napi::Result<Buffer> {
        let mut request = self
            .client
            .generate_temporary_volume_credentials(
                volume_id,
                <generate_temporary_volume_credentials_request::Operation as buffa::Enumeration>::from_i32(
                        operation,
                    )
                    .ok_or_else(|| napi::Error::new(
                        napi::Status::GenericFailure,
                        "invalid enum value",
                    ))?,
            );
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi(catch_unwind)]
    pub async fn list_volumes(
        &self,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<Vec<Buffer>> {
        let mut request = self.client.list_volumes(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        request
            .into_stream()
            .map_ok(|item| Buffer::from(item.encode_to_vec()))
            .try_collect::<Vec<_>>()
            .await
            .default_error()
    }
    #[napi(catch_unwind)]
    pub fn list_volumes_stream(
        &self,
        env: Env,
        catalog_name: String,
        schema_name: String,
        max_results: Option<i32>,
        include_browse: Option<bool>,
    ) -> napi::Result<ReadableStream<'_, Buffer>> {
        let mut request = self.client.list_volumes(catalog_name, schema_name);
        request = request.with_max_results(max_results);
        request = request.with_include_browse(include_browse);
        ReadableStream::new(
            &env,
            request.into_stream().map(|item| {
                item.map(|v| Buffer::from(v.encode_to_vec()))
                    .map_err(|e| crate::error::convert_error(&e))
            }),
        )
    }
    #[napi(catch_unwind)]
    pub async fn create_volume(
        &self,
        catalog_name: String,
        schema_name: String,
        name: String,
        volume_type: i32,
        storage_location: Option<String>,
        comment: Option<String>,
    ) -> napi::Result<Buffer> {
        let mut request = self.client.create_volume(
            catalog_name,
            schema_name,
            name,
            <VolumeType as buffa::Enumeration>::from_i32(volume_type).ok_or_else(|| {
                napi::Error::new(napi::Status::GenericFailure, "invalid enum value")
            })?,
        );
        request = request.with_storage_location(storage_location);
        request = request.with_comment(comment);
        request
            .await
            .map(|item| Buffer::from(item.encode_to_vec()))
            .default_error()
    }
    #[napi]
    pub fn agent_skill(
        &self,
        catalog_name: String,
        schema_name: String,
        agent_skill_name: String,
    ) -> NapiAgentSkillClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, agent_skill_name);
        NapiAgentSkillClient {
            client: self.client.agent_skill_from_full_name(full_name),
        }
    }
    #[napi]
    pub fn agent(
        &self,
        catalog_name: String,
        schema_name: String,
        agent_name: String,
    ) -> NapiAgentClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, agent_name);
        NapiAgentClient {
            client: self.client.agent_from_full_name(full_name),
        }
    }
    #[napi]
    pub fn catalog(&self, catalog_name: String) -> NapiCatalogClient {
        NapiCatalogClient {
            client: self.client.catalog(catalog_name),
        }
    }
    #[napi]
    pub fn credential(&self, credential_name: String) -> NapiCredentialClient {
        NapiCredentialClient {
            client: self.client.credential(credential_name),
        }
    }
    #[napi]
    pub fn external_location(&self, external_location_name: String) -> NapiExternalLocationClient {
        NapiExternalLocationClient {
            client: self.client.external_location(external_location_name),
        }
    }
    #[napi]
    pub fn function(
        &self,
        catalog_name: String,
        schema_name: String,
        function_name: String,
    ) -> NapiFunctionClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, function_name);
        NapiFunctionClient {
            client: self.client.function_from_full_name(full_name),
        }
    }
    #[napi]
    pub fn policy(&self, policy_name: String) -> NapiPolicyClient {
        NapiPolicyClient {
            client: self.client.policy(policy_name),
        }
    }
    #[napi]
    pub fn provider(&self, provider_name: String) -> NapiProviderClient {
        NapiProviderClient {
            client: self.client.provider(provider_name),
        }
    }
    #[napi]
    pub fn recipient(&self, recipient_name: String) -> NapiRecipientClient {
        NapiRecipientClient {
            client: self.client.recipient(recipient_name),
        }
    }
    #[napi]
    pub fn schema(&self, catalog_name: String, schema_name: String) -> NapiSchemaClient {
        let full_name = format!("{}.{}", catalog_name, schema_name);
        NapiSchemaClient {
            client: self.client.schema_from_full_name(full_name),
        }
    }
    #[napi]
    pub fn share(&self, share_name: String) -> NapiShareClient {
        NapiShareClient {
            client: self.client.share(share_name),
        }
    }
    #[napi]
    pub fn staging_table(&self, staging_table_name: String) -> NapiStagingTableClient {
        NapiStagingTableClient {
            client: self.client.staging_table(staging_table_name),
        }
    }
    #[napi]
    pub fn table(
        &self,
        catalog_name: String,
        schema_name: String,
        table_name: String,
    ) -> NapiTableClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, table_name);
        NapiTableClient {
            client: self.client.table_from_full_name(full_name),
        }
    }
    #[napi]
    pub fn tag_policy(&self, tag_policy_name: String) -> NapiTagPolicyClient {
        NapiTagPolicyClient {
            client: self.client.tag_policy(tag_policy_name),
        }
    }
    #[napi]
    pub fn volume(
        &self,
        catalog_name: String,
        schema_name: String,
        volume_name: String,
    ) -> NapiVolumeClient {
        let full_name = format!("{}.{}.{}", catalog_name, schema_name, volume_name);
        NapiVolumeClient {
            client: self.client.volume_from_full_name(full_name),
        }
    }
}
