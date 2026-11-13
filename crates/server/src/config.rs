use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct EnvValue {
    pub env: String,
}

/// A leaf value in the configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ConfigValue {
    Value(String),
    Environment(EnvValue),
}

impl ConfigValue {
    /// Get the resolved value of the config value.
    ///
    /// Returns the specified value directly or resolves it from the environment
    /// variable specified in the `Environment` variant.
    pub fn value(&self) -> Option<String> {
        match self {
            ConfigValue::Value(value) => Some(value.clone()),
            ConfigValue::Environment(env) => std::env::var(&env.env).ok(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    /// The host address to bind the server to.
    #[serde(default)]
    pub host: Option<String>,

    /// The port to bind the server to.
    #[serde(default)]
    pub port: Option<u16>,

    /// The backend configuration.
    #[serde(default)]
    pub backend: Backend,

    /// Envelope-encryption configuration for secrets at rest.
    ///
    /// Required whenever secrets are stored (i.e. always, in practice). Defines the active
    /// key-encryption key (KEK) used to wrap per-secret data keys, plus any retired KEKs kept
    /// available for decryption during rotation.
    #[serde(default)]
    pub encryption: Option<EncryptionConfig>,

    /// Upstream Unity Catalog instance to delegate selected surfaces to.
    ///
    /// Required when [`Config::routing`] marks any surface as
    /// [`RoutingMode::Upstream`]; ignored otherwise.
    #[serde(default)]
    pub upstream: Option<UpstreamConfig>,

    /// Per-surface routing: whether each API surface is served locally or
    /// proxied to the [`upstream`](Config::upstream) instance.
    ///
    /// Defaults to all-local, so existing configs behave exactly as before.
    #[serde(default)]
    pub routing: RoutingConfig,

    /// Allowlist governing which host filesystem paths may back a `file://`
    /// storage location. Empty (the default) denies all local storage.
    #[serde(default)]
    pub local_storage: LocalStorageConfig,

    /// Metastore-level managed storage root.
    ///
    /// The default managed storage location for the metastore as a whole. A
    /// managed catalog created without an explicit `storage_root` inherits this
    /// root, mirroring the Unity Catalog metastore → catalog → schema hierarchy.
    /// When unset (the default), every managed catalog must supply its own
    /// `storage_root`. A `file://` root must sit within an allowed
    /// [`local_storage`](Config::local_storage) root.
    #[serde(default)]
    pub managed_storage_root: Option<String>,

    /// Name of the registered storage credential governing
    /// [`managed_storage_root`](Config::managed_storage_root).
    ///
    /// Managed tables created under the metastore root are vended from this
    /// credential when no external location covers their path, so a managed
    /// catalog without its own `storage_root` works without registering an
    /// external location over the root. When unset, such paths must be covered by
    /// an external location. The credential is looked up when vending, so it may
    /// be registered after startup.
    #[serde(default)]
    pub managed_storage_root_credential: Option<String>,

    /// Bundled web-UI serving settings (see [`UiConfig`]).
    #[serde(default)]
    pub ui: UiConfig,

    /// Same-origin storage byte-proxy settings (see [`StorageProxyConfig`]).
    ///
    /// Defaults to disabled: the server announces `storageAccess: "direct"` and
    /// mounts no proxy surface, so existing deployments are unaffected.
    #[serde(default)]
    pub storage_proxy: StorageProxyConfig,

    /// Embedded Files API settings (see [`FilesConfig`]).
    ///
    /// Defaults to disabled: the server mounts no Files API surface, so existing
    /// deployments are unaffected.
    #[serde(default)]
    pub files: FilesConfig,

    /// Request authentication settings (see [`AuthConfig`]).
    ///
    /// Defaults to anonymous: every request is [`Principal::anonymous`]. Set
    /// `mode: reverse-proxy` when running behind a trusted proxy that forwards
    /// the caller identity — recommended before enabling the storage proxy in a
    /// shared deployment so vends are attributed to a real user.
    ///
    /// [`Principal::anonymous`]: crate::policy::Principal::anonymous
    #[serde(default)]
    pub auth: AuthConfig,
}

/// Environment variable holding the configuration as inline YAML ([`Config::from_env`]).
pub const CONFIG_YAML_ENV: &str = "UC_SERVER_CONFIG_YAML";
/// Environment variable naming the configuration file.
pub const CONFIG_PATH_ENV: &str = "UC_SERVER_CONFIG";
/// Environment variable with a Postgres URL that overrides the configured
/// backend in [`Config::from_env`].
pub const DATABASE_URL_ENV: &str = "DATABASE_URL";
/// Environment variable with the direct (unpooled) Postgres URL used for
/// migrations and the startup schema check in [`Config::from_env`].
pub const DATABASE_URL_UNPOOLED_ENV: &str = "DATABASE_URL_UNPOOLED";

/// Default bind host used when neither the config file nor a CLI flag sets one.
pub const DEFAULT_HOST: &str = "0.0.0.0";
/// Default listen port used when neither the config file nor a CLI flag sets one.
pub const DEFAULT_PORT: u16 = 8080;

impl Config {
    /// Load configuration from an optional YAML file path, falling back to
    /// [`Config::default`] when the path is `None` or does not exist.
    ///
    /// This is the single config entry point shared by every subcommand
    /// (`serve` / `migrate` / `healthcheck`), so they all resolve host, port,
    /// backend, and UI settings identically.
    pub fn load(path: Option<&String>) -> Result<Self, String> {
        let Some(path) = path else {
            return Ok(Config::default());
        };
        let p = std::path::Path::new(path);
        if !p.exists() {
            tracing::info!("config file not found at {}, using defaults", p.display());
            return Ok(Config::default());
        }
        let contents =
            std::fs::read_to_string(p).map_err(|e| format!("reading config `{path}`: {e}"))?;
        serde_yml::from_str(&contents).map_err(|e| format!("parsing config `{path}`: {e}"))
    }

    /// Load configuration entirely from the environment, for hosts with no config
    /// file or CLI (e.g. a Neon Function).
    ///
    /// - [`CONFIG_YAML_ENV`] holds the configuration as an inline YAML document;
    ///   otherwise [`CONFIG_PATH_ENV`] names a file, read like [`Config::load`];
    ///   otherwise [`Config::default`].
    /// - [`DATABASE_URL_ENV`], when set, replaces the backend with Postgres at that
    ///   URL, and [`DATABASE_URL_UNPOOLED_ENV`] (when set) becomes its
    ///   [`migration_url`](PostgresBackendConfig::migration_url). This matches what
    ///   Neon sets for a branch: a pooled URL for requests and a direct one for
    ///   the schema.
    ///
    /// The `uc-server` binary doesn't read `DATABASE_URL`, so a local `.env` meant
    /// for sqlx's offline tooling can't redirect its backend.
    pub fn from_env() -> Result<Self, String> {
        Self::from_vars(|name| std::env::var(name).ok())
    }

    fn from_vars(var: impl Fn(&str) -> Option<String>) -> Result<Self, String> {
        let mut config = match var(CONFIG_YAML_ENV) {
            Some(yaml) => serde_yml::from_str(&yaml)
                .map_err(|e| format!("parsing `{CONFIG_YAML_ENV}`: {e}"))?,
            None => Self::load(var(CONFIG_PATH_ENV).as_ref())?,
        };
        if let Some(url) = var(DATABASE_URL_ENV).filter(|u| !u.is_empty()) {
            config.backend = Backend::Postgres(PostgresBackendConfig {
                url: Some(ConfigValue::Value(url)),
                migration_url: var(DATABASE_URL_UNPOOLED_ENV)
                    .filter(|u| !u.is_empty())
                    .map(ConfigValue::Value),
                ..Default::default()
            });
        }
        Ok(config)
    }

    /// Resolved bind host: the configured value, else [`DEFAULT_HOST`].
    pub fn resolved_host(&self) -> &str {
        self.host.as_deref().unwrap_or(DEFAULT_HOST)
    }

    /// Resolved listen port: the configured value, else [`DEFAULT_PORT`].
    pub fn resolved_port(&self) -> u16 {
        self.port.unwrap_or(DEFAULT_PORT)
    }

    /// The `/health` URL a `healthcheck` probe should GET. A wildcard bind host
    /// (`0.0.0.0` / empty) maps to loopback, since that is not a connectable
    /// address for a client.
    pub fn health_url(&self) -> String {
        let host = match self.resolved_host() {
            "0.0.0.0" | "" | "::" => "127.0.0.1",
            other => other,
        };
        format!("http://{host}:{}/health", self.resolved_port())
    }
}

/// Web UI serving settings.
///
/// By default the bundled single-page app is served from the service root (`/`)
/// as a fallback behind the API routes. Set [`serve`](UiConfig::serve) to `false`
/// to run the service API-only — the SPA routes are not mounted and any on-disk
/// bundle is ignored — for deployments that embed a custom UI (built on the
/// shipped `@open-lakehouse/*` components) or serve none. The CLI `--no-ui` flag
/// sets this to `false`.
///
/// Set [`base_path`](UiConfig::base_path) to serve the UI (and every API route)
/// under a sub-path instead — the "static prefix" pattern used when the server
/// sits behind a gateway at e.g. `https://platform.example.com/catalog/`. The
/// value is normalized on load (leading slash enforced, trailing slash stripped),
/// so `catalog`, `/catalog`, and `/catalog/` all become `/catalog`; empty means
/// "serve at root".
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(default)]
pub struct UiConfig {
    /// URL prefix the UI and all API routes are served under. Empty = root.
    pub base_path: String,
    /// Whether to serve the bundled single-page app. `true` (default) mounts the
    /// SPA routes; `false` runs API-only, even if a bundle is on disk. The CLI
    /// `--no-ui` flag sets this to `false`.
    pub serve: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            base_path: String::new(),
            serve: true,
        }
    }
}

