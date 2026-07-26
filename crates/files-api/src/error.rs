//! Store-level errors and their mapping onto ConnectRPC error codes.

use connectrpc::ConnectError;

/// Errors produced by the file stores.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The requested resource does not exist.
    #[error("{0} not found")]
    NotFound(String),

    /// A resource with the same identity already exists.
    #[error("{0} already exists")]
    AlreadyExists(String),

    /// The request was malformed (missing/invalid fields).
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// The operation is not allowed in the resource's current state (e.g.
    /// deleting a non-empty directory).
    #[error("failed precondition: {0}")]
    FailedPrecondition(String),

    /// An unexpected backend/IO failure (e.g. object-store or credential error).
    #[error("internal error: {0}")]
    Internal(String),
}

impl From<StoreError> for ConnectError {
    fn from(err: StoreError) -> Self {
        match err {
            StoreError::NotFound(msg) => ConnectError::not_found(msg),
            StoreError::AlreadyExists(msg) => ConnectError::already_exists(msg),
            StoreError::InvalidArgument(msg) => ConnectError::invalid_argument(msg),
            StoreError::FailedPrecondition(msg) => ConnectError::failed_precondition(msg),
            StoreError::Internal(msg) => ConnectError::internal(msg),
        }
    }
}

/// Convenience alias for store operations.
pub type StoreResult<T> = Result<T, StoreError>;
