//! Server-launch wiring, shared by the `uc-server` binary and socket-less hosts.
//!
//! Two entry points, one per schema-affecting concern:
//!   - [`serve`] builds the service from [`Config`] and runs it until shutdown.
//!     It does **not** apply migrations to a durable backend, and refuses to
//!     start unless the backend's schema is current.
//!   - [`migrate`] connects the configured backend and applies any pending
//!     database migrations and creates the declared property indexes, then
//!     returns. This is the only schema-mutating path,
//!     kept off the `serve` hot path so concurrent replicas don't race to migrate.
//!
//! The service itself is assembled by [`build_app`] without binding a socket;
//! [`crate::embed::start`] hands that router to hosts that drive it themselves.
//! The REST router (and the `/health` + `/version` operational endpoints) is
//! built by [`build_rest_router`], shared by the local and hybrid paths.
//!
//! Backends and optional surfaces are feature-gated (`postgres`, `sqlite`,
//! `proxy`, `storage-proxy`, `files`, `swagger`, `ui`). A config that asks for
//! one this build doesn't include fails at startup.

use std::sync::Arc;

use axum::Router;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use unitycatalog_common::{Error, Result};

use crate::api::RequestContext;
use crate::config::{AuthMode, Backend, Config, StorageProxyConfig, UiConfig};
use crate::policy::Policy;
use crate::rest::{
    AnonymousAuthenticator, AuthenticationLayer, BearerAuthenticator, OnMissingIdentity,
    ReverseProxyAuthenticator, create_agent_skills_router, create_agents_router,
    create_catalogs_router, create_credentials_router, create_delta_router,
    create_entity_tag_assignments_router, create_external_locations_router,
    create_functions_router, create_grants_router, create_model_versions_router,
    create_policies_router, create_registered_models_router, create_schemas_router,
    create_staging_tables_router, create_tables_router, create_tag_policies_router,
    create_temporary_credentials_router, create_volumes_router,
};
use crate::services::{LocalStoragePolicy, ServerHandler, location::StorageLocationUrl};

/// Directory the bundled single-page app is served from, relative to the process
/// working directory. The Docker image places the built bundle here (see
/// `crates/server/Dockerfile`); when it's absent — e.g. local API-only runs where the UI
/// is served by the Vite dev server instead — the SPA routes simply 404.
#[cfg(feature = "ui")]
const UI_DIR: &str = "web";

/// A local server handler paired with the policy it was built with.
///
/// The policy is surfaced separately so the hybrid proxy can apply the *same*
/// authorization to surfaces it forwards upstream.
pub(crate) type LocalHandler = (
    ServerHandler<RequestContext>,
    Arc<dyn Policy<RequestContext>>,
);

/// Build the service from `config` (see [`build_app`]), bind the configured
/// address, and serve until shutdown. Does not apply migrations.
pub async fn serve(config: Config) -> Result<()> {
    let app = build_app(&config).await?;

    let addr = format!("{}:{}", config.resolved_host(), config.resolved_port());
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|e| Error::Generic(e.to_string()))?;
    let addr = listener
        .local_addr()
        .map_err(|e| Error::Generic(e.to_string()))?;
    tracing::info!("Listening on: {addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| Error::Generic(e.to_string()))?;

    Ok(())
}

