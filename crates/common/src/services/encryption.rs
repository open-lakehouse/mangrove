//! Envelope encryption for secrets at rest.
//!
//! This module re-exports the envelope-encryption primitives from
//! [`olai_store`]. Mangrove previously carried its own verbatim copy of these
//! types; they now live upstream so the same [`EnvelopeEncryptor`] can be handed
//! to [`olai_store::ManagedObjectStore`] to seal a resource's sensitive fields
//! inline on its object row.
//!
//! - Each secret value is encrypted under a fresh, random data encryption key
//!   (DEK, AES-256-GCM); the DEK is wrapped by a key encryption key (KEK) from a
//!   pluggable [`KeyProvider`]. Only the wrapped DEK is persisted with the
//!   ciphertext, so a KEK can be rotated by re-wrapping the DEK
//!   ([`EnvelopeEncryptor::rewrap`]) without re-encrypting the value.
//! - The AAD name is bound into every AEAD operation, so a ciphertext cannot be
//!   silently relocated to a different name/object.

pub use olai_store::encryption::{EnvelopeEncryptor, KekId, KeyProvider, LocalKeyProvider};
