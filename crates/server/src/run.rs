//! Server-launch wiring for the `uc-server` binary.
//!
//! Two entry points, one per schema-affecting concern:
//!   - [`serve`] resolves the backend from [`Config`], builds the REST router
//!     (optionally with upstream-proxied surfaces — see [`crate::hybrid`]), mounts
//!     the bundled web UI when enabled, and runs until shutdown. It does **not**
//!     apply migrations.
//!   - [`migrate`] connects the configured backend and applies any pending
//!     database migrations, then returns. This is the only schema-mutating path,
//!     kept off the `serve` hot path so concurrent replicas don't race to migrate.
//!
//! The REST router itself (and the `/health` + `/version` operational endpoints)
//! is built by [`build_rest_router`], shared by the local and hybrid paths.

use std::path::Path;
use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use swagger_ui_dist::{ApiDefinition, OpenApiSource};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::LatencyUnit;
use tower_http::services::ServeDir;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::services::encryption::EnvelopeEncryptor;
use unitycatalog_common::store::ObjectStoreAdapter;
use unitycatalog_common::{Error, Result};
use unitycatalog_delta_api::DeltaApiHandler;
use unitycatalog_postgres::PgCommitCoordinator;
use unitycatalog_sqlite::SqliteCommitCoordinator;

use crate::api::RequestContext;
use crate::api::agent_skills::AgentSkillHandler;
use crate::api::agents::AgentHandler;
use crate::api::catalogs::CatalogHandler;
use crate::api::credentials::CredentialHandler;
use crate::api::entity_tag_assignments::EntityTagAssignmentHandler;
use crate::api::external_locations::ExternalLocationHandler;
use crate::api::functions::FunctionHandler;
use crate::api::model_versions::ModelVersionHandler;
use crate::api::policies::PolicyHandler;
use crate::api::providers::ProviderHandler;
use crate::api::recipients::RecipientHandler;
use crate::api::registered_models::RegisteredModelHandler;
use crate::api::schemas::SchemaHandler;
use crate::api::shares::ShareHandler;
use crate::api::sharing::{SharingHandler, SharingQueryHandler};
use crate::api::staging_tables::StagingTableHandler;
use crate::api::tables::TableHandler;
use crate::api::tag_policies::TagPolicyHandler;
use crate::api::temporary_credentials::TemporaryCredentialHandler;
use crate::api::volumes::VolumeHandler;
use crate::config::PostgresBackendConfig;
use crate::config::{Backend, Config, SqliteBackendConfig, UiConfig};
use crate::policy::{ConstantPolicy, Policy};
use crate::rest::{
    AnonymousAuthenticator, AuthenticationLayer, create_agent_skills_router, create_agents_router,
    create_catalogs_router, create_credentials_router, create_delta_router,
    create_entity_tag_assignments_router, create_external_locations_router,
    create_functions_router, create_model_versions_router, create_open_sharing_router,
    create_policies_router, create_providers_router, create_recipients_router,
    create_registered_models_router, create_schemas_router, create_shares_router,
    create_sharing_router, create_staging_tables_router, create_tables_router,
    create_tag_policies_router, create_temporary_credentials_router, create_volumes_router,
};
use crate::services::{LocalStoragePolicy, ServerHandler, location::StorageLocationUrl};
use crate::sharing::{SharingSkillHandler, SharingVolumeHandler};

/// Directory the bundled single-page app is served from, relative to the process
/// working directory. The Docker image places the built bundle here (see the
/// server `Dockerfile`); when it's absent — e.g. local API-only runs where the UI
/// is served by the Vite dev server instead — the SPA routes simply 404.
const UI_DIR: &str = "web";

/// A local server handler paired with the policy it was built with.
///
/// The policy is surfaced separately so the hybrid proxy can apply the *same*
/// authorization to surfaces it forwards upstream.
pub(crate) type LocalHandler = (
    ServerHandler<RequestContext>,
    Arc<dyn Policy<RequestContext>>,
);

