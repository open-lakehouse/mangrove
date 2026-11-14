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

    /// Who decides what each caller may do. Defaults to
    /// [`PolicyConfig::Constant`] (allow everything).
    #[serde(default)]
    pub policy: PolicyConfig,

    /// Credential vending settings (see [`VendingConfig`]).
    #[serde(default)]
    pub vending: VendingConfig,
}

/// Credential vending settings.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct VendingConfig {
    /// Trusted vending for Neon object storage (see [`NeonVendingConfig`]).
    #[serde(default)]
    pub neon: NeonVendingConfig,
}

/// Trusted vending for Neon object storage.
///
/// Neon storage has no STS and no prefix-scoped keys. When enabled, a vend
/// against a Neon credential (an `aws_iam_role` whose `role_arn` is
/// `arn:neon:storage::<project_id>:branch/<branch_id>`) mints an expiring key
/// that reaches **every bucket on the branch** (and its descendant branches) at
/// the granted access level, until it expires.
///
/// ```yaml
/// vending:
///   neon:
///     enabled: true
///     project-id: falling-sun-12345678
///     # api-key: { env: NEON_API_KEY }      # the default
///     # branch-id: { env: NEON_BRANCH_ID }  # the default
///     ttl-secs: 900
/// ```
///
/// - **The TTL and `enabled` are the controls.** Neon enforces a key's expiry to
///   the second, but revocation is eventually consistent (a revoked key was seen
///   accepted again ~13 s later, and rejected for good only after ~20 s), so
///   revoking a key does not stop it promptly. Keep the TTL short; to stop
///   vending, set `enabled: false`.
/// - Needs the `neon-trusted-vending` feature.
/// - UC REST credentials carry no endpoint: clients set `AWS_ENDPOINT_URL_S3` to
///   the branch's storage endpoint. Delta API credentials carry `s3.endpoint`,
///   `s3.region` and `s3.path-style-access`.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct NeonVendingConfig {
    /// Master switch. Default `false`: Neon credentials register but don't vend.
    #[serde(default)]
    pub enabled: bool,

    /// The Neon API key minting the keys, never stored with a credential.
    /// Default `{ env: NEON_API_KEY }`.
    #[serde(default)]
    pub api_key: Option<ConfigValue>,

    /// The lifetime of a minted key, in seconds: 900 (15 min) to 43200 (12 h).
    /// Default 3600.
    #[serde(default)]
    pub ttl_secs: Option<u64>,

    /// The Neon project registered credentials default to. Required when
    /// enabled: Neon Functions don't get `NEON_PROJECT_ID`.
    #[serde(default)]
    pub project_id: Option<String>,

    /// The branch registered credentials default to. Default
    /// `{ env: NEON_BRANCH_ID }`, which Neon Functions get for their branch.
    #[serde(default)]
    pub branch_id: Option<ConfigValue>,

    /// The Neon API base URL. Default `https://console.neon.tech/api/v2/`.
    #[serde(default)]
    pub api_url: Option<String>,
}

/// The environment variable [`NeonVendingConfig::api_key`] defaults to.
pub const NEON_API_KEY_ENV: &str = "NEON_API_KEY";
/// The environment variable [`NeonVendingConfig::branch_id`] defaults to.
pub const NEON_BRANCH_ID_ENV: &str = "NEON_BRANCH_ID";
/// The shortest key lifetime [`NeonVendingConfig::ttl_secs`] accepts.
pub const NEON_MIN_TTL_SECS: u64 = 15 * 60;
/// The longest key lifetime [`NeonVendingConfig::ttl_secs`] accepts.
pub const NEON_MAX_TTL_SECS: u64 = 12 * 60 * 60;

impl NeonVendingConfig {
    fn resolve_or(value: &Option<ConfigValue>, env: &str) -> Option<String> {
        match value {
            Some(value) => value.value(),
            None => std::env::var(env).ok(),
        }
        .filter(|v| !v.is_empty())
    }