/// Assemble the whole service from `config` without binding a socket.
///
/// Connects the backend (checking that a durable one's schema is current),
/// builds the API router (locally, or with selected surfaces proxied upstream)
/// with the optional surfaces and the configured authenticator, then mounts
/// Swagger UI, the SPA, the base path, the operational endpoints and the trace
/// layer. Shared by [`serve`] and [`crate::embed::start`].
pub(crate) async fn build_app(config: &Config) -> Result<Router> {
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
    } else if config.managed_storage_root_credential.is_some() {
        return Err(Error::Generic(
            "managed_storage_root_credential is set but managed_storage_root is not".into(),
        ));
    }

    // Refuse to start on an incoherent config (client arm with no client block,
    // unity files backend with no client block) before connecting anything.
    if let Some(err) = config.storage_proxy.startup_error() {
        return Err(Error::Generic(format!("storage-proxy config: {err}")));
    }
    if let Some(err) = config.files.startup_error() {
        return Err(Error::Generic(format!("files config: {err}")));
    }
    if let Some(err) = config.vending.startup_error(&config.auth, &config.policy) {
        return Err(Error::Generic(format!("vending config: {err}")));
    }
    let neon_vending = config
        .vending
        .neon
        .resolve()
        .map_err(|e| Error::Generic(format!("vending config: {e}")))?;

    let (handler, policy) = connect(config).await?;
    let handler = with_grants(handler, config)?;
    let handler = handler
        .with_neon_vending(neon_vending)
        .with_local_storage_policy(local_storage_policy)
        .with_managed_storage_root(config.managed_storage_root.clone())
        .with_managed_storage_root_credential(config.managed_storage_root_credential.clone());

    // Keep a handle for the (optional) storage-proxy local arm, which is a
    // separate surface from the hybrid-routed UC API below.
    #[cfg(feature = "storage-proxy")]
    let proxy_handler = handler.clone();

    // Build the API surface — locally, or with selected surfaces proxied upstream.
    let api_router = if config.routing.any_upstream() {
        build_upstream_router(config, handler, policy)?
    } else {
        build_rest_router(handler)
    };

    // Optionally mount the same-origin storage byte-proxy at the server root
    // (`/storage-proxy/...`) — its own surface, not under `/api/2.1/unity-catalog`.
    #[cfg(feature = "storage-proxy")]
    let api_router = if config.storage_proxy.enabled {
        let proxy = build_storage_proxy_router(config, proxy_handler).await?;
        api_router.merge(proxy)
    } else {
        api_router
    };
    #[cfg(not(feature = "storage-proxy"))]
    if config.storage_proxy.enabled {
        return Err(not_compiled("`storage_proxy.enabled`", "storage-proxy"));
    }

    // Optionally mount the embedded Files API (portal.files.v1.FilesService).
    // It is a ConnectRPC service exposed as a self-contained axum fallback
    // service, so it is `nest_service`-d under its own base path — never `merge`-d
    // (two fallback-carrying routers cannot be merged, and it must not shadow the
    // REST routes or the SPA fallback). An empty configured base path falls back
    // to `/files` for exactly that reason.
    #[cfg(feature = "files")]
    let api_router = if config.files.enabled {
        let files = build_files_router(config).await?;
        let base_path = config.files.resolved_base_path();
        let mount = if base_path.is_empty() {
            "/files".to_string()
        } else {
            base_path
        };
        api_router.nest_service(&mount, files)
    } else {
        api_router
    };
    #[cfg(not(feature = "files"))]
    if config.files.enabled {
        return Err(not_compiled("`files.enabled`", "files"));
    }

    // Apply the configured authenticator. Each arm yields the same erased
    // `Router` type, so they can share one binding despite distinct authenticator
    // types.
    let app = match config.auth.mode {
        AuthMode::Anonymous => api_router.layer(AuthenticationLayer::new(AnonymousAuthenticator)),
        AuthMode::ReverseProxy => {
            let mut authenticator = ReverseProxyAuthenticator::new();
            if let Some(header) = &config.auth.forwarded_user_header {
                authenticator = authenticator.with_header(header.clone());
            }
            if config.auth.allow_missing_identity {
                authenticator = authenticator.with_on_missing(OnMissingIdentity::Anonymous);
            }
            api_router.layer(AuthenticationLayer::new(authenticator))
        }
        AuthMode::Bearer => {
            let tokens = config
                .auth
                .bearer_tokens()
                .map_err(|e| Error::Generic(format!("auth config: {e}")))?;
            api_router.layer(AuthenticationLayer::new(BearerAuthenticator::new(tokens)))
        }
        #[cfg(feature = "olai-auth")]
        AuthMode::OlaiAuth => {
            let olai = config
                .auth
                .olai_auth(&config.backend)
                .map_err(|e| Error::Generic(format!("auth config: {e}")))?;
            crate::rest::olai_auth::layer(api_router, &olai)
                .map_err(|e| Error::Generic(format!("auth config: {e}")))?
        }
        #[cfg(not(feature = "olai-auth"))]
        AuthMode::OlaiAuth => return Err(not_compiled("`auth.mode: olai-auth`", "olai-auth")),
    };

    Ok(assemble(app, &config.ui, &config.storage_proxy))
}

