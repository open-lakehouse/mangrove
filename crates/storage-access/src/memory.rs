use std::collections::BTreeMap;
use std::sync::{PoisonError, RwLock};

use async_trait::async_trait;
use olai_storage_credentials::StorageLocation;
use unitycatalog_common::models::credentials::v1::{Credential, Purpose};
use unitycatalog_common::models::external_locations::v1::ExternalLocation;

use crate::rules::{credential_in_use_error, new_id, now_millis, overlap_error};
use crate::{
    CredentialSecrets, CredentialUpdate, Error, ExternalLocationUpdate, Governed, LocationKey,
    Page, Result, StorageAccessStore,
};

/// A [`StorageAccessStore`] held in process memory.
///
/// Every lookup scans the registered locations, which suits tests and small,
/// ephemeral servers.
#[derive(Debug, Default)]
pub struct InMemoryStorageAccess {
    state: RwLock<State>,
}

impl InMemoryStorageAccess {
    /// An empty store.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Records keyed by lowercased name, so names compare case-insensitively.
#[derive(Debug, Default)]
struct State {
    credentials: BTreeMap<String, StoredCredential>,
    locations: BTreeMap<String, StoredLocation>,
}

#[derive(Debug)]
struct StoredCredential {
    /// Without secrets.
    credential: Credential,
    secrets: CredentialSecrets,
}

#[derive(Debug)]
struct StoredLocation {
    /// `credential_name` is filled in from the credential on read.
    location: ExternalLocation,
    key: LocationKey,
}

fn fold(name: &str) -> String {
    name.to_lowercase()
}

impl State {
    fn credential_by_id(&self, id: &str) -> Option<&StoredCredential> {
        self.credentials
            .values()
            .find(|c| c.credential.id.as_deref() == Some(id))
    }

    fn read_location(&self, stored: &StoredLocation) -> ExternalLocation {
        let mut location = stored.location.clone();
        if let Some(credential) = self.credential_by_id(&location.credential_id) {
            location.credential_name = credential.credential.name.clone();
        }
        location
    }

    fn check_no_overlap(
        &self,
        url: &str,
        key: &LocationKey,
        except_id: Option<&str>,
    ) -> Result<()> {
        let overlapping = self.locations.values().find(|l| {
            l.location.external_location_id.as_deref() != except_id && l.key.overlaps(key)
        });
        match overlapping {
            Some(l) => Err(overlap_error(url, &l.location.name, &l.location.url)),
            None => Ok(()),
        }
    }

    fn covering(&self, location: &StorageLocation) -> Option<&StoredLocation> {
        let key = LocationKey::new(location);
        // Locations never overlap, so at most one covers `location`.
        self.locations.values().find(|l| l.key.covers(&key))
    }

    fn credential_id(&self, name: &str) -> Result<String> {
        self.credentials
            .get(&fold(name))
            .and_then(|c| c.credential.id.clone())
            .ok_or(Error::NotFound)
    }
}

/// The page of `entries` (in key order) after `page.token`, and the next token.
fn paginate<T: Clone>(
    entries: &BTreeMap<String, T>,
    page: &Page,
    keep: impl Fn(&T) -> bool,
) -> (Vec<(String, T)>, Option<String>) {
    let after = page.token.as_deref().map(fold);
    let mut items: Vec<(String, T)> = entries
        .iter()
        .filter(|(k, _)| after.as_ref().is_none_or(|a| *k > a))
        .filter(|(_, v)| keep(v))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    let next = match page.max_results {
        Some(max) if items.len() > max => {
            items.truncate(max);
            items.last().map(|(k, _)| k.clone())
        }
        _ => None,
    };
    (items, next)
}

#[async_trait]
impl StorageAccessStore for InMemoryStorageAccess {
    async fn create_credential(&self, mut credential: Credential) -> Result<Credential> {
        let mut state = self.state.write().unwrap_or_else(PoisonError::into_inner);
        let key = fold(&credential.name);
        if state.credentials.contains_key(&key) {
            return Err(Error::AlreadyExists);
        }
        let secrets = CredentialSecrets::take(&mut credential);
        let now = now_millis();
        credential.id = Some(new_id());
        credential.full_name = credential.name.clone();
        credential.created_at = Some(now);
        credential.updated_at = Some(now);
        state.credentials.insert(
            key,
            StoredCredential {
                credential: credential.clone(),
                secrets,
            },
        );
        Ok(credential)
    }

