//! Assemble the server as a socket-less [`Router`], for any host: the
//! `sharing-server` binary, an embedding application, or a serverless function.

use std::sync::Arc;
use std::time::Duration;

use axum::Json;
use axum::Router;
use axum::extract::{Path, State};
use axum::routing::get;
use open_sharing_api::auth::{Recipient, require_recipient};
use open_sharing_api::{SharingService, sharing_router};

use crate::Result;
use crate::access::{SourceStorageAccess, open_credential};
use crate::catalog::StoreCatalog;
use crate::config::{Config, DatabaseConfig, EncryptionConfig};
use crate::recipients::{
    ACTIVATION_PATH, SharingProfile, StoreAuthenticator, activate, rewrap_tokens,
};
use crate::secrets::Secrets;
use crate::source::SourceRegistry;
use crate::store::ShareStore;
use crate::store::memory::InMemoryStore;

/// Everything the routes need, besides the store.
#[derive(Clone)]
pub struct Settings {
    /// The externally reachable base URL, without a trailing slash.
    pub public_url: String,
    /// Accepted provider (admin) tokens. Used by the admin API (feature
    /// `unity`).
    pub provider_tokens: Vec<String>,
    /// The longest a presigned URL stays valid.
    pub presign_ttl: Duration,
    /// The S3 region used for presigning.
    pub aws_region: Option<String>,
}

/// An assembled server.
pub struct Service {
    pub router: Router,
    pub store: Arc<dyn ShareStore>,
    pub secrets: Secrets,
}

/// The router over an existing store, secrets and sources:
///
/// - the sharing protocols under `/api/v1/delta-sharing` and
///   `/api/1.0/opensharing`, behind recipient tokens;
/// - activation links under `/api/1.0/opensharing/activations/{code}`;
/// - with the `unity` feature, the provider admin API under
///   `/api/2.1/unity-catalog`;
/// - `GET /health`.
///
/// Without `unity` there is no admin API: the host manages shares, recipients
/// and grants through the [`ShareStore`] (see
/// [`issue_token`](crate::recipients::issue_token)) and can mount its own admin
/// routes next to this router.
pub fn router(
    store: Arc<dyn ShareStore>,
    secrets: Secrets,
    sources: SourceRegistry,
    settings: Settings,
) -> Router {
    let catalog = Arc::new(StoreCatalog::new(store.clone(), sources.clone()));
    let access = Arc::new(SourceStorageAccess::new(
        sources,
        store.clone(),
        secrets.clone(),
        settings.presign_ttl,
        settings.aws_region.clone(),
    ));
    let service = SharingService::<Recipient>::new(catalog, access);
    let sharing = require_recipient(
        sharing_router::<SharingService<Recipient>, Recipient>(service),
        Arc::new(StoreAuthenticator::new(store.clone())),
    );

    let activation = Router::new()
        .route(&format!("{ACTIVATION_PATH}/{{code}}"), get(activation))
        .with_state(ActivationState {
            store: store.clone(),
            secrets: secrets.clone(),
            public_url: settings.public_url.clone(),
        });

    let router = Router::new()
        .merge(sharing)
        .merge(activation)
        .route("/health", get(|| async { "ok" }));

    #[cfg(feature = "unity")]
    let router = {
        use crate::tokens::token_hash;
        use crate::unity::admin::{ADMIN_PREFIX, ProviderState};
        let provider = ProviderState {
            store,
            secrets,
            public_url: settings.public_url,
            provider_tokens: Arc::new(
                settings
                    .provider_tokens
                    .iter()
                    .map(|t| token_hash(t))
                    .collect(),
            ),
        };
        router.nest(ADMIN_PREFIX, crate::unity::admin::router(provider))
    };

    router
}

/// What the activation route needs.
#[derive(Clone)]
struct ActivationState {
    store: Arc<dyn ShareStore>,
    secrets: Secrets,
    public_url: String,
}

async fn activation(
    State(state): State<ActivationState>,
    Path(code): Path<String>,
) -> Result<Json<SharingProfile>> {
    Ok(Json(
        activate(
            state.store.as_ref(),
            &state.secrets,
            &state.public_url,
            &code,
        )
        .await?,
    ))
}

impl Config {
    /// The secrets encryptor. The in-memory store falls back to an insecure
    /// development key; anything persistent requires `encryption`.
    pub fn secrets(&self) -> std::result::Result<Secrets, String> {
        let encryption = match (&self.encryption, &self.database) {
            (Some(encryption), _) => encryption.clone(),
            (None, DatabaseConfig::Memory) => {
                tracing::warn!("no `encryption` configured; using an insecure development key");
                EncryptionConfig::insecure_dev()
            }
            (None, _) => {
                return Err("`encryption` must be configured for a persistent store".to_string());
            }
        };
        Ok(Secrets::new(encryption.build()?))
    }

    pub fn settings(&self) -> std::result::Result<Settings, String> {
        Ok(Settings {
            public_url: self.public_url(),
            provider_tokens: self.provider_tokens()?,
            presign_ttl: Duration::from_secs(self.presign_ttl_secs),
            aws_region: self.aws_region.clone(),
        })
    }

    /// Connect the configured store. For Postgres this checks the schema is
    /// current (over the migration URL) but never migrates.
    pub async fn store(&self) -> std::result::Result<Arc<dyn ShareStore>, String> {
        match &self.database {
            DatabaseConfig::Memory => Ok(Arc::new(InMemoryStore::new())),
            #[cfg(feature = "postgres")]
            DatabaseConfig::Postgres {
                url,
                migration_url,
                max_connections,
            } => {
                use crate::store::postgres;
                let url = url.value().ok_or("database url is not set")?;
                let check_url = migration_url
                    .as_ref()
                    .and_then(|u| u.value())
                    .unwrap_or_else(|| url.clone());
                let check = postgres::connect(&check_url, 1)
                    .await
                    .map_err(|e| e.to_string())?;
                postgres::ensure_schema_current(&check)
                    .await
                    .map_err(|e| e.to_string())?;
                check.close().await;
                let pool = postgres::connect(&url, *max_connections)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(Arc::new(postgres::PostgresStore::new(pool)))
            }
            #[cfg(not(feature = "postgres"))]
            DatabaseConfig::Postgres { .. } => {
                Err("this build has no Postgres support (feature `postgres`)".to_string())
            }
        }
    }
}

/// Build the whole service from configuration.
pub async fn start(config: &Config) -> std::result::Result<Service, String> {
    let store = config.store().await?;
    let secrets = config.secrets()?;
    let router = router(
        store.clone(),
        secrets.clone(),
        config.sources()?,
        config.settings()?,
    );
    Ok(Service {
        router,
        store,
        secrets,
    })
}

/// Re-wrap every stored secret under the active key. Returns how many were
/// re-wrapped (credentials, tokens awaiting activation).
pub async fn rotate_keys(store: &dyn ShareStore, secrets: &Secrets) -> Result<(usize, usize)> {
    let mut credentials = 0;
    for credential in store.list_credentials().await? {
        if secrets.rewrap(&credential.sealed).await?.is_some() {
            // Opening writes the re-wrapped value back.
            open_credential(store, secrets, &credential.name).await?;
            credentials += 1;
        }
    }
    let tokens = rewrap_tokens(store, secrets).await?;
    Ok((credentials, tokens))
}
