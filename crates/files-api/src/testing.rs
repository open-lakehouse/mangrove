//! Test helpers: spin up the Files API ConnectRPC server over an in-memory store.
//!
//! Gated behind the `testing` feature so the helpers (and their `axum`/`tokio`
//! use) never reach a production build.

use std::sync::Arc;

use crate::service::AppState;
use crate::store::MemoryStore;

/// Bind an ephemeral local port, serve the `FilesService` over a fresh
/// [`MemoryStore`], and return the base URI (`http://127.0.0.1:<port>`) the
/// generated `FilesServiceClient` can point at.
///
/// The server task is detached; it lives until the test process exits.
pub async fn spawn_memory_server() -> String {
    let store = Arc::new(MemoryStore::new());
    let state = AppState::new(store);
    let connect = state.register_all(connectrpc::Router::new());
    let app = axum::Router::new().fallback_service(connect.into_axum_service());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    format!("http://{addr}")
}
