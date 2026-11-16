//! `uc mcp serve`: an MCP server for agents, as the signed-in user.
//!
//! By default the MCP client spawns `uc mcp serve` and talks to it over stdio.
//! With `--http <ADDR>` it serves Streamable HTTP instead, so several clients
//! (an agent and an MCP Apps host) can share one server; see [`http`] for why
//! that is for use behind a gateway only. Either way it talks to the Unity
//! Catalog server with the CLI's own credentials (`--token` / `UC_TOKEN` /
//! `DATABRICKS_TOKEN`), fixed at startup. The first surface is grants: an agent asked to grant
//! privileges opens a prefilled grant form (`open_grant_form`), an A2UI
//! surface rendered by an MCP App in hosts that support the MCP Apps
//! extension; the user reviews and submits it, and only then does the form
//! apply the grant (`apply_grants`). See `docs-mcp-server-design.md`.

mod a2ui;
mod http;
mod scim;
mod server;
mod tools;

#[cfg(test)]
mod tests;

use std::net::SocketAddr;

use clap::{Args, Subcommand};
use rmcp::ServiceExt;

use crate::GlobalOpts;
use crate::error::{Error, Result};

pub use server::UcMcpServer;
pub use tools::Tools;

#[derive(Args)]
pub struct McpCommand {
    #[clap(subcommand)]
    command: McpSubcommand,
}

#[derive(Subcommand)]
enum McpSubcommand {
    /// Serve MCP over stdio (the client spawns `uc mcp serve`), or over
    /// Streamable HTTP with `--http`.
    Serve(ServeArgs),
}

#[derive(Args)]
struct ServeArgs {
    /// Serve Streamable HTTP on this address (e.g. `127.0.0.1:8090`) instead
    /// of stdio. There is no inbound auth: every client acts as the server's
    /// own identity, so run it behind a gateway only.
    #[clap(long, value_name = "ADDR")]
    http: Option<SocketAddr>,

    /// The path of the MCP endpoint (with `--http`). `GET /health` is served
    /// beside it.
    #[clap(long, default_value = "/mcp", requires = "http")]
    path: String,

    /// A `Host` header value to accept, as `host` (any port) or `host:port`
    /// (with `--http`; repeatable). Defaults to loopback only (`localhost`,
    /// `127.0.0.1`, `::1`); a list replaces that default, so add loopback
    /// back if local clients need it. `GET /health` is not checked.
    #[clap(long = "allowed-host", value_name = "HOST", requires = "http")]
    allowed_hosts: Vec<String>,
}

/// The tools for the server `opts` points at.
pub fn tools(opts: &GlobalOpts) -> Result<Tools> {
    Ok(Tools::new(
        opts.client()?,
        scim::Scim::new(opts.cloud(), &opts.server_url()?),
    ))
}

pub async fn handle_mcp(cmd: &McpCommand, opts: GlobalOpts) -> Result<()> {
    match &cmd.command {
        McpSubcommand::Serve(args) => {
            let server = UcMcpServer::new(tools(&opts)?);
            if let Some(addr) = args.http {
                return http::serve(server, addr, &args.path, &args.allowed_hosts).await;
            }
            let running = server
                .serve(rmcp::transport::stdio())
                .await
                .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
            running
                .waiting()
                .await
                .map_err(|e| Error::Io(std::io::Error::other(e.to_string())))?;
            Ok(())
        }
    }
}