    /// The vending settings this config asks for, or why it can't be served.
    ///
    /// Fails when enabled in a build without the `neon-trusted-vending` feature,
    /// or without a project, an API key, a usable `api-url`, or with a TTL out of
    /// range.
    pub fn resolve(&self) -> Result<crate::services::NeonVending, String> {
        let defaults = crate::services::NeonVending::disabled(
            self.project_id.clone(),
            Self::resolve_or(&self.branch_id, NEON_BRANCH_ID_ENV),
        );
        if !self.enabled {
            return Ok(defaults);
        }
        self.enable(defaults)
    }

    #[cfg(not(feature = "neon-trusted-vending"))]
    fn enable(
        &self,
        _defaults: crate::services::NeonVending,
    ) -> Result<crate::services::NeonVending, String> {
        Err(
            "`vending.neon.enabled` needs the `neon-trusted-vending` feature, \
             which this build was compiled without"
                .into(),
        )
    }

    #[cfg(feature = "neon-trusted-vending")]
    fn enable(
        &self,
        defaults: crate::services::NeonVending,
    ) -> Result<crate::services::NeonVending, String> {
        use olai_credential_vending::neon::NeonMinterOptions;

        if defaults.project_id().is_none() {
            return Err("`vending.neon.enabled` needs `vending.neon.project-id` \
                 (Neon Functions don't get `NEON_PROJECT_ID`)"
                .into());
        }
        let api_key = Self::resolve_or(&self.api_key, NEON_API_KEY_ENV).ok_or(
            "`vending.neon.api-key` could not be resolved (default `{ env: NEON_API_KEY }`)",
        )?;
        let ttl = self.ttl_secs.unwrap_or(3600);
        if !(NEON_MIN_TTL_SECS..=NEON_MAX_TTL_SECS).contains(&ttl) {
            return Err(format!(
                "`vending.neon.ttl-secs` is {ttl}; it must be {NEON_MIN_TTL_SECS} to {NEON_MAX_TTL_SECS}"
            ));
        }
        let mut options =
            NeonMinterOptions::default().with_default_ttl(std::time::Duration::from_secs(ttl));
        if let Some(url) = &self.api_url {
            let url =
                url::Url::parse(url).map_err(|e| format!("`vending.neon.api-url` `{url}`: {e}"))?;
            options = options.with_api_url(url);
        }
        Ok(defaults.enabled(api_key, options))
    }
}

/// The authorization policy every request is checked against.
///
/// ```yaml
/// policy:
///   kind: authorizer
///   url: https://authz.example.com
///   token: { env: UC_PDP_TOKEN }
/// ```
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum PolicyConfig {
    /// Allow everything. The default, for local development and deployments
    /// guarded by something else.
    #[default]
    Constant,
    /// Ask a Cedar policy decision point over HTTPS (olai-auth's PDP wire API).
    /// Needs `auth.mode: olai-auth` and the `olai-auth` feature. Denies whatever
    /// it can't decide.
    Authorizer(AuthorizerConfig),
}

/// The decision point of [`PolicyConfig::Authorizer`].
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AuthorizerConfig {
    /// The base URL of the decision point; requests go to `{url}/v1/authorize`.
    pub url: ConfigValue,
    /// The server's bearer secret for the decision point.
    pub token: ConfigValue,
    /// How long a decision is reused, in seconds. Default 5; 0 disables the
    /// cache.
    #[serde(default)]
    pub cache_ttl_secs: Option<u64>,
    /// How long to wait for a decision, in milliseconds, before denying.
    /// Default 2000.
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

/// The decision cache TTL of [`AuthorizerConfig`] when none is configured.
pub const DEFAULT_DECISION_CACHE_TTL_SECS: u64 = 5;

impl AuthorizerConfig {
    /// The decision point's URL and token, resolved.
    pub fn endpoint(&self) -> Result<(String, String), String> {
        let resolve = |what: &str, v: &ConfigValue| {
            v.value()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| format!("`policy.{what}` could not be resolved"))
        };
        Ok((resolve("url", &self.url)?, resolve("token", &self.token)?))
    }
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
            policy: PolicyConfig::default(),
            vending: VendingConfig::default(),
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

    /// The verifier settings of [`AuthMode::OlaiAuth`]. Required in that mode;
    /// ignored otherwise.
    #[serde(default)]
    pub olai: Option<OlaiAuthConfig>,
}

