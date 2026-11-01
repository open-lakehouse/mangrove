//! Shares, recipients, their bearer tokens and the grants between them.
//!
//! A Delta Sharing provider maps shares onto catalog assets, issues recipients
//! bearer tokens and grants them shares. This crate holds that state and the
//! rules for the tokens, apart from any protocol surface or server:
//!
//! - the store ports, split by resource: [`ShareStore`] holds shares and grants,
//!   and its read-only view [`ShareReader`] is all a sharing protocol needs to
//!   list and authorize shares. [`RecipientStore`] holds recipients and tokens,
//!   and its read-only view [`TokenReader`] is all bearer authentication needs.
//!   [`SharingStore`] bundles both;
//! - [`recipients`]: issuing, rotating and redeeming tokens through one-time
//!   activation codes, and re-wrapping the sealed ones after a key rotation;
//! - [`tokens`]: the token and activation-code digests.
//!
//! The store keeps shares as catalog-independent [`ShareObject`]s, serialized in
//! the shape of the Unity Catalog `shares.v1.DataObject`. Secrets are stored only
//! sealed, by an [`EnvelopeEncryptor`](olai_store::encryption::EnvelopeEncryptor)
//! bound to the token id, and tokens and activation codes only as digests.
//!
//! Every write that must not be observed half-done is one atomic store
//! operation: creating a recipient with its first token, rotating a token,
//! redeeming an activation code.
//!
//! Implementations:
//!
//! - [`InMemorySharingStore`] (feature `memory`), for tests and ephemeral servers;
//! - `PgSharingStore` (feature `postgres`) and `SqliteSharingStore` (feature
//!   `sqlite`), over a pool the host provides, each with tables of its own and a
//!   `MIGRATOR` (versioned 03xx) to merge into the host's.
//!
//! The `testing` feature exposes the conformance suite every backend runs
//! (`testing::round_trips`).

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub use unitycatalog_common::{Error, Result};

pub mod recipients;
pub mod tokens;

#[cfg(feature = "memory")]
mod memory;
#[cfg(feature = "memory")]
pub use crate::memory::InMemorySharingStore;

#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "postgres")]
pub use crate::postgres::PgSharingStore;

#[cfg(feature = "sqlite")]
pub mod sqlite;
#[cfg(feature = "sqlite")]
pub use crate::sqlite::SqliteSharingStore;

#[cfg(any(feature = "postgres", feature = "sqlite"))]
mod sql;

#[cfg(feature = "testing")]
pub mod testing;

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
    /// The sha256 digest of the single-use activation code, until activated.
    pub activation_code_hash: Option<Vec<u8>>,
    /// When the token stops being accepted (ms since epoch); `None` never expires.
    pub expires_at: Option<i64>,
    pub created_at: i64,
}

impl TokenRecord {
    /// Whether the token is accepted at `now` (ms since epoch).
    pub fn is_live(&self, now: i64) -> bool {
        self.expires_at.is_none_or(|at| at > now)
    }
}

/// A change to a recipient. `None` leaves a field as it is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipientUpdate {
    pub comment: Option<String>,
    /// A new expiry for the recipient's newest token (`Some(None)`: never
    /// expires). Older tokens, still in a rotation's grace period, keep theirs.
    pub expires_at: Option<Option<i64>>,
}

// The store is split by resource, with read-only views for what a sharing
// protocol needs. Lookups of something missing return [`Error::NotFound`];
// creating something that exists returns [`Error::AlreadyExists`].

/// The shares as a sharing protocol reads them: which shares a recipient sees,
/// and whether it may read one.
#[async_trait]
pub trait ShareReader: Send + Sync + 'static {
    async fn get_share(&self, name: &str) -> Result<ShareRecord>;
    /// The shares granted to a recipient, ordered by name.
    async fn shares_of(&self, recipient_id: &str) -> Result<Vec<ShareRecord>>;
    async fn is_granted(&self, share: &str, recipient_id: &str) -> Result<bool>;
}

/// Shares and their grants, as the provider manages them.
#[async_trait]
pub trait ShareStore: ShareReader {
    async fn create_share(&self, name: &str, comment: Option<String>) -> Result<ShareRecord>;
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

    async fn grant(&self, share: &str, recipient: &str) -> Result<()>;
    async fn revoke(&self, share: &str, recipient: &str) -> Result<()>;
    /// The names of the recipients granted `share`, ordered.
    async fn recipients_of(&self, share: &str) -> Result<Vec<String>>;
}

/// Recipient tokens as bearer authentication reads them.
#[async_trait]
pub trait TokenReader: Send + Sync + 'static {
    /// The token with this digest ([`tokens::token_hash`]) and its recipient.
    async fn token_by_hash(&self, token_hash: &[u8]) -> Result<(TokenRecord, RecipientRecord)>;
}

/// Recipients and their tokens: issuing, activation, rotation and re-wrapping.
///
/// The domain rules (generating and sealing tokens, expiry) live in
/// [`recipients`]; the store makes each operation atomic.
#[async_trait]
pub trait RecipientStore: TokenReader {
    /// Create `recipient` together with its first token, or neither: a taken
    /// name leaves no token behind.
    async fn create_recipient(&self, recipient: RecipientRecord, token: TokenRecord) -> Result<()>;
    async fn get_recipient(&self, name: &str) -> Result<RecipientRecord>;
    /// All recipients, ordered by name.
    async fn list_recipients(&self) -> Result<Vec<RecipientRecord>>;
    async fn update_recipient(
        &self,
        name: &str,
        update: RecipientUpdate,
    ) -> Result<RecipientRecord>;
    /// Delete a recipient with its tokens and grants.
    async fn delete_recipient(&self, name: &str) -> Result<()>;

    /// The recipient's tokens, newest first.
    async fn tokens_for_recipient(&self, recipient_id: &str) -> Result<Vec<TokenRecord>>;
    /// Add `token` for its recipient and retire the recipient's other tokens,
    /// in one step.
    ///
    /// With `existing_expire_at`, the other tokens stay valid until then at
    /// the latest (an earlier expiry is kept); without, they are deleted. Tokens
    /// already expired at `token.created_at` are deleted either way.
    async fn rotate_token(&self, token: TokenRecord, existing_expire_at: Option<i64>)
    -> Result<()>;
    /// Consume the activation code with this digest, if its token is live at
    /// `now`: clear the code and the sealed copy, and return the token as it
    /// was, sealed copy included.
    ///
    /// Exactly one of any number of concurrent redemptions of a code succeeds;
    /// the others, and any later one, get [`Error::NotFound`].
    async fn redeem_activation(&self, code_hash: &[u8], now: i64) -> Result<TokenRecord>;
    /// Every token still holding a sealed copy (for key rotation).
    async fn sealed_tokens(&self) -> Result<Vec<TokenRecord>>;
    /// Replace a token's sealed copy, if it still is `current`. Returns whether
    /// it was replaced; a token activated in the meantime keeps none.
    async fn reseal_token(&self, token_id: &str, current: &[u8], sealed: Vec<u8>) -> Result<bool>;
}

/// Both stores: all of a provider's sharing state. Implemented for every type
/// that implements both.
///
/// An `Arc<dyn SharingStore>` coerces to any of the narrower stores and readers.
pub trait SharingStore: ShareStore + RecipientStore {}

impl<T: ShareStore + RecipientStore + ?Sized> SharingStore for T {}

/// The current time in milliseconds since the epoch.
pub fn now_millis() -> i64 {
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