impl UiConfig {
    /// Normalize [`base_path`](UiConfig::base_path): trim, and if non-empty force
    /// a single leading slash and drop the trailing slash. Idempotent.
    pub fn normalized_base_path(&self) -> String {
        let trimmed = self.base_path.trim().trim_matches('/');
        if trimmed.is_empty() {
            String::new()
        } else {
            format!("/{trimmed}")
        }
    }
}

/// Configuration for local (`file://`) storage locations.
///
/// Deny-by-default: with no allowed roots, the server rejects every `file://`
/// storage location (external locations, external tables/volumes, and managed
/// catalog/schema roots). List one or more absolute host paths to permit local
/// storage beneath them — typically a single dev data directory.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct LocalStorageConfig {
    /// Absolute host paths under which `file://` storage locations are allowed.
    /// Each must exist at startup; paths are matched on whole-component
    /// boundaries after resolving symlinks.
    #[serde(default)]
    pub allowed_roots: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            backend: Backend::default(),
            // No config file: fall back to a dev KEK so the default ephemeral
            // SQLite server runs out of the box. Real deployments supply their
            // own `encryption` config (see `EncryptionConfig::dev_default`).
            encryption: Some(EncryptionConfig::dev_default()),
            upstream: None,
            routing: RoutingConfig::default(),
            local_storage: LocalStorageConfig::default(),
            managed_storage_root: None,
            managed_storage_root_credential: None,
            ui: UiConfig::default(),
            storage_proxy: StorageProxyConfig::default(),
            files: FilesConfig::default(),
            auth: AuthConfig::default(),
        }
    }
}