/// Resolve the backend from `config`, build the router, mount the bundled UI when
/// enabled, and run the REST server until shutdown. Does not apply migrations.
pub async fn serve(config: Config) -> Result<()> {
    let host = config.resolved_host().to_string();
    let port = config.resolved_port();

    // A configured KEK is required for durable backends (secrets must survive a
    // restart). For an ephemeral `:memory:` backend, secrets never persist, so
    // fall back to the dev KEK when none is configured — this keeps a UI-only or
    // otherwise minimal config file usable against the dev backend, matching the
    // config-less default (which also ships the dev KEK). See
    // [`crate::config::EncryptionConfig::dev_default`].
    let encryptor = match config.encryption.as_ref() {
        Some(enc) => enc.build_encryptor().map_err(Error::Generic)?,
        None if config.backend.is_ephemeral() => crate::config::EncryptionConfig::dev_default()
            .build_encryptor()
            .map_err(Error::Generic)?,
        None => {
            return Err(Error::Generic(
                "missing `encryption` configuration: an active KEK is required to store secrets \
                 with a durable backend"
                    .into(),
            ));
        }
    };

    // Build the local-storage allowlist from config. Empty ⇒ deny all file://.
    // A configured root that does not exist is a hard startup error.
    let local_storage_policy = LocalStoragePolicy::new(&config.local_storage.allowed_roots)
        .map_err(|e| Error::Generic(format!("invalid local_storage config: {e}")))?;

    // A configured metastore managed storage root must parse and, if it is a
    // local (file://) path, sit within an allowed local root — same governance as
    // catalog/schema roots. Validate at startup so a misconfigured root is a hard
    // error rather than surfacing later at catalog-create time.
    if let Some(root) = config
        .managed_storage_root
        .as_deref()
        .filter(|s| !s.is_empty())
    {
        let url = StorageLocationUrl::parse(root)
            .map_err(|e| Error::Generic(format!("invalid managed_storage_root '{root}': {e}")))?;
        local_storage_policy
            .check(&url)
            .map_err(|e| Error::Generic(format!("invalid managed_storage_root '{root}': {e}")))?;
    }

    // `serve` does not migrate durable backends — the operator runs `migrate`
    // first. The exception is an ephemeral `:memory:` SQLite database: it is
    // created fresh in this process, so a prior `migrate` step could not have
    // reached it; auto-migrate it here so a config-less `serve` works out of the
    // box. See [`Backend::is_ephemeral`].
    let migrate_on_connect = config.backend.is_ephemeral();
    if migrate_on_connect {
        tracing::info!("ephemeral in-memory backend: applying migrations at startup");
    }
    let (handler, policy) = match &config.backend {
        Backend::Postgres(pg) => connect_postgres(pg, encryptor, migrate_on_connect).await?,
        Backend::Sqlite(cfg) => connect_sqlite(cfg, encryptor, migrate_on_connect).await?,
    };
    let handler = handler
        .with_local_storage_policy(local_storage_policy)
        .with_managed_storage_root(config.managed_storage_root.clone());

    // Build the API surface — locally, or with selected surfaces proxied upstream.
    let api_router = if config.routing.any_upstream() {
        let unsupported = config.routing.unsupported_upstream();
        if !unsupported.is_empty() {
            return Err(Error::Generic(format!(
                "upstream routing is not yet implemented for: {}",
                unsupported.join(", ")
            )));
        }
        let upstream = config.upstream.as_ref().ok_or_else(|| {
            Error::Generic(
                "routing marks surfaces as upstream but no `upstream` config is set".to_string(),
            )
        })?;
        let upstream_url = upstream
            .url
            .parse()
            .map_err(|e| Error::Generic(format!("invalid upstream url: {e}")))?;
        // Upstream needs no auth; authorization is enforced locally via `policy`.
        let client = UnityCatalogClient::new_unauthenticated(upstream_url);
        crate::hybrid::build_hybrid_router(handler, policy, client, &config.routing)
    } else {
        build_rest_router(handler)
    };

    let app = api_router.layer(AuthenticationLayer::new(AnonymousAuthenticator));

    run(app, &config.ui, &host, port).await
}

/// Connect the configured backend and apply any pending migrations, then return.
/// The only schema-mutating path (see the module docs).
pub async fn migrate(config: Config) -> Result<()> {
    match &config.backend {
        Backend::Postgres(pg) => {
            let db_url = pg.connection_string().ok_or_else(|| {
                Error::Generic("incomplete postgres backend configuration".into())
            })?;
            let pool = unitycatalog_postgres::connect_pool(&db_url)
                .await
                .map_err(|e| Error::Generic(format!("connecting to database: {e}")))?;
            unitycatalog_postgres::unified_migrator()
                .run(&pool)
                .await
                .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
        }
        Backend::Sqlite(cfg) => {
            let path = cfg
                .database_path()
                .ok_or_else(|| Error::Generic("incomplete sqlite backend configuration".into()))?;
            let pool = unitycatalog_sqlite::connect_pool(&path)
                .await
                .map_err(|e| Error::Generic(format!("opening sqlite database: {e}")))?;
            unitycatalog_sqlite::unified_migrator()
                .run(&pool)
                .await
                .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
        }
    }
    tracing::info!("migrations applied");
    Ok(())
}

