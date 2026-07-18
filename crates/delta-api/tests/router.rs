//! Tests for the state-agnostic axum router.
//!
//! These cover the two properties issue #135 is about:
//! 1. **Composition** — the router builds over an arbitrary host state `S` and can
//!    be `.nest`ed into a `Router<S>` *before* `.with_state`, without the host
//!    writing any `FromRequestParts` glue.
//! 2. **Extraction** — driving the composed router over HTTP exercises the
//!    context/path/query/body extraction closures end-to-end.

// A `ContextExtractor` closure returns `Result<Cx, Response>`; the `Response` Err
// is intentional (see `router.rs`).
#![allow(clippy::result_large_err)]

use std::sync::Arc;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use tower::ServiceExt; // oneshot

use unitycatalog_delta_api::testing::InMemoryDeltaBackend;
use unitycatalog_delta_api::{ContextExtractor, router_from_extension_at, router_with_context_at};

/// The in-memory backend's context type is `()`.
type Cx = ();

/// A trivial always-anonymous extractor.
fn unit_extractor() -> ContextExtractor<Cx> {
    Arc::new(|_parts| Ok(()))
}

/// A host state distinct from `()` — the router must build over it without ever
/// making the handler the axum `State`.
#[derive(Clone)]
struct HostState {
    #[allow(dead_code)]
    marker: u8,
}

/// The #135 composition guard: an unstated `Router<HostState>` nests into a
/// `Router<HostState>` and only then gets `.with_state`d — the exact shape a host
/// (e.g. lakekeeper) needs and that the old fully-stated `Router<()>` could not do.
fn app() -> Router {
    let delta: Router<HostState> = router_with_context_at(
        "", // relative routes; the host adds the `/delta/v1` prefix via `.nest`
        Arc::new(InMemoryDeltaBackend::new()),
        unit_extractor(),
    );

    Router::<HostState>::new()
        .nest("/delta/v1", delta)
        .with_state(HostState { marker: 7 })
}

#[tokio::test]
async fn get_config_routes_and_extracts_query() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/delta/v1/config?catalog=catalog&protocol-versions=1.1,2.3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    // `catalog` is the in-memory backend's default catalog, so a well-formed
    // request with a negotiable version succeeds — proving routing + query
    // extraction + context extraction all fired.
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.get("protocol-version").is_some(), "body: {json}");
}

#[tokio::test]
async fn missing_query_param_is_a_400() {
    // No `catalog` query param → the `Query<GetConfigParams>` extraction fails,
    // and its rejection is returned as a Response by the extraction helper.
    let response = app()
        .oneshot(
            Request::builder()
                .uri("/delta/v1/config?protocol-versions=1.1,2.3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn head_table_missing_is_404() {
    // Routes to `table_exists` (HEAD on the `/tables/{table}` method fan-out) and
    // exercises path extraction into `TableRef`. No such table → 404.
    let response = app()
        .oneshot(
            Request::builder()
                .method("HEAD")
                .uri("/delta/v1/catalogs/c/schemas/s/tables/absent")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

/// The extension convenience path short-circuits with 401 when the context
/// extension is absent — confirming `router_from_extension_at` wires the
/// missing-context rejection.
#[tokio::test]
async fn extension_router_without_context_is_401() {
    let delta: Router<()> =
        router_from_extension_at::<(), Cx>("/delta/v1", Arc::new(InMemoryDeltaBackend::new()));
    let app = delta.with_state(());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/delta/v1/config?catalog=main&protocol-versions=1.1,2.3")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