/// Which backend arm serves the storage byte-proxy.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProxyArm {
    /// Serve from the in-process server handler: authorize + vend + stream
    /// locally (default). Requires no extra configuration.
    #[default]
    Local,
    /// Serve via a [`UnityObjectStoreFactory`] pointed at any UC server, using
    /// [`StorageProxyClientConfig`]. Portable, UC-server-agnostic.
    Client,
}

/// Same-origin storage byte-proxy configuration.
///
/// When [`enabled`](Self::enabled) is `false` (the default) the server announces
/// `storageAccess: "direct"` at `/capabilities` and mounts no `/storage-proxy`
/// surface — no behavior change for existing deployments. Enabling it announces
/// `"proxy"` and mounts the surface served by the selected [`arm`](Self::arm).
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct StorageProxyConfig {
    /// Master switch. Default `false`.
    #[serde(default)]
    pub enabled: bool,

    /// Which backend arm serves the proxy. Defaults to [`ProxyArm::Local`].
    #[serde(default)]
    pub arm: ProxyArm,

    /// Upstream connection, **required** when [`arm`](Self::arm) is
    /// [`ProxyArm::Client`]; ignored otherwise.
    #[serde(default)]
    pub client: Option<StorageProxyClientConfig>,
}

/// Upstream connection for the client arm of the storage proxy.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct StorageProxyClientConfig {
    /// Base URL of the UC server to resolve + vend through, e.g.
    /// `http://uc-java:8080/api/2.1/unity-catalog/`.
    pub base_url: String,

    /// Optional bearer token (inline or `{env: ...}`). Absent = unauthenticated
    /// (same-origin cookie forwarding).
    #[serde(default)]
    pub token: Option<ConfigValue>,
}

impl StorageProxyConfig {
    /// Whether a client-arm configuration is present.
    pub fn has_client(&self) -> bool {
        self.client.is_some()
    }

    /// The config error (if any) that should refuse server startup: the client
    /// arm is selected but no `client` block is configured. Returns the offending
    /// message, or `None` when the config is coherent.
    pub fn startup_error(&self) -> Option<&'static str> {
        if self.enabled && self.arm == ProxyArm::Client && self.client.is_none() {
            Some("storage-proxy arm is `client` but no `client` block is configured")
        } else {
            None
        }
    }
}

/// Which backend serves the embedded Files API.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FilesBackend {
    /// Serve Unity Catalog volumes through a [`FilesClientConfig`]-configured
    /// upstream UC (default). Requires a `client` block.
    #[default]
    Unity,
    /// Serve a process-local in-memory store — non-durable; for demos/tests.
    Memory,
}

/// Embedded `portal.files.v1.FilesService` configuration.
///
/// When [`enabled`](Self::enabled) is `false` (the default) the server mounts no
/// Files API surface — no behavior change for existing deployments. Enabling it
/// mounts the ConnectRPC service under [`base_path`](Self::base_path), served by
/// the selected [`backend`](Self::backend).
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct FilesConfig {
    /// Master switch. Default `false`.
    #[serde(default)]
    pub enabled: bool,

    /// Which backend serves the Files API. Defaults to [`FilesBackend::Unity`].
    #[serde(default)]
    pub backend: FilesBackend,

    /// Upstream connection, **required** when [`backend`](Self::backend) is
    /// [`FilesBackend::Unity`]; ignored otherwise.
    #[serde(default)]
    pub client: Option<FilesClientConfig>,

    /// URL prefix the FilesService is mounted under. Empty (the default) mounts
    /// at root — the ConnectRPC method paths are fully-qualified, so no prefix is
    /// required. Normalized like the storage-proxy `base_path`.
    #[serde(default)]
    pub base_path: Option<String>,
}

/// Upstream connection for the Unity backend of the embedded Files API.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct FilesClientConfig {
    /// Base URL of the UC server to vend volume credentials through, e.g.
    /// `http://uc:8080/api/2.1/unity-catalog/`.
    pub base_url: String,

    /// Optional bearer token (inline or `{env: ...}`). Absent = unauthenticated.
    #[serde(default)]
    pub token: Option<ConfigValue>,

    /// Optional AWS region override for vended credentials.
    #[serde(default)]
    pub region: Option<String>,
}

impl FilesConfig {
    /// The config error (if any) that should refuse server startup: the Unity
    /// backend is selected but no `client` block is configured.
    pub fn startup_error(&self) -> Option<&'static str> {
        if self.enabled && self.backend == FilesBackend::Unity && self.client.is_none() {
            Some("files backend is `unity` but no `client` block is configured")
        } else {
            None
        }
    }

    /// Resolved mount prefix: normalized to a single leading slash with no
    /// trailing slash; empty means "mount at root".
    pub fn resolved_base_path(&self) -> String {
        let raw = self.base_path.as_deref().unwrap_or("");
        let trimmed = raw.trim().trim_matches('/');
        if trimmed.is_empty() {
            String::new()
        } else {
            format!("/{trimmed}")
        }
    }
}

/// How incoming requests are authenticated.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AuthConfig {
    /// The authentication mode. Defaults to [`AuthMode::Anonymous`].
    #[serde(default)]
    pub mode: AuthMode,

    /// Header the reverse-proxy mode reads the forwarded user from. Defaults to
    /// `x-forwarded-user`. Ignored unless `mode` is
    /// [`AuthMode::ReverseProxy`].
    #[serde(default)]
    pub forwarded_user_header: Option<String>,

    /// In reverse-proxy mode, treat a request lacking the forwarded-user header
    /// as anonymous instead of rejecting it (`401`). Default `false` (reject).
    #[serde(default)]
    pub allow_missing_identity: bool,

    /// The accepted bearer tokens, each with the principal it authenticates.
    /// Required (non-empty) in [`AuthMode::Bearer`]; ignored otherwise.
    #[serde(default)]
    pub tokens: Vec<BearerTokenConfig>,
}

