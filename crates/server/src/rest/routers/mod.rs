use crate::api::{
    AgentHandler, AgentSkillHandler, CatalogHandler, CredentialHandler, EntityTagAssignmentHandler,
    ExternalLocationHandler, FunctionHandler, ModelVersionHandler, PolicyHandler, ProviderHandler,
    RecipientHandler, RegisteredModelHandler, SchemaHandler, ShareHandler, StagingTableHandler,
    TableHandler, TagPolicyHandler, TemporaryCredentialHandler, VolumeHandler,
};
use axum::routing::{delete, get, patch, post};

pub use sharing::get_router as create_sharing_router;
pub use sharing::open_sharing_router as create_open_sharing_router;
pub use unitycatalog_delta_api::get_router as create_delta_router;

mod sharing;

pub fn create_catalogs_router<T, Cx>(handler: T) -> axum::Router
where
    T: CatalogHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::catalogs::server::*;

    axum::Router::new()
        .route("/catalogs", get(list_catalogs::<T, Cx>))
        .route("/catalogs", post(create_catalog::<T, Cx>))
        .route("/catalogs/{name}", get(get_catalog::<T, Cx>))
        .route("/catalogs/{name}", patch(update_catalog::<T, Cx>))
        .route("/catalogs/{name}", delete(delete_catalog::<T, Cx>))
        .with_state(handler)
}

pub fn create_credentials_router<T, Cx>(handler: T) -> axum::Router
where
    T: CredentialHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::credentials::server::*;

    axum::Router::new()
        .route("/credentials", get(list_credentials::<T, Cx>))
        .route("/credentials", post(create_credential::<T, Cx>))
        .route("/credentials/{name}", get(get_credential::<T, Cx>))
        .route("/credentials/{name}", patch(update_credential::<T, Cx>))
        .route("/credentials/{name}", delete(delete_credential::<T, Cx>))
        .with_state(handler)
}

pub fn create_external_locations_router<T, Cx>(handler: T) -> axum::Router
where
    T: ExternalLocationHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::external_locations::server::*;

    axum::Router::new()
        .route("/external-locations", get(list_external_locations::<T, Cx>))
        .route(
            "/external-locations",
            post(create_external_location::<T, Cx>),
        )
        .route(
            "/external-locations/{name}",
            get(get_external_location::<T, Cx>),
        )
        .route(
            "/external-locations/{name}",
            patch(update_external_location::<T, Cx>),
        )
        .route(
            "/external-locations/{name}",
            delete(delete_external_location::<T, Cx>),
        )
        .with_state(handler)
}

pub fn create_recipients_router<T, Cx>(handler: T) -> axum::Router
where
    T: RecipientHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::recipients::server::*;

    axum::Router::new()
        .route("/recipients", get(list_recipients::<T, Cx>))
        .route("/recipients", post(create_recipient::<T, Cx>))
        .route("/recipients/{name}", get(get_recipient::<T, Cx>))
        .route("/recipients/{name}", patch(update_recipient::<T, Cx>))
        .route("/recipients/{name}", delete(delete_recipient::<T, Cx>))
        .with_state(handler)
}

pub fn create_providers_router<T, Cx>(handler: T) -> axum::Router
where
    T: ProviderHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::providers::server::*;

    axum::Router::new()
        .route("/providers", get(list_providers::<T, Cx>))
        .route("/providers", post(create_provider::<T, Cx>))
        .route("/providers/{name}", get(get_provider::<T, Cx>))
        .route("/providers/{name}", patch(update_provider::<T, Cx>))
        .route("/providers/{name}", delete(delete_provider::<T, Cx>))
        .with_state(handler)
}

pub fn create_schemas_router<T, Cx>(handler: T) -> axum::Router
where
    T: SchemaHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::schemas::server::*;

    axum::Router::new()
        .route("/schemas", get(list_schemas::<T, Cx>))
        .route("/schemas", post(create_schema::<T, Cx>))
        .route("/schemas/{name}", get(get_schema::<T, Cx>))
        .route("/schemas/{name}", patch(update_schema::<T, Cx>))
        .route("/schemas/{name}", delete(delete_schema::<T, Cx>))
        .with_state(handler)
}

pub fn create_staging_tables_router<T, Cx>(handler: T) -> axum::Router
where
    T: StagingTableHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::staging_tables::server::*;

    axum::Router::new()
        .route("/staging-tables", post(create_staging_table::<T, Cx>))
        .with_state(handler)
}

