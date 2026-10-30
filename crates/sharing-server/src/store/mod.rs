//! The [`ShareStore`] port: the sharing server's own state.
//!
//! The store holds what the provider configures — shares (each a list of
//! [`ShareObject`]s naming assets in a source catalog), recipients, their bearer
//! tokens and the share grants — plus, for local storage access, sealed storage
//! credentials and the external locations that map storage URLs onto them.
//! Secrets are stored only sealed (see [`crate::secrets`]) and tokens only as
//! digests.
//!
//! Two implementations: [`memory::InMemoryStore`] (tests, ephemeral demos) and,
//! with the `postgres` feature, [`postgres::PostgresStore`].

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::Result;

pub mod memory;
#[cfg(feature = "postgres")]
pub mod postgres;

/// One entry of a share: an asset, or a whole schema, in a source catalog.
///
/// Serialized with the field names and values of the Unity Catalog
/// `shares.v1.DataObject` (`data_object_type: "TABLE"`, …), and any other field
/// of that object is kept in [`extra`](Self::extra), so shares written through
/// a UC-shaped admin API round-trip unchanged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShareObject {
    /// The source-catalog name: `catalog.schema.name`, or `catalog.schema` for
    /// a [`Schema`](ShareObjectKind::Schema).
    pub name: String,
    #[serde(rename = "data_object_type")]
    pub kind: ShareObjectKind,
    /// The name recipients see: `schema.name` (default: the source name minus
    /// its catalog), or the schema name for a schema.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared_as: Option<String>,
    /// Whether the table's history (time travel, older versions) is shared.
    #[serde(
        default,
        rename = "history_data_sharing_status",
        with = "history_status",
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub history_shared: bool,
    /// The first table version recipients may read.
    #[serde(
        default,
        deserialize_with = "optional_int64",
        skip_serializing_if = "Option::is_none"
    )]
    pub start_version: Option<i64>,
    /// Any further fields, preserved as given (`added_at`, `comment`, …).
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

impl ShareObject {
    /// A shared object of `kind` with nothing but its source name set.
    pub fn new(kind: ShareObjectKind, name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind,
            shared_as: None,
            history_shared: false,
            start_version: None,
            extra: serde_json::Map::new(),
        }
    }
}

/// What a [`ShareObject`] names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShareObjectKind {
    Table,
    /// Every table of a schema, resolved when listed.
    Schema,
    Volume,
    /// An agent skill, stored on a volume.
    AgentSkill,
}

/// `history_shared` as the UC `HistoryStatus` enum (`ENABLED` / `DISABLED`).
mod history_status {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(shared: &bool, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(if *shared { "ENABLED" } else { "DISABLED" })
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<bool, D::Error> {
        Ok(Option::<String>::deserialize(d)?.as_deref() == Some("ENABLED"))
    }
}

/// An `int64` given as a JSON number or, as proto JSON encodes it, a string.
fn optional_int64<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<i64>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Int64 {
        Number(i64),
        String(String),
    }
    match Option::<Int64>::deserialize(d)? {
        None => Ok(None),
        Some(Int64::Number(n)) => Ok(Some(n)),
        Some(Int64::String(s)) => s.parse().map(Some).map_err(serde::de::Error::custom),
    }
}

/// A share and the objects it contains.
#[derive(Debug, Clone, PartialEq)]
pub struct ShareRecord {
    pub id: String,
    pub name: String,
    pub comment: Option<String>,
    /// The shared objects, in the order they were added.
    pub objects: Vec<ShareObject>,
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

/// A storage credential, sealed. The clear value is a
/// [`RegisteredCredential`](olai_credential_vending::RegisteredCredential) (JSON).
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
        objects: Vec<ShareObject>,
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn share_objects_read_the_catalog_shape() {
        // A UC `DataObject` as proto JSON: int64 as strings, enums by name.
        let value = json!({
            "name": "cat.sales.orders",
            "data_object_type": "TABLE",
            "shared_as": "sales.orders",
            "history_data_sharing_status": "ENABLED",
            "start_version": "3",
            "added_at": "1700000000000",
            "comment": "orders",
        });
        let object: ShareObject = serde_json::from_value(value).unwrap();
        assert_eq!(object.kind, ShareObjectKind::Table);
        assert!(object.history_shared);
        assert_eq!(object.start_version, Some(3));
        assert_eq!(object.extra["comment"], "orders");

        let written = serde_json::to_value(&object).unwrap();
        assert_eq!(written["added_at"], "1700000000000");
        assert_eq!(written["history_data_sharing_status"], "ENABLED");
        assert_eq!(
            serde_json::from_value::<ShareObject>(written).unwrap(),
            object
        );

        let schema: ShareObject =
            serde_json::from_value(json!({ "name": "cat.sales", "data_object_type": "SCHEMA" }))
                .unwrap();
        assert_eq!(
            schema,
            ShareObject::new(ShareObjectKind::Schema, "cat.sales")
        );
    }
}