/// The policy `config.policy` asks for.
///
/// An authorizer policy needs verified callers (`auth.mode: olai-auth`): it
/// denies every request without one, so any other mode is a startup error.
fn build_policy(config: &Config) -> Result<Arc<dyn Policy<RequestContext>>> {
    use crate::config::PolicyConfig;

    match &config.policy {
        PolicyConfig::Constant => Ok(Arc::new(crate::policy::ConstantPolicy::default())),
        #[cfg(feature = "olai-auth")]
        PolicyConfig::Authorizer(authorizer) => {
            use std::time::Duration;

            if config.auth.mode != AuthMode::OlaiAuth {
                return Err(Error::Generic(
                    "policy config: `kind: authorizer` needs `auth.mode: olai-auth`, \
                     since it denies every request without a verified caller"
                        .into(),
                ));
            }
            let (url, token) = authorizer
                .endpoint()
                .map_err(|e| Error::Generic(format!("policy config: {e}")))?;
            let ttl = authorizer
                .cache_ttl_secs
                .unwrap_or(crate::config::DEFAULT_DECISION_CACHE_TTL_SECS);
            Ok(Arc::new(crate::policy::AuthorizerPolicy::remote(
                &url,
                &token,
                authorizer.timeout_ms.map(Duration::from_millis),
                Duration::from_secs(ttl),
            )))
        }
        #[cfg(not(feature = "olai-auth"))]
        PolicyConfig::Authorizer(_) => Err(not_compiled("`policy.kind: authorizer`", "olai-auth")),
    }
}

/// `handler` keeping its grants where the deployment's decision point reads them.
///
/// With `auth.mode: olai-auth`, grants live in the `authz` schema of the
/// identity database, next to the groups and service accounts their principals
/// name; otherwise they stay in memory. The connecting role needs `SELECT`,
/// `INSERT`, `UPDATE` and `DELETE` on `authz.grants`, and `SELECT` on the user
/// table's `id` and `email` and on `authz.groups` and `authz.service_accounts`.
fn with_grants(
    handler: ServerHandler<RequestContext>,
    #[cfg_attr(not(feature = "olai-auth"), allow(unused_variables))] config: &Config,
) -> Result<ServerHandler<RequestContext>> {
    #[cfg(feature = "olai-auth")]
    if config.auth.mode == AuthMode::OlaiAuth {
        let olai = config
            .auth
            .olai_auth(&config.backend)
            .map_err(|e| Error::Generic(format!("auth config: {e}")))?;
        let pool = crate::rest::olai_auth::identity_pool(&olai.database_url)
            .map_err(|e| Error::Generic(format!("auth config: {e}")))?;
        let store = olai_auth::PgGrantStore::new(pool.clone()).with_schema(&olai.authz_schema);
        let principals =
            crate::grants::PgPrincipalDirectory::new(pool, &olai.users_schema, &olai.authz_schema);
        return Ok(handler.with_grants(crate::grants::Grants::new(
            Arc::new(store),
            Arc::new(principals),
        )));
    }
    Ok(handler)
}

/// The startup error for a config that asks for `what`, which needs the cargo
/// feature `feature` this build was compiled without.
#[allow(dead_code)] // unused when every feature is on
fn not_compiled(what: &str, feature: &str) -> Error {
    Error::Generic(format!(
        "{what} needs the `{feature}` feature, which this build of the server doesn't include"
    ))
}

/// Connect the configured backend.
///
/// An ephemeral `:memory:` SQLite database is created fresh in this process, so
/// a prior `migrate` step could not have reached it; it is migrated here so a
/// config-less `serve` works out of the box (see [`Backend::is_ephemeral`]). A
/// durable backend is never migrated here — the operator runs `migrate` first —
/// and must already hold this build's schema.
async fn connect(config: &Config) -> Result<LocalHandler> {
    let migrate = config.backend.is_ephemeral();
    if migrate {
        tracing::info!("ephemeral in-memory backend: applying migrations at startup");
    }
    match &config.backend {
        #[cfg(feature = "postgres")]
        Backend::Postgres(pg) => connect_postgres(pg, encryptor(config)?, migrate, config).await,
        #[cfg(not(feature = "postgres"))]
        Backend::Postgres(_) => Err(not_compiled("the postgres backend", "postgres")),
        #[cfg(feature = "sqlite")]
        Backend::Sqlite(cfg) => connect_sqlite(cfg, encryptor(config)?, migrate, config).await,
        #[cfg(not(feature = "sqlite"))]
        Backend::Sqlite(_) => Err(not_compiled("the sqlite backend", "sqlite")),
    }
}

