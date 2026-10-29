//! Server configuration: a YAML file, with secrets resolvable from the
//! environment and a few well-known environment overrides.
//!
//! ```yaml
//! public_url: https://sharing.example.com
//! database:
//!   type: postgres
//!   url: { env: DATABASE_URL }
//!   # The direct (unpooled) URL for migrations / the startup schema check.
//!   migration_url: { env: DATABASE_URL_UNPOOLED }
//! encryption:
//!   provider: local
//!   active: { id: v2, key: { env: SHARING_KEK_V2 } }
//!   retired: [{ id: v1, key: { env: SHARING_KEK_V1 } }]
//! provider_tokens: [{ env: SHARING_PROVIDER_TOKEN }]
//! sources:
//!   - name: uc
//!     type: unity
//!     url: https://uc.example.com/api/2.1/unity-catalog/
//!     token: { env: UC_TOKEN }
//!     catalogs: ["*"]
//!     access: delegated
//! ```
//!
//! Environment overrides: `DATABASE_URL` / `DATABASE_URL_UNPOOLED` set the
//! Postgres URLs, and `SHARING_PROVIDER_TOKEN` adds a provider token.

use std::sync::Arc;

use base64::Engine as _;
use olai_store::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use serde::{Deserialize, Serialize};
#[cfg(feature = "unity")]
use url::Url;

use crate::source::fixed::StaticSource;
use crate::source::{AccessMode, RegisteredSource, SourceRegistry};
#[cfg(feature = "unity")]
use crate::unity::source::UnitySource;

/// A configuration value given inline or read from an environment variable.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum ConfigValue {
    Value(String),
    Env { env: String },
}

impl ConfigValue {
    /// The resolved value, or `None` for an unset environment variable.
    pub fn value(&self) -> Option<String> {
        match self {
            ConfigValue::Value(value) => Some(value.clone()),
            ConfigValue::Env { env } => std::env::var(env).ok(),
        }
    }

    fn require(&self, what: &str) -> Result<String, String> {
        self.value().ok_or_else(|| match self {
            ConfigValue::Env { env } => format!("{what}: environment variable {env} is not set"),
            ConfigValue::Value(_) => format!("{what} is not set"),
        })
    }
}

/// The server configuration. See the module docs.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// The bind host (default `0.0.0.0`).
    #[serde(default)]
    pub host: Option<String>,
    /// The listen port (default `8080`).
    #[serde(default)]
    pub port: Option<u16>,
    /// The externally reachable base URL, used in recipient profiles and
    /// activation links (default `http://localhost:<port>`).
    #[serde(default)]
    pub public_url: Option<String>,
    #[serde(default)]
    pub database: DatabaseConfig,
    /// Secrets-at-rest configuration. Required outside of the in-memory store.
    #[serde(default)]
    pub encryption: Option<EncryptionConfig>,
    /// Bearer tokens accepted on the provider admin API.
    #[serde(default)]
    pub provider_tokens: Vec<ConfigValue>,
    /// The catalogs shared assets are resolved from.
    #[serde(default)]
    pub sources: Vec<SourceConfig>,
    /// How long presigned file URLs stay valid, at most (seconds; default 3600).
    #[serde(default = "default_presign_ttl")]
    pub presign_ttl_secs: u64,
    /// The S3 region used when presigning (else `AWS_REGION`).
    #[serde(default)]
    pub aws_region: Option<String>,
}

fn default_presign_ttl() -> u64 {
    3600
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: None,
            port: None,
            public_url: None,
            database: DatabaseConfig::default(),
            encryption: None,
            provider_tokens: Vec::new(),
            sources: Vec::new(),
            presign_ttl_secs: default_presign_ttl(),
            aws_region: None,
        }
    }
}

/// The share store backend.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum DatabaseConfig {
    /// In-memory: nothing survives a restart.
    #[default]
    Memory,
    /// Postgres (e.g. Neon).
    Postgres {
        /// The connection URL used for request traffic (may be pooled).
        url: ConfigValue,
        /// The direct, unpooled URL for migrations and the startup schema check
        /// (advisory locks need a session; defaults to `url`).
        #[serde(default)]
        migration_url: Option<ConfigValue>,
        #[serde(default = "default_max_connections")]
        max_connections: u32,
    },
}