pub fn create_shares_router<T, Cx>(handler: T) -> axum::Router
where
    T: ShareHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::shares::server::*;

    axum::Router::new()
        .route("/shares", get(list_shares::<T, Cx>))
        .route("/shares", post(create_share::<T, Cx>))
        .route("/shares/{name}", get(get_share::<T, Cx>))
        .route("/shares/{name}", patch(update_share::<T, Cx>))
        .route("/shares/{name}", delete(delete_share::<T, Cx>))
        .route("/shares/{name}/permissions", get(get_permissions::<T, Cx>))
        .route(
            "/shares/{name}/permissions",
            patch(update_permissions::<T, Cx>),
        )
        .with_state(handler)
}

pub fn create_tables_router<T, Cx>(handler: T) -> axum::Router
where
    T: TableHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::tables::server::*;

    axum::Router::new()
        .route("/table-summaries", get(list_table_summaries::<T, Cx>))
        .route("/tables", get(list_tables::<T, Cx>))
        .route("/tables", post(create_table::<T, Cx>))
        .route("/tables/{full_name}", get(get_table::<T, Cx>))
        .route("/tables/{full_name}/exists", get(get_table_exists::<T, Cx>))
        .route("/tables/{full_name}", delete(delete_table::<T, Cx>))
        .with_state(handler)
}

pub fn create_temporary_credentials_router<T, Cx>(handler: T) -> axum::Router
where
    T: TemporaryCredentialHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::temporary_credentials::server::*;

    axum::Router::new()
        .route(
            "/temporary-table-credentials",
            post(generate_temporary_table_credentials::<T, Cx>),
        )
        .route(
            "/temporary-path-credentials",
            post(generate_temporary_path_credentials::<T, Cx>),
        )
        .route(
            "/temporary-volume-credentials",
            post(generate_temporary_volume_credentials::<T, Cx>),
        )
        .route(
            "/temporary-model-version-credentials",
            post(generate_temporary_model_version_credentials::<T, Cx>),
        )
        .with_state(handler)
}

pub fn create_registered_models_router<T, Cx>(handler: T) -> axum::Router
where
    T: RegisteredModelHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::registered_models::server::*;

    axum::Router::new()
        .route("/models", get(list_registered_models::<T, Cx>))
        .route("/models", post(create_registered_model::<T, Cx>))
        .route("/models/{full_name}", get(get_registered_model::<T, Cx>))
        .route(
            "/models/{full_name}",
            patch(update_registered_model::<T, Cx>),
        )
        .route(
            "/models/{full_name}",
            delete(delete_registered_model::<T, Cx>),
        )
        .with_state(handler)
}

pub fn create_model_versions_router<T, Cx>(handler: T) -> axum::Router
where
    T: ModelVersionHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::model_versions::server::*;

    axum::Router::new()
        .route("/models/versions", post(create_model_version::<T, Cx>))
        .route(
            "/models/{full_name}/versions",
            get(list_model_versions::<T, Cx>),
        )
        .route(
            "/models/{full_name}/versions/{version}",
            get(get_model_version::<T, Cx>),
        )
        .route(
            "/models/{full_name}/versions/{version}",
            patch(update_model_version::<T, Cx>),
        )
        .route(
            "/models/{full_name}/versions/{version}",
            delete(delete_model_version::<T, Cx>),
        )
        .route(
            "/models/{full_name}/versions/{version}/finalize",
            patch(finalize_model_version::<T, Cx>),
        )
        .with_state(handler)
}

pub fn create_volumes_router<T, Cx>(handler: T) -> axum::Router
where
    T: VolumeHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::volumes::server::*;

    axum::Router::new()
        .route("/volumes", get(list_volumes::<T, Cx>))
        .route("/volumes", post(create_volume::<T, Cx>))
        .route("/volumes/{name}", get(get_volume::<T, Cx>))
        .route("/volumes/{name}", patch(update_volume::<T, Cx>))
        .route("/volumes/{name}", delete(delete_volume::<T, Cx>))
        .with_state(handler)
}

pub fn create_agent_skills_router<T, Cx>(handler: T) -> axum::Router
where
    T: AgentSkillHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::agent_skills::server::*;

    axum::Router::new()
        .route("/agent-skills", get(list_agent_skills::<T, Cx>))
        .route("/agent-skills", post(create_agent_skill::<T, Cx>))
        .route("/agent-skills/{name}", get(get_agent_skill::<T, Cx>))
        .route("/agent-skills/{name}", patch(update_agent_skill::<T, Cx>))
        .route("/agent-skills/{name}", delete(delete_agent_skill::<T, Cx>))
        .with_state(handler)
}

