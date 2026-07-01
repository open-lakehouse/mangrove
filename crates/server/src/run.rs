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
use axum::routing::get;
use swagger_ui_dist::{ApiDefinition, OpenApiSource};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::LatencyUnit;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::services::encryption::EnvelopeEncryptor;
use unitycatalog_common::store::ObjectStoreAdapter;
use unitycatalog_common::{Error, Result};
use unitycatalog_postgres::GraphStore;
use unitycatalog_sqlite::SqliteStore;

use crate::api::RequestContext;
use crate::api::agent_skills::AgentSkillHandler;
use crate::api::agents::AgentHandler;
use crate::api::catalogs::CatalogHandler;
use crate::api::commits::DeltaCommitHandler;
use crate::api::credentials::CredentialHandler;
use crate::api::delta::DeltaApiHandler;
use crate::api::entity_tag_assignments::EntityTagAssignmentHandler;
use crate::api::external_locations::ExternalLocationHandler;
use crate::api::functions::FunctionHandler;
use crate::api::providers::ProviderHandler;
use crate::api::recipients::RecipientHandler;
use crate::api::schemas::SchemaHandler;
use crate::api::shares::ShareHandler;
use crate::api::sharing::{SharingHandler, SharingQueryHandler};
use crate::api::staging_tables::StagingTableHandler;
use crate::api::tables::TableHandler;
use crate::api::tag_policies::TagPolicyHandler;
use crate::api::temporary_credentials::TemporaryCredentialHandler;
use crate::api::volumes::VolumeHandler;
use crate::config::{Backend, Config, PostgresBackendConfig, SqliteBackendConfig, UiConfig};
use crate::policy::{ConstantPolicy, Policy};
use crate::rest::{
    AnonymousAuthenticator, AuthenticationLayer, create_agent_skills_router, create_agents_router,
    create_catalogs_router, create_commits_router, create_credentials_router, create_delta_router,
    create_entity_tag_assignments_router, create_external_locations_router,
    create_functions_router, create_open_sharing_router, create_providers_router,
    create_recipients_router, create_schemas_router, create_shares_router, create_sharing_router,
    create_staging_tables_router, create_tables_router, create_tag_policies_router,
    create_temporary_credentials_router, create_volumes_router,
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

    let encryptor = config
        .encryption
        .as_ref()
        .ok_or_else(|| {
            Error::Generic(
                "missing `encryption` configuration: an active KEK is required to store secrets"
                    .into(),
            )
        })?
        .build_encryptor()
        .map_err(Error::Generic)?;

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
            let encryptor = migration_encryptor(&config)?;
            let store = GraphStore::connect(&db_url, encryptor)
                .await
                .map_err(|e| Error::Generic(format!("connecting to database: {e}")))?;
            store
                .migrate()
                .await
                .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
        }
        Backend::Sqlite(cfg) => {
            let path = cfg
                .database_path()
                .ok_or_else(|| Error::Generic("incomplete sqlite backend configuration".into()))?;
            let encryptor = migration_encryptor(&config)?;
            let store = SqliteStore::connect(&path, encryptor)
                .await
                .map_err(|e| Error::Generic(format!("opening sqlite database: {e}")))?;
            store
                .migrate()
                .await
                .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
        }
    }
    tracing::info!("migrations applied");
    Ok(())
}

/// Resolve an encryptor for the *migration* path only.
///
/// Schema migration never reads or writes secrets, so the KEK is irrelevant to
/// it — the store type merely requires *an* encryptor to construct. When no
/// `encryption` config is present (e.g. a minimal migrate-only config file) fall
/// back to the dev KEK rather than failing; a real KEK is enforced by `serve`.
fn migration_encryptor(config: &Config) -> Result<EnvelopeEncryptor> {
    match config.encryption.as_ref() {
        Some(enc) => enc.build_encryptor().map_err(Error::Generic),
        None => crate::config::EncryptionConfig::dev_default()
            .build_encryptor()
            .map_err(Error::Generic),
    }
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
        + DeltaCommitHandler<Cx>
        + DeltaApiHandler<Cx>
        + TagPolicyHandler<Cx>
        + EntityTagAssignmentHandler<Cx>
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
        .merge(create_recipients_router(handler.clone()))
        .merge(create_providers_router(handler.clone()))
        .merge(create_shares_router(handler.clone()))
        .merge(create_commits_router(handler.clone()))
        .merge(create_delta_router(handler.clone()))
        .merge(create_entity_tag_assignments_router(handler.clone()));

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
    let router = swagger_api_defs()
        .into_iter()
        .fold(api_router, |router, api| {
            router.merge(swagger_ui_dist::generate_routes(api))
        });

    // Operational endpoints first so they can never be shadowed by the SPA fallback.
    let mut router = operational_router().merge(router);

    // Optionally mount the bundled SPA as a fallback: real files come off disk,
    // any other path falls back to index.html for the SPA's client-side router.
    // Absent a bundle on disk, these routes simply 404.
    if ui.serve {
        let index = Path::new(UI_DIR).join("index.html");
        let serve_dir = ServeDir::new(UI_DIR).not_found_service(ServeFile::new(index.clone()));
        router = router.fallback_service(serve_dir);
        if !index.exists() {
            tracing::warn!(
                "ui.serve is enabled but no bundle found at `{}/index.html`; SPA routes will 404",
                UI_DIR
            );
        }
    }

    // Mount the whole surface under the base path when configured (behind a
    // gateway sub-path). Empty base_path ⇒ served at root.
    let base_path = ui.normalized_base_path();
    let router = if base_path.is_empty() {
        router
    } else {
        Router::new().nest(&base_path, router)
    };

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

    let store = Arc::new(
        GraphStore::connect(&db_url, encryptor)
            .await
            .map_err(|e| Error::Generic(format!("connecting to database: {e}")))?,
    );
    if migrate {
        store
            .migrate()
            .await
            .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
    }
    let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
    // The Postgres store also implements `CommitCoordinator`, so Delta
    // catalog-managed commits are persisted in the database rather than memory.
    let handler = ServerHandler::try_new_tokio_with_coordinator(
        policy.clone(),
        store.clone(),
        store.clone(),
        store,
    )
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

    let store = Arc::new(
        SqliteStore::connect(&path, encryptor)
            .await
            .map_err(|e| Error::Generic(format!("opening sqlite database: {e}")))?,
    );
    if migrate {
        store
            .migrate()
            .await
            .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
    }
    let policy: Arc<dyn Policy<RequestContext>> = Arc::new(ConstantPolicy::default());
    // `SqliteStore` implements the generic object/association stores (lifted to
    // `ResourceStore` by `ObjectStoreAdapter`), `SecretManager`, and
    // `CommitCoordinator`, but the adapter does not forward the latter two — so
    // those roles are wired from the same shared store separately. Like the
    // Postgres backend, Delta catalog-managed commits are persisted in the
    // database rather than in memory.
    let resource_store = Arc::new(ObjectStoreAdapter::new(store.clone()));
    let handler = ServerHandler::try_new_tokio_with_coordinator(
        policy.clone(),
        resource_store,
        store.clone(),
        store,
    )
    .map_err(|e| Error::Generic(e.to_string()))?;
    Ok((handler, policy))
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
