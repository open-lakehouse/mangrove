//! Envelope encryption for secrets at rest.
//!
//! Secrets are protected with a standard envelope-encryption scheme:
//!
//! - Each secret value is encrypted under a fresh, random **data encryption key** (DEK,
//!   AES-256-GCM) with a random 96-bit nonce.
//! - The DEK is itself encrypted ("wrapped") by a **key encryption key** (KEK) obtained from a
//!   pluggable [`KeyProvider`]. Only the wrapped DEK is persisted alongside the ciphertext.
//!
//! The persisted blob is self-describing: it records which KEK wrapped the DEK so a
//! value can always be decrypted, and so a KEK can be rotated by re-wrapping the DEK without
//! re-encrypting the (potentially large) value — see [`EnvelopeEncryptor::rewrap`].
//!
//! The secret name is bound into every AEAD operation as associated data (AAD), so a ciphertext
//! cannot be silently relocated to a different secret name.
//!
//! The default [`KeyProvider`] is [`LocalKeyProvider`], which holds KEK material in process. The
//! [`KeyProvider`] trait is async so external key stores (Azure Key Vault, AWS KMS, GCP KMS) can be
//! plugged in later to perform remote wrap/unwrap without changing callers.

use aes_gcm::aead::consts::U12;
use aes_gcm::aead::rand_core::RngCore;
use aes_gcm::aead::{Aead, KeyInit, OsRng, Payload};
use aes_gcm::{AeadCore, Aes256Gcm, Key, Nonce};
use std::collections::HashMap;
use std::sync::Arc;
use zeroize::Zeroizing;

use crate::{Error, Result};

/// Identifier for a key encryption key (KEK).
///
/// A short, stable tag (e.g. `"v1"`) recorded in every envelope so the wrapping key can be
/// located at decrypt time and so rotation can detect blobs sealed under a retired key.
pub type KekId = String;

const ENVELOPE_VERSION: u8 = 1;
/// AES-256 key length in bytes.
const KEY_LEN: usize = 32;
/// AES-GCM nonce length in bytes (96 bits).
const NONCE_LEN: usize = 12;

fn crypto_err(msg: &str) -> Error {
    // Deliberately opaque: never leak key material or plaintext details into errors/logs.
    Error::generic(format!("encryption error: {msg}"))
}

/// Wraps and unwraps data encryption keys (DEKs) under a key encryption key (KEK).
///
/// Implementations may hold key material locally ([`LocalKeyProvider`]) or delegate to an external
/// key store. The trait is async so remote providers can perform network round-trips.
#[async_trait::async_trait]
pub trait KeyProvider: Send + Sync + 'static {
    /// The KEK that new writes should wrap their DEK under.
    fn active_kek_id(&self) -> KekId;

    /// Encrypt (wrap) a DEK under the KEK identified by `kek_id`.
    async fn wrap_dek(&self, dek: &[u8], kek_id: &str) -> Result<Vec<u8>>;

    /// Decrypt (unwrap) a previously wrapped DEK using the KEK identified by `kek_id`.
    async fn unwrap_dek(&self, wrapped: &[u8], kek_id: &str) -> Result<Zeroizing<Vec<u8>>>;
}

/// A [`KeyProvider`] that keeps KEK material in process.
///
/// Holds one active KEK plus zero or more retired KEKs, all keyed by [`KekId`]. Retired keys remain
/// available so values sealed under them can still be unwrapped (and lazily re-wrapped under the
/// active key during rotation).
pub struct LocalKeyProvider {
    active: KekId,
    keys: HashMap<KekId, Zeroizing<[u8; KEY_LEN]>>,
}

impl LocalKeyProvider {
    /// Build a provider from an active KEK and any number of retired KEKs.
    ///
    /// Each key must be exactly 32 bytes (AES-256). The active key must be present in `keys`.
    pub fn new(
        active: impl Into<KekId>,
        keys: impl IntoIterator<Item = (KekId, Vec<u8>)>,
    ) -> Result<Self> {
        let active = active.into();
        let mut map = HashMap::new();
        for (id, bytes) in keys {
            if bytes.len() != KEY_LEN {
                return Err(Error::invalid_argument(format!(
                    "KEK '{id}' must be {KEY_LEN} bytes, got {}",
                    bytes.len()
                )));
            }
            let mut arr = [0u8; KEY_LEN];
            arr.copy_from_slice(&bytes);
            map.insert(id, Zeroizing::new(arr));
        }
        if !map.contains_key(&active) {
            return Err(Error::invalid_argument(format!(
                "active KEK '{active}' was not provided among the configured keys"
            )));
        }
        Ok(Self { active, keys: map })
    }