/// The envelope encryptor for secrets at rest.
///
/// A configured KEK is required for durable backends (secrets must survive a
/// restart). For an ephemeral `:memory:` backend, secrets never persist, so
/// fall back to the dev KEK when none is configured — this keeps a UI-only or
/// otherwise minimal config file usable against the dev backend, matching the
/// config-less default (which also ships the dev KEK). See
/// [`crate::config::EncryptionConfig::dev_default`].
#[cfg(any(feature = "postgres", feature = "sqlite"))]
fn encryptor(
    config: &Config,
) -> Result<unitycatalog_common::services::encryption::EnvelopeEncryptor> {
    match config.encryption.as_ref() {
        Some(enc) => enc.build_encryptor().map_err(Error::Generic),
        None if config.backend.is_ephemeral() => crate::config::EncryptionConfig::dev_default()
            .build_encryptor()
            .map_err(Error::Generic),
        None => Err(Error::Generic(
            "missing `encryption` configuration: an active KEK is required to store secrets \
             with a durable backend"
                .into(),
        )),
    }
}

/// The REST router with the surfaces `config.routing` marks `upstream` proxied
/// to `config.upstream` (see [`crate::hybrid`]).
#[cfg(feature = "proxy")]
fn build_upstream_router(
    config: &Config,
    handler: ServerHandler<RequestContext>,
    policy: Arc<dyn Policy<RequestContext>>,
) -> Result<Router> {
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
    let client = unitycatalog_client::UnityCatalogClient::new_unauthenticated(upstream_url);
    Ok(crate::hybrid::build_hybrid_router(
        handler,
        policy,
        client,
        &config.routing,
    ))
}

#[cfg(not(feature = "proxy"))]
fn build_upstream_router(
    _: &Config,
    _: ServerHandler<RequestContext>,
    _: Arc<dyn Policy<RequestContext>>,
) -> Result<Router> {
    Err(not_compiled("upstream `routing`", "proxy"))
}

/// Build the storage byte-proxy router for the configured arm.
///
/// - [`ProxyArm::Local`](crate::config::ProxyArm::Local): the in-process
///   `ServerHandler` (authorize + vend + stream locally).
/// - [`ProxyArm::Client`](crate::config::ProxyArm::Client): a portable backend
///   pointed at any UC server via `{base_url, token}`.
#[cfg(feature = "storage-proxy")]
async fn build_storage_proxy_router(
    config: &Config,
    handler: ServerHandler<RequestContext>,
) -> Result<Router> {
    use crate::config::ProxyArm;

    match config.storage_proxy.arm {
        ProxyArm::Local => Ok(crate::rest::create_storage_proxy_router(handler)),
        ProxyArm::Client => {
            let client = config.storage_proxy.client.as_ref().ok_or_else(|| {
                Error::Generic("storage-proxy client arm requires a `client` block".into())
            })?;
            let token = client.token.as_ref().and_then(|t| t.value());
            let backend = unitycatalog_storage_proxy::UnityFactoryProxyBackend::connect(
                &client.base_url,
                token,
            )
            .await
            .map_err(|e| Error::Generic(format!("storage-proxy client connect: {e}")))?;
            use std::sync::Arc;
            use unitycatalog_storage_proxy::{ContextExtractor, router_with_context};
            let extract_cx: ContextExtractor<RequestContext> = Arc::new(|parts| {
                let cx = RequestContext::from_extensions(&parts.extensions);
                Box::pin(async move { Ok(cx) })
            });
            Ok(router_with_context::<(), _>(Arc::new(backend), extract_cx).with_state(()))
        }
    }
}

