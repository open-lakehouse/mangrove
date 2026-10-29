//! End-to-end: a provider sets up shares and recipients through the admin API, a
//! recipient activates its token and reads a real Delta table through both
//! protocol surfaces — with local (static source) and delegated (catalog-vended)
//! storage access.

use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use async_trait::async_trait;
use axum::Router;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use tower::ServiceExt;
use unitycatalog_common::models::temporary_credentials::v1::TemporaryCredential;
use unitycatalog_credentials::local_path_credential;
use unitycatalog_sharing_server::Result;
use unitycatalog_sharing_server::config::{
    ConfigValue, EncryptionConfig, KeyConfig, KeyProviderKind,
};
use unitycatalog_sharing_server::embed::{self, Settings};
use unitycatalog_sharing_server::secrets::Secrets;
use unitycatalog_sharing_server::source::fixed::{StaticSource, StaticTable};
use unitycatalog_sharing_server::source::{
    AccessMode, CatalogSource, RegisteredSource, SourceRegistry, SourceTable, SourceVolume,
};
use unitycatalog_sharing_server::store::ShareStore;
use unitycatalog_sharing_server::store::memory::InMemoryStore;

const ADMIN: &str = "provider-secret";
const PUBLIC_URL: &str = "https://sharing.example.com";

fn table_location() -> String {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../sharing-api/tests/data/basic_partitioned")
        .canonicalize()
        .unwrap();
    url::Url::from_directory_path(dir).unwrap().to_string()
}

fn settings() -> Settings {
    Settings {
        public_url: PUBLIC_URL.to_string(),
        provider_tokens: vec![ADMIN.to_string()],
        presign_ttl: Duration::from_secs(300),
        aws_region: None,
    }
}

fn key(id: &str, byte: u8) -> KeyConfig {
    use base64::Engine as _;
    KeyConfig {
        id: id.to_string(),
        key: ConfigValue::Value(base64::engine::general_purpose::STANDARD.encode([byte; 32])),
    }
}

fn secrets(active: KeyConfig, retired: Vec<KeyConfig>) -> Secrets {
    Secrets::new(
        EncryptionConfig {
            provider: KeyProviderKind::Local,
            active,
            retired,
        }
        .build()
        .unwrap(),
    )
}

fn static_sources() -> SourceRegistry {
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

struct Client {
    app: Router,
}

impl Client {
    async fn call(
        &self,
        method: &str,
        uri: &str,
        token: Option<&str>,
        body: Option<Value>,
    ) -> (StatusCode, Value) {
        let mut req = Request::builder().method(method).uri(uri);
        if let Some(token) = token {
            req = req.header("authorization", format!("Bearer {token}"));
        }
        let req = match body {
            Some(body) => req
                .header("content-type", "application/json")
                .body(Body::from(body.to_string())),
            None => req.body(Body::empty()),
        }
        .unwrap();
        let resp = self.app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX)
            .await
            .unwrap();
        let value = serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            // NDJSON responses: an array of lines.
            Value::Array(
                bytes
                    .split(|b| *b == b'\n')
                    .filter(|l| !l.is_empty())
                    .map(|l| serde_json::from_slice(l).unwrap_or(Value::Null))
                    .collect(),
            )
        });
        (status, value)
    }

    async fn admin(&self, method: &str, path: &str, body: Option<Value>) -> Value {
        let (status, value) = self
            .call(
                method,
                &format!("/api/2.1/unity-catalog{path}"),
                Some(ADMIN),
                body,
            )
            .await;
        assert_eq!(status, StatusCode::OK, "{method} {path}: {value}");
        value
    }

    /// Activate `recipient`'s pending token, returning the bearer token.
    async fn activate(&self, recipient: &Value) -> String {
        let link = recipient["tokens"][0]["activation_url"].as_str().unwrap();
        let path = link
            .strip_prefix(PUBLIC_URL)
            .expect("link under the public url");
        let (status, profile) = self.call("GET", path, None, None).await;
        assert_eq!(status, StatusCode::OK, "{profile}");
        assert_eq!(
            profile["endpoint"],
            format!("{PUBLIC_URL}/api/v1/delta-sharing")
        );
        // An activation link works once.
        assert_eq!(
            self.call("GET", path, None, None).await.0,
            StatusCode::NOT_FOUND
        );
        profile["bearerToken"].as_str().unwrap().to_string()
    }

    /// Create share `share` holding `object`, and recipient `recipient` granted
    /// it; returns the recipient's bearer token.
    async fn provision(&self, share: &str, object: Value, recipient: &str) -> String {
        self.admin("POST", "/shares", Some(json!({"name": share})))
            .await;
        self.admin(
            "PATCH",
            &format!("/shares/{share}"),
            Some(json!({"updates": [{"action": "ADD", "data_object": object}]})),
        )
        .await;
        let created = self
            .admin(
                "POST",
                "/recipients",
                Some(json!({"name": recipient, "authentication_type": "TOKEN"})),
            )
            .await;
        let token = self.activate(&created).await;
        self.admin(
            "PATCH",
            &format!("/shares/{share}/permissions"),
            Some(json!({"changes": [{"principal": recipient, "add": ["SELECT"]}]})),
        )
        .await;
        token
    }

    async fn query(
        &self,
        token: &str,
        share: &str,
        schema: &str,
        table: &str,
        body: Value,
    ) -> (StatusCode, Vec<Value>) {
        let (status, lines) = self
            .call(
                "POST",
                &format!(
                    "/api/v1/delta-sharing/shares/{share}/schemas/{schema}/tables/{table}/query"
                ),
                Some(token),
                Some(body),
            )
            .await;
        let lines = match lines {
            Value::Array(lines) => lines,
            other => vec![other],
        };
        (status, lines)
    }
}

