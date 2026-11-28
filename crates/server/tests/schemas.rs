//! Schema creation over the REST API, against the in-process server
//! ([`embed::start`]) on an in-memory SQLite store: a schema needs an existing
//! parent catalog, and a refused create leaves no orphan behind (#372).
#![cfg(feature = "sqlite")]

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;
use unitycatalog_server::config::Config;
use unitycatalog_server::embed;

const SCHEMAS: &str = "/api/2.1/unity-catalog/schemas";
const CATALOGS: &str = "/api/2.1/unity-catalog/catalogs";

/// A server whose metastore managed root is a local directory under `dir`, so
/// managed catalogs can be created without a storage root of their own.
async fn app(dir: &std::path::Path) -> Router {
    let data = std::fs::canonicalize(dir).unwrap().join("data");
    std::fs::create_dir(&data).unwrap();
    let yaml = format!(
        r#"
managed_storage_root: file://{data}
local_storage:
  allowed-roots: ["{data}"]
"#,
        data = data.display(),
    );
    let mut config: Config = serde_yml::from_str(&yaml).unwrap();
    config.ui.serve = false;
    embed::start(&config).await.unwrap().router
}

async fn call(app: &Router, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(body.map_or_else(Body::empty, |b| Body::from(b.to_string())))
        .unwrap();
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap()
    };
    (status, body)
}

#[tokio::test]
async fn create_schema_in_missing_catalog_is_not_found_and_stores_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path()).await;

    let (status, body) = call(
        &app,
        "POST",
        SCHEMAS,
        Some(json!({ "name": "orphan", "catalog_name": "missing" })),
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    assert_eq!(body["errorCode"], "RESOURCE_NOT_FOUND", "{body}");

    // No orphan schema was persisted under the missing catalog.
    let (status, body) = call(&app, "GET", &format!("{SCHEMAS}/missing.orphan"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
    let (status, body) = call(
        &app,
        "GET",
        &format!("{SCHEMAS}?catalog_name=missing"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let listed = body["schemas"].as_array().map_or(0, Vec::len);
    assert_eq!(listed, 0, "{body}");

    // Creating the catalog afterwards does not surface a stale schema either.
    let (status, body) = call(&app, "POST", CATALOGS, Some(json!({ "name": "missing" }))).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = call(&app, "GET", &format!("{SCHEMAS}/missing.orphan"), None).await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

#[tokio::test]
async fn create_schema_in_existing_catalog_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path()).await;

    let (status, body) = call(&app, "POST", CATALOGS, Some(json!({ "name": "cat" }))).await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, body) = call(
        &app,
        "POST",
        SCHEMAS,
        Some(json!({ "name": "sch", "catalog_name": "cat" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["full_name"], "cat.sch", "{body}");

    let (status, body) = call(&app, "GET", &format!("{SCHEMAS}/cat.sch"), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["catalog_name"], "cat", "{body}");
}
