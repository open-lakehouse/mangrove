//! The server's error type, shared with the handler traits it implements.
pub use unitycatalog_handlers::error::*;

/// Lifts an error from a store port onto the server's own variants, so callers
/// can match `Error::NotFound` and friends whichever store raised it.
pub(crate) fn from_store(e: unitycatalog_common::Error) -> Error {
    use unitycatalog_common::Error as E;
    match e {
        E::NotFound => Error::NotFound,
        E::AlreadyExists => Error::AlreadyExists,
        E::InvalidArgument(message) => Error::InvalidArgument(message),
        other => other.into(),
    }
}
