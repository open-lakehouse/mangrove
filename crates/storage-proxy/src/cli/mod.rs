//! CLI surface for the standalone `storage-proxy` binary.
//!
//! Two subcommands, mirroring the deployable half of the `server` crate's CLI
//! (the proxy has no database, so there is no `migrate`):
//!   - `serve` — run the byte-proxy (see [`serve::serve`]). Flags overlay the
//!     config file, CLI flags taking precedence. A config file is optional when
//!     the required upstream URL is supplied via `--upstream-url`.
//!   - `healthcheck` — probe the configured `/health` endpoint and exit 0
//!     (healthy) or non-zero (unhealthy). This is the probe the distroless
//!     image's Docker `HEALTHCHECK` runs, since distroless has no shell/`curl`.

pub mod config;
pub mod serve;

use std::time::Duration;

use clap::{Args, Parser, Subcommand};

use crate::cli::config::{AuthConfig, Config, DEFAULT_HOST, DEFAULT_PORT, UpstreamConfig};

/// `storage-proxy` — standalone Unity Catalog storage byte-proxy.
#[derive(Debug, Parser)]
#[command(name = "storage-proxy", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the byte-proxy.
    Serve(ServeArgs),
    /// Probe the configured `/health` endpoint; exit 0 if healthy, non-zero otherwise.
    Healthcheck(HealthcheckArgs),
}

/// Arguments for `serve`. Every flag is optional and, when present, overlays the
/// value loaded from the config file (highest precedence). With no config file,
/// `--upstream-url` is required.
#[derive(Debug, Default, Clone, Args)]
pub struct ServeArgs {
    /// Config file path (YAML). Also read from `STORAGE_PROXY_CONFIG`.
    #[arg(short, long, env = "STORAGE_PROXY_CONFIG", value_name = "PATH")]
    pub config: Option<String>,

    /// Host/interface to bind. Overrides config; default 0.0.0.0 (all interfaces).
    #[arg(long)]
    pub host: Option<String>,

    /// TCP port to listen on. Overrides config; default 8080.
    #[arg(long, short)]
    pub port: Option<u16>,

    /// Upstream Unity Catalog base URL to resolve + vend through, e.g.
    /// `http://uc:8080/api/2.1/unity-catalog/`. Overrides config; required when
    /// no config file is supplied.
    #[arg(long, value_name = "URL")]
    pub upstream_url: Option<String>,

    /// Bearer token for the upstream UC. Overrides config. Absent = contact the
    /// upstream unauthenticated. Prefer a config-file `{ env: ... }` reference in
    /// production so the token is not visible in the process arguments.
    #[arg(long, value_name = "TOKEN")]
    pub upstream_token: Option<String>,
}

impl ServeArgs {
    /// Resolve the effective [`Config`]: load the file when given, else start
    /// from the CLI-supplied upstream, then overlay the remaining flags.
    pub fn resolve_config(&self) -> Result<Config, String> {
        let mut cfg = match &self.config {
            Some(path) => Config::load(path)?,
            None => {
                let base_url = self.upstream_url.clone().ok_or_else(|| {
                    "no config file and no --upstream-url: the upstream Unity Catalog URL is \
                     required"
                        .to_string()
                })?;
                Config {
                    host: None,
                    port: None,
                    base_path: None,
                    upstream: UpstreamConfig {
                        base_url,
                        token: None,
                    },
                    auth: AuthConfig::default(),
                }
            }
        };
        self.overlay(&mut cfg);
        Ok(cfg)
    }

    /// Overlay the provided flags onto `cfg` (highest precedence).
    fn overlay(&self, cfg: &mut Config) {
        if let Some(host) = &self.host {
            cfg.host = Some(host.clone());
        }
        if let Some(port) = self.port {
            cfg.port = Some(port);
        }
        if let Some(url) = &self.upstream_url {
            cfg.upstream.base_url = url.clone();
        }
        if let Some(token) = &self.upstream_token {
            cfg.upstream.token = Some(config::ConfigValue::Value(token.clone()));
        }
    }
}

/// Arguments for `healthcheck`. Shares the config/host/port resolution inputs so
/// the probe targets the same address the server binds.
#[derive(Debug, Default, Clone, Args)]
pub struct HealthcheckArgs {
    /// Config file path used to resolve the probe target. Also
    /// `STORAGE_PROXY_CONFIG`.
    #[arg(short, long, env = "STORAGE_PROXY_CONFIG", value_name = "PATH")]
    pub config: Option<String>,

    /// Host to connect to (overrides config). A wildcard bind host maps to loopback.
    #[arg(long)]
    pub host: Option<String>,

