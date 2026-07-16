//! `uc` — the Unity Catalog command-line client.
//!
//! A thin CLI over [`unitycatalog_client`] for talking to a running Unity
//! Catalog server: managing securables (catalogs, schemas, tables, …) and
//! exploring the catalog hierarchy. It targets a server by host — the client
//! appends the `/api/2.1/unity-catalog` API prefix itself — and renders results
//! in the format selected by the global output flag.
//!
//! This crate is a binary, not a library; run `uc --help` for the full command
//! surface.

use clap::{Args, Parser, Subcommand};
use unitycatalog_client::UnityCatalogClient;

use crate::client::{ClientCommand, handle_client};
use crate::error::{Error, Result};
use crate::explore::{ExploreCommand, handle_explore};
use crate::render::OutputFormat;

/// REST path prefix under which the Unity Catalog 2.1 API is served. The client
/// resolves resource paths relative to its base URL, so the base must include
/// this prefix — callers pass only the host (e.g. `http://localhost:8080`).
const UC_API_PREFIX: &str = "/api/2.1/unity-catalog";

mod client;
mod error;
mod explore;
mod render;
// mod test;

#[derive(Parser)]
#[command(name = "unity-catalog", version, about = "CLI to manage delta.sharing services.", long_about = None)]
struct Cli {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Commands,
}

#[derive(Debug, Args)]
struct GlobalOpts {
    /// Server URL (host only; the `/api/2.1/unity-catalog` prefix is added automatically)
    #[clap(
        long,
        global = true,
        env = "UC_SERVER_URL",
        default_value = "http://localhost:8080"
    )]
    server: String,

    /// Output format (`auto` renders a table on a terminal, JSON when piped)
    #[clap(
        long,
        short,
        global = true,
        env = "UC_OUTPUT",
        default_value = "auto",
        value_enum
    )]
    output: OutputFormat,
}

impl GlobalOpts {
    /// Build an unauthenticated client for the configured server, ensuring the
    /// base URL carries the [`UC_API_PREFIX`]. A `--server` value that already
    /// ends with the prefix is used as-is, so passing either `http://host:8080`
    /// or `http://host:8080/api/2.1/unity-catalog` works.
    fn client(&self) -> Result<UnityCatalogClient> {
        let mut url = url::Url::parse(&self.server)
            .map_err(|e| Error::Generic(format!("invalid server url `{}`: {e}", self.server)))?;
        let path = url.path().trim_end_matches('/');
        if !path.ends_with(UC_API_PREFIX) {
            url.set_path(&format!("{path}{UC_API_PREFIX}"));
        }
        Ok(UnityCatalogClient::new(
            olai_http::CloudClient::new_unauthenticated(),
            url,
        ))
    }
}

/// Subcommands for the `uc` client CLI.
///
/// This is a thin HTTP client for a running Unity Catalog server (mirroring
/// headwaters' `hw`); the server itself — including `serve`, `migrate`, and
/// `healthcheck` — lives in the separate `uc-server` binary (crate
/// `olai-uc-server`).
#[derive(Subcommand)]
enum Commands {
    #[clap(
        arg_required_else_help = true,
        about = "execute requests against a sharing server"
    )]
    Client(ClientCommand),

    #[clap(about = "interactively browse the catalog hierarchy in a TUI")]
    Explore(ExploreCommand),
}

#[derive(Parser)]
struct ClientArgs {
    #[clap(help = "Sets the server address")]
    endpoint: String,
}

#[tokio::main(flavor = "multi_thread", worker_threads = 10)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    match &args.command {
        Commands::Client(client_args) => {
            handle_client(client_args, args.global_opts).await?;
        }
        Commands::Explore(cmd) => {
            handle_explore(cmd, args.global_opts).await?;
        }
    };
    Ok(())
}

// Handle the profile command.
// async fn handle_profile(args: &ProfileArgs) -> Result<()> {
//     let token_manager = TokenManager::new_from_secret(args.secret.as_bytes(), None);
//     let profile_manager = DeltaProfileManager::new(args.endpoint.clone(), 1, token_manager);
//
//     let exp = args
//         .validity
//         .and_then(|days| chrono::Utc::now().checked_add_days(Days::new(days)));
//     let shares = args
//         .shares
//         .split(',')
//         .map(|s| s.trim().to_ascii_lowercase())
//         .collect();
//     let claims = DefaultClaims {
//         sub: args.subject.clone(),
//         issued_at: chrono::Utc::now().timestamp(),
//         admin: args.admin,
//         exp: exp.as_ref().map(|dt| dt.timestamp() as u64),
//         shares,
//     };
//     let profile = profile_manager.issue_profile(&claims, exp).await?;
//     std::fs::write("profile.json", serde_json::to_string_pretty(&profile)?)?;
//     Ok(())
// }