/// A bearer token accepted in [`AuthMode::Bearer`].
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct BearerTokenConfig {
    /// The user a request presenting this token acts as.
    pub principal: String,

    /// The token, usually from the environment (`{ env: UC_ADMIN_TOKEN }`).
    pub token: ConfigValue,
}

impl AuthConfig {
    /// Resolve the configured bearer tokens into `(token, principal)` pairs.
    ///
    /// Fails if none is configured, or if one doesn't resolve to a non-empty
    /// value, so a missing secret is a startup error rather than a token nobody
    /// can present.
    pub fn bearer_tokens(&self) -> Result<Vec<(String, String)>, String> {
        if self.tokens.is_empty() {
            return Err("bearer auth needs at least one entry in `auth.tokens`".into());
        }
        self.tokens
            .iter()
            .map(|t| match t.token.value().filter(|v| !v.is_empty()) {
                Some(token) => Ok((token, t.principal.clone())),
                None => Err(format!(
                    "the bearer token for principal `{}` could not be resolved",
                    t.principal
                )),
            })
            .collect()
    }
}

/// The request-authentication strategy.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthMode {
    /// Every request is anonymous. The default; suitable for local development
    /// and single-tenant deployments.
    #[default]
    Anonymous,
    /// Trust a forwarded-identity header set by an upstream reverse proxy that
    /// has already authenticated the caller. Only safe when the server is not
    /// directly reachable.
    ReverseProxy,
    /// Require `Authorization: Bearer <token>` with one of the configured
    /// [`tokens`](AuthConfig::tokens); the request acts as that token's
    /// principal. For a server reachable directly, without a gateway in front.
    Bearer,
}

/// Configuration for the upstream Unity Catalog instance.
///
/// The hybrid server proxies selected surfaces to this instance while serving
/// the rest from its local [`Backend`]. The upstream connection is
/// unauthenticated by design: authorization is enforced in *this* server's
/// policy layer before any request is forwarded, which lets the hybrid server
/// act as a test-bed for policy-engine integrations against real upstream data.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UpstreamConfig {
    /// Base URL of the upstream Unity Catalog REST API, e.g.
    /// `http://uc-java:8080/api/2.1/unity-catalog`.
    pub url: String,
}

/// Whether an API surface is served from the local store or proxied upstream.
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RoutingMode {
    /// Serve the surface from the local backend store (default).
    #[default]
    Local,
    /// Proxy the surface to the upstream instance.
    Upstream,
}

/// Per-surface routing configuration.
///
/// One field per Unity Catalog surface mounted by the REST server. Each
/// defaults to [`RoutingMode::Local`], so an empty or absent `routing` section
/// preserves the all-local behavior.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct RoutingConfig {
    #[serde(default)]
    pub catalogs: RoutingMode,
    #[serde(default)]
    pub schemas: RoutingMode,
    #[serde(default)]
    pub tables: RoutingMode,
    #[serde(default)]
    pub credentials: RoutingMode,
    #[serde(default)]
    pub external_locations: RoutingMode,
    #[serde(default)]
    pub functions: RoutingMode,
}

impl RoutingConfig {
    /// Returns `true` if any surface is routed upstream.
    pub fn any_upstream(&self) -> bool {
        self.surfaces()
            .iter()
            .any(|(_, mode)| *mode == RoutingMode::Upstream)
    }

    /// Names of all surfaces currently routed upstream (for display).
    pub fn upstream_surfaces(&self) -> Vec<&'static str> {
        self.surfaces()
            .into_iter()
            .filter(|(_, mode)| *mode == RoutingMode::Upstream)
            .map(|(name, _)| name)
            .collect()
    }

    fn surfaces(&self) -> [(&'static str, RoutingMode); 6] {
        [
            ("catalogs", self.catalogs),
            ("schemas", self.schemas),
            ("tables", self.tables),
            ("credentials", self.credentials),
            ("external-locations", self.external_locations),
            ("functions", self.functions),
        ]
    }
}

/// Backend configuration for the unity catalog server.
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case", tag = "engine")]
pub enum Backend {
    /// Postgres backend configuration.
    Postgres(PostgresBackendConfig),

    /// Embedded SQLite backend configuration.
    ///
    /// Durable, file-based storage that runs in-process — no external database
    /// to operate. Suitable for a capable local single-binary server. The
    /// special path `:memory:` opens an ephemeral in-process database, which is
    /// the default when no config file is supplied — a feature-complete,
    /// non-persistent dev backend (it coordinates Delta commits like the
    /// file/Postgres backends, unlike the former bespoke in-memory store).
    Sqlite(SqliteBackendConfig),
}

impl Backend {
    /// Whether this backend is an ephemeral, in-process SQLite database
    /// (`:memory:`).
    ///
    /// An ephemeral backend is created fresh per process, so a separate
    /// `migrate` step is meaningless — nothing it wrote would be visible to a
    /// later `serve` process. [`crate::run::serve`] therefore auto-migrates only
    /// this case, and keeps the strict "run `migrate` first" split for durable
    /// backends (file-based SQLite and Postgres).
    pub fn is_ephemeral(&self) -> bool {
        matches!(
            self,
            Backend::Sqlite(cfg) if cfg.database_path().as_deref() == Some(":memory:")
        )
    }
}

