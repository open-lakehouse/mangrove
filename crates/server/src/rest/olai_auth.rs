//! `auth.mode: olai-auth`: verified callers through [`olai_auth`].
//!
//! A request presents `Authorization: Bearer` with an EdDSA JWT of the configured
//! issuer (Neon Auth) or an `olk_` API key of a service account. The verified
//! [`olai_auth::Principal`] goes into the request extensions, where
//! [`RequestContext::from_extensions`](crate::api::RequestContext::from_extensions)
//! picks it up for the policy and the Delta API backend.
//!
//! Whether a user is an admin or banned comes from Neon Auth's `user` table
//! ([`PgNeonUserStatus`]), never from token claims.
//!
//! With `auth.olai.trusted-compute`, a request may also carry the workload token
//! of the compute it runs on (`X-Olai-Actor-Token`), which becomes the
//! principal's [`actor`](olai_auth::Principal::actor). See
//! [`TrustedComputeConfig`](crate::config::TrustedComputeConfig).

use std::time::Duration;

use axum::Router;
use olai_auth::{
    ApiKeyStore, Authenticator, BoxError, HttpKeySource, JwtActorVerifier, JwtVerifier, KeySource,
    PgApiKeyStore, ProtectedResourceMetadata, UserStatus, UserStatusStore, async_trait, auth_layer,
};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

use crate::config::ResolvedOlaiAuth;

/// The connections the identity pool keeps open at most. Each request looks up
/// at most one user or API key, and the results are cached.
const MAX_IDENTITY_CONNECTIONS: u32 = 4;

/// How long a request waits for an identity-database connection before it is
/// answered with a 503.
const IDENTITY_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// A lazy pool on the identity database at `url` (`auth.olai.database-url`).
pub(crate) fn identity_pool(url: &str) -> Result<PgPool, String> {
    PgPoolOptions::new()
        .max_connections(MAX_IDENTITY_CONNECTIONS)
        .acquire_timeout(IDENTITY_ACQUIRE_TIMEOUT)
        .connect_lazy(url)
        .map_err(|e| format!("auth.olai.database-url: {e}"))
}

/// The `olai-auth` layer over `router`, plus the public protected-resource
/// metadata document when [`resource`](ResolvedOlaiAuth::resource) is set.
pub(crate) fn layer(router: Router, config: &ResolvedOlaiAuth) -> Result<Router, String> {
    let pool = identity_pool(&config.database_url)?;
    let api_keys = config
        .api_keys
        .then(|| PgApiKeyStore::new(pool.clone()).with_schema(&config.authz_schema));
    let authenticator = build_authenticator(
        config,
        HttpKeySource::new(&config.jwks_url),
        |url: &str| HttpKeySource::new(url),
        PgNeonUserStatus::new(pool, &config.users_schema),
        api_keys,
    )?;
    Ok(with_authenticator(router, config, authenticator))
}

/// `router` behind `authenticator`, with the metadata document merged outside it.
pub(crate) fn with_authenticator(
    router: Router,
    config: &ResolvedOlaiAuth,
    authenticator: Authenticator,
) -> Router {
    let protected = auth_layer(router, authenticator);
    match &config.resource {
        Some(resource) => protected.merge(
            ProtectedResourceMetadata::new(resource.clone(), config.issuer.clone()).router(),
        ),
        None => protected,
    }
}

