//! Serve wiring for the standalone `storage-proxy` binary.
//!
//! Assemble the byte-proxy surface (the client arm pointed at the configured
//! upstream Unity Catalog), wrap it in the configured request-auth layer, mount
//! the operational endpoints (`/health`, `/version`, `/capabilities`), bind, and
//! serve until shutdown. This is the distilled `ProxyArm::Client` path the
//! `server` crate mounts internally, minus the DB / UI / hybrid routing.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

use crate::auth::{AuthLayer, ForwardedIdentity};
use crate::cli::config::Config;
use crate::client::UnityFactoryProxyBackend;
use crate::router::router_from_extension_at;

/// Build the app, bind `host:port`, and serve until shutdown.
pub async fn serve(config: Config) -> Result<(), String> {
    let host = config.resolved_host().to_string();
    let port = config.resolved_port();
    let base_path = config.resolved_base_path();

    // Connect the client arm to the upstream UC. The token (if any) authenticates
    // the proxy's own resolve/vend calls; absent, the upstream is contacted
    // unauthenticated (anonymous UC). In reverse-proxy mode, a per-request
    // forwarded identity is re-emitted upstream under the SAME header the proxy
    // reads it from, so UC attributes the vend to the end user; an anonymous
    // request falls back to the token above.
    let token = config.upstream.token.as_ref().and_then(|t| t.value());
    let forwarded_header = config.auth.resolved_forwarded_header();
    let backend = UnityFactoryProxyBackend::connect_with_forwarded_header(
        &config.upstream.base_url,
        token,
        forwarded_header,
    )
    .await
    .map_err(|e| {
        format!(
            "connecting to upstream UC `{}`: {e}",
            config.upstream.base_url
        )
    })?;

    // The auth layer inserts a `ForwardedIdentity` extension on every request;
    // the router reads it back as the per-request context. Anonymous mode still
    // inserts `ForwardedIdentity(None)`, so the context is always present.
    let proxy: Router =
        router_from_extension_at::<(), ForwardedIdentity>(&base_path, Arc::new(backend))
            .with_state(())
            .layer(AuthLayer::new(config.auth.to_mode()));

    // Operational endpoints at the root, outside the proxy surface, so `/health`
    // (the healthcheck probe target) and `/capabilities` are always reachable.
    let app = operational_router(&base_path).merge(proxy).layer(
        TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().include_headers(true))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(
                DefaultOnResponse::new()
                    .level(Level::INFO)
                    .latency_unit(LatencyUnit::Micros),
            ),
    );

    let listener = TcpListener::bind(format!("{host}:{port}"))
        .await
        .map_err(|e| format!("binding {host}:{port}: {e}"))?;
    let addr = listener
        .local_addr()
        .map_err(|e| format!("resolving local address: {e}"))?;
    tracing::info!("storage-proxy listening on {addr}, byte-proxy mounted at `{base_path}/`");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| format!("serving: {e}"))?;
    Ok(())
}

/// Operational endpoints served regardless of the proxy surface.
///
/// `/health` returns the literal `OK` body the `healthcheck` subcommand (and the
/// Docker `HEALTHCHECK`) expects; `/version` returns the crate version;
/// `/capabilities` announces the storage-access posture — always `"proxy"` for
/// this deployment, since it exists to serve the byte-proxy.
fn operational_router(base_path: &str) -> Router {
    let capabilities = capabilities_body(base_path);
    Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/version", get(|| async { env!("CARGO_PKG_VERSION") }))
        .route(
            "/capabilities",
            get(move || async move {
                (
                    [(axum::http::header::CONTENT_TYPE, "application/json")],
                    capabilities,
                )
            }),
        )
}

/// The `/capabilities` JSON body. `basePath` is the byte-proxy mount;
/// `conditionalWrites` advertises the server-enforced `If-Match` relay.
fn capabilities_body(base_path: &str) -> String {
    // The wasm client keys off `storageAccess` and reads `storageProxy.basePath`
    // to build request URLs. An empty (root) mount reports `basePath: ""`.
    format!(
        r#"{{"storageAccess":"proxy","storageProxy":{{"basePath":"{base_path}","conditionalWrites":true}}}}"#
    )
}

/// Resolve when the process receives Ctrl-C or (on Unix) SIGTERM, so the server
/// drains in-flight requests before exiting.
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_reports_proxy_posture() {
        assert_eq!(
            capabilities_body("/storage-proxy"),
            r#"{"storageAccess":"proxy","storageProxy":{"basePath":"/storage-proxy","conditionalWrites":true}}"#
        );
    }

    #[test]
    fn capabilities_handles_root_mount() {
        assert_eq!(
            capabilities_body(""),
            r#"{"storageAccess":"proxy","storageProxy":{"basePath":"","conditionalWrites":true}}"#
        );
    }
}