/// Build the embedded Files API router: construct the configured file store
/// backend, register the `FilesService` on a ConnectRPC router, and expose it as
/// a single axum fallback service (the caller nests it under a base path).
///
/// - [`FilesBackend::Unity`](crate::config::FilesBackend::Unity): a
///   `UnityVolumeStore` over a factory pointed at the configured upstream UC.
/// - [`FilesBackend::Memory`](crate::config::FilesBackend::Memory): a
///   process-local in-memory store (non-durable; demos/tests).
#[cfg(feature = "files")]
async fn build_files_router(config: &Config) -> Result<Router> {
    use crate::config::FilesBackend;
    use unitycatalog_files_api::service::AppState;
    use unitycatalog_files_api::store::{FileStore, MemoryStore, UnityVolumeStore};

    let files: Arc<dyn FileStore> = match config.files.backend {
        FilesBackend::Memory => Arc::new(MemoryStore::new()),
        FilesBackend::Unity => {
            let client = config.files.client.as_ref().ok_or_else(|| {
                Error::Generic("files unity backend requires a `client` block".into())
            })?;
            let token = client.token.as_ref().and_then(|t| t.value());
            let mut builder = unitycatalog_object_store::UnityObjectStoreFactory::builder()
                .with_uri(client.base_url.clone())
                .with_io_runtime(tokio::runtime::Handle::current());
            match token.filter(|t| !t.is_empty()) {
                Some(token) => builder = builder.with_token(token),
                None => builder = builder.with_allow_unauthenticated(true),
            }
            if let Some(region) = client.region.as_ref().filter(|r| !r.is_empty()) {
                builder = builder.with_aws_region(region.clone());
            }
            let factory = builder
                .build()
                .await
                .map_err(|e| Error::Generic(format!("files unity factory: {e}")))?;
            Arc::new(UnityVolumeStore::new(Arc::new(factory)))
        }
    };

    Ok(AppState::new(files).into_axum_router())
}

/// Connect the configured backend, apply any pending migrations and create the
/// declared property indexes, then return. The only schema-mutating path (see the
/// module docs).
pub async fn migrate(config: Config) -> Result<()> {
    match &config.backend {
        #[cfg(feature = "postgres")]
        Backend::Postgres(pg) => {
            // Over the direct connection: the migrator holds a session-level
            // advisory lock, which a transaction-mode pooler can't.
            let db_url = pg.migration_connection_string().ok_or_else(|| {
                Error::Generic("incomplete postgres backend configuration".into())
            })?;
            let pool = unitycatalog_postgres::connect_pool(&db_url)
                .await
                .map_err(|e| Error::Generic(format!("connecting to database: {e}")))?;
            unitycatalog_postgres::unified_migrator()
                .run(&pool)
                .await
                .map_err(|e| Error::Generic(format!("running migrations: {e}")))?;
            unitycatalog_postgres::ensure_property_indexes(&pool)
                .await
                .map_err(|e| Error::Generic(format!("creating property indexes: {e}")))?;
        }
        #[cfg(not(feature = "postgres"))]
        Backend::Postgres(_) => Err::<(), _>(not_compiled("the postgres backend", "postgres"))?,
        #[cfg(feature = "sqlite")]
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
            unitycatalog_sqlite::ensure_property_indexes(&pool)
                .await
                .map_err(|e| Error::Generic(format!("creating property indexes: {e}")))?;
        }
        #[cfg(not(feature = "sqlite"))]
        Backend::Sqlite(_) => Err::<(), _>(not_compiled("the sqlite backend", "sqlite"))?,
    }
    tracing::info!("migrations applied");
    Ok(())
}

/// The OpenAPI definitions mounted as Swagger UI, shared by the local and hybrid
/// paths. Kept as a free function so both router builders stay in sync.
#[cfg(feature = "swagger")]
pub(crate) fn swagger_api_defs() -> Vec<swagger_ui_dist::ApiDefinition<&'static str>> {
    use swagger_ui_dist::{ApiDefinition, OpenApiSource};

    vec![
        ApiDefinition {
            uri_prefix: "/api/2.1/unity-catalog",
            // In-crate copy of the canonical repo-root `openapi/openapi.yaml`, so
            // the embed survives `cargo publish` (which packages only files under
            // the crate; a `../../../` read escapes the tarball). Kept in sync by
            // `just sync-server-openapi` + a CI drift guard — see justfile.
            api_definition: OpenApiSource::Inline(include_str!("../openapi/openapi.yaml")),
            title: Some("Unity Catalog API"),
        },
        // The Delta REST API routes live at `/delta/v1/...` under the UC base
        // path, but its Swagger UI + spec are hosted under a distinct prefix so
        // the swagger-ui asset routes don't collide with the main UC API's.
        ApiDefinition {
            uri_prefix: "/api/2.1/unity-catalog/delta",
            // In-crate copy of repo-root `openapi/delta.yaml` (same publish-safety
            // reason as above).
            api_definition: OpenApiSource::Inline(include_str!("../openapi/delta.yaml")),
            title: Some("UC Delta API"),
        },
    ]
}

