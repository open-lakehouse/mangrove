//! Recipient tokens: issuing, authenticating and activating them.
//!
//! A recipient is handed an *activation link* once (at creation or rotation).
//! Opening it returns the Delta Sharing profile — endpoint plus bearer token —
//! and consumes the link: the clear token, kept sealed only for that moment, is
//! dropped, and from then on the server holds just its sha256 digest.

use std::sync::Arc;

use async_trait::async_trait;
use open_sharing_api::auth::{Recipient, RecipientAuthenticator};
use serde::Serialize;

use crate::secrets::Secrets;
use crate::store::{RecipientRecord, ShareStore, TokenRecord, now_millis};
use crate::tokens::{random_secret, token_hash};
use crate::{Error, Result};

/// Where activation links are served, under the server's public URL.
pub const ACTIVATION_PATH: &str = "/api/1.0/opensharing/activations";

/// Authenticates recipient bearer tokens against the store.
pub struct StoreAuthenticator {
    store: Arc<dyn ShareStore>,
}

impl StoreAuthenticator {
    pub fn new(store: Arc<dyn ShareStore>) -> Self {
        Self { store }
    }
}

#[async_trait]
impl RecipientAuthenticator for StoreAuthenticator {
    async fn authenticate(&self, token: &str) -> Result<Recipient> {
        let (token, recipient) = match self.store.token_by_hash(&token_hash(token)).await {
            Ok(found) => found,
            Err(Error::NotFound) => return Err(Error::Unauthenticated),
            Err(e) => return Err(e),
        };
        if token.expires_at.is_some_and(|at| at <= now_millis()) {
            return Err(Error::Unauthenticated);
        }
        Ok(Recipient {
            id: recipient.id,
            name: recipient.name,
        })
    }
}

/// The AAD a token awaiting activation is sealed under.
fn token_aad(token_id: &str) -> String {
    format!("recipient-token:{token_id}")
}

/// Issue a new token for `recipient`. The returned record carries the activation
/// code; [`activation_url`] turns it into the link handed to the recipient.
pub async fn issue_token(
    store: &dyn ShareStore,
    secrets: &Secrets,
    recipient: &RecipientRecord,
    expires_at: Option<i64>,
) -> Result<TokenRecord> {
    let id = uuid::Uuid::new_v4().to_string();
    let token = random_secret();
    let activation_code = random_secret();
    let record = TokenRecord {
        id: id.clone(),
        recipient_id: recipient.id.clone(),
        token_hash: token_hash(&token),
        sealed_token: Some(secrets.seal(&token_aad(&id), token.as_bytes()).await?),
        activation_code: Some(activation_code),
        expires_at,
        created_at: now_millis(),
    };
    store.insert_token(record.clone()).await?;
    Ok(record)
}

/// The activation link for a token, until it is used.
pub fn activation_url(public_url: &str, token: &TokenRecord) -> Option<String> {
    token
        .activation_code
        .as_ref()
        .map(|code| format!("{public_url}{ACTIVATION_PATH}/{code}"))
}

/// A Delta Sharing profile (the contents of a `.share` file).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SharingProfile {
    pub share_credentials_version: u32,
    pub endpoint: String,
    pub bearer_token: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<String>,
}

/// Consume an activation code, returning the recipient's sharing profile.
pub async fn activate(
    store: &dyn ShareStore,
    secrets: &Secrets,
    public_url: &str,
    code: &str,
) -> Result<SharingProfile> {
    let token = store.token_by_activation_code(code).await?;
    let sealed = token.sealed_token.as_deref().ok_or(Error::NotFound)?;
    let (clear, _) = secrets.open(&token_aad(&token.id), sealed).await?;
    store.mark_activated(&token.id).await?;
    Ok(SharingProfile {
        share_credentials_version: 1,
        endpoint: format!(
            "{public_url}{}",
            open_sharing_api::router::DELTA_SHARING_PREFIX
        ),
        bearer_token: String::from_utf8(clear)
            .map_err(|_| Error::generic("stored token is not valid UTF-8"))?,
        expiration_time: token
            .expires_at
            .and_then(chrono::DateTime::from_timestamp_millis)
            .map(|at| at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
    })
}

/// Re-wrap every token awaiting activation under the active key.
pub async fn rewrap_tokens(store: &dyn ShareStore, secrets: &Secrets) -> Result<usize> {
    let mut rewrapped = 0;
    for token in store.sealed_tokens().await? {
        let Some(sealed) = &token.sealed_token else {
            continue;
        };
        if let Some(sealed) = secrets.rewrap(sealed).await? {
            store.update_sealed_token(&token.id, sealed).await?;
            rewrapped += 1;
        }
    }
    Ok(rewrapped)
}
