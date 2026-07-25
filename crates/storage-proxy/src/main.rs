//! Entry point for the standalone `storage-proxy` binary.
//!
//! Thin dispatch over the [`cli`](unitycatalog_storage_proxy::cli) surface:
//! parse the subcommand, then either run the async `serve` loop or the blocking
//! `healthcheck` probe. Only compiled for the `bin` feature (see the crate's
//! `Cargo.toml` `[[bin]]` `required-features`).

use clap::Parser;
use unitycatalog_storage_proxy::cli::{Cli, Command, run_healthcheck};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Structured logging; honor `RUST_LOG`, default to `info`.
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    match Cli::parse().command {
        // The probe is synchronous (blocking reqwest) and needs no runtime.
        Command::Healthcheck(args) => std::process::exit(match run_healthcheck(&args) {
            Ok(()) => 0,
            Err(e) => {
                eprintln!("healthcheck failed: {e}");
                1
            }
        }),
        Command::Serve(args) => {
            let config = args.resolve_config()?;
            let runtime = tokio::runtime::Runtime::new()?;
            runtime.block_on(unitycatalog_storage_proxy::cli::serve::serve(config))?;
            Ok(())
        }
    }
}
