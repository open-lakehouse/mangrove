//! Entry point for the standalone `files-api` binary (the `bin` feature).
//!
//! Dispatches the two subcommands: `serve` builds the effective config and runs
//! the server on a Tokio runtime; `healthcheck` runs a blocking probe and maps
//! its result to the process exit code (the Docker `HEALTHCHECK` relies on this).

use clap::Parser;
use tracing_subscriber::EnvFilter;

use unitycatalog_files_api::cli::serve::serve;
use unitycatalog_files_api::cli::{Cli, Command, run_healthcheck};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    match Cli::parse().command {
        Command::Healthcheck(args) => {
            std::process::exit(match run_healthcheck(&args) {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("healthcheck failed: {e}");
                    1
                }
            });
        }
        Command::Serve(args) => {
            let config = args.resolve_config()?;
            tokio::runtime::Runtime::new()?.block_on(serve(config))?;
            Ok(())
        }
    }
}
