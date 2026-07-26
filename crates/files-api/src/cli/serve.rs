//! Serve wiring for the standalone `files-api` binary.
//!
//! Build the file store backend from config (Unity Catalog volumes, or the
//! in-memory store), register the `FilesService` on a ConnectRPC router, mount it
//! as an axum fallback service under the configured base path, wrap it in the
//! request-auth layer, add the operational endpoints (`/health`, `/version`,
//! `/capabilities`), bind, and serve until shutdown.

use std::sync::Arc;

use axum::Router;
use axum::routing::get;
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::LatencyUnit;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use unitycatalog_object_store::UnityObjectStoreFactory;

use crate::auth::AuthLayer;
use crate::cli::config::{Backend, Config};
use crate::service::AppState;
use crate::store::{FileStore, MemoryStore, UnityVolumeStore};

/// Build the app, bind `host:port`, and serve until shutdown.
pub async fn serve(config: Config) -> Result<(), String> {
    config.validate()?;
    let host = config.resolved_host().to_string();
    let port = config.resolved_port();
    let base_path = config.resolved_base_path();

    // Build the file store backend.
    let files: Arc<dyn FileStore> = match config.backend {
        Backend::Unity => unity_backend(&config).await?,
        Backend::Memory => {
            tracing::info!("files backed by in-memory store (non-durable)");
            Arc::new(MemoryStore::new())
        }
    };

    // Expose the FilesService as a single axum fallback service, nested under the
    // base path so it occupies its own subtree and never shadows the operational
    // routes below.
    let files_service = AppState::new(files).into_axum_router();
    let mounted: Router = if base_path.is_empty() {
        files_service
    } else {
        Router::new().nest_service(&base_path, files_service)
    };

    // The auth layer inserts a `ForwardedIdentity` extension on every request (and
    // rejects a missing reverse-proxy identity with 401 before it reaches the
    // service). Operational endpoints stay outside it so `/health` (the probe
    // target) is always reachable.
    let app = operational_router(&base_path)
        .merge(mounted.layer(AuthLayer::new(config.auth.to_mode())))
        .layer(
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
    let mount = if base_path.is_empty() {
        "/"
    } else {
        &base_path
    };
    tracing::info!("files-api listening on {addr}, FilesService mounted at `{mount}`");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| format!("serving: {e}"))?;
    Ok(())
}

/// Build a Unity Catalog volume-backed [`FileStore`] from the resolved config.
///
/// The endpoint must be the Unity Catalog REST base URL (e.g.
/// `https://<host>/api/2.1/unity-catalog/`); use `https://` when a token is set —
/// an `http://` endpoint 301-redirects and drops the bearer token. The token is
/// optional (omit for a local unauthenticated OSS server); `region` overrides the
/// AWS region for vended credentials.
async fn unity_backend(config: &Config) -> Result<Arc<dyn FileStore>, String> {
    // `validate()` (called by `serve`) guarantees `upstream` is present for the
    // unity backend.
    let upstream = config
        .upstream
        .as_ref()
        .ok_or_else(|| "the `unity` backend requires an `upstream` block".to_string())?;
    let token = upstream.token.as_ref().and_then(|t| t.value());

    let mut builder = UnityObjectStoreFactory::builder()
        .with_uri(upstream.base_url.clone())
        .with_io_runtime(tokio::runtime::Handle::current());
    match token.filter(|t| !t.is_empty()) {
        Some(token) => builder = builder.with_token(token),
        None => builder = builder.with_allow_unauthenticated(true),
    }
    if let Some(region) = upstream.region.as_ref().filter(|r| !r.is_empty()) {
        builder = builder.with_aws_region(region.clone());
    }

    let factory = builder.build().await.map_err(|e| {
        format!(
            "building Unity Catalog object-store factory for `{}`: {e}",
            upstream.base_url
        )
    })?;
    tracing::info!(
        "files backed by Unity Catalog volumes at `{}`",
        upstream.base_url
    );
    Ok(Arc::new(UnityVolumeStore::new(Arc::new(factory))))
}

/// Operational endpoints served regardless of the FilesService mount.
///
/// `/health` returns the literal `OK` body the `healthcheck` subcommand (and the
/// Docker `HEALTHCHECK`) expects; `/version` returns the crate version;
/// `/capabilities` announces the mount so a client can locate the service.
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

/// The `/capabilities` JSON body: the service name and its mount base path (empty
/// for a root mount).
fn capabilities_body(base_path: &str) -> String {
    format!(r#"{{"service":"portal.files.v1.FilesService","basePath":"{base_path}"}}"#)
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
    fn capabilities_reports_mount() {
        assert_eq!(
            capabilities_body("/files"),
            r#"{"service":"portal.files.v1.FilesService","basePath":"/files"}"#
        );
    }

    #[test]
    fn capabilities_handles_root_mount() {
        assert_eq!(
            capabilities_body(""),
            r#"{"service":"portal.files.v1.FilesService","basePath":""}"#
        );
    }
}
