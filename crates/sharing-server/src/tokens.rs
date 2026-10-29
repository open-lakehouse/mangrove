//! Recipient bearer tokens and activation codes.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// A fresh, unguessable secret (≈244 random bits), URL-safe.
pub(crate) fn random_secret() -> String {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    URL_SAFE_NO_PAD.encode(bytes)
}

/// The digest a token is stored and looked up by.
pub(crate) fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secrets_are_unique_and_hash_stably() {
        let (a, b) = (random_secret(), random_secret());
        assert_ne!(a, b);
        assert!(a.len() >= 40);
        assert_eq!(token_hash(&a), token_hash(&a));
        assert_ne!(token_hash(&a), token_hash(&b));
    }
}