/// The settings of [`AuthMode::OlaiAuth`].
///
/// Callers present `Authorization: Bearer` with either an EdDSA JWT from the
/// configured issuer (Neon Auth) or an `olk_` API key of a service account. Users'
/// admin and ban status come from `neon_auth."user"`, never from token claims.
///
/// ```yaml
/// auth:
///   mode: olai-auth
///   olai:
///     issuer: https://ep-example.neonauth.us-east-2.aws.neon.tech/neondb/auth
///     jwks-url: https://ep-example.neonauth.us-east-2.aws.neon.tech/neondb/auth/.well-known/jwks.json
///     audiences: [https://unitycatalog.example.com]
///     database-url: { env: AUTH_DATABASE_URL }
/// ```
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct OlaiAuthConfig {
    /// The exact `iss` of accepted tokens, e.g. the Neon Auth URL
    /// (`…/neondb/auth`) for OAuth access tokens.
    pub issuer: ConfigValue,

    /// Where the issuer publishes its signing keys (JWKS).
    pub jwks_url: ConfigValue,

    /// Accepted `aud` values: a token is accepted when it names any of them. At
    /// least one is required.
    pub audiences: Vec<ConfigValue>,

    /// Also accept Neon Auth session tokens (the web app's): `iss` and `aud` are
    /// both the origin of [`issuer`](Self::issuer). Default `false`.
    #[serde(default)]
    pub accept_session_tokens: bool,

    /// Accept `olk_` API keys of service accounts. Default `true`.
    #[serde(default = "default_true")]
    pub api_keys: bool,

    /// The schema of the `service_accounts` and `api_keys` tables. Default
    /// `authz`.
    #[serde(default)]
    pub authz_schema: Option<String>,

    /// The schema of Neon Auth's `user` table. Default `neon_auth`.
    #[serde(default)]
    pub users_schema: Option<String>,

    /// Postgres holding the identity tables (`authz.*`, `neon_auth."user"`).
    /// Defaults to the Postgres storage backend's URL; required with any other
    /// backend. On Neon the catalog lives in its own database while the identity
    /// tables are in the branch's `neondb`, so this is usually set.
    #[serde(default)]
    pub database_url: Option<ConfigValue>,

    /// The URL of this server's OAuth protected-resource metadata (RFC 9728),
    /// advertised in the `WWW-Authenticate` challenge of a 401.
    #[serde(default)]
    pub resource_metadata_url: Option<String>,

    /// This server's resource identifier. When set, the server publishes
    /// `/.well-known/oauth-protected-resource` (outside authentication) naming it
    /// and [`issuer`](Self::issuer) as its authorization server.
    #[serde(default)]
    pub resource: Option<String>,
}

fn default_true() -> bool {
    true
}

/// [`OlaiAuthConfig`] with every value resolved, as the authenticator is built from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOlaiAuth {
    pub issuer: String,
    pub jwks_url: String,
    pub audiences: Vec<String>,
    pub accept_session_tokens: bool,
    pub api_keys: bool,
    pub authz_schema: String,
    pub users_schema: String,
    pub database_url: String,
    pub resource_metadata_url: Option<String>,
    pub resource: Option<String>,
}