/// Build the all-local REST router (every surface served from `handler`).
// Concrete over `ServerHandler<RequestContext>` — the only handler/context this is
// ever built with. The sibling `create_*_router` helpers stay generic over
// `Cx: FromRequestParts<T>`; `create_delta_router`, however, supplies the
// Delta router's context via an explicit `Principal → RequestContext` extractor
// closure (the Delta router is no longer an axum `FromRequestParts` consumer — see
// issue #135), which pins this builder to the concrete context type.
pub(crate) fn build_rest_router(handler: ServerHandler<RequestContext>) -> Router {
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
        .merge(create_delta_router(handler.clone()))
        .merge(create_entity_tag_assignments_router(handler.clone()))
        .merge(create_policies_router(handler.clone()))
        .merge(create_grants_router(handler.clone()));

    Router::new()
        .nest("/api/2.1/unity-catalog", api_routes)
        // Tag Policies (governed tag definitions) live under /api/2.1, not /unity-catalog.
        .nest("/api/2.1", create_tag_policies_router(handler))
}

/// Operational endpoints served regardless of backend or routing.
///
/// `/health` returns the literal `OK` body the `healthcheck` subcommand (and the
/// Docker `HEALTHCHECK`) expects; `/version` returns the crate version so a
/// running service can be matched to a release (binary + bundled UI).
///
/// `/capabilities` is a server-level discovery endpoint announcing the
/// `storageAccess` posture (`"direct"` when the proxy is off, `"proxy"` when on).
/// It is mounted at the root alongside `/health`/`/version` — server-level, not a
/// UC securable surface, and reachable independent of `ui.base_path`.
fn operational_router(storage_proxy: &StorageProxyConfig) -> Router {
    let capabilities = capabilities_body(storage_proxy);
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
        .route(
            "/capabilities",
            get(move || async move {
                (
                    [(axum::http::header::CONTENT_TYPE, "application/json")],
                    capabilities,
                )
            }),
        )
}

/// The `/capabilities` JSON body for the configured storage-proxy posture.
fn capabilities_body(storage_proxy: &StorageProxyConfig) -> String {
    if storage_proxy.enabled {
        // `basePath` is the byte-proxy mount; `conditionalWrites` advertises the
        // server-enforced `If-Match` relay.
        r#"{"storageAccess":"proxy","storageProxy":{"basePath":"/storage-proxy","conditionalWrites":true}}"#
            .to_string()
    } else {
        r#"{"storageAccess":"direct"}"#.to_string()
    }
}

/// Assemble the final service — API + operational + Swagger UI + (optional) SPA —
/// mounted under the configured base path, with the trace layer.
fn assemble(api_router: Router, ui: &UiConfig, storage_proxy: &StorageProxyConfig) -> Router {
    // Swagger UI asset routes are merged onto the API router.
    #[cfg(feature = "swagger")]
    let api_router = swagger_api_defs()
        .into_iter()
        .fold(api_router, |router, api| {
            router.merge(swagger_ui_dist::generate_routes(api))
        });
    let mut router = api_router;

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
    let router = operational_router(storage_proxy).merge(router);

    router.layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros),
            ),
    )
}