    /// Convenience constructor for a single active KEK with no retired keys.
    pub fn single(active: impl Into<KekId>, key: Vec<u8>) -> Result<Self> {
        let active = active.into();
        Self::new(active.clone(), [(active, key)])
    }

    fn cipher(&self, kek_id: &str) -> Result<Aes256Gcm> {
        let key = self
            .keys
            .get(kek_id)
            .ok_or_else(|| crypto_err("unknown KEK id"))?;
        Ok(Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(key.as_ref())))
    }
}

#[async_trait::async_trait]
impl KeyProvider for LocalKeyProvider {
    fn active_kek_id(&self) -> KekId {
        self.active.clone()
    }

    async fn wrap_dek(&self, dek: &[u8], kek_id: &str) -> Result<Vec<u8>> {
        let cipher = self.cipher(kek_id)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        // Bind the KEK id as AAD so a wrapped DEK can't be reinterpreted under another key.
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: dek,
                    aad: kek_id.as_bytes(),
                },
            )
            .map_err(|_| crypto_err("DEK wrap failed"))?;
        let mut out = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        out.extend_from_slice(nonce.as_slice());
        out.extend_from_slice(&ciphertext);
        Ok(out)
    }

    async fn unwrap_dek(&self, wrapped: &[u8], kek_id: &str) -> Result<Zeroizing<Vec<u8>>> {
        if wrapped.len() < NONCE_LEN {
            return Err(crypto_err("wrapped DEK too short"));
        }
        let cipher = self.cipher(kek_id)?;
        let (nonce_bytes, ciphertext) = wrapped.split_at(NONCE_LEN);
        let nonce = Nonce::<U12>::from_slice(nonce_bytes);
        let dek = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: ciphertext,
                    aad: kek_id.as_bytes(),
                },
            )
            .map_err(|_| crypto_err("DEK unwrap failed"))?;
        Ok(Zeroizing::new(dek))
    }
}

/// A decoded envelope: the wrapped DEK plus the value ciphertext and the metadata needed to open
/// it. Serialized to/from the self-describing on-disk format via its `encode` / `decode` methods.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Envelope {
    kek_id: KekId,
    nonce: Vec<u8>,
    wrapped_dek: Vec<u8>,
    ciphertext: Vec<u8>,
}

impl Envelope {
    /// Serialize to the explicit, versioned binary layout:
    ///
    /// ```text
    /// version: u8
    /// kek_id_len: u8,  kek_id: bytes
    /// nonce_len: u8,   nonce: bytes
    /// wrapped_len: u32 (BE), wrapped_dek: bytes
    /// ciphertext: remaining bytes
    /// ```
    fn encode(&self) -> Result<Vec<u8>> {
        if self.kek_id.len() > u8::MAX as usize {
            return Err(crypto_err("KEK id too long"));
        }
        if self.nonce.len() > u8::MAX as usize {
            return Err(crypto_err("nonce too long"));
        }
        let wrapped_len = u32::try_from(self.wrapped_dek.len())
            .map_err(|_| crypto_err("wrapped DEK too long"))?;
        let mut out = Vec::with_capacity(
            1 + 1
                + self.kek_id.len()
                + 1
                + self.nonce.len()
                + 4
                + self.wrapped_dek.len()
                + self.ciphertext.len(),
        );
        out.push(ENVELOPE_VERSION);
        out.push(self.kek_id.len() as u8);
        out.extend_from_slice(self.kek_id.as_bytes());
        out.push(self.nonce.len() as u8);
        out.extend_from_slice(&self.nonce);
        out.extend_from_slice(&wrapped_len.to_be_bytes());
        out.extend_from_slice(&self.wrapped_dek);
        out.extend_from_slice(&self.ciphertext);
        Ok(out)
    }