impl Default for Backend {
    /// Default to an ephemeral in-process SQLite database (`:memory:`).
    ///
    /// This replaces the former bespoke in-memory store: it exercises the same
    /// store and commit-coordinator code paths as the file and Postgres
    /// backends, so a config-less `uc server` behaves like a real deployment
    /// minus persistence.
    fn default() -> Self {
        Backend::Sqlite(SqliteBackendConfig {
            path: ConfigValue::Value(":memory:".to_string()),
        })
    }
}

/// SQLite backend configuration.
#[derive(Debug, Deserialize, Serialize)]
pub struct SqliteBackendConfig {
    /// Filesystem path to the SQLite database file.
    ///
    /// The file (and any missing schema) is created on first use. The special
    /// value `:memory:` opens an ephemeral in-memory database.
    pub path: ConfigValue,
}

impl SqliteBackendConfig {
    /// Resolve the configured database path.
    pub fn database_path(&self) -> Option<String> {
        self.path.value()
    }
}

/// Postgres backend configuration.
///
/// Either a full connection [`url`](Self::url), or the individual `host`, `port`,
/// `user`, `password` and `database` fields; the URL wins when both are set.
#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PostgresBackendConfig {
    /// The full connection URL, e.g. Neon's `DATABASE_URL`.
    #[serde(default)]
    pub url: Option<ConfigValue>,

    /// A direct connection URL for `migrate` and the startup schema check, for
    /// when [`url`](Self::url) goes through a transaction-mode pooler (PgBouncer,
    /// Neon's pooled URL), which can't hold the migrator's session-level advisory
    /// lock. Defaults to the request connection.
    #[serde(default)]
    pub migration_url: Option<ConfigValue>,

    /// The host of the server.
    #[serde(default)]
    pub host: Option<ConfigValue>,

    /// The port of the server.
    #[serde(default)]
    pub port: Option<ConfigValue>,

    /// The database user.
    #[serde(default)]
    pub user: Option<ConfigValue>,

    /// Password for the user.
    #[serde(default)]
    pub password: Option<ConfigValue>,

    /// The database name.
    #[serde(default)]
    pub database: Option<ConfigValue>,
}

impl PostgresBackendConfig {
    /// The connection string requests are served over: [`url`](Self::url), else
    /// one assembled from the individual fields. `None` if neither resolves.
    pub fn connection_string(&self) -> Option<String> {
        if let Some(url) = &self.url {
            return url.value();
        }
        let value = |v: &Option<ConfigValue>| v.as_ref().and_then(ConfigValue::value);
        let host = value(&self.host)?;
        let port = value(&self.port)?;
        let user = value(&self.user)?;
        let password = value(&self.password)?;
        let database = value(&self.database)?;

        Some(format!(
            "postgres://{user}:{password}@{host}:{port}/{database}"
        ))
    }

    /// The connection string for migrations and the schema check:
    /// [`migration_url`](Self::migration_url), else
    /// [`connection_string`](Self::connection_string).
    pub fn migration_connection_string(&self) -> Option<String> {
        match &self.migration_url {
            Some(url) => url.value(),
            None => self.connection_string(),
        }
    }
}

/// Envelope-encryption configuration for secrets at rest.
///
/// Secrets are sealed with a per-secret data key that is wrapped by a key-encryption key (KEK).
/// Exactly one KEK is `active` (used for new writes); any number of `retired` KEKs may be listed so
/// values previously sealed under them can still be decrypted and lazily re-wrapped during
/// rotation.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct EncryptionConfig {
    /// The active KEK that new secrets are wrapped under.
    pub active: KeyConfig,

    /// Retired KEKs retained for decryption during rotation.
    #[serde(default)]
    pub retired: Vec<KeyConfig>,
}

impl EncryptionConfig {
    /// A fixed, well-known KEK for local development only.
    ///
    /// Used by [`Config::default`] so `uc server` runs without a config file
    /// against the default ephemeral SQLite backend, where secrets do not
    /// survive a restart. **Never use this in production** — supply a real KEK
    /// via a config file.
    pub fn dev_default() -> Self {
        // 32 zero-derived bytes (`0..32`), base64-encoded. Deliberately not secret.
        const DEV_KEK: &str = "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=";
        Self {
            active: KeyConfig {
                id: "dev".to_string(),
                key: ConfigValue::Value(DEV_KEK.to_string()),
            },
            retired: Vec::new(),
        }
    }
}

/// A single key-encryption key: a stable id plus its 32-byte material (base64-encoded).
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct KeyConfig {
    /// Stable identifier for this KEK (e.g. `"v1"`), recorded in every sealed secret.
    pub id: String,

    /// The KEK material: 32 bytes (AES-256), base64-encoded. Resolved from an inline value or an
    /// environment variable via [`ConfigValue`].
    pub key: ConfigValue,
}

