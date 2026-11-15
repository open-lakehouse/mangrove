//! `uc mcp serve`: an MCP server for agents, over stdio, as the signed-in user.
//!
//! The MCP client spawns `uc mcp serve`, which talks to the Unity Catalog
//! server with the CLI's own credentials (`--token` / `UC_TOKEN` /
//! `DATABRICKS_TOKEN`). The first surface is grants: an agent asked to grant
//! privileges opens a prefilled grant form (`open_grant_form`), an A2UI
//! surface rendered by an MCP App in hosts that support the MCP Apps
//! extension; the user reviews and submits it, and only then does the form
//! apply the grant (`apply_grants`). See `docs-mcp-server-design.md`.

mod a2ui;
mod scim;
mod server;
mod tools;

#[cfg(test)]
mod tests;

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
    /// Serve MCP over stdio (the client spawns `uc mcp serve`).
    Serve,
}

/// The tools for the server `opts` points at.
pub fn tools(opts: &GlobalOpts) -> Result<Tools> {
    Ok(Tools::new(
        opts.client()?,
        scim::Scim::new(opts.cloud(), &opts.server_url()?),
    ))
}

pub async fn handle_mcp(cmd: &McpCommand, opts: GlobalOpts) -> Result<()> {
    match cmd.command {
        McpSubcommand::Serve => {
            let server = UcMcpServer::new(tools(&opts)?);
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
