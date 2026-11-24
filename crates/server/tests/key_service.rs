//! Wrapping data keys through trestle's key service, against an in-process
//! secret server (`olai-secret-server`) on a Unix socket: startup, sealing and
//! opening, a local `kek-1` blob opening through its alias, rotation reaching a
//! running server, the rewrap sweep over all three sealed stores, retirement
//! through `min_decrypt`, and the DEK cache across a secret-server restart.
#![cfg(all(unix, feature = "sqlite", feature = "key-service"))]

use std::path::{Path, PathBuf};
use std::time::Duration;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use base64::Engine as _;
use olai_secret_server::{
    CallerId, Config as SecretServerConfig, KeyHandle, Keyring, Policy, serve,
};
use olai_store::{ObjectStore, ResourceName, SecretObjectReader};
use serde_json::{Value, json};
use tokio::sync::oneshot;
use tower::ServiceExt;
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::rotation::{SealedStore, sweep};
use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};
use unitycatalog_server::config::{
    AuthConfig, AuthMode, Backend, BearerTokenConfig, Config, ConfigValue, DekCacheConfig,
    EncryptionConfig, SqliteBackendConfig,
};
use unitycatalog_server::embed;
use unitycatalog_server::rest::admin::SealedObjects;
use unitycatalog_sharing::recipients::{
    SealedRecipientTokens, create_recipient, pending_activation,
};
use unitycatalog_sharing::{RecipientStore, SqliteSharingStore};
use unitycatalog_storage_access::testing::{credential, location};
use unitycatalog_storage_access::{SealedCredentials, SqliteStorageAccess, StorageAccessStore};

const KEY: &str = "uc-kek";
const CONSUMER: &str = "uc";
const TOKEN: &str = "test-token";

/// The material of keyring version `v`. Version 1 is also the local `kek-1`.
fn material(v: u32) -> Vec<u8> {
    vec![v as u8; 32]
}

/// A keyring with `versions`, `active` and `min_decrypt`; version 1, when
/// present, keeps the alias of the local KEK it was imported from.
fn keyring(versions: &[u32], active: u32, min_decrypt: Option<u32>) -> String {
    let b64 = base64::engine::general_purpose::STANDARD;
    let versions: serde_json::Map<String, Value> = versions
        .iter()
        .map(|v| (v.to_string(), json!(b64.encode(material(*v)))))
        .collect();
    let mut ring = json!({ "versions": versions, "active": active });
    if let Some(min) = min_decrypt {
        ring["min_decrypt"] = json!(min);
    }
    if versions.contains_key("1") {
        ring["aliases"] = json!({ "kek-1": 1 });
    }
    ring.to_string()
}

/// An in-process secret server serving the key `uc-kek` to the consumer `uc`.
struct KeyServer {
    _dir: tempfile::TempDir,
    socket: PathBuf,
    config: SecretServerConfig,
    policy: Policy,
    stop: Option<oneshot::Sender<()>>,
    task: Option<tokio::task::JoinHandle<()>>,
}