impl OlaiAuthConfig {
    /// Resolve every value, with the identity database defaulting to `backend`'s
    /// Postgres URL.
    ///
    /// Fails on an unresolved or empty issuer, JWKS URL or audience, no audience
    /// at all, or no identity database.
    pub fn resolve(&self, backend: &Backend) -> Result<ResolvedOlaiAuth, String> {
        let required = |what: &str, v: &ConfigValue| {
            v.value()
                .filter(|v| !v.is_empty())
                .ok_or_else(|| format!("`auth.olai.{what}` could not be resolved"))
        };
        let audiences = self
            .audiences
            .iter()
            .map(|a| required("audiences", a))
            .collect::<Result<Vec<_>, _>>()?;
        if audiences.is_empty() {
            return Err("`auth.olai.audiences` needs at least one audience".into());
        }
        let database_url = match (&self.database_url, backend) {
            (Some(url), _) => required("database-url", url)?,
            (None, Backend::Postgres(pg)) => pg.connection_string().ok_or(
                "`auth.olai.database-url` is unset and the Postgres backend has no connection string",
            )?,
            (None, _) => {
                return Err(
                    "`auth.olai.database-url` is required unless the storage backend is Postgres"
                        .into(),
                );
            }
        };
        Ok(ResolvedOlaiAuth {
            issuer: required("issuer", &self.issuer)?,
            jwks_url: required("jwks-url", &self.jwks_url)?,
            audiences,
            accept_session_tokens: self.accept_session_tokens,
            api_keys: self.api_keys,
            authz_schema: self.authz_schema.clone().unwrap_or_else(|| "authz".into()),
            users_schema: self
                .users_schema
                .clone()
                .unwrap_or_else(|| "neon_auth".into()),
            database_url,
            resource_metadata_url: self.resource_metadata_url.clone(),
            resource: self.resource.clone(),
        })
    }
}

impl AuthConfig {
    /// The resolved [`OlaiAuthConfig`] of [`AuthMode::OlaiAuth`].
    pub fn olai_auth(&self, backend: &Backend) -> Result<ResolvedOlaiAuth, String> {
        self.olai
            .as_ref()
            .ok_or("`auth.mode: olai-auth` needs an `auth.olai` block")?
            .resolve(backend)
    }
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
    /// Verify `Authorization: Bearer` with `olai-auth`: JWTs of the configured
    /// issuer and service-account API keys (see [`OlaiAuthConfig`]). The request
    /// acts as the verified caller. Needs the `olai-auth` feature.
    OlaiAuth,
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

    fn olai_auth_yaml(extra: &str) -> AuthConfig {
        serde_yml::from_str(&format!(
            "mode: olai-auth\nolai:\n  issuer: https://auth.example.com/neondb/auth\n  \
             jwks-url: https://auth.example.com/neondb/auth/.well-known/jwks.json\n  \
             audiences: [https://uc.example.com]\n{extra}"
        ))
        .unwrap()
    }

    #[test]
    fn policy_defaults_to_constant() {
        let config: Config = serde_yml::from_str("{}").unwrap();
        assert_eq!(config.policy, PolicyConfig::Constant);
        let config: Config = serde_yml::from_str("policy: { kind: constant }").unwrap();
        assert_eq!(config.policy, PolicyConfig::Constant);
    }

    #[test]
    fn policy_authorizer() {
        let config: Config = serde_yml::from_str(
            "policy:\n  kind: authorizer\n  url: https://authz.example.com\n  token: s3cr3t\n  cache-ttl-secs: 0\n",
        )
        .unwrap();
        let PolicyConfig::Authorizer(authorizer) = &config.policy else {
            panic!("{:?}", config.policy);
        };
        assert_eq!(
            authorizer.endpoint().unwrap(),
            (
                "https://authz.example.com".to_string(),
                "s3cr3t".to_string()
            )
        );
        assert_eq!(authorizer.cache_ttl_secs, Some(0));

        let config: Config =
            serde_yml::from_str("policy: { kind: authorizer, url: https://a, token: '' }").unwrap();
        let PolicyConfig::Authorizer(authorizer) = &config.policy else {
            panic!("{:?}", config.policy);
        };
        let err = authorizer.endpoint().unwrap_err();
        assert!(err.contains("policy.token"), "{err}");
    }

    fn neon(yaml: &str) -> NeonVendingConfig {
        let config: Config = serde_yml::from_str(&format!("vending:\n  neon:\n{yaml}")).unwrap();
        config.vending.neon
    }

    #[test]
    fn neon_vending_defaults_to_disabled() {
        let config = Config::default();
        assert!(!config.vending.neon.enabled);
        let vending = config.vending.neon.resolve().unwrap();
        assert!(!vending.is_enabled());
        assert_eq!(vending.project_id(), None);
    }

    #[test]
    fn disabled_neon_vending_still_carries_registration_defaults() {
        let vending = neon("    project-id: proj\n    branch-id: br-1\n")
            .resolve()
            .unwrap();
        assert!(!vending.is_enabled());
        assert_eq!(vending.project_id(), Some("proj"));
        assert_eq!(vending.branch_id(), Some("br-1"));
    }