    fn decode(blob: &[u8]) -> Result<Self> {
        let mut cur = blob;
        let version = take_u8(&mut cur)?;
        if version != ENVELOPE_VERSION {
            return Err(crypto_err("unsupported envelope version"));
        }
        let kek_len = take_u8(&mut cur)? as usize;
        let kek_bytes = take(&mut cur, kek_len)?;
        let kek_id =
            String::from_utf8(kek_bytes.to_vec()).map_err(|_| crypto_err("invalid KEK id"))?;
        let nonce_len = take_u8(&mut cur)? as usize;
        let nonce = take(&mut cur, nonce_len)?.to_vec();
        let wrapped_len = take_u32(&mut cur)? as usize;
        let wrapped_dek = take(&mut cur, wrapped_len)?.to_vec();
        let ciphertext = cur.to_vec();
        Ok(Self {
            kek_id,
            nonce,
            wrapped_dek,
            ciphertext,
        })
    }
}

fn take<'a>(cur: &mut &'a [u8], n: usize) -> Result<&'a [u8]> {
    if cur.len() < n {
        return Err(crypto_err("truncated envelope"));
    }
    let (head, tail) = cur.split_at(n);
    *cur = tail;
    Ok(head)
}

fn take_u8(cur: &mut &[u8]) -> Result<u8> {
    Ok(take(cur, 1)?[0])
}

fn take_u32(cur: &mut &[u8]) -> Result<u32> {
    let bytes = take(cur, 4)?;
    Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
}

/// High-level envelope-encryption API used by secret stores.
///
/// Seals plaintext into a self-describing envelope blob and opens it again, delegating DEK
/// wrap/unwrap to the configured [`KeyProvider`].
#[derive(Clone)]
pub struct EnvelopeEncryptor {
    provider: Arc<dyn KeyProvider>,
}

impl std::fmt::Debug for EnvelopeEncryptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Never expose key material.
        f.debug_struct("EnvelopeEncryptor")
            .field("active_kek_id", &self.provider.active_kek_id())
            .finish()
    }
}

impl EnvelopeEncryptor {
    pub fn new(provider: Arc<dyn KeyProvider>) -> Self {
        Self { provider }
    }

    /// Convenience constructor wrapping a [`LocalKeyProvider`].
    pub fn local(provider: LocalKeyProvider) -> Self {
        Self::new(Arc::new(provider))
    }

