//! An in-process `uc-server` for engine-interop suites (`tests/engines`).
//!
//! ```text
//! uc-test-server [--root <dir>] [--port <port>] [--seed <catalog>.<schema>.<table>]...
//!                [--log-requests]
//! ```
//!
//! Serves on `127.0.0.1` with an in-memory SQLite backend and `<dir>` (default: a
//! temporary directory) as the managed `file://` storage root. Every `--seed`
//! creates the catalog and schema if needed, then a catalog-managed Delta table
//! written through `/delta/v1` with rows `(1, alice), (2, bob), (3, carol)`.
//! `--log-requests` prints `<METHOD> <path> -> <status>` to stderr per request.
//!
//! Once ready it prints a single JSON line to stdout,
//! `{"url": ..., "storage_root": ..., "root": ...}`, and serves until it receives
//! Ctrl-C/SIGTERM or stdin closes, so a crashed parent doesn't leave it running.

use std::collections::HashSet;
use std::path::PathBuf;

use tokio::io::AsyncReadExt;
use unitycatalog_acceptance::checks::managed_delta::create_commit_read;
use unitycatalog_acceptance::server::TestServer;
use unitycatalog_acceptance::{AcceptanceError, AcceptanceResult};

struct Args {
    root: Option<PathBuf>,
    port: u16,
    log_requests: bool,
    seeds: Vec<(String, String, String)>,
}

fn usage(msg: &str) -> AcceptanceError {
    AcceptanceError::JourneyValidation(format!(
        "{msg}\nusage: uc-test-server [--root <dir>] [--port <port>] \
         [--seed <catalog>.<schema>.<table>]... [--log-requests]"
    ))
}

fn parse_args() -> AcceptanceResult<Args> {
    let mut args = Args {
        root: None,
        port: 0,
        log_requests: false,
        seeds: Vec::new(),
    };
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let mut value = || {
            it.next()
                .ok_or_else(|| usage(&format!("{flag} needs a value")))
        };
        match flag.as_str() {
            "--log-requests" => args.log_requests = true,
            "--root" => args.root = Some(PathBuf::from(value()?)),
            "--port" => {
                args.port = value()?
                    .parse()
                    .map_err(|e| usage(&format!("invalid --port: {e}")))?
            }
            "--seed" => {
                let name = value()?;
                let parts: Vec<&str> = name.split('.').collect();
                let [catalog, schema, table] = parts[..] else {
                    return Err(usage(&format!(
                        "--seed {name:?} is not <catalog>.<schema>.<table>"
                    )));
                };
                args.seeds
                    .push((catalog.into(), schema.into(), table.into()));
            }
            other => return Err(usage(&format!("unknown argument {other:?}"))),
        }
    }
    Ok(args)
}

async fn seed(server: &TestServer, seeds: &[(String, String, String)]) -> AcceptanceResult<()> {
    let ctx = server.context()?;
    let client = ctx.client();
    let mut catalogs = HashSet::new();
    let mut schemas = HashSet::new();
    for (catalog, schema, table) in seeds {
        if catalogs.insert(catalog.clone()) {
            ctx.create_catalog(catalog).await?;
        }
        if schemas.insert((catalog.clone(), schema.clone())) {
            client.create_schema(schema, catalog).await?;
        }
        let rows = create_commit_read(client, catalog, schema, table).await?;
        if rows != 3 {
            return Err(AcceptanceError::JourneyExecution(format!(
                "seeding {catalog}.{schema}.{table} read back {rows} rows, expected 3"
            )));
        }
    }
    Ok(())
}

/// Resolves when stdin reaches EOF (the parent exited or closed the pipe).
async fn stdin_closed() {
    let mut stdin = tokio::io::stdin();
    let mut buf = [0u8; 256];
    while let Ok(n) = stdin.read(&mut buf).await {
        if n == 0 {
            break;
        }
    }
}

#[cfg(unix)]
async fn terminated() {
    use tokio::signal::unix::{SignalKind, signal};
    match signal(SignalKind::terminate()) {
        Ok(mut sig) => {
            sig.recv().await;
        }
        Err(_) => std::future::pending().await,
    }
}

#[cfg(not(unix))]
async fn terminated() {
    std::future::pending().await
}

#[tokio::main]
async fn main() -> AcceptanceResult<()> {
    let args = parse_args()?;
    // A temporary root lives as long as the server; `_tempdir` drops last.
    let (_tempdir, root) = match &args.root {
        Some(root) => {
            std::fs::create_dir_all(root)?;
            (None, root.clone())
        }
        None => {
            let dir = tempfile::tempdir()?;
            let path = dir.path().to_path_buf();
            (Some(dir), path)
        }
    };
    let server = TestServer::start_at(&root, args.port, args.log_requests).await?;
    seed(&server, &args.seeds).await?;

    println!(
        "{}",
        serde_json::json!({
            "url": server.url(),
            "storage_root": server.storage_root(),
            "root": server.root(),
        })
    );

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = terminated() => {}
        _ = stdin_closed() => {}
    }
    server.shutdown().await
}