    async fn get_credential(&self, name: &str) -> Result<Credential> {
        let state = self.state.read().unwrap_or_else(PoisonError::into_inner);
        state
            .credentials
            .get(&fold(name))
            .map(|c| c.credential.clone())
            .ok_or(Error::NotFound)
    }

    async fn list_credentials(
        &self,
        purpose: Option<Purpose>,
        page: Page,
    ) -> Result<(Vec<Credential>, Option<String>)> {
        let state = self.state.read().unwrap_or_else(PoisonError::into_inner);
        let credentials: BTreeMap<String, Credential> = state
            .credentials
            .iter()
            .map(|(k, v)| (k.clone(), v.credential.clone()))
            .collect();
        let (items, next) = paginate(&credentials, &page, |c| {
            purpose.is_none_or(|p| c.purpose.as_known() == Some(p))
        });
        Ok((items.into_iter().map(|(_, c)| c).collect(), next))
    }

    async fn update_credential(&self, name: &str, update: CredentialUpdate) -> Result<Credential> {
        let mut state = self.state.write().unwrap_or_else(PoisonError::into_inner);
        let key = fold(name);
        let new_key = update.new_name.as_deref().map(fold);
        if let Some(new_key) = &new_key
            && *new_key != key
            && state.credentials.contains_key(new_key)
        {
            return Err(Error::AlreadyExists);
        }
        let mut stored = state.credentials.remove(&key).ok_or(Error::NotFound)?;
        let credential = &mut stored.credential;
        if let Some(new_name) = update.new_name {
            credential.full_name = new_name.clone();
            credential.name = new_name;
        }
        if let Some(comment) = update.comment {
            credential.comment = Some(comment);
        }
        if let Some(read_only) = update.read_only {
            credential.read_only = read_only;
        }
        if let Some(owner) = update.owner {
            credential.owner = Some(owner);
        }
        if let Some(secrets) = update.secrets {
            stored.secrets = secrets;
        }
        credential.updated_at = Some(now_millis());
        let updated = credential.clone();
        state.credentials.insert(new_key.unwrap_or(key), stored);
        Ok(updated)
    }

    async fn delete_credential(&self, name: &str) -> Result<()> {
        let mut state = self.state.write().unwrap_or_else(PoisonError::into_inner);
        let key = fold(name);
        let id = state.credential_id(name)?;
        let users: Vec<String> = state
            .locations
            .values()
            .filter(|l| l.location.credential_id == id)
            .map(|l| l.location.name.clone())
            .collect();
        if !users.is_empty() {
            return Err(credential_in_use_error(name, &users));
        }
        state.credentials.remove(&key);
        Ok(())
    }

    async fn create_external_location(
        &self,
        mut location: ExternalLocation,
    ) -> Result<ExternalLocation> {
        let mut state = self.state.write().unwrap_or_else(PoisonError::into_inner);
        let name_key = fold(&location.name);
        if state.locations.contains_key(&name_key) {
            return Err(Error::AlreadyExists);
        }
        let key = LocationKey::parse(&location.url)?;
        state.check_no_overlap(&location.url, &key, None)?;
        location.credential_id = state.credential_id(&location.credential_name)?;
        let now = now_millis();
        location.external_location_id = Some(new_id());
        location.created_at = Some(now);
        location.updated_at = Some(now);
        let stored = StoredLocation { location, key };
        let read = state.read_location(&stored);
        state.locations.insert(name_key, stored);
        Ok(read)
    }

