//! Secrets at rest: envelope encryption with lazy key rotation.
//!
//! Every secret the server persists — storage credentials (local access) and
//! recipient tokens awaiting activation — is sealed by an [`EnvelopeEncryptor`]:
//! a fresh data key per value, wrapped by a key-encryption key (KEK) from a
//! pluggable [`KeyProvider`](olai_store::encryption::KeyProvider).
//! The AAD binds each ciphertext to its name, so a sealed value cannot be moved
//! to another row.
//!
//! Rotation is lazy: make a new KEK active and keep the old one as retired.
//! [`Secrets::open`] still decrypts values sealed under the retired KEK and hands
//! back a re-wrapped blob for the caller to write back, so values migrate to the
//! active KEK as they are read; `sharing-server rotate-keys` re-wraps everything
//! eagerly, after which the retired KEK can be dropped.

use std::sync::Arc;

use olai_store::encryption::EnvelopeEncryptor;

use crate::{Error, Result};

/// Seals and opens secrets. Cheap to clone.
#[derive(Clone)]
pub struct Secrets {
    encryptor: Arc<EnvelopeEncryptor>,
}

impl Secrets {
    pub fn new(encryptor: EnvelopeEncryptor) -> Self {
        Self {
            encryptor: Arc::new(encryptor),
        }
    }

    /// Seal `plaintext` under the active KEK, bound to `name`.
    pub async fn seal(&self, name: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        self.encryptor
            .seal(name, plaintext)
            .await
            .map_err(|e| Error::generic(format!("sealing secret: {e}")))
    }

    /// Open a sealed value bound to `name`.
    ///
    /// Returns the plaintext and, when `sealed` was wrapped under a retired KEK,
    /// the value re-wrapped under the active one, which the caller should persist.
    pub async fn open(&self, name: &str, sealed: &[u8]) -> Result<(Vec<u8>, Option<Vec<u8>>)> {
        let plaintext = self
            .encryptor
            .open(name, sealed)
            .await
            .map_err(|e| Error::generic(format!("opening secret: {e}")))?;
        let rewrapped = self
            .encryptor
            .rewrap(sealed)
            .await
            .map_err(|e| Error::generic(format!("re-wrapping secret: {e}")))?;
        Ok((plaintext.to_vec(), rewrapped))
    }

    /// Re-wrap `sealed` under the active KEK, or `None` if it already is.
    pub async fn rewrap(&self, sealed: &[u8]) -> Result<Option<Vec<u8>>> {
        self.encryptor
            .rewrap(sealed)
            .await
            .map_err(|e| Error::generic(format!("re-wrapping secret: {e}")))
    }
}