    #[cfg(not(feature = "neon-trusted-vending"))]
    #[test]
    fn enabled_neon_vending_needs_the_feature() {
        let err = neon("    enabled: true\n    project-id: proj\n    api-key: k\n")
            .resolve()
            .unwrap_err();
        assert!(err.contains("neon-trusted-vending"), "{err}");
    }

    #[cfg(feature = "neon-trusted-vending")]
    #[test]
    fn enabled_neon_vending_resolves() {
        let config = neon(
            "    enabled: true\n    project-id: proj\n    branch-id: br-1\n    api-key: k\n    \
             ttl-secs: 900\n    api-url: http://127.0.0.1:1/\n",
        );
        assert_eq!(config.ttl_secs, Some(900));
        let vending = config.resolve().unwrap();
        assert!(vending.is_enabled());
        // The key never shows in debug output.
        let debug = format!("{vending:?}");
        assert!(!debug.contains("\"k\""), "{debug}");
    }

    #[cfg(feature = "neon-trusted-vending")]
    #[test]
    fn enabled_neon_vending_refuses_incomplete_config() {
        let cases = [
            (
                "    enabled: true\n    api-key: k\n",
                "vending.neon.project-id",
            ),
            (
                "    enabled: true\n    project-id: p\n    api-key: { env: UC_TEST_UNSET_NEON_KEY }\n",
                "vending.neon.api-key",
            ),
            (
                "    enabled: true\n    project-id: p\n    api-key: k\n    ttl-secs: 60\n",
                "vending.neon.ttl-secs",
            ),
            (
                "    enabled: true\n    project-id: p\n    api-key: k\n    ttl-secs: 86400\n",
                "vending.neon.ttl-secs",
            ),
            (
                "    enabled: true\n    project-id: p\n    api-key: k\n    api-url: 'not a url'\n",
                "vending.neon.api-url",
            ),
        ];
        for (yaml, expected) in cases {
            let err = neon(yaml).resolve().unwrap_err();
            assert!(err.contains(expected), "{yaml}: {err}");
        }
    }

    #[test]
    fn olai_auth_defaults_to_the_postgres_backend() {
        let auth = olai_auth_yaml("");
        assert_eq!(auth.mode, AuthMode::OlaiAuth);
        let backend = Backend::Postgres(PostgresBackendConfig {
            url: Some(ConfigValue::Value("postgres://db/unitycatalog".into())),
            ..Default::default()
        });
        let resolved = auth.olai_auth(&backend).unwrap();
        assert_eq!(resolved.database_url, "postgres://db/unitycatalog");
        assert_eq!(resolved.audiences, ["https://uc.example.com"]);
        assert!(resolved.api_keys);
        assert!(!resolved.accept_session_tokens);
        assert_eq!(resolved.authz_schema, "authz");
        assert_eq!(resolved.users_schema, "neon_auth");
    }

    #[test]
    fn olai_auth_takes_its_own_database() {
        let auth = olai_auth_yaml(
            "  database-url: postgres://db/neondb\n  api-keys: false\n  accept-session-tokens: true\n",
        );
        let resolved = auth.olai_auth(&Backend::default()).unwrap();
        assert_eq!(resolved.database_url, "postgres://db/neondb");
        assert!(!resolved.api_keys);
        assert!(resolved.accept_session_tokens);
    }

    #[test]
    fn olai_auth_refuses_incomplete_config() {
        let err = olai_auth_yaml("")
            .olai_auth(&Backend::default())
            .unwrap_err();
        assert!(err.contains("database-url"), "{err}");

        let mut auth = olai_auth_yaml("  database-url: postgres://db/x\n");
        auth.olai.as_mut().unwrap().audiences.clear();
        let err = auth.olai_auth(&Backend::default()).unwrap_err();
        assert!(err.contains("audiences"), "{err}");

        let auth: AuthConfig = serde_yml::from_str("mode: olai-auth").unwrap();
        let err = auth.olai_auth(&Backend::default()).unwrap_err();
        assert!(err.contains("auth.olai"), "{err}");
    }

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
