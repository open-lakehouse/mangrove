//! The encryption admin API: which key-encryption key (KEK) wraps the server's
//! sealed values, and the sweep that moves them onto the active KEK.
//!
//! - `GET  /api/admin/v1/encryption/kek-usage`: per store, how many sealed
//!   values each KEK wraps. Read from envelope headers; nothing is decrypted.
//! - `POST /api/admin/v1/encryption/rewrap`: re-wrap every sealed value under
//!   the active KEK. Per store: usage before, then what was rewrapped, already
//!   active, superseded by a concurrent write or failed, and usage after.
//!
//! The stores are the object graph's sensitive fields (`objects`), storage
//! credentials (`storage_credentials`) and, where the host wires a sharing
//! store, recipient tokens (`recipient_tokens`). A store that isn't configured
//! is listed in `not_configured`.
//!
//! Both are asked of the policy on the metastore (`GetKekUsage`,
//! `RewrapSecrets`). A sweep is idempotent: running it again resumes an
//! interrupted one. A value that fails is reported and skipped; a store that
//! fails as a whole is reported and the next store still runs.

use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use olai_store::{ManagedObjectStore, ObjectStore, RewrapOptions};
use serde::Serialize;
use unitycatalog_common::models::ObjectLabel;
use unitycatalog_common::rotation::{
    KekUsage, RewrapReport, SealedStore, SealedStoreError, StoreSweep, sweep,
};
use unitycatalog_common::services::encryption::EnvelopeEncryptor;

use crate::Result;
use crate::api::RequestContext;
use crate::policy::{Facts, Policy, Securable, ServerAction};

/// Where the admin API is mounted.
pub const ADMIN_BASE_PATH: &str = "/api/admin/v1";

/// The object graph's sealed sensitive fields, as a [`SealedStore`].
pub struct SealedObjects<S>(pub ManagedObjectStore<ObjectLabel, S>);

#[async_trait::async_trait]
impl<S: ObjectStore<ObjectLabel>> SealedStore for SealedObjects<S> {
    fn name(&self) -> &str {
        "objects"
    }

    async fn kek_usage(&self) -> std::result::Result<KekUsage, SealedStoreError> {
        Ok(self.0.kek_usage().await?.into())
    }

    async fn rewrap(&self) -> std::result::Result<RewrapReport, SealedStoreError> {
        Ok(self.0.rewrap_all(RewrapOptions::default()).await?.into())
    }
}

/// The server's sealed stores and the encryptor they seal with.
#[derive(Clone)]
pub struct EncryptionAdmin {
    encryptor: EnvelopeEncryptor,
    stores: Vec<Arc<dyn SealedStore>>,
    not_configured: Vec<String>,
}

impl EncryptionAdmin {
    /// The stores sealed by `encryptor`.
    pub fn new(encryptor: EnvelopeEncryptor, stores: Vec<Arc<dyn SealedStore>>) -> Self {
        Self {
            encryptor,
            stores,
            not_configured: Vec::new(),
        }
    }

    /// Report the store `name` as not configured in this deployment.
    pub fn with_not_configured(mut self, name: impl Into<String>) -> Self {
        self.not_configured.push(name.into());
        self
    }

    /// The KEK new values are sealed under.
    pub fn active_kek_id(&self) -> String {
        self.encryptor.active_kek_id()
    }

    /// Every store's KEK usage.
    pub async fn kek_usage(&self) -> KekUsageResponse {
        let mut stores = Vec::with_capacity(self.stores.len());
        for store in &self.stores {
            let (kek_usage, error) = match store.kek_usage().await {
                Ok(usage) => (Some(usage), None),
                Err(e) => (None, Some(e.to_string())),
            };
            stores.push(StoreUsage {
                store: store.name().to_string(),
                kek_usage,
                error,
            });
        }
        KekUsageResponse {
            active_kek_id: self.active_kek_id(),
            stores,
            not_configured: self.not_configured.clone(),
        }
    }

    /// Re-wrap every store under the active KEK.
    pub async fn rewrap(&self) -> RewrapResponse {
        let active_kek_id = self.active_kek_id();
        let stores: Vec<&dyn SealedStore> = self.stores.iter().map(|s| s.as_ref()).collect();
        let stores = sweep(&stores).await;
        for entry in &stores {
            match (&entry.report, &entry.error) {
                (_, Some(error)) => {
                    tracing::warn!(store = %entry.store, %error, "KEK rewrap sweep: store failed")
                }
                (Some(report), None) => tracing::info!(
                    store = %entry.store,
                    active = %active_kek_id,
                    rewrapped = report.rewrapped,
                    already_active = report.already_active,
                    superseded = report.superseded,
                    failed = report.failed.len(),
                    remaining = report.kek_usage.not_under(&active_kek_id),
                    "KEK rewrap sweep"
                ),
                (None, None) => {}
            }
        }
        RewrapResponse {
            active_kek_id,
            stores,
            not_configured: self.not_configured.clone(),
        }
    }
}

