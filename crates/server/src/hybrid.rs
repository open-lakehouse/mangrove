//! Hybrid proxy router wiring.
//!
//! Composes a local Unity Catalog server with an upstream instance: each REST
//! surface is wired at startup to either the local [`ServerHandler`] or the
//! upstream's service client wrapped in [`Authorized`], decided by
//! [`RoutingConfig`]. The decorator enforces this server's policy locally before
//! forwarding; see that type for details. Any surface not marked `upstream` is
//! always served locally.

use std::sync::Arc;

use axum::Router;
use unitycatalog_client::UnityCatalogClient;

use crate::api::RequestContext;
use crate::config::{RoutingConfig, RoutingMode};
use crate::handlers::Authorized;
use crate::policy::Policy;
use crate::rest::{
    create_catalogs_router, create_credentials_router, create_delta_router,
    create_entity_tag_assignments_router, create_external_locations_router,
    create_functions_router, create_policies_router, create_schemas_router,
    create_staging_tables_router, create_tables_router, create_tag_policies_router,
    create_temporary_credentials_router,
};
use crate::services::ServerHandler;

/// Build the REST router with selected surfaces proxied to an upstream instance.
///
/// Each resource router is given either the local `handler` or the upstream's
/// policy-checked service client, decided by `routing`. Any surface not marked
/// `upstream` is always served locally. The returned router still needs the
/// authentication layer applied by the caller.
pub(crate) fn build_hybrid_router(
    handler: ServerHandler<RequestContext>,
    policy: Arc<dyn Policy<RequestContext>>,
    client: UnityCatalogClient,
    routing: &RoutingConfig,
) -> Router {
    // Mount `$create` over the local handler, or over the upstream service client
    // `$client` behind the local policy.
    macro_rules! route {
        ($mode:expr, $create:ident, $client:ident) => {
            match $mode {
                RoutingMode::Local => $create(handler.clone()),
                RoutingMode::Upstream => $create(Authorized::new(client.$client(), policy.clone())),
            }
        };
    }

    let api_routes = route!(routing.catalogs, create_catalogs_router, catalogs_client)
        .merge(route!(
            routing.schemas,
            create_schemas_router,
            schemas_client
        ))
        .merge(route!(routing.tables, create_tables_router, tables_client))
        .merge(route!(
            routing.credentials,
            create_credentials_router,
            credentials_client
        ))
        .merge(route!(
            routing.external_locations,
            create_external_locations_router,
            external_locations_client
        ))
        .merge(route!(
            routing.functions,
            create_functions_router,
            functions_client
        ))
        // The remaining surfaces are always local.
        .merge(create_staging_tables_router(handler.clone()))
        .merge(create_temporary_credentials_router(handler.clone()))
        .merge(create_delta_router(handler.clone()))
        .merge(create_entity_tag_assignments_router(handler.clone()))
        .merge(create_policies_router(handler.clone()));

    Router::new()
        .nest("/api/2.1/unity-catalog", api_routes)
        // Tag Policies (governed tag definitions) are local-only and live under /api/2.1.
        .nest("/api/2.1", create_tag_policies_router(handler))
}