impl KeyServer {
    async fn start(keyring_json: &str) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let socket = dir.path().join(CONSUMER).join("sds.sock");
        let config = SecretServerConfig::from_yaml(&format!(
            "version: 1\nlisteners:\n  - socket: {}\n    consumer: {CONSUMER}\n    keys:\n      \
             - {{name: {KEY}, env: UC_KEK}}\n",
            socket.display()
        ))
        .unwrap();
        let ring = keyring_json.to_string();
        let policy = config
            .resolve(|name| (name == "UC_KEK").then(|| ring.clone().into()))
            .unwrap();
        let mut server = Self {
            _dir: dir,
            socket,
            config,
            policy,
            stop: None,
            task: None,
        };
        server.serve().await;
        server
    }

    /// Serve (again) on the socket, with the current keyring.
    async fn serve(&mut self) {
        let (stop, stopped) = oneshot::channel::<()>();
        let config = self.config.clone();
        let policy = self.policy.clone();
        self.task = Some(tokio::spawn(async move {
            serve(&config, policy, async {
                let _ = stopped.await;
            })
            .await
            .unwrap();
        }));
        self.stop = Some(stop);
        for _ in 0..200 {
            if tokio::net::UnixStream::connect(&self.socket).await.is_ok() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("secret server did not come up on {}", self.socket.display());
    }

    async fn shutdown(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(task) = self.task.take() {
            task.await.unwrap();
        }
    }

    fn handle(&self) -> KeyHandle {
        self.policy
            .key(&CallerId::new(CONSUMER), KEY)
            .unwrap()
            .clone()
    }

    /// Replace the keyring the running server holds; watching clients see it.
    fn rotate(&self, keyring_json: &str) {
        self.handle()
            .replace(Keyring::from_json(KEY, keyring_json.as_bytes()).unwrap())
            .unwrap();
    }

    fn socket(&self) -> &Path {
        &self.socket
    }

    fn encryption(&self) -> EncryptionConfig {
        EncryptionConfig::key_service(self.socket.to_str().unwrap(), KEY)
    }

    async fn encryptor(&self) -> EnvelopeEncryptor {
        self.encryption().build_encryptor().await.unwrap()
    }
}