/// `GET …/encryption/kek-usage`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct KekUsageResponse {
    /// The KEK new values are sealed under.
    pub active_kek_id: String,
    pub stores: Vec<StoreUsage>,
    /// Stores this deployment doesn't hold.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub not_configured: Vec<String>,
}

/// One store's KEK usage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StoreUsage {
    pub store: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kek_usage: Option<KekUsage>,
    /// What stopped the count, if anything did.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// `POST …/encryption/rewrap`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RewrapResponse {
    /// The KEK the sweep moved values to.
    pub active_kek_id: String,
    pub stores: Vec<StoreSweep>,
    /// Stores this deployment doesn't hold.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub not_configured: Vec<String>,
}

#[derive(Clone)]
struct AdminState {
    admin: EncryptionAdmin,
    policy: Arc<dyn Policy<RequestContext>>,
}

impl AdminState {
    async fn authorize(&self, action: ServerAction, cx: &RequestContext) -> Result<()> {
        self.policy
            .authorize_checked(action, &Securable::root(), &Facts::default(), cx)
            .await
    }
}

/// The admin API's router, asking `policy` on the metastore.
pub fn router(admin: EncryptionAdmin, policy: Arc<dyn Policy<RequestContext>>) -> Router {
    let routes = Router::new()
        .route("/encryption/kek-usage", get(kek_usage))
        .route("/encryption/rewrap", post(rewrap))
        .with_state(AdminState { admin, policy });
    Router::new().nest(ADMIN_BASE_PATH, routes)
}

async fn kek_usage(
    State(state): State<AdminState>,
    cx: RequestContext,
) -> Result<Json<KekUsageResponse>> {
    state.authorize(ServerAction::GetKekUsage, &cx).await?;
    Ok(Json(state.admin.kek_usage().await))
}

async fn rewrap(
    State(state): State<AdminState>,
    cx: RequestContext,
) -> Result<Json<RewrapResponse>> {
    state.authorize(ServerAction::RewrapSecrets, &cx).await?;
    Ok(Json(state.admin.rewrap().await))
}

#[cfg(test)]
mod tests {
    use axum::body::{Body, to_bytes};
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;
    use unitycatalog_common::services::encryption::LocalKeyProvider;

    use super::*;
    use crate::policy::{ConstantPolicy, Decision};
    use crate::rest::{AnonymousAuthenticator, AuthenticationLayer};

    struct Failing;

    #[async_trait::async_trait]
    impl SealedStore for Failing {
        fn name(&self) -> &str {
            "failing"
        }
        async fn kek_usage(&self) -> std::result::Result<KekUsage, SealedStoreError> {
            Err("listing failed".into())
        }
        async fn rewrap(&self) -> std::result::Result<RewrapReport, SealedStoreError> {
            Err("listing failed".into())
        }
    }

    fn app(decision: Decision) -> Router {
        let encryptor =
            EnvelopeEncryptor::local(LocalKeyProvider::single("v1", vec![7; 32]).unwrap());
        let admin = EncryptionAdmin::new(encryptor, vec![Arc::new(Failing)])
            .with_not_configured("recipient_tokens");
        router(admin, Arc::new(ConstantPolicy::new(decision)))
            .layer(AuthenticationLayer::new(AnonymousAuthenticator))
    }

    async fn call(app: &Router, method: &str, path: &str) -> (StatusCode, serde_json::Value) {
        let request = Request::builder()
            .method(method)
            .uri(format!("{ADMIN_BASE_PATH}{path}"))
            .body(Body::empty())
            .unwrap();
        let response = app.clone().oneshot(request).await.unwrap();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap_or_default())
    }

    #[tokio::test]
    async fn a_denying_policy_refuses_both_operations() {
        let app = app(Decision::Deny);
        let (status, _) = call(&app, "GET", "/encryption/kek-usage").await;
        assert_eq!(status, StatusCode::FORBIDDEN);
        let (status, _) = call(&app, "POST", "/encryption/rewrap").await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn a_failing_store_is_reported_not_fatal() {
        let app = app(Decision::Allow);
        let (status, usage) = call(&app, "GET", "/encryption/kek-usage").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(usage["active_kek_id"], "v1");
        assert_eq!(usage["stores"][0]["error"], "listing failed");
        assert_eq!(
            usage["not_configured"],
            serde_json::json!(["recipient_tokens"])
        );

        let (status, swept) = call(&app, "POST", "/encryption/rewrap").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(swept["stores"][0]["store"], "failing");
        assert_eq!(swept["stores"][0]["error"], "listing failed");
        // Rewrap is a POST.
        let (status, _) = call(&app, "GET", "/encryption/rewrap").await;
        assert_eq!(status, StatusCode::METHOD_NOT_ALLOWED);
    }
}
