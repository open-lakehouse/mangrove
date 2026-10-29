//! An in-memory [`ShareStore`], for tests and ephemeral demos.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

use async_trait::async_trait;

use super::{
    ExternalLocationRecord, RecipientRecord, SealedCredential, ShareObject, ShareRecord,
    ShareStore, TokenRecord, now_millis,
};
use crate::{Error, Result};

#[derive(Default)]
struct State {
    shares: BTreeMap<String, ShareRecord>,
    recipients: BTreeMap<String, RecipientRecord>,
    tokens: Vec<TokenRecord>,
    /// (share name, recipient name)
    grants: BTreeSet<(String, String)>,
    credentials: BTreeMap<String, SealedCredential>,
    locations: BTreeMap<String, ExternalLocationRecord>,
}

impl State {
    fn recipient_by_id(&self, id: &str) -> Result<&RecipientRecord> {
        self.recipients
            .values()
            .find(|r| r.id == id)
            .ok_or(Error::NotFound)
    }
}

/// An in-memory [`ShareStore`]. Nothing survives a restart.
#[derive(Default)]
pub struct InMemoryStore {
    state: Mutex<State>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.state.lock().expect("store lock poisoned")
    }
}

fn exists(what: &str, name: &str) -> Error {
    Error::invalid_argument(format!("{what} '{name}' already exists"))
}

#[async_trait]
impl ShareStore for InMemoryStore {
    async fn create_share(&self, name: &str, comment: Option<String>) -> Result<ShareRecord> {
        let mut state = self.state();
        if state.shares.contains_key(name) {
            return Err(exists("share", name));
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

    async fn get_share(&self, name: &str) -> Result<ShareRecord> {
        self.state()
            .shares
            .get(name)
            .cloned()
            .ok_or(Error::NotFound)
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

    async fn create_recipient(
        &self,
        name: &str,
        comment: Option<String>,
    ) -> Result<RecipientRecord> {
        let mut state = self.state();
        if state.recipients.contains_key(name) {
            return Err(exists("recipient", name));
        }
        let recipient = RecipientRecord {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            comment,
            created_at: now_millis(),
        };
        state.recipients.insert(name.to_string(), recipient.clone());
        Ok(recipient)
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

    async fn delete_recipient(&self, name: &str) -> Result<()> {
        let mut state = self.state();
        let recipient = state.recipients.remove(name).ok_or(Error::NotFound)?;
        state.tokens.retain(|t| t.recipient_id != recipient.id);
        state.grants.retain(|(_, r)| r != name);
        Ok(())
    }

    async fn insert_token(&self, token: TokenRecord) -> Result<()> {
        self.state().tokens.push(token);
        Ok(())
    }

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

    async fn token_by_activation_code(&self, code: &str) -> Result<TokenRecord> {
        self.state()
            .tokens
            .iter()
            .find(|t| t.activation_code.as_deref() == Some(code))
            .cloned()
            .ok_or(Error::NotFound)
    }

    async fn mark_activated(&self, token_id: &str) -> Result<()> {
        let mut state = self.state();
        let token = state
            .tokens
            .iter_mut()
            .find(|t| t.id == token_id)
            .ok_or(Error::NotFound)?;
        token.sealed_token = None;
        token.activation_code = None;
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
        tokens.sort_by_key(|t| std::cmp::Reverse(t.created_at));
        Ok(tokens)
    }

    async fn delete_tokens(&self, recipient_id: &str) -> Result<()> {
        self.state()
            .tokens
            .retain(|t| t.recipient_id != recipient_id);
        Ok(())
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

    async fn update_sealed_token(&self, token_id: &str, sealed: Vec<u8>) -> Result<()> {
        let mut state = self.state();
        let token = state
            .tokens
            .iter_mut()
            .find(|t| t.id == token_id)
            .ok_or(Error::NotFound)?;
        token.sealed_token = Some(sealed);
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

    async fn put_credential(&self, credential: SealedCredential) -> Result<()> {
        self.state()
            .credentials
            .insert(credential.name.clone(), credential);
        Ok(())
    }

    async fn get_credential(&self, name: &str) -> Result<SealedCredential> {
        self.state()
            .credentials
            .get(name)
            .cloned()
            .ok_or(Error::NotFound)
    }

    async fn list_credentials(&self) -> Result<Vec<SealedCredential>> {
        Ok(self.state().credentials.values().cloned().collect())
    }

    async fn delete_credential(&self, name: &str) -> Result<()> {
        self.state()
            .credentials
            .remove(name)
            .map(|_| ())
            .ok_or(Error::NotFound)
    }

    async fn put_external_location(&self, location: ExternalLocationRecord) -> Result<()> {
        self.state()
            .locations
            .insert(location.name.clone(), location);
        Ok(())
    }

    async fn list_external_locations(&self) -> Result<Vec<ExternalLocationRecord>> {
        Ok(self.state().locations.values().cloned().collect())
    }

    async fn delete_external_location(&self, name: &str) -> Result<()> {
        self.state()
            .locations
            .remove(name)
            .map(|_| ())
            .ok_or(Error::NotFound)
    }
}
