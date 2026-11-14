//! The socket-less server ([`embed::start`]) driven through `oneshot`, as a host
//! such as a Neon Function's Node-API addon serves it: bearer authentication,
//! the startup schema check, and refusing surfaces the build doesn't include.
#![cfg(feature = "sqlite")]

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode, header};
use tower::ServiceExt;
use unitycatalog_server::config::{
    AuthConfig, AuthMode, Backend, BearerTokenConfig, Config, ConfigValue, SqliteBackendConfig,
};
use unitycatalog_server::embed;

const TOKEN: &str = "test-token";

/// A config with bearer auth for `admin` over the SQLite database at `path`.
fn config(path: &str) -> Config {
    let mut config = Config {
        backend: Backend::Sqlite(SqliteBackendConfig {
            path: ConfigValue::Value(path.to_string()),
        }),
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

/// A durable config also needs a KEK.
fn durable_config(path: &str) -> Config {
    let mut config = config(path);
    config.encryption = Some(unitycatalog_server::config::EncryptionConfig::dev_default());
    config
}

async fn send(
    router: &Router,
    method: &str,
    uri: &str,
    token: Option<&str>,
    body: Option<&str>,
) -> (StatusCode, String) {
    let mut request = Request::builder().method(method).uri(uri);
    if let Some(token) = token {
        request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
    }
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
    (status, String::from_utf8_lossy(&bytes).into_owned())
}

#[tokio::test]
async fn serves_the_api_behind_bearer_auth() {
    let service = embed::start(&config(":memory:")).await.unwrap();
    let router = service.router;

    // Operational endpoints sit outside the authentication layer.
    let (status, body) = send(&router, "GET", "/health", None, None).await;
    assert_eq!((status, body.as_str()), (StatusCode::OK, "OK"));

    let catalogs = "/api/2.1/unity-catalog/catalogs";
    let create = Some(r#"{"name":"embedded"}"#);
    let (status, _) = send(&router, "POST", catalogs, None, create).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = send(&router, "GET", catalogs, Some("wrong"), None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);

    // An authenticated request reaches the handlers and the migrated store.
    let (status, body) = send(&router, "GET", catalogs, Some(TOKEN), None).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let (status, body) = send(
        &router,
        "GET",
        &format!("{catalogs}/missing"),
        Some(TOKEN),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::NOT_FOUND, "{body}");
}

#[tokio::test]
async fn refuses_an_unmigrated_database_until_migrated() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("uc.db");
    let path = path.to_str().unwrap();

    let err = embed::start(&durable_config(path)).await.err().unwrap();
    assert!(err.to_string().contains("pending"), "{err}");

    embed::migrate(durable_config(path)).await.unwrap();
    let service = embed::start(&durable_config(path)).await.unwrap();
    let (status, _) = send(&service.router, "GET", "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn bearer_auth_without_tokens_is_a_startup_error() {
    let mut config = config(":memory:");
    config.auth.tokens.clear();
    let err = embed::start(&config).await.err().unwrap();
    assert!(err.to_string().contains("auth.tokens"), "{err}");
}

#[cfg(not(feature = "storage-proxy"))]
#[tokio::test]
async fn a_surface_that_is_not_compiled_in_is_a_startup_error() {
    let mut config = config(":memory:");
    config.storage_proxy.enabled = true;
    let err = embed::start(&config).await.err().unwrap();
    assert!(err.to_string().contains("`storage-proxy` feature"), "{err}");
}

/// `olai-auth` from config: requests without a valid token are answered before
/// the identity database is ever reached (its pool connects lazily).
#[cfg(feature = "olai-auth")]
#[tokio::test]
async fn olai_auth_rejects_requests_without_a_valid_token() {
    use unitycatalog_server::config::OlaiAuthConfig;

    let mut config = config(":memory:");
    config.auth = AuthConfig {
        mode: AuthMode::OlaiAuth,
        olai: Some(OlaiAuthConfig {
            issuer: ConfigValue::Value("https://auth.example.com/neondb/auth".into()),
            jwks_url: ConfigValue::Value("http://127.0.0.1:1/jwks.json".into()),
            audiences: vec![ConfigValue::Value("https://uc.example.com".into())],
            accept_session_tokens: false,
            api_keys: true,
            authz_schema: None,
            users_schema: None,
            // Nothing listens there; no request below gets far enough to connect.
            database_url: Some(ConfigValue::Value("postgres://127.0.0.1:1/neondb".into())),
            resource_metadata_url: None,
            resource: Some("https://uc.example.com".into()),
        }),
        ..Default::default()
    };
    let router = embed::start(&config).await.unwrap().router;

    let (status, _) = send(&router, "GET", "/health", None, None).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = send(
        &router,
        "GET",
        "/.well-known/oauth-protected-resource",
        None,
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let catalogs = "/api/2.1/unity-catalog/catalogs";
    for token in [None, Some("garbage"), Some("a.b.c")] {
        let (status, _) = send(&router, "GET", catalogs, token, None).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{token:?}");
    }
}

/// `auth.mode: olai-auth` needs its `auth.olai` block, and an identity database
/// when the backend isn't Postgres.
#[cfg(feature = "olai-auth")]
#[tokio::test]
async fn olai_auth_without_its_config_is_a_startup_error() {
    let mut config = config(":memory:");
    config.auth = AuthConfig {
        mode: AuthMode::OlaiAuth,
        ..Default::default()
    };
    let err = embed::start(&config).await.err().unwrap();
    assert!(err.to_string().contains("auth.olai"), "{err}");
}

#[cfg(not(feature = "olai-auth"))]
#[tokio::test]
async fn olai_auth_that_is_not_compiled_in_is_a_startup_error() {
    let mut config = config(":memory:");
    config.auth = AuthConfig {
        mode: AuthMode::OlaiAuth,
        ..Default::default()
    };
    let err = embed::start(&config).await.err().unwrap();
    assert!(err.to_string().contains("`olai-auth` feature"), "{err}");
}

/// The authorizer policy denies every request without a verified caller, so it
/// refuses to start without `auth.mode: olai-auth`.
#[tokio::test]
async fn the_authorizer_policy_needs_olai_auth() {
    use unitycatalog_server::config::{AuthorizerConfig, PolicyConfig};

    let mut config = config(":memory:");
    config.policy = PolicyConfig::Authorizer(AuthorizerConfig {
        url: ConfigValue::Value("http://127.0.0.1:1".into()),
        token: ConfigValue::Value("secret".into()),
        cache_ttl_secs: None,
        timeout_ms: None,
    });
    let err = embed::start(&config).await.err().unwrap();
    #[cfg(feature = "olai-auth")]
    assert!(err.to_string().contains("auth.mode: olai-auth"), "{err}");
    #[cfg(not(feature = "olai-auth"))]
    assert!(err.to_string().contains("`olai-auth` feature"), "{err}");
}
