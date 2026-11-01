//! An in-memory [`SharingStore`](crate::SharingStore), for tests and ephemeral servers.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use async_trait::async_trait;

use crate::{
    Error, RecipientRecord, RecipientStore, RecipientUpdate, Result, ShareObject, ShareReader,
    ShareRecord, ShareStore, TokenReader, TokenRecord, now_millis,
};

#[derive(Default)]
struct State {
    shares: BTreeMap<String, ShareRecord>,
    recipients: BTreeMap<String, RecipientRecord>,
    tokens: Vec<TokenRecord>,
    /// (share name, recipient name)
    grants: BTreeSet<(String, String)>,
}

impl State {
    fn recipient_by_id(&self, id: &str) -> Result<&RecipientRecord> {
        self.recipients
            .values()
            .find(|r| r.id == id)
            .ok_or(Error::NotFound)
    }
}

/// An in-memory [`SharingStore`](crate::SharingStore). Nothing survives a restart.
#[derive(Default)]
pub struct InMemorySharingStore {
    state: Mutex<State>,
}

impl InMemorySharingStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().expect("store lock poisoned")
    }
}

#[async_trait]
impl ShareReader for InMemorySharingStore {
    async fn get_share(&self, name: &str) -> Result<ShareRecord> {
        self.state()
            .shares
            .get(name)
            .cloned()
            .ok_or(Error::NotFound)
    }

    async fn shares_of(&self, recipient_id: &str) -> Result<Vec<ShareRecord>> {
        let state = self.state();
        let recipient = state.recipient_by_id(recipient_id)?.name.clone();
        Ok(state
            .grants
            .iter()
            .filter(|(_, r)| *r == recipient)
            .filter_map(|(s, _)| state.shares.get(s).cloned())
            .collect())
    }

    async fn is_granted(&self, share: &str, recipient_id: &str) -> Result<bool> {
        let state = self.state();
        let Ok(recipient) = state.recipient_by_id(recipient_id) else {
            return Ok(false);
        };
        Ok(state
            .grants
            .contains(&(share.to_string(), recipient.name.clone())))
    }
}

#[async_trait]
impl ShareStore for InMemorySharingStore {
    async fn create_share(&self, name: &str, comment: Option<String>) -> Result<ShareRecord> {
        let mut state = self.state();
        if state.shares.contains_key(name) {
            return Err(Error::AlreadyExists);
        }
        let now = now_millis();
        let share = ShareRecord {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            comment,
            objects: Vec::new(),
            created_at: now,
            updated_at: now,
        };
        state.shares.insert(name.to_string(), share.clone());
        Ok(share)
    }

    async fn list_shares(&self) -> Result<Vec<ShareRecord>> {
        Ok(self.state().shares.values().cloned().collect())
    }

    async fn update_share(
        &self,
        name: &str,
        objects: Vec<ShareObject>,
        comment: Option<String>,
    ) -> Result<ShareRecord> {
        let mut state = self.state();
        let share = state.shares.get_mut(name).ok_or(Error::NotFound)?;
        share.objects = objects;
        share.comment = comment;
        share.updated_at = now_millis();
        Ok(share.clone())
    }

    async fn delete_share(&self, name: &str) -> Result<()> {
        let mut state = self.state();
        state.shares.remove(name).ok_or(Error::NotFound)?;
        state.grants.retain(|(share, _)| share != name);
        Ok(())
    }

    async fn grant(&self, share: &str, recipient: &str) -> Result<()> {
        let mut state = self.state();
        if !state.shares.contains_key(share) || !state.recipients.contains_key(recipient) {
            return Err(Error::NotFound);
        }
        state
            .grants
            .insert((share.to_string(), recipient.to_string()));
        Ok(())
    }

    async fn revoke(&self, share: &str, recipient: &str) -> Result<()> {
        self.state()
            .grants
            .remove(&(share.to_string(), recipient.to_string()));
        Ok(())
    }

    async fn recipients_of(&self, share: &str) -> Result<Vec<String>> {
        Ok(self
            .state()
            .grants
            .iter()
            .filter(|(s, _)| s == share)
            .map(|(_, r)| r.clone())
            .collect())
    }
}

#[async_trait]
impl TokenReader for InMemorySharingStore {
    async fn token_by_hash(&self, token_hash: &[u8]) -> Result<(TokenRecord, RecipientRecord)> {
        let state = self.state();
        let token = state
            .tokens
            .iter()
            .find(|t| t.token_hash == token_hash)
            .ok_or(Error::NotFound)?;
        let recipient = state.recipient_by_id(&token.recipient_id)?;
        Ok((token.clone(), recipient.clone()))
    }
}

