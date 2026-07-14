//! Unity Catalog acceptance testing.
//!
//! An API-coverage conformance battery run against a live Unity Catalog server.
//! See [`conformance`] for the check machinery and batteries, [`checks`] for the
//! per-securable checks, and `tests/conformance.rs` for the gated entry points.

pub mod checks;
pub mod conformance;
pub mod context;

pub use context::JourneyContext;

/// Result type used throughout the crate.
pub type AcceptanceResult<T> = Result<T, AcceptanceError>;

/// Errors surfaced by conformance checks.
#[derive(Debug, thiserror::Error)]
pub enum AcceptanceError {
    /// A check step failed (client call or assertion). Also carries the
    /// conformance skip sentinel; see [`conformance::skip`].
    #[error("Journey execution failed: {0}")]
    JourneyExecution(String),

    /// A check precondition or context was invalid (e.g. a malformed server URL).
    #[error("Journey validation failed: {0}")]
    JourneyValidation(String),

    /// JSON (de)serialization error.
    #[error("JSON parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error returned by the Unity Catalog client.
    #[error("Unity Catalog client error: {0}")]
    UnityCatalog(String),
}

impl From<unitycatalog_client::Error> for AcceptanceError {
    fn from(err: unitycatalog_client::Error) -> Self {
        AcceptanceError::UnityCatalog(err.to_string())
    }
}