    /// Port to connect to (overrides config).
    #[arg(long, short)]
    pub port: Option<u16>,

    /// Probe the full health URL directly, bypassing config load entirely. Use
    /// when no config file is available to the probe process (e.g. the Docker
    /// `HEALTHCHECK`, which relies on the default host/port).
    #[arg(long, value_name = "URL")]
    pub url: Option<String>,

    /// Probe timeout in seconds.
    #[arg(long, default_value_t = 3)]
    pub timeout_secs: u64,
}

impl HealthcheckArgs {
    /// The URL to GET: `--url` verbatim if given, else assembled from the loaded
    /// config (or defaults) with `--host`/`--port` overlaid.
    fn target_url(&self) -> Result<String, String> {
        if let Some(url) = &self.url {
            return Ok(url.clone());
        }
        // Without a config file the probe targets the default bind address —
        // which is exactly what the container's HEALTHCHECK relies on.
        let host = self
            .host
            .clone()
            .or_else(|| {
                self.config
                    .as_ref()
                    .and_then(|p| Config::load(p).ok())
                    .and_then(|c| c.host.clone())
            })
            .unwrap_or_else(|| DEFAULT_HOST.to_string());
        let host = match host.as_str() {
            "0.0.0.0" | "" | "::" => "127.0.0.1".to_string(),
            _ => host,
        };
        let port = self
            .port
            .or_else(|| {
                self.config
                    .as_ref()
                    .and_then(|p| Config::load(p).ok())
                    .and_then(|c| c.port)
            })
            .unwrap_or(DEFAULT_PORT);
        Ok(format!("http://{host}:{port}/health"))
    }
}

/// Run the health probe. `Ok(())` iff the endpoint returns a 2xx with body `OK`;
/// the caller maps any `Err` to a non-zero exit.
pub fn run_healthcheck(args: &HealthcheckArgs) -> Result<(), String> {
    let url = args.target_url()?;
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(args.timeout_secs))
        .build()
        .map_err(|e| e.to_string())?;
    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("health endpoint returned {status}"));
    }
    let body = resp.text().map_err(|e| e.to_string())?;
    if body.trim() != "OK" {
        return Err(format!("unexpected health body {body:?}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_arg_tree_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn serve_parses_with_upstream_url() {
        let cli = Cli::try_parse_from([
            "storage-proxy",
            "serve",
            "--upstream-url",
            "http://uc/",
            "--port",
            "9000",
        ])
        .unwrap();
        let Command::Serve(args) = cli.command else {
            panic!("expected serve");
        };
        assert_eq!(args.upstream_url.as_deref(), Some("http://uc/"));
        assert_eq!(args.port, Some(9000));
    }

    #[test]
    fn serve_config_requires_upstream_without_file() {
        let args = ServeArgs::default();
        assert!(args.resolve_config().is_err());
    }

    #[test]
    fn serve_flags_overlay_and_build_config() {
        let args = ServeArgs {
            upstream_url: Some("http://uc/".into()),
            upstream_token: Some("secret".into()),
            host: Some("127.0.0.1".into()),
            port: Some(9000),
            ..ServeArgs::default()
        };
        let cfg = args.resolve_config().unwrap();
        assert_eq!(cfg.upstream.base_url, "http://uc/");
        assert_eq!(
            cfg.upstream.token.as_ref().unwrap().value().as_deref(),
            Some("secret")
        );
        assert_eq!(cfg.resolved_host(), "127.0.0.1");
        assert_eq!(cfg.resolved_port(), 9000);
    }

    #[test]
    fn healthcheck_parses() {
        let cli = Cli::try_parse_from(["storage-proxy", "healthcheck", "--port", "9000"]).unwrap();
        let Command::Healthcheck(args) = cli.command else {
            panic!("expected healthcheck");
        };
        assert_eq!(args.port, Some(9000));
        assert_eq!(args.timeout_secs, 3);
    }

    #[test]
    fn healthcheck_url_overrides_config() {
        let args = HealthcheckArgs {
            url: Some("http://example.test:1234/health".into()),
            ..HealthcheckArgs::default()
        };
        assert_eq!(
            args.target_url().unwrap(),
            "http://example.test:1234/health"
        );
    }

    #[test]
    fn healthcheck_defaults_to_loopback_default_port() {
        let args = HealthcheckArgs::default();
        assert_eq!(args.target_url().unwrap(), "http://127.0.0.1:8080/health");
    }

    #[test]
    fn no_subcommand_is_an_error() {
        assert!(Cli::try_parse_from(["storage-proxy"]).is_err());
    }
}
