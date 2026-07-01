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

    /// Bundled web-UI serving settings (see [`UiConfig`]).
    #[serde(default)]
    pub ui: UiConfig,
}

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
            ui: UiConfig::default(),
        }
    }
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
    #[serde(default)]
    pub recipients: RoutingMode,
    #[serde(default)]
    pub shares: RoutingMode,
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

    /// Surfaces that are routed upstream but do not yet have a proxy adapter
    /// implemented. Setting any of these to [`RoutingMode::Upstream`] is a
    /// configuration error and the server should refuse to start.
    ///
    /// v1 implements adapters for catalogs, schemas, and tables only.
    pub fn unsupported_upstream(&self) -> Vec<&'static str> {
        [
            ("credentials", self.credentials),
            ("external-locations", self.external_locations),
            ("functions", self.functions),
            ("recipients", self.recipients),
            ("shares", self.shares),
        ]
        .into_iter()
        .filter(|(_, mode)| *mode == RoutingMode::Upstream)
        .map(|(name, _)| name)
        .collect()
    }

    fn surfaces(&self) -> [(&'static str, RoutingMode); 8] {
        [
            ("catalogs", self.catalogs),
            ("schemas", self.schemas),
            ("tables", self.tables),
            ("credentials", self.credentials),
            ("external-locations", self.external_locations),
            ("functions", self.functions),
            ("recipients", self.recipients),
            ("shares", self.shares),
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
#[derive(Debug, Deserialize, Serialize)]
pub struct PostgresBackendConfig {
    /// The host of the server.
    pub host: ConfigValue,

    /// The port of the server.
    pub port: ConfigValue,

    /// The database user.
    pub user: ConfigValue,

    /// Password for the user.
    pub password: ConfigValue,

    /// The database name.
    pub database: ConfigValue,
}

impl PostgresBackendConfig {
    /// Get the full connection string for the Postgres backend.
    pub fn connection_string(&self) -> Option<String> {
        let host = self.host.value()?;
        let port = self.port.value()?;
        let user = self.user.value()?;
        let password = self.password.value()?;
        let database = self.database.value()?;

        Some(format!(
            "postgres://{user}:{password}@{host}:{port}/{database}"
        ))
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
        assert_eq!(backend.host.value().unwrap(), "localhost");
        assert_eq!(backend.port.value().unwrap(), "5432");
        assert_eq!(backend.user.value().unwrap(), "user");
        assert_eq!(
            backend.password,
            ConfigValue::Environment(EnvValue {
                env: "PG_PASSWORD".to_string()
            })
        );
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
        assert!(routing.unsupported_upstream().is_empty());
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
        assert!(config.routing.unsupported_upstream().is_empty());

        // round-trips back to YAML and re-parses identically
        let serialized = serde_yml::to_string(&config).unwrap();
        let reparsed: Config = serde_yml::from_str(&serialized).unwrap();
        assert_eq!(reparsed.routing, config.routing);
        assert_eq!(reparsed.upstream, config.upstream);
    }

    #[test]
    fn test_unsupported_upstream_surfaces_detected() {
        let yaml = r#"
            routing:
              catalogs: upstream
              functions: upstream
              shares: upstream
        "#;
        let config: Config = serde_yml::from_str(yaml).unwrap();
        assert!(config.routing.any_upstream());
        assert_eq!(
            config.routing.unsupported_upstream(),
            vec!["functions", "shares"]
        );
    }
}
