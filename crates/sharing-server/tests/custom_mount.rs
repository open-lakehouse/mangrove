//! A host without Unity Catalog: it provisions shares and recipients straight
//! through the store (as a catalog with its own admin surface would) and serves
//! the protocol router, with no admin API mounted. Runs without the `unity`
//! feature.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use open_sharing_server::config::EncryptionConfig;
use open_sharing_server::embed::{self, Settings};
use open_sharing_server::recipients::{activation_url, issue_token};
use open_sharing_server::secrets::Secrets;
use open_sharing_server::source::fixed::{StaticSource, StaticTable};
use open_sharing_server::source::{AccessMode, RegisteredSource, SourceRegistry};
use open_sharing_server::store::memory::InMemoryStore;
use open_sharing_server::store::{ShareObject, ShareObjectKind, ShareStore};
use serde_json::Value;
use tower::ServiceExt;

const PUBLIC_URL: &str = "https://sharing.example.com";

fn table_location() -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../sharing-api/tests/data/basic_partitioned")
        .canonicalize()
        .unwrap();
    url::Url::from_directory_path(dir).unwrap().to_string()
}

fn sources() -> SourceRegistry {
    let mut source = StaticSource::default();
    source.tables.insert(
        "demo.sales.letters".to_string(),
        StaticTable {
            location: table_location(),
            format: "delta".to_string(),
        },
    );
    SourceRegistry::new(vec![RegisteredSource {
        name: "static".to_string(),
        catalogs: vec!["demo".to_string()],
        access: AccessMode::Local,
        source: Arc::new(source),
    }])
}

async fn call(app: &Router, method: &str, uri: &str, token: Option<&str>) -> (StatusCode, String) {
    let mut req = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        req = req.header("authorization", format!("Bearer {token}"));
    }
    if method == "POST" {
        req = req.header("content-type", "application/json");
    }
    let body = if method == "POST" { "{}" } else { "" };
    let resp = app
        .clone()
        .oneshot(req.body(Body::from(body)).unwrap())
        .await
        .unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

#[tokio::test]
async fn serves_shares_provisioned_through_the_store() {
    let store: Arc<dyn ShareStore> = Arc::new(InMemoryStore::new());
    let secrets = Secrets::new(EncryptionConfig::insecure_dev().build().unwrap());
    let app = embed::router(
        store.clone(),
        secrets.clone(),
        sources(),
        Settings {
            public_url: PUBLIC_URL.to_string(),
            provider_tokens: Vec::new(),
            presign_ttl: Duration::from_secs(300),
            aws_region: None,
        },
    );

    // Provision: a share holding one table, and a recipient granted it.
    store.create_share("letters", None).await.unwrap();
    store
        .update_share(
            "letters",
            vec![ShareObject::new(
                ShareObjectKind::Table,
                "demo.sales.letters",
            )],
            None,
        )
        .await
        .unwrap();
    let recipient = store.create_recipient("acme", None).await.unwrap();
    let token = issue_token(store.as_ref(), &secrets, &recipient, None)
        .await
        .unwrap();
    store.grant("letters", "acme").await.unwrap();

    // The recipient activates its link and reads the table.
    let link = activation_url(PUBLIC_URL, &token).unwrap();
    let (status, profile) = call(&app, "GET", link.strip_prefix(PUBLIC_URL).unwrap(), None).await;
    assert_eq!(status, StatusCode::OK, "{profile}");
    let profile: Value = serde_json::from_str(&profile).unwrap();
    let bearer = profile["bearerToken"].as_str().unwrap();

    let (status, shares) = call(&app, "GET", "/api/v1/delta-sharing/shares", Some(bearer)).await;
    assert_eq!(status, StatusCode::OK, "{shares}");
    let shares: Value = serde_json::from_str(&shares).unwrap();
    assert_eq!(shares["items"][0]["name"], "letters");

    let (status, lines) = call(
        &app,
        "POST",
        "/api/v1/delta-sharing/shares/letters/schemas/sales/tables/letters/query",
        Some(bearer),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{lines}");
    let files = lines
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .filter(|l| l.get("file").is_some())
        .count();
    assert!(files > 0, "expected file lines: {lines}");

    // No admin API is mounted without the `unity` feature.
    #[cfg(not(feature = "unity"))]
    assert_eq!(
        call(&app, "GET", "/api/2.1/unity-catalog/shares", None)
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}
