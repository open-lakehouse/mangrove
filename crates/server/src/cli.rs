//! CLI surface for the `uc-server` binary.
//!
//! Three subcommands, one per operational concern:
//!   - `serve` — run the service (see [`crate::run::serve`]). Flags overlay the
//!     config file (`--host`/`--port`/`--config`, plus `--no-ui`), CLI flags
//!     taking precedence. `serve` does **not** apply migrations.
//!   - `migrate` — apply any pending database migrations, then exit (see
//!     [`crate::run::migrate`]). Run this once before/at deploy time; it is the
//!     only path that mutates the schema, keeping migrations off the `serve` hot
//!     path so concurrent instances don't race to migrate.
//!   - `healthcheck` — probe the configured `/health` endpoint and exit 0
//!     (healthy) or non-zero (unhealthy). This is the probe the distroless
//!     image's Docker `HEALTHCHECK` runs, since distroless has no shell/`curl`.

use std::time::Duration;

use clap::{Args, Parser, Subcommand};

use crate::config::Config;

/// `uc-server` — Unity Catalog REST/Delta-Sharing server.
#[derive(Debug, Parser)]
#[command(name = "uc-server", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Run the server.
    Serve(ServeArgs),
    /// Apply any pending database migrations, then exit.
    Migrate(MigrateArgs),
    /// Probe the configured `/health` endpoint; exit 0 if healthy, non-zero otherwise.
    Healthcheck(HealthcheckArgs),
}

/// Arguments for `migrate`. Migrations need only the backend DSN, which the
/// config file resolves; host, port, and UI settings are irrelevant here, so the
/// only flag is the config path.
#[derive(Debug, Default, Clone, Args)]
pub struct MigrateArgs {
    /// Config file path (YAML). Also read from `UC_SERVER_CONFIG`.
    #[arg(short, long, env = "UC_SERVER_CONFIG", value_name = "PATH")]
    pub config: Option<String>,
}

/// Arguments for `serve`. Every flag is optional and, when present, overlays the
/// value loaded from the config file (highest precedence).
#[derive(Debug, Default, Clone, Args)]
pub struct ServeArgs {
    /// Config file path (YAML). Also read from `UC_SERVER_CONFIG`.
    #[arg(short, long, env = "UC_SERVER_CONFIG", value_name = "PATH")]
    pub config: Option<String>,

    /// Host/interface to bind. Overrides config; default 0.0.0.0 (all interfaces).
    #[arg(long)]
    pub host: Option<String>,

    /// TCP port to listen on. Overrides config; default 8080.
    #[arg(long, short)]
    pub port: Option<u16>,

    /// Run API-only: do not serve the bundled web UI. Overrides config; equivalent
    /// to `ui.serve = false`. Use when embedding a custom UI built on the shipped
    /// components, or serving no UI at all.
    #[arg(long)]
    pub no_ui: bool,
}

impl ServeArgs {
    /// Overlay the provided flags onto a loaded [`Config`] (highest precedence).
    pub fn overlay(&self, cfg: &mut Config) {
        if let Some(host) = &self.host {
            cfg.host = Some(host.clone());
        }
        if let Some(port) = self.port {
            cfg.port = Some(port);
        }
        // A bare `--no-ui` opts out of serving the UI; absent, the config value
        // (default `true`) stands. The flag can only disable, never re-enable, so
        // its absence never clobbers a configured `serve = true`.
        if self.no_ui {
            cfg.ui.serve = false;
        }
    }
}

/// Arguments for `healthcheck`. Shares the config/host/port resolution inputs so
/// the probe targets the same address the server binds.
#[derive(Debug, Default, Clone, Args)]
pub struct HealthcheckArgs {
    /// Config file path used to resolve the probe target. Also `UC_SERVER_CONFIG`.
    #[arg(short, long, env = "UC_SERVER_CONFIG", value_name = "PATH")]
    pub config: Option<String>,

