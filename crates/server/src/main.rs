//! The `uc-server` binary: the deployable Unity Catalog service.
//!
//! Thin entry point over [`unitycatalog_server`]'s `bin` support — parses the
//! CLI ([`cli::Cli`]) and dispatches to `serve` / `migrate` / `healthcheck`.
//! Tracing is initialized here (in the binary), not in the library, so an
//! embedder that already installed a subscriber isn't fought.

use clap::Parser;

use unitycatalog_server::cli::{Cli, Command, MigrateArgs, ServeArgs, run_healthcheck};
use unitycatalog_server::config::Config;
use unitycatalog_server::telemetry::init_tracing;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match Cli::parse().command {
        // Synchronous: a `reqwest::blocking` probe needs no tokio runtime, so the
        // healthcheck path stays cheap. Map the result to a process exit code so
        // Docker/Compose can gate on it.
        Command::Healthcheck(args) => std::process::exit(match run_healthcheck(&args) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("healthcheck failed: {e}");
                1
            }
        }),
        Command::Serve(args) => serve(args),
        Command::Migrate(args) => migrate(args),
    }
}

#[tokio::main]
async fn serve(args: ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let mut cfg = Config::load(args.config.as_ref())?;
    args.overlay(&mut cfg);
    unitycatalog_server::run::serve(cfg).await?;
    Ok(())
}

#[tokio::main]
async fn migrate(args: MigrateArgs) -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();
    let cfg = Config::load(args.config.as_ref())?;
    unitycatalog_server::run::migrate(cfg).await?;
    Ok(())
}