/// Wait until `encryptor` seals under `kek_id`.
async fn await_active(encryptor: &EnvelopeEncryptor, kek_id: &str) {
    for _ in 0..500 {
        if encryptor.active_kek_id() == kek_id {
            return;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    panic!(
        "active KEK stayed {}, expected {kek_id}",
        encryptor.active_kek_id()
    );
}

fn local_kek_1() -> EnvelopeEncryptor {
    EnvelopeEncryptor::local(LocalKeyProvider::single("kek-1", material(1)).unwrap())
}

/// A migrated SQLite database file.
async fn sqlite(dir: &Path) -> sqlx::SqlitePool {
    let path = dir.join("uc.db");
    let pool = unitycatalog_sqlite::connect_pool(path.to_str().unwrap())
        .await
        .unwrap();
    unitycatalog_sqlite::unified_migrator()
        .run(&pool)
        .await
        .unwrap();
    pool
}

async fn sharing(dir: &Path) -> SqliteSharingStore {
    let path = dir.join("sharing.db");
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .unwrap();
    unitycatalog_sharing::sqlite::MIGRATOR
        .run(&pool)
        .await
        .unwrap();
    SqliteSharingStore::new(pool)
}

type Graph = unitycatalog_sqlite::SqliteGraphStore;

/// The three sealed stores over `pool` and `tokens`, sealing with `encryptor`.
struct Stores {
    graph: Graph,
    credentials: std::sync::Arc<SqliteStorageAccess>,
    tokens: std::sync::Arc<SqliteSharingStore>,
}

impl Stores {
    fn new(
        pool: &sqlx::SqlitePool,
        tokens: &SqliteSharingStore,
        encryptor: &EnvelopeEncryptor,
    ) -> Self {
        Self {
            graph: unitycatalog_sqlite::connect_graph(pool.clone(), encryptor.clone()),
            credentials: std::sync::Arc::new(SqliteStorageAccess::new(
                pool.clone(),
                encryptor.clone(),
            )),
            tokens: std::sync::Arc::new(tokens.clone()),
        }
    }

    fn sealed(&self, encryptor: &EnvelopeEncryptor) -> Vec<Box<dyn SealedStore>> {
        vec![
            Box::new(SealedObjects(self.graph.clone())),
            Box::new(SealedCredentials(self.credentials.clone())),
            Box::new(SealedRecipientTokens {
                store: self.tokens.clone(),
                encryptor: encryptor.clone(),
            }),
        ]
    }

    /// Seal one value in each store.
    async fn seed(&self, encryptor: &EnvelopeEncryptor) -> uuid::Uuid {
        let mut request = credential("cred", "arn:1");
        request.aws_iam_role = unitycatalog_common::models::credentials::v1::AwsIamRoleConfig {
            role_arn: "arn:1".into(),
            secret_access_key: Some("secret".into()),
            ..Default::default()
        }
        .into();
        self.credentials.create_credential(request).await.unwrap();
        self.credentials
            .create_external_location(location("loc", "s3://bucket/data", "cred"))
            .await
            .unwrap();
        create_recipient(self.tokens.as_ref(), encryptor, "acme", None, None)
            .await
            .unwrap();
        self.graph
            .create(
                ObjectLabel::Credential,
                &ResourceName::new(["graph-cred"]),
                Some(json!({ "name": "graph-cred", "aws_iam_role": { "role_arn": "arn:graph" } })),
                None,
                None,
            )
            .await
            .unwrap()
            .id
    }

    /// Open every seeded value in `pool` and `self.tokens` through `encryptor`.
    async fn open_all(
        &self,
        pool: &sqlx::SqlitePool,
        encryptor: &EnvelopeEncryptor,
        object: uuid::Uuid,
    ) {
        let store = SqliteStorageAccess::new(pool.clone(), encryptor.clone());
        let governed = store
            .governing_location(&"s3://bucket/data/x".parse().unwrap())
            .await
            .unwrap();
        assert_eq!(
            governed
                .credential
                .aws_iam_role
                .as_option()
                .unwrap()
                .secret_access_key
                .as_deref(),
            Some("secret")
        );

        let tokens = self.tokens.sealed_tokens(None, 10).await.unwrap();
        assert_eq!(tokens.len(), 1);
        assert!(
            pending_activation(encryptor, &tokens[0])
                .await
                .unwrap()
                .is_some()
        );

        let graph = unitycatalog_sqlite::connect_graph(pool.clone(), encryptor.clone());
        let full = graph.get_with_secrets(&object).await.unwrap();
        assert_eq!(
            full.properties.unwrap()["aws_iam_role"]["role_arn"],
            json!("arn:graph")
        );
    }
}

#[tokio::test]
async fn seals_and_opens_through_the_key_service() {
    let server = KeyServer::start(&keyring(&[1], 1, None)).await;
    let encryptor = server.encryptor().await;
    assert_eq!(encryptor.active_kek_id(), "uc-kek@v1");

    let sealed = encryptor.seal("name", b"value").await.unwrap();
    assert_eq!(
        olai_store::encryption::envelope_kek_id(&sealed).unwrap(),
        "uc-kek@v1"
    );
    assert_eq!(encryptor.open("name", &sealed).await.unwrap(), b"value");
    // Bound to its name.
    assert!(encryptor.open("other", &sealed).await.is_err());
}

#[tokio::test]
async fn startup_fails_clearly_without_a_usable_key_service() {
    // No socket.
    let err = EncryptionConfig::key_service("/nonexistent/uc/sds.sock", KEY)
        .build_encryptor()
        .await
        .unwrap_err();
    assert!(err.contains("does not exist"), "{err}");

    // A key the secret server doesn't list for this consumer.
    let server = KeyServer::start(&keyring(&[1], 1, None)).await;
    let err = EncryptionConfig::key_service(server.socket().to_str().unwrap(), "someone-elses")
        .build_encryptor()
        .await
        .unwrap_err();
    assert!(err.contains("refused key `someone-elses`"), "{err}");

    // The same through the server's startup.
    let mut config = http_config();
    config.backend = Backend::Sqlite(SqliteBackendConfig {
        path: ConfigValue::Value(":memory:".into()),
    });
    config.encryption = Some(EncryptionConfig::key_service(
        server.socket().to_str().unwrap(),
        "someone-elses",
    ));
    let err = embed::start(&config).await.err().unwrap();
    assert!(err.to_string().contains("refused key"), "{err}");
}

#[tokio::test]
async fn a_local_kek_1_value_opens_through_its_alias() {
    let dir = tempfile::tempdir().unwrap();
    let pool = sqlite(dir.path()).await;
    let tokens = sharing(dir.path()).await;
    let local = local_kek_1();
    let object = Stores::new(&pool, &tokens, &local).seed(&local).await;

    let server = KeyServer::start(&keyring(&[1], 1, None)).await;
    let encryptor = server.encryptor().await;
    let stores = Stores::new(&pool, &tokens, &encryptor);
    stores.open_all(&pool, &encryptor, object).await;

    // Still counted under the local id until a sweep moves them.
    for store in stores.sealed(&encryptor) {
        let usage = store.kek_usage().await.unwrap();
        assert_eq!(usage.count("kek-1"), 1, "{}: {usage:?}", store.name());
    }
}

/// The whole rotation, as the runbook has it: import the local KEK, rotate,
/// sweep, check the counts, retire.
#[tokio::test]
async fn rotation_sweep_and_retirement_across_all_three_stores() {
    let dir = tempfile::tempdir().unwrap();
    let pool = sqlite(dir.path()).await;
    let tokens = sharing(dir.path()).await;
    let local = local_kek_1();
    let object = Stores::new(&pool, &tokens, &local).seed(&local).await;

    let server = KeyServer::start(&keyring(&[1], 1, None)).await;
    let encryptor = server.encryptor().await;
    let stores = Stores::new(&pool, &tokens, &encryptor);
    // A value sealed through the service under v1 before the rotation.
    let probe = encryptor.seal("probe", b"v1").await.unwrap();

    // Rotate: v2 becomes active while the server runs.
    server.rotate(&keyring(&[1, 2], 2, None));
    await_active(&encryptor, "uc-kek@v2").await;
    let fresh = encryptor.seal("fresh", b"v2").await.unwrap();
    assert_eq!(
        olai_store::encryption::envelope_kek_id(&fresh).unwrap(),
        "uc-kek@v2"
    );

    // Sweep all three stores.
    let sealed = stores.sealed(&encryptor);
    let refs: Vec<&dyn SealedStore> = sealed.iter().map(|s| s.as_ref()).collect();
    let report = sweep(&refs).await;
    let names: Vec<&str> = report.iter().map(|s| s.store.as_str()).collect();
    assert_eq!(
        names,
        ["objects", "storage_credentials", "recipient_tokens"]
    );
    for entry in &report {
        assert!(entry.error.is_none(), "{entry:?}");
        assert_eq!(
            entry.before.as_ref().unwrap().count("kek-1"),
            1,
            "{entry:?}"
        );
        let done = entry.report.as_ref().unwrap();
        assert_eq!((done.rewrapped, done.already_active), (1, 0), "{entry:?}");
        assert!(done.failed.is_empty(), "{entry:?}");
        assert_eq!(done.kek_usage.count("uc-kek@v2"), 1, "{entry:?}");
        assert_eq!(done.kek_usage.not_under("uc-kek@v2"), 0, "{entry:?}");
    }

    // Running it again moves nothing.
    for entry in sweep(&refs).await {
        let done = entry.report.unwrap();
        assert_eq!(
            (done.rewrapped, done.already_active),
            (0, 1),
            "{}",
            entry.store
        );
    }

    // The probe still opens under v1 (and lands in the DEK cache) ...
    assert_eq!(encryptor.open("probe", &probe).await.unwrap(), b"v1");
    // ... until v1 is retired: min_decrypt refuses it, cache or not.
    server.rotate(&keyring(&[1, 2], 2, Some(2)));
    let mut refused = false;
    for _ in 0..500 {
        if encryptor.open("probe", &probe).await.is_err() {
            refused = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(
        refused,
        "a v1 value still opened after min_decrypt rose to 2"
    );
    // Everything swept opens under v2.
    stores.open_all(&pool, &encryptor, object).await;
    assert_eq!(encryptor.open("fresh", &fresh).await.unwrap(), b"v2");
}

#[tokio::test]
async fn the_dek_cache_rides_out_a_secret_server_restart() {
    let mut server = KeyServer::start(&keyring(&[1], 1, None)).await;
    let encryptor = server.encryptor().await;
    let sealed = encryptor.seal("name", b"value").await.unwrap();
    assert_eq!(encryptor.open("name", &sealed).await.unwrap(), b"value");

    // The secret server goes away (as on rotation, when it is recreated).
    server.shutdown().await;
    assert_eq!(encryptor.open("name", &sealed).await.unwrap(), b"value");

    // And comes back.
    server.serve().await;
    let other = encryptor.seal("other", b"again").await.unwrap();
    assert_eq!(encryptor.open("other", &other).await.unwrap(), b"again");
}

#[test]
fn the_dek_cache_is_on_unless_turned_off() {
    let Some(ks) = EncryptionConfig::key_service("s", KEY)
        .provider
        .and_then(|p| p.key_service)
    else {
        unreachable!()
    };
    assert_eq!(ks.dek_cache, DekCacheConfig::default());
}

// --- Over HTTP: the server's startup and its admin API. ---

fn http_config() -> Config {
    let mut config = Config {
        auth: AuthConfig {
            mode: AuthMode::Bearer,
            tokens: vec![BearerTokenConfig {
                principal: "admin".into(),
                token: ConfigValue::Value(TOKEN.into()),
            }],
            ..Default::default()
        },
        ..Default::default()
    };
    config.ui.serve = false;
    config
}

async fn send(
    router: &Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let request = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {TOKEN}"));
    let request = match body {
        Some(body) => request
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(body.to_string())),
        None => request.body(Body::empty()),
    }
    .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or_else(|_| json!(String::from_utf8_lossy(&bytes))),
    )
}

fn store<'a>(response: &'a Value, name: &str) -> &'a Value {
    response["stores"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["store"] == name)
        .unwrap_or_else(|| panic!("no store {name} in {response}"))
}

