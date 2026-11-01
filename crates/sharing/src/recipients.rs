//! Recipient tokens: issuing, rotating and redeeming them.
//!
//! A recipient is handed an *activation code* once (at creation or rotation),
//! typically as a link. Redeeming it hands out the clear bearer token and
//! consumes the code: the clear token, kept sealed only for that moment, is
//! dropped, and from then on the store holds just its sha256 digest. The
//! activation code is derived from the token ([`activation_code`]) and stored
//! only as a digest too.
//!
//! These functions hold the rules (generating and sealing tokens, expiry, the
//! rotation grace period); each store write they make is one atomic
//! [`RecipientStore`] operation.

use olai_store::encryption::EnvelopeEncryptor;

use crate::tokens::{activation_code, random_secret, token_hash};
use crate::{Error, RecipientRecord, RecipientStore, Result, TokenRecord, now_millis};

/// The AAD a token awaiting activation is sealed under.
fn token_aad(token_id: &str) -> String {
    format!("recipient-token:{token_id}")
}

fn sealing(e: olai_store::Error) -> Error {
    Error::generic(format!("sealing recipient token: {e}"))
}

/// A new token for the recipient `recipient_id`, sealed for activation, not
/// yet stored. Ids are time-ordered, so tokens created within the same
/// millisecond still sort by creation.
pub async fn new_token(
    encryptor: &EnvelopeEncryptor,
    recipient_id: &str,
    expires_at: Option<i64>,
) -> Result<TokenRecord> {
    let id = uuid::Uuid::now_v7().to_string();
    let token = random_secret();
    let sealed = encryptor
        .seal(&token_aad(&id), token.as_bytes())
        .await
        .map_err(sealing)?;
    Ok(TokenRecord {
        id,
        recipient_id: recipient_id.to_string(),
        token_hash: token_hash(&token),
        sealed_token: Some(sealed),
        activation_code_hash: Some(token_hash(&activation_code(&token))),
        expires_at,
        created_at: now_millis(),
    })
}

/// Create the recipient `name` with its first token, whose activation code
/// [`pending_activation`] gives.
pub async fn create_recipient(
    store: &dyn RecipientStore,
    encryptor: &EnvelopeEncryptor,
    name: &str,
    comment: Option<String>,
    expires_at: Option<i64>,
) -> Result<RecipientRecord> {
    let recipient = RecipientRecord {
        id: uuid::Uuid::now_v7().to_string(),
        name: name.to_string(),
        comment,
        created_at: now_millis(),
    };
    let token = new_token(encryptor, &recipient.id, expires_at).await?;
    store.create_recipient(recipient.clone(), token).await?;
    Ok(recipient)
}

/// Give `recipient` a fresh token, with the expiry of its newest one, and
/// retire the others.
///
/// They stay valid for `existing_expire_in_seconds` more at most (never longer
/// than they would have); `None` or `0` revokes them at once.
pub async fn rotate_token(
    store: &dyn RecipientStore,
    encryptor: &EnvelopeEncryptor,
    recipient: &RecipientRecord,
    existing_expire_in_seconds: Option<i64>,
) -> Result<()> {
    let grace = match existing_expire_in_seconds {
        Some(seconds) if seconds < 0 => {
            return Err(Error::invalid_argument(
                "existing_token_expire_in_seconds must not be negative",
            ));
        }
        Some(seconds) if seconds > 0 => Some(seconds.saturating_mul(1000)),
        _ => None,
    };
    let expires_at = store
        .tokens_for_recipient(&recipient.id)
        .await?
        .first()
        .and_then(|t| t.expires_at);
    let token = new_token(encryptor, &recipient.id, expires_at).await?;
    let existing_expire_at = grace.map(|grace| token.created_at.saturating_add(grace));
    store.rotate_token(token, existing_expire_at).await
}

/// The activation code of a token, until it is redeemed.
pub async fn pending_activation(
    encryptor: &EnvelopeEncryptor,
    token: &TokenRecord,
) -> Result<Option<String>> {
    let Some(sealed) = &token.sealed_token else {
        return Ok(None);
    };
    let clear = open_token(encryptor, &token.id, sealed).await?;
    Ok(Some(activation_code(&clear)))
}

/// A redeemed activation code: the token, as it was before, and the clear
/// bearer token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activated {
    pub token: TokenRecord,
    pub bearer_token: String,
}

/// Consume an activation code, returning the clear bearer token.
///
/// A code is good once, and only while its token is live; otherwise
/// [`Error::NotFound`].
pub async fn redeem(
    store: &dyn RecipientStore,
    encryptor: &EnvelopeEncryptor,
    code: &str,
) -> Result<Activated> {
    let token = store
        .redeem_activation(&token_hash(code), now_millis())
        .await?;
    let sealed = token.sealed_token.as_deref().ok_or(Error::NotFound)?;
    let bearer_token = open_token(encryptor, &token.id, sealed).await?;
    Ok(Activated {
        token,
        bearer_token,
    })
}

async fn open_token(
    encryptor: &EnvelopeEncryptor,
    token_id: &str,
    sealed: &[u8],
) -> Result<String> {
    let clear = encryptor
        .open(&token_aad(token_id), sealed)
        .await
        .map_err(sealing)?;
    String::from_utf8(clear.to_vec()).map_err(|_| Error::generic("stored token is not valid UTF-8"))
}

/// Re-wrap every token awaiting activation under the active key. Returns how
/// many were re-wrapped.
pub async fn rewrap_tokens(
    store: &dyn RecipientStore,
    encryptor: &EnvelopeEncryptor,
) -> Result<usize> {
    let mut rewrapped = 0;
    for token in store.sealed_tokens().await? {
        let Some(current) = &token.sealed_token else {
            continue;
        };
        if let Some(sealed) = encryptor.rewrap(current).await.map_err(sealing)?
            && store.reseal_token(&token.id, current, sealed).await?
        {
            rewrapped += 1;
        }
    }
    Ok(rewrapped)
}