    async fn get_external_location(&self, name: &str) -> Result<ExternalLocation> {
        let state = self.state.read().unwrap_or_else(PoisonError::into_inner);
        let stored = state.locations.get(&fold(name)).ok_or(Error::NotFound)?;
        Ok(state.read_location(stored))
    }

    async fn list_external_locations(
        &self,
        page: Page,
    ) -> Result<(Vec<ExternalLocation>, Option<String>)> {
        let state = self.state.read().unwrap_or_else(PoisonError::into_inner);
        let locations: BTreeMap<String, ExternalLocation> = state
            .locations
            .iter()
            .map(|(k, v)| (k.clone(), state.read_location(v)))
            .collect();
        let (items, next) = paginate(&locations, &page, |_| true);
        Ok((items.into_iter().map(|(_, l)| l).collect(), next))
    }

    async fn update_external_location(
        &self,
        name: &str,
        update: ExternalLocationUpdate,
    ) -> Result<ExternalLocation> {
        let mut state = self.state.write().unwrap_or_else(PoisonError::into_inner);
        let name_key = fold(name);
        let current = state.locations.get(&name_key).ok_or(Error::NotFound)?;
        let id = current.location.external_location_id.clone();
        let new_name_key = update.new_name.as_deref().map(fold);
        if let Some(new_name_key) = &new_name_key
            && *new_name_key != name_key
            && state.locations.contains_key(new_name_key)
        {
            return Err(Error::AlreadyExists);
        }
        let key = match &update.url {
            Some(url) => {
                let key = LocationKey::parse(url)?;
                state.check_no_overlap(url, &key, id.as_deref())?;
                Some(key)
            }
            None => None,
        };
        let credential_id = update
            .credential_name
            .as_deref()
            .map(|n| state.credential_id(n))
            .transpose()?;

        let mut stored = state.locations.remove(&name_key).ok_or(Error::NotFound)?;
        let location = &mut stored.location;
        if let Some(new_name) = update.new_name {
            location.name = new_name;
        }
        if let (Some(url), Some(key)) = (update.url, key) {
            location.url = url;
            stored.key = key;
        }
        if let Some(credential_id) = credential_id {
            location.credential_id = credential_id;
        }
        if let Some(read_only) = update.read_only {
            location.read_only = read_only;
        }
        if let Some(comment) = update.comment {
            location.comment = Some(comment);
        }
        if let Some(owner) = update.owner {
            location.owner = Some(owner);
        }
        location.updated_at = Some(now_millis());
        let read = state.read_location(&stored);
        state
            .locations
            .insert(new_name_key.unwrap_or(name_key), stored);
        Ok(read)
    }

    async fn delete_external_location(&self, name: &str) -> Result<()> {
        let mut state = self.state.write().unwrap_or_else(PoisonError::into_inner);
        state
            .locations
            .remove(&fold(name))
            .map(|_| ())
            .ok_or(Error::NotFound)
    }

    async fn governing_location(&self, location: &StorageLocation) -> Result<Governed> {
        let state = self.state.read().unwrap_or_else(PoisonError::into_inner);
        let stored = state.covering(location).ok_or(Error::NotFound)?;
        let credential = state
            .credential_by_id(&stored.location.credential_id)
            .ok_or(Error::NotFound)?;
        let mut with_secrets = credential.credential.clone();
        credential.secrets.clone().apply(&mut with_secrets);
        Ok(Governed {
            location: state.read_location(stored),
            credential: with_secrets,
        })
    }

    async fn covering_location(
        &self,
        location: &StorageLocation,
    ) -> Result<Option<ExternalLocation>> {
        let state = self.state.read().unwrap_or_else(PoisonError::into_inner);
        Ok(state.covering(location).map(|l| state.read_location(l)))
    }

    /// Secrets are held in the clear, so there is nothing to re-wrap.
    async fn rewrap_secrets(&self) -> Result<usize> {
        Ok(0)
    }
}

#[cfg(all(test, feature = "testing"))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn conformance() {
        crate::testing::conformance(|| async { InMemoryStorageAccess::new() }).await;
    }
}