// Each operation runs under the one lock, so each is atomic.
#[async_trait]
impl RecipientStore for InMemorySharingStore {
    async fn create_recipient(&self, recipient: RecipientRecord, token: TokenRecord) -> Result<()> {
        let mut state = self.state();
        if state.recipients.contains_key(&recipient.name) {
            return Err(Error::AlreadyExists);
        }
        state.recipients.insert(recipient.name.clone(), recipient);
        state.tokens.push(token);
        Ok(())
    }

    async fn get_recipient(&self, name: &str) -> Result<RecipientRecord> {
        self.state()
            .recipients
            .get(name)
            .cloned()
            .ok_or(Error::NotFound)
    }

    async fn list_recipients(&self) -> Result<Vec<RecipientRecord>> {
        Ok(self.state().recipients.values().cloned().collect())
    }

    async fn update_recipient(
        &self,
        name: &str,
        update: RecipientUpdate,
    ) -> Result<RecipientRecord> {
        let mut state = self.state();
        let recipient = state.recipients.get_mut(name).ok_or(Error::NotFound)?;
        if let Some(comment) = update.comment {
            recipient.comment = Some(comment);
        }
        let recipient = recipient.clone();
        if let Some(expires_at) = update.expires_at
            && let Some(newest) = state
                .tokens
                .iter_mut()
                .filter(|t| t.recipient_id == recipient.id)
                .max_by(|a, b| newest_last(a, b))
        {
            newest.expires_at = expires_at;
        }
        Ok(recipient)
    }

    async fn delete_recipient(&self, name: &str) -> Result<()> {
        let mut state = self.state();
        let recipient = state.recipients.remove(name).ok_or(Error::NotFound)?;
        state.tokens.retain(|t| t.recipient_id != recipient.id);
        state.grants.retain(|(_, r)| r != name);
        Ok(())
    }

    async fn tokens_for_recipient(&self, recipient_id: &str) -> Result<Vec<TokenRecord>> {
        let mut tokens: Vec<_> = self
            .state()
            .tokens
            .iter()
            .filter(|t| t.recipient_id == recipient_id)
            .cloned()
            .collect();
        tokens.sort_by(|a, b| newest_last(b, a));
        Ok(tokens)
    }

    async fn rotate_token(
        &self,
        token: TokenRecord,
        existing_expire_at: Option<i64>,
    ) -> Result<()> {
        let mut state = self.state();
        state.recipient_by_id(&token.recipient_id)?;
        let now = token.created_at;
        state.tokens.retain_mut(|t| {
            if t.recipient_id != token.recipient_id {
                return true;
            }
            let Some(at) = existing_expire_at else {
                return false;
            };
            t.expires_at = Some(t.expires_at.map_or(at, |e| e.min(at)));
            t.is_live(now)
        });
        state.tokens.push(token);
        Ok(())
    }

    async fn redeem_activation(&self, code_hash: &[u8], now: i64) -> Result<TokenRecord> {
        let mut state = self.state();
        let token = state
            .tokens
            .iter_mut()
            .find(|t| t.activation_code_hash.as_deref() == Some(code_hash) && t.is_live(now))
            .ok_or(Error::NotFound)?;
        let redeemed = token.clone();
        token.sealed_token = None;
        token.activation_code_hash = None;
        Ok(redeemed)
    }

    async fn sealed_tokens(&self) -> Result<Vec<TokenRecord>> {
        Ok(self
            .state()
            .tokens
            .iter()
            .filter(|t| t.sealed_token.is_some())
            .cloned()
            .collect())
    }

    async fn reseal_token(&self, token_id: &str, current: &[u8], sealed: Vec<u8>) -> Result<bool> {
        let mut state = self.state();
        match state
            .tokens
            .iter_mut()
            .find(|t| t.id == token_id && t.sealed_token.as_deref() == Some(current))
        {
            Some(token) => {
                token.sealed_token = Some(sealed);
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

/// Orders tokens by creation, the SQL stores' `created_at, id` order.
fn newest_last(a: &TokenRecord, b: &TokenRecord) -> std::cmp::Ordering {
    (a.created_at, &a.id).cmp(&(b.created_at, &b.id))
}