impl EncryptionConfig {
    /// Resolve all configured KEKs and build an [`EnvelopeEncryptor`].
    ///
    /// Fails if the active KEK or any retired KEK cannot be resolved or is not valid 32-byte
    /// base64 material.
    pub fn build_encryptor(
        &self,
    ) -> Result<unitycatalog_common::services::encryption::EnvelopeEncryptor, String> {
        use base64::Engine as _;
        use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

        let mut keys = Vec::new();
        for key in std::iter::once(&self.active).chain(self.retired.iter()) {
            let encoded = key
                .key
                .value()
                .ok_or_else(|| format!("KEK '{}' material could not be resolved", key.id))?;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(encoded.trim())
                .map_err(|e| format!("KEK '{}' is not valid base64: {e}", key.id))?;
            keys.push((key.id.clone(), bytes));
        }
        let provider = LocalKeyProvider::new(self.active.id.clone(), keys)
            .map_err(|e| format!("invalid encryption configuration: {e}"))?;
        Ok(EnvelopeEncryptor::local(provider))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_config() {
        let config = r#"
            {
                "host": "0.0.0.0",
                "port": 8080,
                "backend": {
                    "engine": "postgres",
                    "database": "postgres",
                    "host": "localhost",
                    "port": "5432",
                    "user": "user",
                    "password": {
                        "env": "PG_PASSWORD"
                    }
                }
            }
        "#;

        let config: Config = serde_json::from_str(config).unwrap();
        assert_eq!(config.host.as_deref(), Some("0.0.0.0"));
        assert_eq!(config.port, Some(8080));
        assert!(matches!(config.backend, Backend::Postgres(_)));
        let backend = match config.backend {
            Backend::Postgres(backend) => backend,
            _ => unreachable!(),
        };
        let value = |v: &Option<ConfigValue>| v.as_ref().and_then(ConfigValue::value);
        assert_eq!(value(&backend.host).unwrap(), "localhost");
        assert_eq!(value(&backend.port).unwrap(), "5432");
        assert_eq!(value(&backend.user).unwrap(), "user");
        assert_eq!(
            backend.password,
            Some(ConfigValue::Environment(EnvValue {
                env: "PG_PASSWORD".to_string()
            }))
        );
        assert_eq!(backend.url, None);
    }

    #[test]
    fn test_deserialize_sqlite_config() {
        let config = r#"
            backend:
              engine: sqlite
              path: /var/lib/unitycatalog/catalog.db
        "#;
        let config: Config = serde_yml::from_str(config).unwrap();
        let backend = match config.backend {
            Backend::Sqlite(backend) => backend,
            other => panic!("expected sqlite backend, got {other:?}"),
        };
        assert_eq!(
            backend.database_path().as_deref(),
            Some("/var/lib/unitycatalog/catalog.db")
        );
    }

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.host.is_none());
        assert!(config.port.is_none());
        // The config-less default is an ephemeral in-process SQLite database.
        let backend = match config.backend {
            Backend::Sqlite(ref b) => b,
            ref other => panic!("expected sqlite backend, got {other:?}"),
        };
        assert_eq!(backend.database_path().as_deref(), Some(":memory:"));
        assert!(config.upstream.is_none());
        assert_eq!(config.routing, RoutingConfig::default());
        assert!(!config.routing.any_upstream());
        // The default ships a dev KEK so the server runs without a config file.
        let enc = config.encryption.as_ref().expect("dev encryption present");
        assert_eq!(enc.active.id, "dev");
        assert!(enc.build_encryptor().is_ok());
        // The storage proxy is off by default: announces "direct", no mount.
        assert_eq!(config.storage_proxy, StorageProxyConfig::default());
        assert!(!config.storage_proxy.enabled);
        assert_eq!(config.storage_proxy.arm, ProxyArm::Local);
        assert!(config.storage_proxy.startup_error().is_none());
    }

    #[test]
    fn test_minimal_config() {
        let config = r#"{}"#;
        let config: Config = serde_json::from_str(config).unwrap();
        // An empty config gets the default ephemeral SQLite backend.
        assert!(matches!(config.backend, Backend::Sqlite(_)));
        assert!(!config.routing.any_upstream());
    }

    #[test]
    fn test_encryption_config_builds_encryptor() {
        use base64::Engine as _;
        let active = base64::engine::general_purpose::STANDARD.encode([0x11u8; 32]);
        let retired = base64::engine::general_purpose::STANDARD.encode([0x22u8; 32]);
        let yaml = format!(
            r#"
            backend:
              engine: sqlite
              path: ":memory:"
            encryption:
              active:
                id: v2
                key: "{active}"
              retired:
                - id: v1
                  key: "{retired}"
            "#
        );
        let config: Config = serde_yml::from_str(&yaml).unwrap();
        let enc = config.encryption.as_ref().expect("encryption present");
        assert_eq!(enc.active.id, "v2");
        assert_eq!(enc.retired.len(), 1);
        // Builds a working encryptor.
        assert!(enc.build_encryptor().is_ok());

        // Round-trips through YAML.
        let reparsed: Config =
            serde_yml::from_str(&serde_yml::to_string(&config).unwrap()).unwrap();
        assert_eq!(reparsed.encryption, config.encryption);
    }

    #[test]
    fn test_encryption_config_rejects_bad_key() {
        let yaml = r#"
            encryption:
              active:
                id: v1
                key: "not-base64-and-wrong-size!!"
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert!(config.encryption.unwrap().build_encryptor().is_err());
    }

    #[test]
    fn test_managed_storage_root_roundtrips() {
        let yaml = r#"
            backend:
              engine: sqlite
              path: ":memory:"
            managed_storage_root: "s3://bucket/meta"
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(
            config.managed_storage_root.as_deref(),
            Some("s3://bucket/meta")
        );

        // Round-trips back through YAML.
        let reparsed: Config =
            serde_yml::from_str(&serde_yml::to_string(&config).unwrap()).unwrap();
        assert_eq!(reparsed.managed_storage_root, config.managed_storage_root);

        // Absent ⇒ None (default).
        let bare: Config =
            serde_yml::from_str("backend:\n  engine: sqlite\n  path: \":memory:\"\n").unwrap();
        assert!(bare.managed_storage_root.is_none());
    }

    #[test]
    fn test_backend_is_ephemeral() {
        // The config-less default is the ephemeral in-memory backend.
        assert!(Config::default().backend.is_ephemeral());

        // An explicit `:memory:` path is ephemeral.
        let mem: Config =
            serde_yml::from_str("backend:\n  engine: sqlite\n  path: \":memory:\"\n").unwrap();
        assert!(mem.backend.is_ephemeral());

        // A file-backed SQLite database is durable, not ephemeral.
        let file: Config =
            serde_yml::from_str("backend:\n  engine: sqlite\n  path: /tmp/uc.db\n").unwrap();
        assert!(!file.backend.is_ephemeral());
    }

    #[test]
    fn test_ui_config_defaults_and_normalization() {
        // Default: serve at root, UI enabled.
        let ui = UiConfig::default();
        assert!(ui.serve);
        assert_eq!(ui.normalized_base_path(), "");

        // Base-path normalization: leading slash forced, trailing stripped.
        for raw in ["catalog", "/catalog", "/catalog/", "  catalog/  "] {
            let ui = UiConfig {
                base_path: raw.to_string(),
                serve: true,
            };
            assert_eq!(ui.normalized_base_path(), "/catalog", "input {raw:?}");
        }
    }

    #[test]
    fn test_health_url_maps_wildcard_host_to_loopback() {
        let mut cfg = Config {
            host: Some("0.0.0.0".into()),
            port: Some(9000),
            ..Config::default()
        };
        assert_eq!(cfg.health_url(), "http://127.0.0.1:9000/health");

        cfg.host = Some("example.test".into());
        assert_eq!(cfg.health_url(), "http://example.test:9000/health");
    }

    #[test]
    fn test_routing_defaults_to_local() {
        let routing = RoutingConfig::default();
        assert_eq!(routing.catalogs, RoutingMode::Local);
        assert!(!routing.any_upstream());
    }

    #[test]
    fn test_hybrid_config_roundtrip() {
        let yaml = r#"
            backend:
              engine: sqlite
              path: ":memory:"
            upstream:
              url: "http://uc-java:8080/api/2.1/unity-catalog"
            routing:
              catalogs: upstream
              schemas: local
              tables: upstream
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(
            config.upstream,
            Some(UpstreamConfig {
                url: "http://uc-java:8080/api/2.1/unity-catalog".to_string(),
            })
        );
        assert_eq!(config.routing.catalogs, RoutingMode::Upstream);
        assert_eq!(config.routing.schemas, RoutingMode::Local);
        assert_eq!(config.routing.tables, RoutingMode::Upstream);
        assert!(config.routing.any_upstream());

        // round-trips back to YAML and re-parses identically
        let serialized = serde_yml::to_string(&config).unwrap();
        let reparsed: Config = serde_yml::from_str(&serialized).unwrap();
        assert_eq!(reparsed.routing, config.routing);
        assert_eq!(reparsed.upstream, config.upstream);
    }

    #[test]
    fn test_storage_proxy_config_roundtrips() {
        // Enabled + local arm.
        let yaml = r#"
            backend:
              engine: sqlite
              path: ":memory:"
            storage_proxy:
              enabled: true
              arm: local
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert!(config.storage_proxy.enabled);
        assert_eq!(config.storage_proxy.arm, ProxyArm::Local);
        assert!(config.storage_proxy.startup_error().is_none());

        // Enabled + client arm with an env-sourced token.
        let yaml = r#"
            backend:
              engine: sqlite
              path: ":memory:"
            storage_proxy:
              enabled: true
              arm: client
              client:
                base_url: "http://uc-java:8080/api/2.1/unity-catalog/"
                token:
                  env: "UC_TOKEN"
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.storage_proxy.arm, ProxyArm::Client);
        let client = config.storage_proxy.client.as_ref().expect("client block");
        assert_eq!(
            client.base_url,
            "http://uc-java:8080/api/2.1/unity-catalog/"
        );
        assert_eq!(
            client.token,
            Some(ConfigValue::Environment(EnvValue {
                env: "UC_TOKEN".to_string()
            }))
        );
        assert!(config.storage_proxy.startup_error().is_none());

        // Round-trips through YAML.
        let reparsed: Config =
            serde_yml::from_str(&serde_yml::to_string(&config).unwrap()).unwrap();
        assert_eq!(reparsed.storage_proxy, config.storage_proxy);
    }

    #[test]
    fn test_storage_proxy_client_arm_without_client_is_startup_error() {
        let yaml = r#"
            backend:
              engine: sqlite
              path: ":memory:"
            storage_proxy:
              enabled: true
              arm: client
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert!(config.storage_proxy.startup_error().is_some());

        // Disabled + client arm (no client block) is NOT an error — the proxy is
        // not mounted at all.
        let yaml = r#"
            storage_proxy:
              enabled: false
              arm: client
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert!(config.storage_proxy.startup_error().is_none());
    }

    #[test]
    fn test_files_config_defaults_to_disabled() {
        let config = Config::default();
        assert!(!config.files.enabled);
        assert!(config.files.startup_error().is_none());
    }

    #[test]
    fn test_files_config_roundtrips() {
        let yaml = r#"
            files:
              enabled: true
              backend: unity
              base-path: /files
              client:
                base_url: "http://uc:8080/api/2.1/unity-catalog/"
                token:
                  env: UC_TOKEN
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert!(config.files.enabled);
        assert_eq!(config.files.backend, FilesBackend::Unity);
        assert_eq!(config.files.resolved_base_path(), "/files");
        assert!(config.files.client.is_some());
        assert!(config.files.startup_error().is_none());
    }

    #[test]
    fn test_files_memory_backend_needs_no_client() {
        let yaml = r#"
            files:
              enabled: true
              backend: memory
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.files.backend, FilesBackend::Memory);
        assert!(config.files.startup_error().is_none());
        // Empty base path resolves to root; the mount logic then falls back to
        // `/files` to avoid a fallback collision.
        assert_eq!(config.files.resolved_base_path(), "");
    }

    #[test]
    fn test_files_unity_backend_without_client_is_startup_error() {
        let yaml = r#"
            files:
              enabled: true
              backend: unity
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert!(config.files.startup_error().is_some());

        // Disabled + unity backend (no client block) is NOT an error — the Files
        // API is not mounted at all.
        let yaml = r#"
            files:
              enabled: false
              backend: unity
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert!(config.files.startup_error().is_none());
    }

    #[test]
    fn test_every_surface_routes_upstream() {
        let yaml = r#"
            routing:
              catalogs: upstream
              schemas: upstream
              tables: upstream
              credentials: upstream
              external-locations: upstream
              functions: upstream
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(
            config.routing.upstream_surfaces(),
            vec![
                "catalogs",
                "schemas",
                "tables",
                "credentials",
                "external-locations",
                "functions"
            ]
        );
    }

    fn vars(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
        let pairs: Vec<(String, String)> = pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        move |name| {
            pairs
                .iter()
                .find(|(k, _)| k == name)
                .map(|(_, v)| v.clone())
        }
    }

    #[test]
    fn test_from_env_without_vars_is_default() {
        let config = Config::from_vars(vars(&[])).unwrap();
        assert!(config.backend.is_ephemeral());
        assert_eq!(config.auth, AuthConfig::default());
    }

    #[test]
    fn test_from_env_reads_inline_yaml() {
        let yaml = "port: 9000\nui:\n  serve: false\nauth:\n  mode: bearer\n  tokens:\n    - principal: admin\n      token: { env: UC_ADMIN_TOKEN }\n";
        let config = Config::from_vars(vars(&[(CONFIG_YAML_ENV, yaml)])).unwrap();
        assert_eq!(config.port, Some(9000));
        assert!(!config.ui.serve);
        assert_eq!(config.auth.mode, AuthMode::Bearer);
        assert_eq!(
            config.auth.tokens,
            vec![BearerTokenConfig {
                principal: "admin".into(),
                token: ConfigValue::Environment(EnvValue {
                    env: "UC_ADMIN_TOKEN".into()
                }),
            }]
        );
    }

    #[test]
    fn test_from_env_inline_yaml_wins_over_path_and_reports_parse_errors() {
        let config = Config::from_vars(vars(&[
            (CONFIG_YAML_ENV, "port: 9001"),
            (CONFIG_PATH_ENV, "/definitely/not/here.yaml"),
        ]))
        .unwrap();
        assert_eq!(config.port, Some(9001));

        let err = Config::from_vars(vars(&[(CONFIG_YAML_ENV, "port: [")])).unwrap_err();
        assert!(err.contains(CONFIG_YAML_ENV), "{err}");
    }

    #[test]
    fn test_from_env_database_url_overrides_backend() {
        let yaml = "backend:\n  engine: sqlite\n  path: /tmp/uc.db\n";
        let config = Config::from_vars(vars(&[
            (CONFIG_YAML_ENV, yaml),
            (DATABASE_URL_ENV, "postgres://pooled/db"),
            (DATABASE_URL_UNPOOLED_ENV, "postgres://direct/db"),
        ]))
        .unwrap();
        let Backend::Postgres(pg) = &config.backend else {
            panic!("expected the postgres backend, got {:?}", config.backend);
        };
        assert_eq!(
            pg.connection_string().as_deref(),
            Some("postgres://pooled/db")
        );
        assert_eq!(
            pg.migration_connection_string().as_deref(),
            Some("postgres://direct/db")
        );

        // Without an unpooled URL, migrations use the request connection.
        let config =
            Config::from_vars(vars(&[(DATABASE_URL_ENV, "postgres://pooled/db")])).unwrap();
        let Backend::Postgres(pg) = &config.backend else {
            panic!("expected the postgres backend");
        };
        assert_eq!(
            pg.migration_connection_string().as_deref(),
            Some("postgres://pooled/db")
        );
    }

    #[test]
    fn test_postgres_url_or_fields() {
        let fields: PostgresBackendConfig = serde_yml::from_str(
            "host: db\nport: '5432'\nuser: uc\npassword: secret\ndatabase: catalog\n",
        )
        .unwrap();
        assert_eq!(
            fields.connection_string().as_deref(),
            // Assembled so no credential-bearing URL literal sits in the source.
            Some(format!("postgres://{}:{}@db:5432/catalog", "uc", "secret").as_str())
        );
        assert_eq!(
            fields.migration_connection_string(),
            fields.connection_string()
        );

        let url: PostgresBackendConfig =
            serde_yml::from_str("url: postgres://a/b\nhost: ignored\n").unwrap();
        assert_eq!(url.connection_string().as_deref(), Some("postgres://a/b"));

        let incomplete: PostgresBackendConfig = serde_yml::from_str("host: db\n").unwrap();
        assert_eq!(incomplete.connection_string(), None);
    }

    #[test]
    fn test_bearer_tokens_resolve_or_fail() {
        let auth: AuthConfig = serde_yml::from_str(
            "mode: bearer\ntokens:\n  - principal: alice\n    token: secret-a\n",
        )
        .unwrap();
        assert_eq!(
            auth.bearer_tokens().unwrap(),
            vec![("secret-a".to_string(), "alice".to_string())]
        );

        let none = AuthConfig {
            mode: AuthMode::Bearer,
            ..Default::default()
        };
        assert!(none.bearer_tokens().unwrap_err().contains("auth.tokens"));

        let unresolved: AuthConfig = serde_yml::from_str(
            "mode: bearer\ntokens:\n  - principal: bob\n    token: { env: UC_TEST_TOKEN_THAT_IS_NEVER_SET }\n",
        )
        .unwrap();
        assert!(unresolved.bearer_tokens().unwrap_err().contains("`bob`"));

        let empty: AuthConfig =
            serde_yml::from_str("mode: bearer\ntokens:\n  - principal: carol\n    token: ''\n")
                .unwrap();
        assert!(empty.bearer_tokens().unwrap_err().contains("`carol`"));
    }
}
