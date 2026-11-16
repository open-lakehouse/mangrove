//! `uc mcp serve --http`: MCP over Streamable HTTP.
//!
//! One server, many clients: an agent and an MCP Apps host can share it, and
//! each connection (or, for the stateless 2026-07-28 protocol, each request)
//! sees the app-only tools only if it declared the MCP Apps extension.
//!
//! There is no inbound authentication. Every client acts as the identity the
//! server was started with (`--server` / `--token`), so run it behind a
//! gateway that authenticates its callers, never as a shared deployment.
//! The `Host` allowlist (`--allowed-host`, loopback by default) guards against
//! DNS rebinding, not against callers.

use std::net::SocketAddr;
use std::time::Duration;

use axum::Router;
use axum::routing::get;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tokio::net::TcpListener;

use super::server::UcMcpServer;
use crate::error::{Error, Result};

/// How often an open SSE stream is pinged, so proxies keep it open.
const SSE_KEEP_ALIVE: Duration = Duration::from_secs(15);

/// The Streamable HTTP config: SSE keep-alive, and `allowed_hosts` for the
/// inbound `Host` check (rmcp's loopback-only default when empty).
pub fn config(allowed_hosts: &[String]) -> StreamableHttpServerConfig {
    let config = StreamableHttpServerConfig::default().with_sse_keep_alive(Some(SSE_KEEP_ALIVE));
    if allowed_hosts.is_empty() {
        config
    } else {
        config.with_allowed_hosts(allowed_hosts.iter().cloned())
    }
}

/// The MCP endpoint at `path`, and `GET /health`, which answers `OK` like the server's, so
/// `uc-server healthcheck --url` can probe it.
pub fn router(server: UcMcpServer, path: &str, config: StreamableHttpServerConfig) -> Router {
    let mcp = StreamableHttpService::new(
        move || Ok(server.clone()),
        LocalSessionManager::default().into(),
        config,
    );
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .nest_service(path, mcp)
}

/// Serve `server` on `addr` until ctrl-c.
pub async fn serve(
    server: UcMcpServer,
    addr: SocketAddr,
    path: &str,
    allowed_hosts: &[String],
) -> Result<()> {
    let config = config(allowed_hosts);
    let shutdown = config.cancellation_token.clone();
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    eprintln!("uc mcp: serving Streamable HTTP on http://{local}{path} (health: /health)");
    axum::serve(listener, router(server, path, config))
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            // Ends the open sessions and their SSE streams, so the server can drain.
            shutdown.cancel();
        })
        .await
        .map_err(Error::Io)
}
