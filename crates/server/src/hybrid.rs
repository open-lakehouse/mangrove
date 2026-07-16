//! Hybrid proxy router wiring.
//!
//! Composes a local Unity Catalog server with an upstream instance: each REST
//! surface is wired at startup to either the local [`ServerHandler`] or a
//! per-surface upstream adapter (from [`crate::handlers::upstream`]), decided by
//! [`RoutingConfig`]. The adapters enforce this server's policy locally before
//! forwarding; see that module for details. Any surface not marked `upstream` is
//! always served locally.

use std::sync::Arc;

use axum::Router;
use unitycatalog_client::UnityCatalogClient;

use crate::api::RequestContext;
use crate::config::{RoutingConfig, RoutingMode};
use crate::handlers::upstream::{
    UpstreamCatalogHandler, UpstreamSchemaHandler, UpstreamTableHandler,
};
use crate::policy::Policy;
use crate::rest::{
    create_catalogs_router, create_credentials_router, create_delta_router,
    create_entity_tag_assignments_router, create_external_locations_router,
    create_functions_router, create_policies_router, create_providers_router,
    create_recipients_router, create_schemas_router, create_shares_router,
    create_staging_tables_router, create_tables_router, create_tag_policies_router,
    create_temporary_credentials_router,
};
use crate::services::ServerHandler;

/// Build the REST router with selected surfaces proxied to an upstream instance.
///
/// Each resource router is given either the local `handler` or a per-surface
/// upstream adapter, decided by `routing`. Any surface not marked `upstream` is
/// always served locally. The returned router still needs the authentication
/// layer applied by the caller.
pub(crate) fn build_hybrid_router(
    handler: ServerHandler<RequestContext>,
    policy: Arc<dyn Policy<RequestContext>>,
    client: UnityCatalogClient,
    routing: &RoutingConfig,
) -> Router {
    let catalogs = match routing.catalogs {
        RoutingMode::Local => create_catalogs_router(handler.clone()),
        RoutingMode::Upstream => create_catalogs_router(UpstreamCatalogHandler::new(
            policy.clone(),
            client.catalogs_client(),
        )),
    };
    let schemas = match routing.schemas {
        RoutingMode::Local => create_schemas_router(handler.clone()),
        RoutingMode::Upstream => create_schemas_router(UpstreamSchemaHandler::new(
            policy.clone(),
            client.schemas_client(),
        )),
    };
    let tables = match routing.tables {
        RoutingMode::Local => create_tables_router(handler.clone()),
        RoutingMode::Upstream => create_tables_router(UpstreamTableHandler::new(
            policy.clone(),
            client.tables_client(),
        )),
    };

    // Remaining surfaces are local-only in v1 (validated upstream of here).
    let api_routes = catalogs
        .merge(schemas)
        .merge(tables)
        .merge(create_staging_tables_router(handler.clone()))
        .merge(create_credentials_router(handler.clone()))
        .merge(create_external_locations_router(handler.clone()))
        .merge(create_temporary_credentials_router(handler.clone()))
        .merge(create_functions_router(handler.clone()))
        .merge(create_recipients_router(handler.clone()))
        .merge(create_providers_router(handler.clone()))
        .merge(create_shares_router(handler.clone()))
        .merge(create_delta_router(handler.clone()))
        .merge(create_entity_tag_assignments_router(handler.clone()))
        .merge(create_policies_router(handler.clone()));

    Router::new()
        .nest("/api/2.1/unity-catalog", api_routes)
        // Tag Policies (governed tag definitions) are local-only and live under /api/2.1.
        .nest("/api/2.1", create_tag_policies_router(handler))
}
