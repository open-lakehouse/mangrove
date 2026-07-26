//! YAML configuration for the standalone `storage-proxy` binary.
//!
//! A single optional config file, dotenv-style env indirection for secrets, and
//! sensible defaults so a config-less run works with a couple of CLI flags. The
//! shape is deliberately small — the proxy has no database, so it configures
//! only the bind address, the mount prefix, the upstream Unity Catalog to
//! resolve + vend through, and how incoming requests are authenticated.
//!
//! This mirrors the relevant subset of the `server` crate's config (the
//! `ConfigValue` env indirection, `StorageProxyClientConfig`, `AuthConfig`) so
//! operators see a familiar surface, without depending on that crate.

use serde::{Deserialize, Serialize};

use crate::auth::{AuthMode, DEFAULT_FORWARDED_USER_HEADER};

/// A `{ env: "VAR_NAME" }` reference resolved from the process environment.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct EnvValue {
    pub env: String,
}

/// A leaf configuration value: either an inline literal or an environment
/// variable reference. Keeps secrets (the upstream token) out of the file.
#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum ConfigValue {
    Value(String),
    Environment(EnvValue),
}

impl ConfigValue {
    /// Resolve the value: the inline string, or the named environment variable
    /// (`None` if unset).
    pub fn value(&self) -> Option<String> {
        match self {
            ConfigValue::Value(value) => Some(value.clone()),
            ConfigValue::Environment(env) => std::env::var(&env.env).ok(),
        }
    }
}

/// Default bind host when neither the config file nor a CLI flag sets one.
pub const DEFAULT_HOST: &str = "0.0.0.0";
/// Default listen port when neither the config file nor a CLI flag sets one.
pub const DEFAULT_PORT: u16 = 8080;
/// Default mount prefix for the byte-proxy surface — matches the path a UC
/// server announces at `/capabilities` (`storageProxy.basePath`).
pub const DEFAULT_BASE_PATH: &str = "/storage-proxy";

/// Standalone storage-proxy configuration.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    /// Host/interface to bind. Defaults to [`DEFAULT_HOST`] (all interfaces).
    #[serde(default)]
    pub host: Option<String>,

    /// TCP port to listen on. Defaults to [`DEFAULT_PORT`].
    #[serde(default)]
    pub port: Option<u16>,

    /// URL prefix the byte-proxy surface is mounted under. Defaults to
    /// [`DEFAULT_BASE_PATH`].
    #[serde(default)]
    pub base_path: Option<String>,

    /// Upstream Unity Catalog to resolve securables and vend credentials
    /// through. Required.
    pub upstream: UpstreamConfig,

    /// How incoming requests are authenticated. Defaults to anonymous.
    #[serde(default)]
    pub auth: AuthConfig,
}

/// Upstream Unity Catalog connection.
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct UpstreamConfig {
    /// Base URL of the UC REST API, e.g.
    /// `http://uc:8080/api/2.1/unity-catalog/`.
    pub base_url: String,

    /// Optional bearer token (inline or `{ env: ... }`). Absent = the upstream
    /// is contacted unauthenticated (anonymous UC).
    #[serde(default)]
    pub token: Option<ConfigValue>,
}

/// How the proxy authenticates incoming requests.
#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AuthConfig {
    /// The authentication mode. Defaults to [`AuthKind::Anonymous`].
    #[serde(default)]
    pub mode: AuthKind,

    /// Header the reverse-proxy mode reads the forwarded user from. Defaults to
    /// `x-forwarded-user`. Ignored unless `mode` is `reverse-proxy`.
    #[serde(default)]
    pub forwarded_user_header: Option<String>,

    /// In reverse-proxy mode, treat a request lacking the forwarded-user header
    /// as anonymous instead of rejecting it (`401`). Default `false` (reject).
    #[serde(default)]
    pub allow_missing_identity: bool,
}

/// The request-authentication strategy (config surface). Maps to the runtime
/// [`AuthMode`] via [`AuthConfig::to_mode`].
#[derive(Debug, Deserialize, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum AuthKind {
    /// Every request is anonymous. The default.
    #[default]
    Anonymous,
    /// Trust a forwarded-identity header set by a trusted upstream reverse proxy.
    ReverseProxy,
}

impl AuthConfig {
    /// The header name the forwarded end-user identity is read from (and, in
    /// turn, re-emitted under on upstream UC requests): the configured
    /// `forwarded-user-header`, else [`DEFAULT_FORWARDED_USER_HEADER`].
    ///
    /// Meaningful only in `reverse-proxy` mode; in `anonymous` mode no forwarded
    /// identity is ever present, so the value is unused.
    pub fn resolved_forwarded_header(&self) -> String {
        self.forwarded_user_header
            .clone()
            .unwrap_or_else(|| DEFAULT_FORWARDED_USER_HEADER.to_string())
    }

