//! The `sharing-server` binary: `serve` | `migrate` | `healthcheck` | `rotate-keys`.

use std::process::ExitCode;

use clap::{Parser, Subcommand};
use open_sharing_server::config::DatabaseConfig;
use open_sharing_server::{Config, embed};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Parser)]
#[command(
    name = "sharing-server",
    version,
    about = "An Open Sharing / Delta Sharing server"
)]
struct Cli {
    /// Path to the YAML configuration file.
    #[arg(long, short, env = "SHARING_SERVER_CONFIG", global = true)]
    config: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the sharing protocols and the admin API.
    Serve {
        /// Override the configured bind host.
        #[arg(long)]
        host: Option<String>,
        /// Override the configured port.
        #[arg(long)]
        port: Option<u16>,
    },
    /// Apply pending database migrations (over the migration URL).
    Migrate,
    /// Exit 0 if a local server answers `GET /health`.
    Healthcheck,
    /// Re-wrap every stored secret under the active key-encryption key.
    RotateKeys,
}

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let cli = Cli::parse();
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{e}");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> Result<(), String> {
    let mut config = Config::load(cli.config.as_deref())?;
    match cli.command {
        Command::Serve { host, port } => {
            config.host = host.or(config.host);
            config.port = port.or(config.port);
            let service = embed::start(&config).await?;
            let addr = format!("{}:{}", config.host(), config.port());
            let listener = tokio::net::TcpListener::bind(&addr)
                .await
                .map_err(|e| format!("binding {addr}: {e}"))?;
            tracing::info!("serving on {addr} (public url {})", config.public_url());
            axum::serve(listener, service.router)
                .with_graceful_shutdown(async {
                    let _ = tokio::signal::ctrl_c().await;
                })
                .await
                .map_err(|e| e.to_string())
        }
        Command::Migrate => migrate(&config).await,
        Command::Healthcheck => healthcheck(config.port()).await,
        Command::RotateKeys => {
            let service = embed::start(&config).await?;
            let (credentials, tokens) =
                embed::rotate_keys(service.store.as_ref(), &service.secrets)
                    .await
                    .map_err(|e| e.to_string())?;
            tracing::info!("re-wrapped {credentials} credential(s) and {tokens} pending token(s)");
            Ok(())
        }
    }
}

async fn migrate(config: &Config) -> Result<(), String> {
    use open_sharing_server::store::postgres;
    let DatabaseConfig::Postgres {
        url, migration_url, ..
    } = &config.database
    else {
        return Err("`migrate` needs a postgres database".to_string());
    };
    let url = migration_url
        .as_ref()
        .and_then(|u| u.value())
        .or_else(|| url.value())
        .ok_or("database url is not set")?;
    let pool = postgres::connect(&url, 1)
        .await
        .map_err(|e| e.to_string())?;
    postgres::migrate(&pool).await.map_err(|e| e.to_string())?;
    tracing::info!("migrations applied");
    Ok(())
}

/// A minimal `GET /health` over TCP, so the distroless image needs no curl.
async fn healthcheck(port: u16) -> Result<(), String> {
    let mut stream = tokio::net::TcpStream::connect(("127.0.0.1", port))
        .await
        .map_err(|e| format!("connecting: {e}"))?;
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n")
        .await
        .map_err(|e| e.to_string())?;
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .await
        .map_err(|e| e.to_string())?;
    if response.starts_with("HTTP/1.1 200") {
        Ok(())
    } else {
        Err(format!(
            "unhealthy: {}",
            response.lines().next().unwrap_or_default()
        ))
    }
}