/// The OpenAPI definitions mounted as Swagger UI, shared by the local and hybrid
/// paths. Kept as a free function so both router builders stay in sync.
pub(crate) fn swagger_api_defs() -> Vec<ApiDefinition<&'static str>> {
    vec![
        ApiDefinition {
            uri_prefix: "/api/2.1/unity-catalog",
            api_definition: OpenApiSource::Inline(include_str!("../../../openapi/openapi.yaml")),
            title: Some("Unity Catalog API"),
        },
        ApiDefinition {
            uri_prefix: "/api/v1/delta-sharing",
            api_definition: OpenApiSource::Inline(include_str!("../../../openapi/sharing.yaml")),
            title: Some("Delta Sharing API"),
        },
        // Open Sharing is a superset of Delta Sharing served at its own prefix;
        // the tabular surface is wire-compatible, so it reuses the same spec.
        ApiDefinition {
            uri_prefix: "/api/v1/open-sharing",
            api_definition: OpenApiSource::Inline(include_str!("../../../openapi/sharing.yaml")),
            title: Some("Open Sharing API"),
        },
        // The Delta REST API routes live at `/delta/v1/...` under the UC base
        // path, but its Swagger UI + spec are hosted under a distinct prefix so
        // the swagger-ui asset routes don't collide with the main UC API's.
        ApiDefinition {
            uri_prefix: "/api/2.1/unity-catalog/delta",
            api_definition: OpenApiSource::Inline(include_str!("../../../openapi/delta.yaml")),
            title: Some("UC Delta API"),
        },
    ]
}

/// Build the all-local REST router (every surface served from `handler`).
pub(crate) fn build_rest_router<T, Cx>(handler: T) -> Router
where
    T: CatalogHandler<Cx>
        + CredentialHandler<Cx>
        + FunctionHandler<Cx>
        + SharingHandler<Cx>
        + SharingQueryHandler<Cx>
        + SharingVolumeHandler<Cx>
        + SharingSkillHandler<Cx>
        + ShareHandler<Cx>
        + SchemaHandler<Cx>
        + StagingTableHandler<Cx>
        + TableHandler<Cx>
        + VolumeHandler<Cx>
        + AgentSkillHandler<Cx>
        + AgentHandler<Cx>
        + ExternalLocationHandler<Cx>
        + RecipientHandler<Cx>
        + ProviderHandler<Cx>
        + DeltaApiHandler<Cx>
        + TagPolicyHandler<Cx>
        + EntityTagAssignmentHandler<Cx>
        + PolicyHandler<Cx>
        + RegisteredModelHandler<Cx>
        + ModelVersionHandler<Cx>
        + TemporaryCredentialHandler<Cx>
        + Clone,
    Cx: axum::extract::FromRequestParts<T> + Send + 'static,
{
    let api_routes = create_catalogs_router(handler.clone())
        .merge(create_schemas_router(handler.clone()))
        .merge(create_staging_tables_router(handler.clone()))
        .merge(create_tables_router(handler.clone()))
        .merge(create_volumes_router(handler.clone()))
        .merge(create_agent_skills_router(handler.clone()))
        .merge(create_agents_router(handler.clone()))
        .merge(create_credentials_router(handler.clone()))
        .merge(create_external_locations_router(handler.clone()))
        .merge(create_temporary_credentials_router(handler.clone()))
        .merge(create_functions_router(handler.clone()))
        .merge(create_registered_models_router(handler.clone()))
        .merge(create_model_versions_router(handler.clone()))
        .merge(create_recipients_router(handler.clone()))
        .merge(create_providers_router(handler.clone()))
        .merge(create_shares_router(handler.clone()))
        .merge(create_delta_router(handler.clone()))
        .merge(create_entity_tag_assignments_router(handler.clone()))
        .merge(create_policies_router(handler.clone()));

    Router::new()
        .nest("/api/2.1/unity-catalog", api_routes)
        // Tag Policies (governed tag definitions) live under /api/2.1, not /unity-catalog.
        .nest("/api/2.1", create_tag_policies_router(handler.clone()))
        .nest(
            "/api/v1/delta-sharing",
            create_sharing_router(handler.clone()),
        )
        // Open Sharing: superset surface sharing the same tabular handlers.
        .nest("/api/v1/open-sharing", create_open_sharing_router(handler))
}

/// Operational endpoints served regardless of backend or routing.
///
/// `/health` returns the literal `OK` body the `healthcheck` subcommand (and the
/// Docker `HEALTHCHECK`) expects; `/version` returns the crate version so a
/// running service can be matched to a release (binary + bundled UI).
fn operational_router() -> Router {
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
}

