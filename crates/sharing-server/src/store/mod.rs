//! The [`ShareStore`] port: the sharing server's own state.
//!
//! The store holds what the provider configures — shares (each a list of UC
//! `DataObject`s naming assets in a source catalog), recipients, their bearer
//! tokens and the share grants — plus, for local storage access, sealed storage
//! credentials and the external locations that map storage URLs onto them.
//! Secrets are stored only sealed (see [`crate::secrets`]) and tokens only as
//! digests.
//!
//! Two implementations: [`memory::InMemoryStore`] (tests, ephemeral demos) and,
//! with the `postgres` feature, [`postgres::PostgresStore`].

use async_trait::async_trait;
use unitycatalog_common::models::shares::v1::DataObject;

use crate::Result;

pub mod memory;
#[cfg(feature = "postgres")]
pub mod postgres;

/// A share and the data objects it contains.
#[derive(Debug, Clone, PartialEq)]
pub struct ShareRecord {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    /// The shared objects, in the order they were added.
    pub objects: Vec<DataObject>,
    pub created_at: i64,
    pub updated_at: i64,
}

/// A recipient of shares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipientRecord {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    pub created_at: i64,
}

/// A recipient bearer token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenRecord {
    pub id: String,
    pub recipient_id: String,
    /// The sha256 digest of the token; the token itself is never stored in clear.
    pub token_hash: Vec<u8>,
    /// The token, sealed, kept only until the recipient activates it (the
    /// activation link is the one time the clear token is handed out).
    pub sealed_token: Option<Vec<u8>>,
    /// The single-use activation code, until activated.
    pub activation_code: Option<String>,
    /// When the token stops being accepted (ms since epoch); `None` never expires.
    pub expires_at: Option<i64>,
    pub created_at: i64,
}

/// A storage credential, sealed. The clear value is a UC `Credential` (JSON).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedCredential {
    pub name: String,
    pub sealed: Vec<u8>,
}

/// A storage URL prefix governed by a stored credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalLocationRecord {
    pub name: String,
    pub url: String,
    pub credential_name: String,
}

/// The sharing server's state. See the module docs.
///
/// Lookups of something missing return [`Error::NotFound`](crate::Error::NotFound);
/// creating something that exists returns an invalid-argument error.
#[async_trait]
pub trait ShareStore: Send + Sync + 'static {
    // -- shares ---------------------------------------------------------------

    async fn create_share(&self, name: &str, comment: Option<String>) -> Result<ShareRecord>;
    async fn get_share(&self, name: &str) -> Result<ShareRecord>;
    /// All shares, ordered by name.
    async fn list_shares(&self) -> Result<Vec<ShareRecord>>;
    /// Replace a share's objects and comment.
    async fn update_share(
        &self,
        name: &str,
        objects: Vec<DataObject>,
        comment: Option<String>,
    ) -> Result<ShareRecord>;
    async fn delete_share(&self, name: &str) -> Result<()>;

    // -- recipients -----------------------------------------------------------

    async fn create_recipient(
        &self,
        name: &str,
        comment: Option<String>,
    ) -> Result<RecipientRecord>;
    async fn get_recipient(&self, name: &str) -> Result<RecipientRecord>;
    /// All recipients, ordered by name.
    async fn list_recipients(&self) -> Result<Vec<RecipientRecord>>;
    /// Delete a recipient with its tokens and grants.
    async fn delete_recipient(&self, name: &str) -> Result<()>;

    // -- tokens ---------------------------------------------------------------

    async fn insert_token(&self, token: TokenRecord) -> Result<()>;
    /// The token with this digest and its recipient.
    async fn token_by_hash(&self, token_hash: &[u8]) -> Result<(TokenRecord, RecipientRecord)>;
    /// The not-yet-activated token with this activation code.
    async fn token_by_activation_code(&self, code: &str) -> Result<TokenRecord>;
    /// Consume a token's activation: drop its sealed copy and activation code.
    async fn mark_activated(&self, token_id: &str) -> Result<()>;
    /// The recipient's tokens, newest first.
    async fn tokens_for_recipient(&self, recipient_id: &str) -> Result<Vec<TokenRecord>>;
    /// Delete all of a recipient's tokens.
    async fn delete_tokens(&self, recipient_id: &str) -> Result<()>;
    /// Every token still holding a sealed copy (for key rotation).
    async fn sealed_tokens(&self) -> Result<Vec<TokenRecord>>;
    async fn update_sealed_token(&self, token_id: &str, sealed: Vec<u8>) -> Result<()>;

    // -- grants ---------------------------------------------------------------

    async fn grant(&self, share: &str, recipient: &str) -> Result<()>;
    async fn revoke(&self, share: &str, recipient: &str) -> Result<()>;
    /// The names of the recipients granted `share`, ordered.
    async fn recipients_of(&self, share: &str) -> Result<Vec<String>>;
    /// The shares granted to a recipient, ordered by name.
    async fn shares_of(&self, recipient_id: &str) -> Result<Vec<ShareRecord>>;
    async fn is_granted(&self, share: &str, recipient_id: &str) -> Result<bool>;

    // -- storage credentials (local access) -------------------------------------

    async fn put_credential(&self, credential: SealedCredential) -> Result<()>;
    async fn get_credential(&self, name: &str) -> Result<SealedCredential>;
    /// All credentials, ordered by name.
    async fn list_credentials(&self) -> Result<Vec<SealedCredential>>;
    async fn delete_credential(&self, name: &str) -> Result<()>;

    async fn put_external_location(&self, location: ExternalLocationRecord) -> Result<()>;
    /// All external locations, ordered by name.
    async fn list_external_locations(&self) -> Result<Vec<ExternalLocationRecord>>;
    async fn delete_external_location(&self, name: &str) -> Result<()>;
}

/// The current time in milliseconds since the epoch.
pub(crate) fn now_millis() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