/// Builds the authenticator from the resolved config and its stores.
///
/// Tokens of [`issuer`](ResolvedOlaiAuth::issuer) are accepted for any of the
/// configured audiences. With
/// [`accept_session_tokens`](ResolvedOlaiAuth::accept_session_tokens), Neon Auth
/// session tokens are accepted too: their `iss` and `aud` are both the origin of
/// the issuer URL.
///
/// With [`trusted_compute`](ResolvedOlaiAuth::trusted_compute), workload tokens
/// of its issuers are accepted as actor credentials, with keys from
/// `actor_keys(jwks_url)`, and its rules decide which actors are trusted compute.
pub(crate) fn build_authenticator<K, A>(
    config: &ResolvedOlaiAuth,
    keys: K,
    actor_keys: impl Fn(&str) -> A,
    user_status: impl UserStatusStore,
    api_keys: Option<impl ApiKeyStore>,
) -> Result<Authenticator, String>
where
    K: KeySource + Clone,
    A: KeySource,
{
    let mut access = JwtVerifier::new(&config.issuer, keys.clone());
    for audience in &config.audiences {
        access = access.with_audience(audience);
    }

    let mut builder = Authenticator::builder(user_status);
    if config.accept_session_tokens {
        let origin = origin(&config.issuer)?;
        if origin == config.issuer {
            // The issuer is an origin already: the same verifier also takes the
            // origin as an audience.
            access = access.with_audience(origin);
        } else {
            builder = builder.issuer(JwtVerifier::new(&origin, keys).with_audience(&origin));
        }
    }
    builder = builder.issuer(access);
    if let Some(store) = api_keys {
        builder = builder.api_keys(store);
    }
    if let Some(url) = &config.resource_metadata_url {
        builder = builder.resource_metadata_url(url);
    }
    if let Some(trusted) = &config.trusted_compute {
        let verifiers = trusted.issuers.iter().map(|issuer| {
            issuer.audiences.iter().fold(
                JwtVerifier::new(&issuer.issuer, actor_keys(&issuer.jwks_url)),
                |verifier, audience| verifier.with_audience(audience),
            )
        });
        let actors = JwtActorVerifier::new(verifiers)
            .map_err(|e| format!("auth.olai.trusted-compute: {e}"))?
            .with_max_lifetime(trusted.max_lifetime);
        builder = builder
            .actor_verifier(actors)
            .trusted_compute(trusted.rules.clone());
    }
    builder.build().map_err(|e| format!("auth.olai: {e}"))
}

/// The origin (`scheme://host[:port]`) of `url`.
fn origin(url: &str) -> Result<String, String> {
    let parsed = url::Url::parse(url).map_err(|e| format!("auth.olai.issuer `{url}`: {e}"))?;
    match parsed.origin() {
        origin @ url::Origin::Tuple(..) => Ok(origin.ascii_serialization()),
        url::Origin::Opaque(_) => Err(format!("auth.olai.issuer `{url}` has no origin")),
    }
}

/// A [`UserStatusStore`] over Neon Auth's user table (`neon_auth."user"` by
/// default).
///
/// A user is an admin when their `role` is (or lists) `admin`, and banned while
/// `banned` is set and `banExpires` is unset or in the future. A missing user is
/// unknown, so their token is rejected. The issuer is not consulted: one server
/// trusts one Neon Auth instance, whose users are all in this table.
///
/// The connecting role needs `SELECT` on `id`, `role`, `banned` and `banExpires`,
/// which luminescent-lakehouse's `authz_reader` holds.
#[derive(Debug, Clone)]
pub struct PgNeonUserStatus {
    pool: PgPool,
    query: String,
}

impl PgNeonUserStatus {
    /// Reads the `user` table of `schema` through `pool`.
    pub fn new(pool: PgPool, schema: &str) -> Self {
        Self {
            pool,
            query: user_status_query(schema),
        }
    }
}

/// The status lookup against `schema`, quoted as an identifier.
fn user_status_query(schema: &str) -> String {
    let schema = format!("\"{}\"", schema.replace('"', "\"\""));
    format!(
        "SELECT role, \
                COALESCE(banned, false) AND (\"banExpires\" IS NULL OR \"banExpires\" > now()) \
         FROM {schema}.\"user\" WHERE id::text = $1"
    )
}