/// Assemble the final service — API + operational + Swagger UI + (optional) SPA —
/// mount it under the configured base path, add the trace layer, bind, and serve
/// until shutdown.
pub(crate) async fn run(api_router: Router, ui: &UiConfig, host: &str, port: u16) -> Result<()> {
    // Swagger UI asset routes are merged onto the API router.
    let mut router = swagger_api_defs()
        .into_iter()
        .fold(api_router, |router, api| {
            router.merge(swagger_ui_dist::generate_routes(api))
        });

    // Optionally mount the bundled SPA: real (hashed) asset files come off disk,
    // and any non-file path falls back to `index.html` with a 200 so the SPA's
    // client-side router takes over (deep links like `/catalog` must load the app,
    // not 404). Absent a bundle on disk, the fallback 404s and the server stays up.
    if ui.serve {
        router = mount_spa(router);
    }

    // Mount the app (API + Swagger + SPA) under the base path when configured
    // (behind a gateway sub-path). Empty base_path ⇒ served at root.
    let router = mount_under_base(router, &ui.normalized_base_path());

    // Operational endpoints are mounted at the ROOT, *outside* the base-path
    // wrapper, so `/health` and `/version` are always reachable at the address the
    // server binds — regardless of `ui.base_path`. This is what the `healthcheck`
    // subcommand and the Docker `HEALTHCHECK` probe (both target root `/health`
    // via `Config::health_url`), and it keeps liveness independent of the gateway
    // routing. Merged last so nothing (including the SPA fallback) can shadow them.
    let router = operational_router().merge(router);

    let router = router.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros),
            ),
    );

    let listener = TcpListener::bind(format!("{host}:{port}"))
        .await
        .map_err(|e| Error::Generic(e.to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|e| Error::Generic(e.to_string()))?;
    tracing::info!("Listening on: {addr}");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| Error::Generic(e.to_string()))?;

    Ok(())
}

async fn connect_postgres(
    pg: &PostgresBackendConfig,
    encryptor: EnvelopeEncryptor,
    migrate: bool,
) -> Result<LocalHandler> {
    let db_url = pg
        .connection_string()
        .ok_or_else(|| Error::Generic("incomplete postgres backend configuration".into()))?;

    let pool = unitycatalog_postgres::connect_pool(&db_url)
        .await
        .map_err(|e| Error::Generic(format!("connecting to database: {e}")))?;
    if migrate {
        unitycatalog_postgres::unified_migrator()
            .run(&pool)
            .await
            .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
    }
    let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
    // The object/association graph store (lifted to `ResourceStore` by
    // `ObjectStoreAdapter`) and the Delta `CommitCoordinator` are two independent
    // handles on the same pool. Sensitive fields (credentials, tokens) are sealed
    // inline on the stored resources rather than in a separate secret store; Delta
    // catalog-managed commits are persisted in the `delta_commits` table.
    let graph = unitycatalog_postgres::connect_graph(pool.clone(), encryptor);
    let resource_store = Arc::new(ObjectStoreAdapter::new(graph));
    let coordinator = Arc::new(PgCommitCoordinator::new(pool));
    let handler =
        ServerHandler::try_new_tokio_with_coordinator(policy.clone(), resource_store, coordinator)
            .map_err(|e| Error::Generic(e.to_string()))?;
    Ok((handler, policy))
}

async fn connect_sqlite(
    cfg: &SqliteBackendConfig,
    encryptor: EnvelopeEncryptor,
    migrate: bool,
) -> Result<LocalHandler> {
    let path = cfg
        .database_path()
        .ok_or_else(|| Error::Generic("incomplete sqlite backend configuration".into()))?;

    let pool = unitycatalog_sqlite::connect_pool(&path)
        .await
        .map_err(|e| Error::Generic(format!("opening sqlite database: {e}")))?;
    if migrate {
        unitycatalog_sqlite::unified_migrator()
            .run(&pool)
            .await
            .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
    }
    let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
    // The object/association graph store (lifted to `ResourceStore` by
    // `ObjectStoreAdapter`) and the Delta `CommitCoordinator` are two independent
    // handles on the same pool. Sensitive fields (credentials, tokens) are sealed
    // inline on the stored resources rather than in a separate secret store; like
    // the Postgres backend, Delta catalog-managed commits are persisted in the
    // `delta_commits` table rather than in memory.
    let graph = unitycatalog_sqlite::connect_graph(pool.clone(), encryptor);
    let resource_store = Arc::new(ObjectStoreAdapter::new(graph));
    let coordinator = Arc::new(SqliteCommitCoordinator::new(pool));
    let handler =
        ServerHandler::try_new_tokio_with_coordinator(policy.clone(), resource_store, coordinator)
            .map_err(|e| Error::Generic(e.to_string()))?;
    Ok((handler, policy))
}