fn default_max_connections() -> u32 {
    10
}

/// Envelope encryption for secrets at rest.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct EncryptionConfig {
    /// Where key-encryption keys live. Only `local` (KEKs in this config)
    /// exists so far; an external key store (cloud KMS) plugs in here.
    #[serde(default)]
    pub provider: KeyProviderKind,
    /// The KEK new secrets are sealed under.
    pub active: KeyConfig,
    /// Retired KEKs, kept so older secrets can still be opened (and are
    /// re-wrapped under `active` when read or by `rotate-keys`).
    #[serde(default)]
    pub retired: Vec<KeyConfig>,
}

/// The source of key-encryption keys.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KeyProviderKind {
    /// KEKs are supplied in this configuration (inline or from the environment).
    #[default]
    Local,
}

/// A key-encryption key: a stable id and 32 bytes of key material (base64).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct KeyConfig {
    pub id: String,
    pub key: ConfigValue,
}

impl EncryptionConfig {
    /// A fixed, publicly known KEK for development and tests only.
    pub fn insecure_dev() -> Self {
        Self {
            provider: KeyProviderKind::Local,
            active: KeyConfig {
                id: "dev".to_string(),
                key: ConfigValue::Value("AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=".to_string()),
            },
            retired: Vec::new(),
        }
    }

    /// Build the encryptor these keys describe.
    pub fn build(&self) -> Result<EnvelopeEncryptor, String> {
        match self.provider {
            KeyProviderKind::Local => {
                let mut keys = Vec::new();
                for key in std::iter::once(&self.active).chain(&self.retired) {
                    let encoded = key.key.require(&format!("KEK '{}'", key.id))?;
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
    }
}

/// A catalog shared assets are resolved from.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct SourceConfig {
    pub name: String,
    /// The catalog names this source serves; `*` (the default) serves all.
    #[serde(default = "all_catalogs")]
    pub catalogs: Vec<String>,
    /// Delegated (catalog-vended) or local (own credentials) storage access.
    #[serde(default)]
    pub access: Option<AccessMode>,
    #[serde(flatten)]
    pub kind: SourceKind,
}

fn all_catalogs() -> Vec<String> {
    vec!["*".to_string()]
}

/// The kind of a catalog source.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SourceKind {
    /// A Unity Catalog REST server. Defaults to delegated access. Needs the
    /// `unity` feature.
    #[cfg(feature = "unity")]
    Unity {
        /// The UC REST base URL (`…/api/2.1/unity-catalog/`).
        url: String,
        #[serde(default)]
        token: Option<ConfigValue>,
    },
    /// Assets declared in this configuration. Always local access.
    Static(StaticSource),
}

impl Config {
    /// Load the configuration from `path`, or the defaults when `None`.
    pub fn load(path: Option<&str>) -> Result<Self, String> {
        let mut config: Config = match path {
            None => Config::default(),
            Some(path) => {
                let contents = std::fs::read_to_string(path)
                    .map_err(|e| format!("reading config `{path}`: {e}"))?;
                serde_yml::from_str(&contents)
                    .map_err(|e| format!("parsing config `{path}`: {e}"))?
            }
        };
        config.apply_env();
        Ok(config)
    }

    /// Load the configuration from the environment alone, for hosts without a
    /// config file (serverless functions): the YAML document in
    /// `SHARING_SERVER_CONFIG_YAML`, else the file at `SHARING_SERVER_CONFIG`,
    /// else the defaults — then the usual environment overrides.
    pub fn from_env() -> Result<Self, String> {
        if let Ok(yaml) = std::env::var("SHARING_SERVER_CONFIG_YAML") {
            let mut config: Config = serde_yml::from_str(&yaml)
                .map_err(|e| format!("parsing SHARING_SERVER_CONFIG_YAML: {e}"))?;
            config.apply_env();
            return Ok(config);
        }
        Self::load(std::env::var("SHARING_SERVER_CONFIG").ok().as_deref())
    }