const USAGE: &str = "/api/admin/v1/encryption/kek-usage";
const REWRAP: &str = "/api/admin/v1/encryption/rewrap";

async fn create_credential(router: &Router, name: &str) {
    let (status, body) = send(
        router,
        "POST",
        "/api/2.1/unity-catalog/credentials",
        Some(json!({
            "name": name,
            "purpose": "STORAGE",
            "aws_iam_role": { "role_arn": format!("arn:aws:iam::123456789012:role/{name}") },
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
}

#[tokio::test]
async fn the_admin_api_follows_a_rotation_and_sweeps() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("uc.db");
    let path = path.to_str().unwrap();
    let server = KeyServer::start(&keyring(&[1], 1, None)).await;
    let mut config = http_config();
    config.backend = Backend::Sqlite(SqliteBackendConfig {
        path: ConfigValue::Value(path.into()),
    });
    config.encryption = Some(server.encryption());
    embed::migrate(http_config_with(&config)).await.unwrap();
    let router = embed::start(&config).await.unwrap().router;

    create_credential(&router, "one").await;
    create_credential(&router, "two").await;
    let (status, usage) = send(&router, "GET", USAGE, None).await;
    assert_eq!(status, StatusCode::OK, "{usage}");
    assert_eq!(usage["active_kek_id"], "uc-kek@v1");
    assert_eq!(
        store(&usage, "storage_credentials")["kek_usage"]["by_kek"],
        json!({ "uc-kek@v1": 2 })
    );
    assert_eq!(store(&usage, "objects")["kek_usage"]["by_kek"], json!({}));
    assert_eq!(usage["not_configured"], json!(["recipient_tokens"]));

    // Rotation reaches the running server without a restart.
    server.rotate(&keyring(&[1, 2], 2, None));
    let mut active = Value::Null;
    for _ in 0..500 {
        active = send(&router, "GET", USAGE, None).await.1["active_kek_id"].clone();
        if active == "uc-kek@v2" {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(active, "uc-kek@v2");
    create_credential(&router, "three").await;

    let (status, swept) = send(&router, "POST", REWRAP, None).await;
    assert_eq!(status, StatusCode::OK, "{swept}");
    assert_eq!(swept["active_kek_id"], "uc-kek@v2");
    let credentials = store(&swept, "storage_credentials");
    assert_eq!(
        credentials["before"]["by_kek"],
        json!({ "uc-kek@v1": 2, "uc-kek@v2": 1 })
    );
    assert_eq!(credentials["report"]["rewrapped"], 2);
    assert_eq!(credentials["report"]["already_active"], 1);
    assert_eq!(credentials["report"]["superseded"], 0);
    assert_eq!(credentials["report"]["failed"], json!([]));
    assert_eq!(
        credentials["report"]["kek_usage"]["by_kek"],
        json!({ "uc-kek@v2": 3 })
    );
    assert_eq!(swept["not_configured"], json!(["recipient_tokens"]));

    let (_, usage) = send(&router, "GET", USAGE, None).await;
    assert_eq!(
        store(&usage, "storage_credentials")["kek_usage"]["by_kek"],
        json!({ "uc-kek@v2": 3 })
    );

    // Unauthenticated callers are turned away.
    let request = Request::builder()
        .method("POST")
        .uri(REWRAP)
        .body(Body::empty())
        .unwrap();
    let response = router.clone().oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

/// `migrate` takes the config by value.
fn http_config_with(config: &Config) -> Config {
    let mut copy = http_config();
    copy.backend = Backend::Sqlite(SqliteBackendConfig {
        path: match &config.backend {
            Backend::Sqlite(b) => match &b.path {
                ConfigValue::Value(v) => ConfigValue::Value(v.clone()),
                ConfigValue::Environment(e) => {
                    ConfigValue::Environment(unitycatalog_server::config::EnvValue {
                        env: e.env.clone(),
                    })
                }
            },
            _ => unreachable!(),
        },
    });
    copy.encryption = Some(EncryptionConfig::dev_default());
    copy
}

/// A credential updated between the sweep's read and its write keeps the
/// update, sealed under the active version; the sweep counts it superseded.
#[tokio::test]
async fn a_concurrent_update_during_the_sweep_is_kept() {
    use unitycatalog_common::rotation::rows::{self, SealedRows};
    use unitycatalog_storage_access::{CredentialSecrets, CredentialUpdate, Error, Result};

    let dir = tempfile::tempdir().unwrap();
    let pool = sqlite(dir.path()).await;
    let server = KeyServer::start(&keyring(&[1], 1, None)).await;
    let encryptor = server.encryptor().await;
    let store = SqliteStorageAccess::new(pool.clone(), encryptor.clone());
    store
        .create_credential(credential("cred", "arn:1"))
        .await
        .unwrap();
    server.rotate(&keyring(&[1, 2], 2, None));
    await_active(&encryptor, "uc-kek@v2").await;

    /// Updates the credential right after the sweep reads its first page.
    struct Racing<'a, R> {
        rows: R,
        store: &'a SqliteStorageAccess,
    }
    #[async_trait::async_trait]
    impl<R: SealedRows<Error = Error>> SealedRows for Racing<'_, R> {
        type Error = Error;
        async fn page(&self, after: Option<&str>, limit: usize) -> Result<Vec<(String, Vec<u8>)>> {
            let page = self.rows.page(after, limit).await?;
            if after.is_none() && !page.is_empty() {
                let mut updated = credential("cred", "arn:updated");
                self.store
                    .update_credential(
                        "cred",
                        CredentialUpdate {
                            secrets: Some(CredentialSecrets::take(&mut updated)),
                            ..Default::default()
                        },
                    )
                    .await?;
            }
            Ok(page)
        }
        async fn replace(&self, id: &str, current: &[u8], fresh: &[u8]) -> Result<bool> {
            self.rows.replace(id, current, fresh).await
        }
    }

    let racing = Racing {
        rows: store.sealed_rows(),
        store: &store,
    };
    let report = rows::rewrap(&racing, &encryptor).await.unwrap();
    assert_eq!((report.rewrapped, report.superseded), (0, 1), "{report:?}");
    assert_eq!(report.kek_usage.count("uc-kek@v2"), 1);

    store
        .create_external_location(location("loc", "s3://b/", "cred"))
        .await
        .unwrap();
    let governed = store
        .governing_location(&"s3://b/x".parse().unwrap())
        .await
        .unwrap();
    assert_eq!(
        governed
            .credential
            .aws_iam_role
            .as_option()
            .unwrap()
            .role_arn,
        "arn:updated"
    );
}