    /// Build the runtime [`AuthMode`] from this config.
    pub fn to_mode(&self) -> AuthMode {
        match self.mode {
            AuthKind::Anonymous => AuthMode::Anonymous,
            AuthKind::ReverseProxy => AuthMode::ReverseProxy {
                header: self
                    .forwarded_user_header
                    .clone()
                    .unwrap_or_else(|| DEFAULT_FORWARDED_USER_HEADER.to_string()),
                allow_missing: self.allow_missing_identity,
            },
        }
    }
}

impl Config {
    /// Load configuration from an optional YAML file path.
    ///
    /// `None` (or a path that does not exist) yields an error, because the
    /// upstream URL is required and has no default — a config-less run must
    /// supply it via CLI flags instead (see the serve command's
    /// `--upstream-url`). This is only called when a path is given.
    pub fn load(path: &str) -> Result<Self, String> {
        let p = std::path::Path::new(path);
        if !p.exists() {
            return Err(format!("config file not found at {}", p.display()));
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

    /// Resolved mount prefix: the configured value, else [`DEFAULT_BASE_PATH`].
    /// Normalized to a single leading slash with no trailing slash; an empty
    /// value means "mount at root".
    pub fn resolved_base_path(&self) -> String {
        let raw = self.base_path.as_deref().unwrap_or(DEFAULT_BASE_PATH);
        let trimmed = raw.trim().trim_matches('/');
        if trimmed.is_empty() {
            String::new()
        } else {
            format!("/{trimmed}")
        }
    }

    /// The `/health` URL a `healthcheck` probe should GET. A wildcard bind host
    /// maps to loopback, since that is not a connectable address for a client.
    pub fn health_url(&self) -> String {
        let host = match self.resolved_host() {
            "0.0.0.0" | "" | "::" => "127.0.0.1",
            other => other,
        };
        format!("http://{host}:{}/health", self.resolved_port())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_config_defaults() {
        let yaml = r#"
            upstream:
              base_url: "http://uc:8080/api/2.1/unity-catalog/"
        "#;
        let cfg: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(cfg.resolved_host(), "0.0.0.0");
        assert_eq!(cfg.resolved_port(), 8080);
        assert_eq!(cfg.resolved_base_path(), "/storage-proxy");
        assert!(cfg.upstream.token.is_none());
        assert_eq!(cfg.auth.mode, AuthKind::Anonymous);
        assert_eq!(cfg.auth.to_mode(), AuthMode::Anonymous);
    }

    #[test]
    fn base_path_normalization() {
        for raw in ["storage-proxy", "/storage-proxy", "/storage-proxy/"] {
            let cfg = Config {
                host: None,
                port: None,
                base_path: Some(raw.to_string()),
                upstream: UpstreamConfig {
                    base_url: "http://uc/".into(),
                    token: None,
                },
                auth: AuthConfig::default(),
            };
            assert_eq!(cfg.resolved_base_path(), "/storage-proxy", "input {raw:?}");
        }
        // Empty ⇒ root mount.
        let cfg = Config {
            host: None,
            port: None,
            base_path: Some(String::new()),
            upstream: UpstreamConfig {
                base_url: "http://uc/".into(),
                token: None,
            },
            auth: AuthConfig::default(),
        };
        assert_eq!(cfg.resolved_base_path(), "");
    }

    #[test]
    fn reverse_proxy_auth_roundtrips_and_maps() {
        let yaml = r#"
            upstream:
              base_url: "http://uc:8080/api/2.1/unity-catalog/"
              token:
                env: "UC_TOKEN"
            auth:
              mode: reverse-proxy
              forwarded-user-header: "x-user"
              allow-missing-identity: true
        "#;
        let cfg: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(
            cfg.upstream.token,
            Some(ConfigValue::Environment(EnvValue {
                env: "UC_TOKEN".into()
            }))
        );
        assert_eq!(
            cfg.auth.to_mode(),
            AuthMode::ReverseProxy {
                header: "x-user".into(),
                allow_missing: true
            }
        );

        // Round-trips through YAML.
        let reparsed: Config = serde_yml::from_str(&serde_yml::to_string(&cfg).unwrap()).unwrap();
        assert_eq!(reparsed, cfg);
    }

    #[test]
    fn reverse_proxy_defaults_to_standard_header() {
        let yaml = r#"
            upstream:
              base_url: "http://uc/"
            auth:
              mode: reverse-proxy
        "#;
        let cfg: Config = serde_yml::from_str(yaml).unwrap();
        assert_eq!(
            cfg.auth.to_mode(),
            AuthMode::ReverseProxy {
                header: DEFAULT_FORWARDED_USER_HEADER.into(),
                allow_missing: false
            }
        );
    }

    #[test]
    fn health_url_maps_wildcard_host_to_loopback() {
        let cfg = Config {
            host: Some("0.0.0.0".into()),
            port: Some(9000),
            base_path: None,
            upstream: UpstreamConfig {
                base_url: "http://uc/".into(),
                token: None,
            },
            auth: AuthConfig::default(),
        };
        assert_eq!(cfg.health_url(), "http://127.0.0.1:9000/health");
    }
}