    fn apply_env(&mut self) {
        if let Ok(url) = std::env::var("DATABASE_URL") {
            let migration_url = std::env::var("DATABASE_URL_UNPOOLED")
                .ok()
                .map(ConfigValue::Value);
            let max_connections = match &self.database {
                DatabaseConfig::Postgres {
                    max_connections, ..
                } => *max_connections,
                DatabaseConfig::Memory => default_max_connections(),
            };
            self.database = DatabaseConfig::Postgres {
                url: ConfigValue::Value(url),
                migration_url,
                max_connections,
            };
        }
        if std::env::var("SHARING_PROVIDER_TOKEN").is_ok() {
            self.provider_tokens.push(ConfigValue::Env {
                env: "SHARING_PROVIDER_TOKEN".to_string(),
            });
        }
    }

    pub fn host(&self) -> &str {
        self.host.as_deref().unwrap_or("0.0.0.0")
    }

    pub fn port(&self) -> u16 {
        self.port.unwrap_or(8080)
    }

    /// The externally reachable base URL, without a trailing slash.
    pub fn public_url(&self) -> String {
        self.public_url
            .clone()
            .unwrap_or_else(|| format!("http://localhost:{}", self.port()))
            .trim_end_matches('/')
            .to_string()
    }

    /// The resolved provider tokens.
    pub fn provider_tokens(&self) -> Result<Vec<String>, String> {
        self.provider_tokens
            .iter()
            .map(|t| t.require("provider token"))
            .collect()
    }

    /// Build the catalog source registry.
    pub fn sources(&self) -> Result<SourceRegistry, String> {
        let mut sources = Vec::new();
        for source in &self.sources {
            let (implementation, default_access): (Arc<dyn crate::source::CatalogSource>, _) =
                match &source.kind {
                    #[cfg(feature = "unity")]
                    SourceKind::Unity { url, token } => {
                        let mut url = Url::parse(url)
                            .map_err(|e| format!("source '{}': invalid url: {e}", source.name))?;
                        if !url.path().ends_with('/') {
                            url.set_path(&format!("{}/", url.path()));
                        }
                        let token = token
                            .as_ref()
                            .map(|t| t.require(&format!("source '{}' token", source.name)))
                            .transpose()?;
                        (
                            Arc::new(UnitySource::new(url, token)),
                            AccessMode::Delegated,
                        )
                    }
                    SourceKind::Static(fixed) => (Arc::new(fixed.clone()), AccessMode::Local),
                };
            let access = source.access.unwrap_or(default_access);
            if matches!(source.kind, SourceKind::Static(_)) && access == AccessMode::Delegated {
                return Err(format!(
                    "source '{}': a static source cannot vend credentials; use `access: local`",
                    source.name
                ));
            }
            sources.push(RegisteredSource {
                name: source.name.clone(),
                catalogs: source.catalogs.clone(),
                access,
                source: implementation,
            });
        }
        Ok(SourceRegistry::new(sources))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "unity")]
    #[test]
    fn parses_a_full_config() {
        let config: Config = serde_yml::from_str(
            r#"
public_url: https://sharing.example.com/
database:
  type: postgres
  url: { env: SOME_DB_URL }
encryption:
  active: { id: v2, key: "AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=" }
  retired: [{ id: v1, key: { env: OLD_KEK } }]
provider_tokens: ["provider-secret"]
sources:
  - name: uc
    type: unity
    url: https://uc.example.com/api/2.1/unity-catalog
  - name: local
    type: static
    catalogs: [demo]
    tables:
      demo.sales.orders: { location: "s3://bucket/orders" }
"#,
        )
        .unwrap();
        assert_eq!(config.public_url(), "https://sharing.example.com");
        assert!(matches!(config.database, DatabaseConfig::Postgres { .. }));
        assert_eq!(config.provider_tokens().unwrap(), ["provider-secret"]);
        assert_eq!(config.sources.len(), 2);
        let registry = config.sources().unwrap();
        let demo = registry.for_name("demo.sales.orders").unwrap();
        // `uc` serves every catalog and is listed first.
        assert_eq!(demo.name, "uc");
        assert_eq!(demo.access, AccessMode::Delegated);
    }

    #[test]
    fn rejects_delegated_static_sources() {
        let config: Config = serde_yml::from_str(
            r#"
sources:
  - name: local
    type: static
    access: delegated
"#,
        )
        .unwrap();
        assert!(config.sources().is_err());
    }

    #[test]
    fn builds_the_encryptor() {
        assert!(EncryptionConfig::insecure_dev().build().is_ok());
    }
}
