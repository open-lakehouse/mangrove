//! The directory end to end: a server behind a reverse proxy (Authelia's
//! `Remote-User`) with Authelia's user file as its user source. A user found
//! through SCIM is granted a privilege by email, and the grants name users by
//! email while keeping them by user name.
#![cfg(all(feature = "scim", feature = "sqlite"))]

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;
use unitycatalog_server::config::Config;

const USERS: &str = r#"
users:
  alice:
    displayname: Alice Liddell
    email: alice@example.com
    password: "$argon2id$v=19$m=65536,t=3,p=4$x$y"
  bob:
    displayname: Bob
    email: bob@example.com
    password: "$argon2id$v=19$m=65536,t=3,p=4$x$y"
"#;

async fn app(dir: &std::path::Path) -> Router {
    let root = std::fs::canonicalize(dir).unwrap();
    let users = root.join("users.yml");
    std::fs::write(&users, USERS).unwrap();
    let data = root.join("data");
    std::fs::create_dir(&data).unwrap();
    let yaml = format!(
        r#"
managed_storage_root: file://{data}
local_storage:
  allowed-roots: ["{data}"]
auth:
  mode: reverse-proxy
  forwarded-user-header: remote-user
directory:
  users: {{ source: authelia, users-file: "{users}" }}
"#,
        data = data.display(),
        users = users.display(),
    );
    let config: Config = serde_yml::from_str(&yaml).unwrap();
    unitycatalog_server::embed::start(&config)
        .await
        .unwrap()
        .router
}

async fn call(app: &Router, method: &str, path: &str, body: Option<Value>) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(path)
        .header("remote-user", "alice")
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
async fn grants_name_users_found_through_scim() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path()).await;

    let (status, me) = call(&app, "GET", "/api/2.0/preview/scim/v2/Me", None).await;
    assert_eq!(status, StatusCode::OK, "{me}");
    assert_eq!(me["userName"], "alice@example.com");
    assert_eq!(me["displayName"], "Alice Liddell");

    let (status, found) = call(
        &app,
        "GET",
        "/api/2.0/preview/scim/v2/Users?filter=displayName%20sw%20%22bo%22",
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{found}");
    assert_eq!(found["totalResults"], 1);
    let bob = found["Resources"][0]["userName"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(bob, "bob@example.com");

    let (status, body) = call(
        &app,
        "POST",
        "/api/2.1/unity-catalog/catalogs",
        Some(json!({ "name": "main" })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (status, body) = call(
        &app,
        "PATCH",
        "/api/2.1/unity-catalog/permissions/catalog/main",
        Some(json!({ "changes": [{ "principal": bob, "add": ["USE_CATALOG"] }] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");

    let (_, grants) = call(
        &app,
        "GET",
        "/api/2.1/unity-catalog/permissions/catalog/main",
        None,
    )
    .await;
    // Kept as user `bob`, named by email. (Ownership is not listed.)
    assert_eq!(
        grants["privilege_assignments"],
        json!([{ "principal": "bob@example.com", "privileges": ["USE CATALOG"] }])
    );

    let (status, _) = call(
        &app,
        "PATCH",
        "/api/2.1/unity-catalog/permissions/catalog/main",
        Some(json!({ "changes": [{ "principal": "nobody@example.com", "add": ["SELECT"] }] })),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn scim_needs_the_forwarded_user() {
    let dir = tempfile::tempdir().unwrap();
    let app = app(dir.path()).await;
    let request = Request::get("/api/2.0/preview/scim/v2/Users")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