    /// Host to connect to (overrides config). A wildcard bind host maps to loopback.
    #[arg(long)]
    pub host: Option<String>,

    /// Port to connect to (overrides config).
    #[arg(long, short)]
    pub port: Option<u16>,

    /// Probe the full health URL directly, bypassing config load entirely. Use
    /// when no config file is available to the probe process.
    #[arg(long, value_name = "URL")]
    pub url: Option<String>,

    /// Probe timeout in seconds.
    #[arg(long, default_value_t = 3)]
    pub timeout_secs: u64,
}

impl HealthcheckArgs {
    /// The URL to GET: `--url` verbatim if given, else assembled from the loaded
    /// config with `--host`/`--port` overlaid.
    fn target_url(&self) -> Result<String, String> {
        if let Some(url) = &self.url {
            return Ok(url.clone());
        }
        let mut cfg = Config::load(self.config.as_ref())?;
        if let Some(host) = &self.host {
            cfg.host = Some(host.clone());
        }
        if let Some(port) = self.port {
            cfg.port = Some(port);
        }
        Ok(cfg.health_url())
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

    /// The derived arg tree must be internally consistent (no conflicting flags,
    /// duplicate names, etc.).
    #[test]
    fn cli_arg_tree_is_valid() {
        Cli::command().debug_assert();
    }

    #[test]
    fn serve_parses_with_no_flags() {
        let cli = Cli::try_parse_from(["uc-server", "serve"]).unwrap();
        let Command::Serve(args) = cli.command else {
            panic!("expected serve");
        };
        assert!(args.config.is_none());
        assert!(args.host.is_none());
        assert!(args.port.is_none());
        assert!(!args.no_ui);
    }

    #[test]
    fn no_ui_flag_overlays_serve() {
        // Without the flag, the configured value (default `true`) stands.
        let cli = Cli::try_parse_from(["uc-server", "serve"]).unwrap();
        let Command::Serve(args) = cli.command else {
            panic!("expected serve");
        };
        let mut cfg = Config::default();
        args.overlay(&mut cfg);
        assert!(
            cfg.ui.serve,
            "absent --no-ui leaves serve at its config value"
        );

        // `--no-ui` disables serving the bundled UI.
        let cli = Cli::try_parse_from(["uc-server", "serve", "--no-ui"]).unwrap();
        let Command::Serve(args) = cli.command else {
            panic!("expected serve");
        };
        assert!(args.no_ui);
        let mut cfg = Config::default();
        args.overlay(&mut cfg);
        assert!(!cfg.ui.serve);
    }

    #[test]
    fn serve_flags_overlay_config() {
        let cli = Cli::try_parse_from([
            "uc-server",
            "serve",
            "--host",
            "127.0.0.1",
            "--port",
            "9000",
        ])
        .unwrap();
        let Command::Serve(args) = cli.command else {
            panic!("expected serve");
        };
        let mut cfg = Config::default();
        args.overlay(&mut cfg);
        assert_eq!(cfg.host.as_deref(), Some("127.0.0.1"));
        assert_eq!(cfg.port, Some(9000));
    }

    #[test]
    fn migrate_parses() {
        let cli = Cli::try_parse_from(["uc-server", "migrate"]).unwrap();
        let Command::Migrate(args) = cli.command else {
            panic!("expected migrate");
        };
        assert!(args.config.is_none());
    }

    #[test]
    fn healthcheck_parses() {
        let cli = Cli::try_parse_from(["uc-server", "healthcheck", "--port", "9000"]).unwrap();
        let Command::Healthcheck(args) = cli.command else {
            panic!("expected healthcheck");
        };
        assert_eq!(args.port, Some(9000));
        assert_eq!(args.timeout_secs, 3);
    }

    #[test]
    fn healthcheck_url_overrides_config() {
        // With `--url` the target is verbatim and no config is loaded.
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
    fn no_subcommand_is_an_error() {
        assert!(Cli::try_parse_from(["uc-server"]).is_err());
    }
}