#[cfg(feature = "postgres")]
async fn connect_postgres(
    pg: &crate::config::PostgresBackendConfig,
    encryptor: unitycatalog_common::services::encryption::EnvelopeEncryptor,
    migrate: bool,
    config: &Config,
) -> Result<LocalHandler> {
    use unitycatalog_common::store::ObjectStoreAdapter;
    use unitycatalog_postgres::PgCommitCoordinator;

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
        unitycatalog_postgres::ensure_property_indexes(&pool)
            .await
            .map_err(|e| Error::Generic(format!("creating property indexes: {e}")))?;
    } else {
        check_postgres_schema(pg, &pool).await?;
    }
    // The object/association graph store (lifted to `ResourceStore` by
    // `ObjectStoreAdapter`), the storage-access store and the Delta
    // `CommitCoordinator` are independent handles on the same pool. Sensitive
    // fields (tokens) are sealed inline on the stored resources; credentials and
    // external locations live in their own tables with sealed secrets; Delta
    // catalog-managed commits are persisted in the `delta_commits` table.
    let graph = unitycatalog_postgres::connect_graph(pool.clone(), encryptor.clone());
    let resource_store = Arc::new(ObjectStoreAdapter::new(graph));
    let policy = build_policy(config)?;
    let storage_access = Arc::new(unitycatalog_postgres::PgStorageAccess::new(
        pool.clone(),
        encryptor,
    ));
    let coordinator = Arc::new(PgCommitCoordinator::new(pool));
    let handler = ServerHandler::try_new_tokio_with_coordinator(
        policy.clone(),
        resource_store,
        storage_access,
        coordinator,
    )
    .map_err(|e| Error::Generic(e.to_string()))?;
    Ok((handler, policy))
}

/// Fail unless the Postgres schema is current, checked over the migration URL
/// when one is configured (see
/// [`migration_url`](crate::config::PostgresBackendConfig::migration_url)),
/// else over the request pool.
#[cfg(feature = "postgres")]
async fn check_postgres_schema(
    pg: &crate::config::PostgresBackendConfig,
    pool: &sqlx::PgPool,
) -> Result<()> {
    let migrator = unitycatalog_postgres::unified_migrator();
    if pg.migration_url.is_none() {
        return crate::schema::ensure_current_postgres(&migrator, pool).await;
    }
    let url = pg
        .migration_connection_string()
        .ok_or_else(|| Error::Generic("postgres `migration_url` could not be resolved".into()))?;
    let direct = sqlx::postgres::PgPoolOptions::new()
        .max_connections(1)
        .connect(&url)
        .await
        .map_err(|e| Error::Generic(format!("connecting to database (migration url): {e}")))?;
    let result = crate::schema::ensure_current_postgres(&migrator, &direct).await;
    direct.close().await;
    result
}

#[cfg(feature = "sqlite")]
async fn connect_sqlite(
    cfg: &crate::config::SqliteBackendConfig,
    encryptor: unitycatalog_common::services::encryption::EnvelopeEncryptor,
    migrate: bool,
    config: &Config,
) -> Result<LocalHandler> {
    use unitycatalog_common::store::ObjectStoreAdapter;
    use unitycatalog_sqlite::SqliteCommitCoordinator;

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
        unitycatalog_sqlite::ensure_property_indexes(&pool)
            .await
            .map_err(|e| Error::Generic(format!("creating property indexes: {e}")))?;
    } else {
        crate::schema::ensure_current_sqlite(&unitycatalog_sqlite::unified_migrator(), &pool)
            .await?;
    }
    // The object/association graph store (lifted to `ResourceStore` by
    // `ObjectStoreAdapter`), the storage-access store and the Delta
    // `CommitCoordinator` are independent handles on the same pool, as on the
    // Postgres backend: credentials and external locations live in their own
    // tables, Delta catalog-managed commits in the `delta_commits` table.
    let graph = unitycatalog_sqlite::connect_graph(pool.clone(), encryptor.clone());
    let resource_store = Arc::new(ObjectStoreAdapter::new(graph));
    let policy = build_policy(config)?;
    let storage_access = Arc::new(unitycatalog_sqlite::SqliteStorageAccess::new(
        pool.clone(),
        encryptor,
    ));
    let coordinator = Arc::new(SqliteCommitCoordinator::new(pool));
    let handler = ServerHandler::try_new_tokio_with_coordinator(
        policy.clone(),
        resource_store,
        storage_access,
        coordinator,
    )
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
#[cfg(feature = "ui")]
fn mount_spa(app: Router) -> Router {
    use tower_http::services::ServeDir;

    let index_path = std::path::Path::new(UI_DIR).join("index.html");
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

/// Without the `ui` feature there is no bundle to serve: the API is served alone.
#[cfg(not(feature = "ui"))]
fn mount_spa(app: Router) -> Router {
    tracing::warn!(
        "`ui.serve` is set but this build doesn't include the bundled UI (feature `ui`); \
         serving the API only. Set `ui.serve: false` to silence this."
    );
    app
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