pub fn create_agents_router<T, Cx>(handler: T) -> axum::Router
where
    T: AgentHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::agents::server::*;

    axum::Router::new()
        .route("/agents", get(list_agents::<T, Cx>))
        .route("/agents", post(create_agent::<T, Cx>))
        .route("/agents/{name}", get(get_agent::<T, Cx>))
        .route("/agents/{name}", patch(update_agent::<T, Cx>))
        .route("/agents/{name}", delete(delete_agent::<T, Cx>))
        .with_state(handler)
}

pub fn create_functions_router<T, Cx>(handler: T) -> axum::Router
where
    T: FunctionHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::functions::server::*;

    axum::Router::new()
        .route("/functions", get(list_functions::<T, Cx>))
        .route("/functions", post(create_function::<T, Cx>))
        .route("/functions/{name}", get(get_function::<T, Cx>))
        .route("/functions/{name}", patch(update_function::<T, Cx>))
        .route("/functions/{name}", delete(delete_function::<T, Cx>))
        .with_state(handler)
}

/// Router for the ABAC Policies API (row-filter / column-mask policies bound to securables).
pub fn create_policies_router<T, Cx>(handler: T) -> axum::Router
where
    T: PolicyHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::policies::server::*;

    axum::Router::new()
        .route(
            "/policies/{on_securable_type}/{on_securable_fullname}",
            get(list_policies::<T, Cx>),
        )
        .route(
            "/policies/{on_securable_type}/{on_securable_fullname}",
            post(create_policy::<T, Cx>),
        )
        .route(
            "/policies/{on_securable_type}/{on_securable_fullname}/{name}",
            get(get_policy::<T, Cx>),
        )
        .route(
            "/policies/{on_securable_type}/{on_securable_fullname}/{name}",
            patch(update_policy::<T, Cx>),
        )
        .route(
            "/policies/{on_securable_type}/{on_securable_fullname}/{name}",
            delete(delete_policy::<T, Cx>),
        )
        .with_state(handler)
}

/// Router for the Tag Policies (governed tag definitions) API.
///
/// Unlike the Unity Catalog securable APIs, these routes live under `/api/2.1/tag-policies`
/// (not `/api/2.1/unity-catalog/...`), so this router is mounted at a distinct base by the
/// server setup.
pub fn create_tag_policies_router<T, Cx>(handler: T) -> axum::Router
where
    T: TagPolicyHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::tag_policies::server::*;

    axum::Router::new()
        .route("/tag-policies", get(list_tag_policies::<T, Cx>))
        .route("/tag-policies", post(create_tag_policy::<T, Cx>))
        .route("/tag-policies/{tag_key}", get(get_tag_policy::<T, Cx>))
        .route("/tag-policies/{tag_key}", patch(update_tag_policy::<T, Cx>))
        .route(
            "/tag-policies/{tag_key}",
            delete(delete_tag_policy::<T, Cx>),
        )
        .with_state(handler)
}

/// Router for the Entity Tag Assignments API (applying tags to UC securables).
///
/// These routes live under the Unity Catalog surface
/// (`/api/2.1/unity-catalog/entity-tag-assignments/...`), so this router is merged into the
/// `unity-catalog`-nested route set.
pub fn create_entity_tag_assignments_router<T, Cx>(handler: T) -> axum::Router
where
    T: EntityTagAssignmentHandler<Cx> + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    use crate::codegen::entity_tag_assignments::server::*;

    axum::Router::new()
        .route(
            "/entity-tag-assignments",
            post(create_entity_tag_assignment::<T, Cx>),
        )
        .route(
            "/entity-tag-assignments/{entity_type}/{entity_name}/tags",
            get(list_entity_tag_assignments::<T, Cx>),
        )
        .route(
            "/entity-tag-assignments/{entity_type}/{entity_name}/tags/{tag_key}",
            get(get_entity_tag_assignment::<T, Cx>),
        )
        .route(
            "/entity-tag-assignments/{entity_type}/{entity_name}/tags/{tag_key}",
            patch(update_entity_tag_assignment::<T, Cx>),
        )
        .route(
            "/entity-tag-assignments/{entity_type}/{entity_name}/tags/{tag_key}",
            delete(delete_entity_tag_assignment::<T, Cx>),
        )
        .with_state(handler)
}