fn files(lines: &[Value]) -> Vec<&Value> {
    lines.iter().filter_map(|l| l.get("file")).collect()
}

fn app(store: Arc<dyn ShareStore>, secrets: Secrets, sources: SourceRegistry) -> Client {
    Client {
        app: embed::router(store, secrets, sources, settings()),
    }
}

#[tokio::test]
async fn provider_shares_a_table_with_a_recipient() {
    let client = app(
        Arc::new(InMemoryStore::new()),
        secrets(key("v1", 1), vec![]),
        static_sources(),
    );

    // The admin API requires a provider token.
    let (status, _) = client
        .call("GET", "/api/2.1/unity-catalog/shares", Some("wrong"), None)
        .await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    let token = client
        .provision(
            "sales",
            json!({
                "name": "demo.sales.letters",
                "data_object_type": "TABLE",
                "history_data_sharing_status": "ENABLED"
            }),
            "acme",
        )
        .await;

    // Discovery on both protocol surfaces.
    for prefix in ["/api/v1/delta-sharing", "/api/1.0/opensharing"] {
        let (status, shares) = client
            .call("GET", &format!("{prefix}/shares"), Some(&token), None)
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(shares["items"][0]["name"], "sales", "{shares}");
        let (_, tables) = client
            .call(
                "GET",
                &format!("{prefix}/shares/sales/all-tables"),
                Some(&token),
                None,
            )
            .await;
        assert_eq!(tables["items"][0]["schema"], "sales");
        assert_eq!(tables["items"][0]["name"], "letters");
    }

    // Query the latest version and time-travel to version 0.
    let (status, lines) = client
        .query(&token, "sales", "sales", "letters", json!({}))
        .await;
    assert_eq!(status, StatusCode::OK, "{lines:?}");
    assert_eq!(files(&lines).len(), 6);
    assert!(
        files(&lines)[0]["url"]
            .as_str()
            .unwrap()
            .starts_with("file://")
    );
    let (_, lines) = client
        .query(&token, "sales", "sales", "letters", json!({"version": 0}))
        .await;
    assert_eq!(files(&lines).len(), 3);

    // Temporary table credentials (Open Sharing).
    let (status, credentials) = client
        .call(
            "POST",
            "/api/1.0/opensharing/shares/sales/schemas/sales/tables/letters/temporary-table-credentials",
            Some(&token),
            None,
        )
        .await;
    assert_eq!(status, StatusCode::OK, "{credentials}");
    assert_eq!(credentials["url"], table_location());

    // Revoking the grant hides the share.
    client
        .admin(
            "PATCH",
            "/shares/sales/permissions",
            Some(json!({"changes": [{"principal": "acme", "remove": ["SELECT"]}]})),
        )
        .await;
    let (_, shares) = client
        .call("GET", "/api/v1/delta-sharing/shares", Some(&token), None)
        .await;
    assert!(
        shares["items"].as_array().is_none_or(Vec::is_empty),
        "{shares}"
    );
    let (status, _) = client
        .query(&token, "sales", "sales", "letters", json!({}))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn history_is_shared_only_when_enabled() {
    let client = app(
        Arc::new(InMemoryStore::new()),
        secrets(key("v1", 1), vec![]),
        static_sources(),
    );
    let token = client
        .provision(
            "sales",
            json!({"name": "demo.sales.letters", "data_object_type": "TABLE"}),
            "acme",
        )
        .await;
    let (status, _) = client
        .query(&token, "sales", "sales", "letters", json!({}))
        .await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = client
        .query(&token, "sales", "sales", "letters", json!({"version": 0}))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn schemas_expand_to_their_tables() {
    let client = app(
        Arc::new(InMemoryStore::new()),
        secrets(key("v1", 1), vec![]),
        static_sources(),
    );
    let token = client
        .provision(
            "everything",
            json!({"name": "demo.sales", "data_object_type": "SCHEMA", "shared_as": "shared"}),
            "acme",
        )
        .await;
    let (_, tables) = client
        .call(
            "GET",
            "/api/v1/delta-sharing/shares/everything/schemas/shared/tables",
            Some(&token),
            None,
        )
        .await;
    assert_eq!(tables["items"][0]["name"], "letters", "{tables}");
    let (status, lines) = client
        .query(&token, "everything", "shared", "letters", json!({}))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(files(&lines).len(), 6);
}

#[tokio::test]
async fn rotating_a_token_invalidates_the_old_one() {
    let client = app(
        Arc::new(InMemoryStore::new()),
        secrets(key("v1", 1), vec![]),
        static_sources(),
    );
    let old = client
        .provision(
            "sales",
            json!({"name": "demo.sales.letters", "data_object_type": "TABLE"}),
            "acme",
        )
        .await;
    let rotated = client
        .admin("POST", "/recipients/acme/rotate-token", None)
        .await;
    let new = client.activate(&rotated).await;
    let shares = |token: String| {
        let client = &client;
        async move {
            client
                .call("GET", "/api/v1/delta-sharing/shares", Some(&token), None)
                .await
                .0
        }
    };
    assert_eq!(shares(old).await, StatusCode::UNAUTHORIZED);
    assert_eq!(shares(new).await, StatusCode::OK);
}

/// A catalog that vends (local) credentials and counts how often.
struct VendingCatalog {
    location: String,
    vends: AtomicUsize,
}

#[async_trait]
impl CatalogSource for VendingCatalog {
    async fn get_table(&self, full_name: &str) -> Result<SourceTable> {
        Ok(SourceTable {
            full_name: full_name.to_string(),
            name: full_name.rsplit('.').next().unwrap().to_string(),
            id: Some("11111111-1111-1111-1111-111111111111".to_string()),
            location: self.location.clone(),
            format: unitycatalog_sharing_api::catalog::DataFormat::Delta,
        })
    }

    async fn list_tables(&self, _catalog: &str, _schema: &str) -> Result<Vec<SourceTable>> {
        Ok(Vec::new())
    }

    async fn get_volume(&self, _full_name: &str) -> Result<SourceVolume> {
        Err(unitycatalog_sharing_server::Error::NotFound)
    }

    async fn vend_table(&self, table: &SourceTable) -> Result<TemporaryCredential> {
        self.vends.fetch_add(1, Ordering::SeqCst);
        let mut credential = local_path_credential(&table.location);
        credential.expiration_time = chrono::Utc::now().timestamp_millis() + 3_600_000;
        Ok(credential)
    }
}

#[tokio::test]
async fn delegated_access_uses_catalog_vended_credentials() {
    let catalog = Arc::new(VendingCatalog {
        location: table_location(),
        vends: AtomicUsize::new(0),
    });
    let sources = SourceRegistry::new(vec![RegisteredSource {
        name: "uc".to_string(),
        catalogs: vec!["*".to_string()],
        access: AccessMode::Delegated,
        source: catalog.clone(),
    }]);
    // No storage credentials are stored: everything comes from the catalog.
    let store = Arc::new(InMemoryStore::new());
    let client = app(store.clone(), secrets(key("v1", 1), vec![]), sources);
    let token = client
        .provision(
            "sales",
            json!({"name": "main.sales.letters", "data_object_type": "TABLE"}),
            "acme",
        )
        .await;
    for _ in 0..2 {
        let (status, lines) = client
            .query(&token, "sales", "sales", "letters", json!({}))
            .await;
        assert_eq!(status, StatusCode::OK, "{lines:?}");
        assert_eq!(files(&lines).len(), 6);
    }
    assert!(store.list_credentials().await.unwrap().is_empty());
    // The vended credential is cached until close to its expiry.
    assert_eq!(catalog.vends.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn rotating_keys_rewraps_stored_secrets() {
    let store: Arc<dyn ShareStore> = Arc::new(InMemoryStore::new());

    // Provision under key v1: a stored credential and a pending recipient token.
    let v1 = app(
        store.clone(),
        secrets(key("v1", 1), vec![]),
        static_sources(),
    );
    v1.admin(
        "POST",
        "/credentials",
        Some(json!({
            "name": "storage",
            "azure_storage_key": {"account_name": "acct", "account_key": "a2V5"}
        })),
    )
    .await;
    let recipient = v1
        .admin(
            "POST",
            "/recipients",
            Some(json!({"name": "acme", "authentication_type": "TOKEN"})),
        )
        .await;

    // Rotate: v2 becomes active, v1 is retired.
    let rotating = secrets(key("v2", 2), vec![key("v1", 1)]);
    assert_eq!(
        embed::rotate_keys(store.as_ref(), &rotating).await.unwrap(),
        (1, 1)
    );
    assert_eq!(
        embed::rotate_keys(store.as_ref(), &rotating).await.unwrap(),
        (0, 0),
        "everything is already under v2"
    );

    // With v1 gone entirely, the pending token still activates.
    let v2 = app(store, secrets(key("v2", 2), vec![]), static_sources());
    let token = v2.activate(&recipient).await;
    assert!(!token.is_empty());
}