/// Mount the bundled SPA onto `app`: real asset files off disk, with an
/// `index.html` fallback (200) so client-side deep links load the app.
///
/// `index.html` is read once at startup into an `Arc<Option<String>>`: present ⇒
/// the fallback serves it with a 200; absent (no bundle on disk) ⇒ the fallback
/// 404s and the server still runs. `ServeDir` serves only the real hashed assets;
/// `append_index_html_on_directories(false)` keeps it from serving the raw
/// `index.html` off disk (all index serving goes through the 200 handler).
fn mount_spa(app: Router) -> Router {
    let index_path = Path::new(UI_DIR).join("index.html");
    let index_html: Arc<Option<String>> = Arc::new(std::fs::read_to_string(&index_path).ok());
    if index_html.is_none() {
        tracing::warn!(
            "ui.serve is enabled but no bundle found at `{}`; SPA routes will 404",
            index_path.display()
        );
    }

    let index_handler = move || {
        let index_html = index_html.clone();
        get(move || {
            let index_html = index_html.clone();
            async move {
                match index_html.as_ref() {
                    Some(html) => axum::response::Html(html.clone()).into_response(),
                    None => StatusCode::NOT_FOUND.into_response(),
                }
            }
        })
    };

    let serve_assets = ServeDir::new(UI_DIR)
        .append_index_html_on_directories(false)
        .fallback(index_handler());

    app
        // The SPA entry at the root + its explicit filename (ServeDir would serve
        // these straight off disk; route them through the 200 handler instead).
        .route("/", index_handler())
        .route("/index.html", index_handler())
        // Everything else: real asset files off disk, else the SPA entry (200).
        .fallback_service(serve_assets)
}

/// Mount `app` under `base_path`, or return it unchanged when the prefix is empty
/// (serve at root).
///
/// Rather than `Router::nest` — whose trailing-slash handling at the mount root
/// is fiddly (`/{prefix}` vs `/{prefix}/` resolve inconsistently against the
/// inner `/` route, and a nested `fallback_service` like `ServeDir` doesn't fire
/// for the bare prefix) — this strips the prefix from the request path *before*
/// the inner router routes, then delegates to the unchanged inner `app`. The
/// strip runs as a layer wrapping the whole router **as a service** (via
/// `ServiceBuilder`), not `Router::layer` — the latter runs only after a route is
/// matched, too late to influence routing. The inner router then sees exactly the
/// path it expects, and both `/{prefix}` and `/{prefix}/` map cleanly to `/`.
/// Requests outside the prefix get a 404.
fn mount_under_base(app: Router, base_path: &str) -> Router {
    if base_path.is_empty() {
        return app;
    }
    let prefix = base_path.to_string();
    let stripped = tower::ServiceBuilder::new()
        .layer(axum::middleware::from_fn(
            move |mut req: axum::extract::Request, next: axum::middleware::Next| {
                let prefix = prefix.clone();
                async move {
                    let path = req.uri().path();
                    // Strip the prefix; `/{prefix}` and `/{prefix}/` -> `/`.
                    let new_path = match path.strip_prefix(&prefix) {
                        Some("") => Some("/".to_string()),
                        Some(rest) if rest.starts_with('/') => Some(rest.to_string()),
                        // A path that merely *starts* with the prefix as a
                        // substring (e.g. `/{prefix}foo`) is not under it -> 404.
                        _ => None,
                    };
                    match new_path {
                        Some(new_path) => {
                            rewrite_path(&mut req, &new_path);
                            next.run(req).await
                        }
                        None => StatusCode::NOT_FOUND.into_response(),
                    }
                }
            },
        ))
        .service(app);
    Router::new().fallback_service(stripped)
}

/// Replace a request's path (preserving the query) with `new_path`.
fn rewrite_path(req: &mut axum::extract::Request, new_path: &str) {
    let uri = req.uri();
    let path_and_query = match uri.query() {
        Some(q) => format!("{new_path}?{q}"),
        None => new_path.to_string(),
    };
    let mut parts = uri.clone().into_parts();
    parts.path_and_query = Some(
        path_and_query
            .parse()
            .expect("rewritten path-and-query is valid"),
    );
    if let Ok(new_uri) = axum::http::Uri::from_parts(parts) {
        *req.uri_mut() = new_uri;
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