#[async_trait]
impl UserStatusStore for PgNeonUserStatus {
    async fn status(&self, _issuer: &str, sub: &str) -> Result<Option<UserStatus>, BoxError> {
        let row: Option<(Option<String>, bool)> = sqlx::query_as(&self.query)
            .bind(sub)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|(role, banned)| UserStatus::from_role(role.as_deref(), banned)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_of_the_auth_url() {
        assert_eq!(
            origin("https://ep-x.neonauth.example.com/neondb/auth").unwrap(),
            "https://ep-x.neonauth.example.com"
        );
        assert_eq!(
            origin("http://localhost:4000").unwrap(),
            "http://localhost:4000"
        );
        assert!(origin("not a url").is_err());
    }

    #[test]
    fn quotes_the_users_schema() {
        assert!(user_status_query("neon_auth").contains(r#"FROM "neon_auth"."user""#));
        assert!(user_status_query(r#"a"b"#).contains(r#"FROM "a""b"."user""#));
    }
}

/// The layer in front of the real REST router, with in-memory keys and stores.
#[cfg(all(test, feature = "memory"))]
mod router_tests {
    use std::sync::{Arc, Mutex};

    use axum::body::Body;
    use axum::http::{Request, StatusCode, header};
    use base64::Engine as _;
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use ed25519_dalek::{Signer as _, SigningKey};
    use olai_auth::{ApiKeyRecord, PushKeySource, api_key_digest};
    use serde_json::{Value, json};
    use tower::ServiceExt as _;
    use unitycatalog_common::services::encryption::{EnvelopeEncryptor, LocalKeyProvider};

    use super::*;
    use crate::api::RequestContext;
    use crate::memory::InMemoryResourceStore;
    use crate::policy::{Decision, Facts, Policy, Securable, ServerAction};
    use crate::run::build_rest_router;
    use crate::services::ServerHandler;

    const ISSUER: &str = "https://auth.example.com/neondb/auth";
    const ORIGIN: &str = "https://auth.example.com";
    const AUDIENCE: &str = "https://uc.example.com";
    const USER: &str = "8c1d0f5e-3b7a-4c55-9f40-2d6f1e9b7a10";
    const BANNED: &str = "00000000-0000-0000-0000-00000000dead";
    const API_KEY: &str = concat!(
        "olk_0123456789abcdef_",
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    );

    fn signing_key() -> SigningKey {
        SigningKey::from_bytes(&[7; 32])
    }

    fn keys() -> PushKeySource {
        let keys = PushKeySource::new();
        let jwk = json!({
            "kty": "OKP", "crv": "Ed25519", "alg": "EdDSA", "kid": "k1",
            "x": URL_SAFE_NO_PAD.encode(signing_key().verifying_key().as_bytes()),
        });
        keys.set_jwks(json!({ "keys": [jwk] }).to_string().as_bytes())
            .unwrap();
        keys
    }

    fn now() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    fn token(claims: Value) -> String {
        sign(&claims, &signing_key())
    }

    fn sign(claims: &Value, key: &SigningKey) -> String {
        let header = json!({ "alg": "EdDSA", "kid": "k1", "typ": "at+jwt" });
        let input = format!(
            "{}.{}",
            URL_SAFE_NO_PAD.encode(header.to_string()),
            URL_SAFE_NO_PAD.encode(claims.to_string())
        );
        let signature = key.sign(input.as_bytes());
        format!("{input}.{}", URL_SAFE_NO_PAD.encode(signature.to_bytes()))
    }

    fn claims(iss: &str, aud: &str, sub: &str) -> Value {
        json!({ "iss": iss, "aud": [aud], "sub": sub, "iat": now() - 10, "exp": now() + 600 })
    }

    struct Users;

    #[async_trait]
    impl UserStatusStore for Users {
        async fn status(&self, _iss: &str, sub: &str) -> Result<Option<UserStatus>, BoxError> {
            Ok(match sub {
                USER => Some(UserStatus::from_role(Some("user"), false)),
                BANNED => Some(UserStatus::from_role(None, true)),
                _ => None,
            })
        }
    }

    struct Keys;

    #[async_trait]
    impl ApiKeyStore for Keys {
        async fn lookup(&self, id: &str) -> Result<Option<ApiKeyRecord>, BoxError> {
            Ok((id == "0123456789abcdef").then(|| ApiKeyRecord {
                sha256: api_key_digest(API_KEY),
                account_id: "sa-1".into(),
                account_name: "airflow".into(),
                scopes: vec![],
            }))
        }
    }

    /// Allows everything and records the verified caller of each decision.
    #[derive(Default)]
    struct Recording(Mutex<Vec<Option<olai_auth::Principal>>>);

    #[async_trait]
    impl Policy<RequestContext> for Recording {
        async fn authorize(
            &self,
            _action: ServerAction,
            _securable: &Securable,
            _facts: &Facts,
            cx: &RequestContext,
        ) -> crate::Result<Decision> {
            self.0
                .lock()
                .unwrap()
                .push(cx.verified_principal().cloned());
            Ok(Decision::Allow)
        }
    }

    fn config(accept_session_tokens: bool) -> ResolvedOlaiAuth {
        ResolvedOlaiAuth {
            issuer: ISSUER.into(),
            jwks_url: "unused".into(),
            audiences: vec![AUDIENCE.into()],
            accept_session_tokens,
            api_keys: true,
            authz_schema: "authz".into(),
            users_schema: "neon_auth".into(),
            database_url: "unused".into(),
            resource_metadata_url: Some(format!("{AUDIENCE}/.well-known/oauth-protected-resource")),
            resource: Some(AUDIENCE.into()),
            trusted_compute: None,
        }
    }

    /// The issuer of the workloads' tokens, and the one trusted compute it covers.
    const WORKLOAD_ISSUER: &str = "https://oidc.cluster.example.com";
    const SPARK: &str = "system:serviceaccount:etl:spark";

    fn workload_key() -> SigningKey {
        SigningKey::from_bytes(&[9; 32])
    }

    fn workload_keys(_jwks_url: &str) -> PushKeySource {
        let keys = PushKeySource::new();
        let jwk = json!({
            "kty": "OKP", "crv": "Ed25519", "alg": "EdDSA", "kid": "k1",
            "x": URL_SAFE_NO_PAD.encode(workload_key().verifying_key().as_bytes()),
        });
        keys.set_jwks(json!({ "keys": [jwk] }).to_string().as_bytes())
            .unwrap();
        keys
    }

    fn workload_token(sub: &str) -> String {
        sign(&claims(WORKLOAD_ISSUER, AUDIENCE, sub), &workload_key())
    }

    fn trusting_spark() -> ResolvedOlaiAuth {
        let yaml = format!(
            "issuers: [{{ issuer: {WORKLOAD_ISSUER}, jwks-url: unused }}]\n\
             compute: [{{ id: spark, match: {{ exact: \"{WORKLOAD_ISSUER}#{SPARK}\" }} }}]\n"
        );
        let trusted: crate::config::TrustedComputeConfig = serde_yml::from_str(&yaml).unwrap();
        ResolvedOlaiAuth {
            trusted_compute: Some(trusted.resolve(ISSUER, &[AUDIENCE.into()]).unwrap()),
            ..config(false)
        }
    }

    fn app(policy: Arc<Recording>, accept_session_tokens: bool) -> Router {
        app_with(policy, config(accept_session_tokens))
    }

    fn app_with(policy: Arc<Recording>, config: ResolvedOlaiAuth) -> Router {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("test", vec![0x42; 32]).unwrap());
        let store = Arc::new(InMemoryResourceStore::new(encryptor));
        let handler = ServerHandler::try_new_tokio(policy, store).unwrap();
        let authenticator =
            build_authenticator(&config, keys(), workload_keys, Users, Some(Keys)).unwrap();
        with_authenticator(build_rest_router(handler), &config, authenticator)
    }

    async fn get(app: &Router, uri: &str, token: Option<&str>) -> axum::response::Response {
        get_from(app, uri, token, None).await
    }

    async fn get_from(
        app: &Router,
        uri: &str,
        token: Option<&str>,
        actor: Option<&str>,
    ) -> axum::response::Response {
        let mut request = Request::builder().uri(uri);
        if let Some(token) = token {
            request = request.header(header::AUTHORIZATION, format!("Bearer {token}"));
        }
        if let Some(actor) = actor {
            request = request.header(olai_auth::ACTOR_TOKEN_HEADER, actor);
        }
        app.clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    const CATALOG: &str = "/api/2.1/unity-catalog/catalogs/main";
    const DELTA_CONFIG: &str =
        "/api/2.1/unity-catalog/delta/v1/config?catalog=main&protocol-versions=1.0";

    #[tokio::test]
    async fn no_token_is_challenged() {
        let policy = Arc::new(Recording::default());
        let response = get(&app(policy.clone(), false), CATALOG, None).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let challenge = response.headers()[header::WWW_AUTHENTICATE]
            .to_str()
            .unwrap();
        assert!(challenge.starts_with("Bearer"), "{challenge}");
        assert!(challenge.contains("resource_metadata="), "{challenge}");
        assert!(policy.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn rejected_tokens_are_401() {
        let policy = Arc::new(Recording::default());
        let app = app(policy.clone(), false);
        let other_key = sign(
            &claims(ISSUER, AUDIENCE, USER),
            &SigningKey::from_bytes(&[8; 32]),
        );
        for token in [
            "garbage".to_string(),
            other_key,
            token(claims("https://other.example.com", AUDIENCE, USER)),
            token(claims(ISSUER, "https://other.example.com", USER)),
            token(claims(ISSUER, AUDIENCE, "unknown-user")),
            "olk_0123456789abcdef_0000000000000000000000000000000000000000000000000000000000000000"
                .to_string(),
        ] {
            let response = get(&app, CATALOG, Some(&token)).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{token}");
        }
        assert!(policy.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_banned_user_is_forbidden() {
        let policy = Arc::new(Recording::default());
        let token = token(claims(ISSUER, AUDIENCE, BANNED));
        let response = get(&app(policy.clone(), false), CATALOG, Some(&token)).await;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert!(policy.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn a_valid_jwt_reaches_the_policy_and_the_delta_api() {
        let policy = Arc::new(Recording::default());
        let app = app(policy.clone(), false);
        let token = token(claims(ISSUER, AUDIENCE, USER));

        // Allowed, then not found: the catalog doesn't exist.
        let response = get(&app, CATALOG, Some(&token)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let response = get(&app, DELTA_CONFIG, Some(&token)).await;
        assert_ne!(response.status(), StatusCode::UNAUTHORIZED);

        let seen = policy.0.lock().unwrap();
        assert_eq!(seen.len(), 2, "{seen:?}");
        for principal in seen.iter() {
            let principal = principal.as_ref().expect("a verified principal");
            assert_eq!(principal.subject(), USER);
            assert_eq!(principal.issuer, ISSUER);
            assert_eq!(principal.auth_method, olai_auth::AuthMethod::Jwt);
        }
    }

    #[tokio::test]
    async fn an_api_key_authenticates_a_service_account() {
        let policy = Arc::new(Recording::default());
        let response = get(&app(policy.clone(), false), CATALOG, Some(API_KEY)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let seen = policy.0.lock().unwrap();
        let principal = seen[0].as_ref().unwrap();
        assert_eq!(principal.subject(), "sa-1");
        assert!(matches!(
            principal.kind,
            olai_auth::PrincipalKind::ServiceAccount { .. }
        ));
    }

    #[tokio::test]
    async fn session_tokens_only_when_enabled() {
        let session = token(claims(ORIGIN, ORIGIN, USER));
        let response = get(&app(Arc::default(), false), CATALOG, Some(&session)).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let response = get(&app(Arc::default(), true), CATALOG, Some(&session)).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn serves_the_protected_resource_metadata_unauthenticated() {
        let response = get(
            &app(Arc::default(), false),
            olai_auth::WELL_KNOWN_PATH,
            None,
        )
        .await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn a_workload_token_names_the_compute() {
        let policy = Arc::new(Recording::default());
        let app = app_with(policy.clone(), trusting_spark());
        let user = token(claims(ISSUER, AUDIENCE, USER));

        for (workload, trusted) in [(SPARK, true), ("system:serviceaccount:dev:notebook", false)] {
            let response =
                get_from(&app, CATALOG, Some(&user), Some(&workload_token(workload))).await;
            assert_eq!(response.status(), StatusCode::NOT_FOUND, "{workload}");
            let seen = policy.0.lock().unwrap().pop().unwrap().unwrap();
            assert_eq!(seen.subject(), USER);
            let actor = seen.actor.as_ref().expect("an actor");
            assert_eq!(actor.id, format!("{WORKLOAD_ISSUER}#{workload}"));
            assert_eq!(seen.is_trusted_compute(), trusted, "{workload}");
        }
    }

    #[tokio::test]
    async fn invalid_workload_tokens_are_401() {
        let policy = Arc::new(Recording::default());
        let app = app_with(policy.clone(), trusting_spark());
        let user = token(claims(ISSUER, AUDIENCE, USER));
        let mut long_lived = claims(WORKLOAD_ISSUER, AUDIENCE, SPARK);
        long_lived["exp"] = json!(now() + 7200);
        for actor in [
            "garbage".to_string(),
            // A user's token is not a workload's.
            user.clone(),
            sign(&claims(WORKLOAD_ISSUER, AUDIENCE, SPARK), &signing_key()),
            sign(
                &claims(WORKLOAD_ISSUER, "https://other.example.com", SPARK),
                &workload_key(),
            ),
            sign(&long_lived, &workload_key()),
        ] {
            let response = get_from(&app, CATALOG, Some(&user), Some(&actor)).await;
            assert_eq!(response.status(), StatusCode::UNAUTHORIZED, "{actor}");
            let challenge = response.headers()[header::WWW_AUTHENTICATE]
                .to_str()
                .unwrap();
            assert!(
                challenge.contains("invalid actor credential"),
                "{challenge}"
            );
        }
        // Nor does a workload token stand in for the user.
        let response = get_from(&app, CATALOG, None, Some(&workload_token(SPARK))).await;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        assert!(policy.0.lock().unwrap().is_empty());
    }

    #[tokio::test]
    async fn without_trusted_compute_workload_tokens_are_ignored() {
        let policy = Arc::new(Recording::default());
        let app = app(policy.clone(), false);
        let user = token(claims(ISSUER, AUDIENCE, USER));
        let response = get_from(&app, CATALOG, Some(&user), Some("garbage")).await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let seen = policy.0.lock().unwrap().pop().unwrap().unwrap();
        assert_eq!(seen.actor, None);
    }
}
