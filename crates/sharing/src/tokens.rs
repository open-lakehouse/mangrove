//! Recipient bearer tokens and activation codes, and the digests they are
//! stored and looked up by.

use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// A fresh, unguessable secret (≈244 random bits), URL-safe.
pub fn random_secret() -> String {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    bytes.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
    URL_SAFE_NO_PAD.encode(bytes)
}

/// The digest a token is stored and looked up by (sha256): what
/// [`TokenReader::token_by_hash`](crate::TokenReader::token_by_hash)
/// receives. Activation codes are stored and looked up by it too.
pub fn token_hash(token: &str) -> Vec<u8> {
    Sha256::digest(token.as_bytes()).to_vec()
}

/// The activation code of `token`, derived from it so that the code needs no
/// storage of its own: the store keeps only its digest, and the link is rebuilt
/// from the sealed token while that is kept. The code reveals nothing of the
/// token.
pub fn activation_code(token: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(b"opensharing-activation:");
    digest.update(token.as_bytes());
    URL_SAFE_NO_PAD.encode(digest.finalize())
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

    #[test]
    fn activation_codes_are_derived_and_distinct() {
        let (a, b) = (random_secret(), random_secret());
        assert_eq!(activation_code(&a), activation_code(&a));
        assert_ne!(activation_code(&a), activation_code(&b));
        assert_ne!(activation_code(&a), a);
        assert_ne!(token_hash(&activation_code(&a)), token_hash(&a));
    }
}