    /// Encrypt `plaintext` for the secret named `name`, returning a serialized envelope blob.
    ///
    /// `name` is bound as AEAD associated data so the blob cannot be opened under a different name.
    pub async fn seal(&self, name: &str, plaintext: &[u8]) -> Result<Vec<u8>> {
        // Fresh random DEK per value.
        let mut dek = Zeroizing::new([0u8; KEY_LEN]);
        OsRng.fill_bytes(dek.as_mut_slice());
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(dek.as_slice()));
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: plaintext,
                    aad: name.as_bytes(),
                },
            )
            .map_err(|_| crypto_err("value encryption failed"))?;

        let kek_id = self.provider.active_kek_id();
        let wrapped_dek = self.provider.wrap_dek(dek.as_slice(), &kek_id).await?;

        Envelope {
            kek_id,
            nonce: nonce.to_vec(),
            wrapped_dek,
            ciphertext,
        }
        .encode()
    }

    /// Decrypt a serialized envelope `blob` for the secret named `name`.
    pub async fn open(&self, name: &str, blob: &[u8]) -> Result<Vec<u8>> {
        let envelope = Envelope::decode(blob)?;
        let dek = self
            .provider
            .unwrap_dek(&envelope.wrapped_dek, &envelope.kek_id)
            .await?;
        if dek.len() != KEY_LEN {
            return Err(crypto_err("unwrapped DEK has wrong length"));
        }
        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&dek));
        if envelope.nonce.len() != NONCE_LEN {
            return Err(crypto_err("invalid nonce length"));
        }
        let nonce = Nonce::<U12>::from_slice(&envelope.nonce);
        let plaintext = cipher
            .decrypt(
                nonce,
                Payload {
                    msg: &envelope.ciphertext,
                    aad: name.as_bytes(),
                },
            )
            .map_err(|_| crypto_err("value decryption failed"))?;
        Ok(plaintext)
    }

    /// Re-wrap a blob's DEK under the active KEK if it was sealed under a different (retired) KEK.
    ///
    /// Returns `Ok(Some(new_blob))` when re-wrapping occurred, or `Ok(None)` when the blob is
    /// already sealed under the active KEK. The value ciphertext is never touched, so this is cheap
    /// regardless of value size.
    pub async fn rewrap(&self, blob: &[u8]) -> Result<Option<Vec<u8>>> {
        let envelope = Envelope::decode(blob)?;
        let active = self.provider.active_kek_id();
        if envelope.kek_id == active {
            return Ok(None);
        }
        let dek = self
            .provider
            .unwrap_dek(&envelope.wrapped_dek, &envelope.kek_id)
            .await?;
        let wrapped_dek = self.provider.wrap_dek(dek.as_slice(), &active).await?;
        let rewrapped = Envelope {
            kek_id: active,
            nonce: envelope.nonce,
            wrapped_dek,
            ciphertext: envelope.ciphertext,
        }
        .encode()?;
        Ok(Some(rewrapped))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(byte: u8) -> Vec<u8> {
        vec![byte; KEY_LEN]
    }

    fn encryptor_single() -> EnvelopeEncryptor {
        EnvelopeEncryptor::local(LocalKeyProvider::single("v1", key(1)).unwrap())
    }

    #[tokio::test]
    async fn round_trip() {
        let enc = encryptor_single();
        let blob = enc.seal("my-secret", b"super secret value").await.unwrap();
        // The plaintext must not appear in the stored blob.
        assert!(
            !blob
                .windows(b"super secret value".len())
                .any(|w| w == b"super secret value")
        );
        let out = enc.open("my-secret", &blob).await.unwrap();
        assert_eq!(out, b"super secret value");
    }

    #[tokio::test]
    async fn wrong_name_aad_fails() {
        let enc = encryptor_single();
        let blob = enc.seal("name-a", b"value").await.unwrap();
        assert!(enc.open("name-b", &blob).await.is_err());
    }

    #[tokio::test]
    async fn tampered_ciphertext_fails() {
        let enc = encryptor_single();
        let mut blob = enc.seal("n", b"value").await.unwrap();
        let last = blob.len() - 1;
        blob[last] ^= 0xff;
        assert!(enc.open("n", &blob).await.is_err());
    }

    #[tokio::test]
    async fn tampered_wrapped_dek_fails() {
        let enc = encryptor_single();
        let blob = enc.seal("n", b"value").await.unwrap();
        // Flip a byte inside the wrapped-DEK region (just past the header + small nonce).
        let mut tampered = blob.clone();
        let idx = 1 + 1 + 2 /* "v1" */ + 1 + NONCE_LEN + 4 + 2;
        tampered[idx] ^= 0x01;
        assert!(enc.open("n", &tampered).await.is_err());
    }

    #[tokio::test]
    async fn rewrap_from_retired_to_active() {
        // Seal under v1 (active), then rotate so v2 is active and v1 retired.
        let v1 = EnvelopeEncryptor::local(LocalKeyProvider::single("v1", key(1)).unwrap());
        let blob = v1.seal("n", b"value").await.unwrap();

        let rotated = EnvelopeEncryptor::local(
            LocalKeyProvider::new("v2", [("v1".into(), key(1)), ("v2".into(), key(2))]).unwrap(),
        );
        // Old blob still opens (v1 retained).
        assert_eq!(rotated.open("n", &blob).await.unwrap(), b"value");

        let rewrapped = rotated.rewrap(&blob).await.unwrap().expect("should rewrap");
        // Now sealed under the active key; rewrapping again is a no-op.
        assert!(rotated.rewrap(&rewrapped).await.unwrap().is_none());
        assert_eq!(rotated.open("n", &rewrapped).await.unwrap(), b"value");

        // A provider holding only v2 can open the rewrapped blob but not the original.
        let v2_only = EnvelopeEncryptor::local(LocalKeyProvider::single("v2", key(2)).unwrap());
        assert_eq!(v2_only.open("n", &rewrapped).await.unwrap(), b"value");
        assert!(v2_only.open("n", &blob).await.is_err());
    }

    #[test]
    fn rejects_bad_key_length() {
        assert!(LocalKeyProvider::single("v1", vec![0u8; 16]).is_err());
    }

    #[test]
    fn rejects_missing_active() {
        assert!(LocalKeyProvider::new("v9", [("v1".into(), key(1))]).is_err());
    }
}
