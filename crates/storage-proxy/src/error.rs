//! The storage-proxy error contract, decoupled from any server's internal error.
//!
//! [`ProxyError`] is the error half of every handler `Result`. A backend
//! [port](crate::backend::StorageProxyBackend) returns it directly (a server
//! adapter maps its own internal error into one of these variants), and its
//! [`IntoResponse`] emits the HTTP status the wire contract mandates.
//!
//! [`object_store::Error`] outcomes flow through the [`ProxyError::Storage`]
//! variant, which is the single place cloud-storage results are mapped to a
//! status — most importantly the `If-Match` relay: a conditional-write mismatch
//! surfaces as [`object_store::Error::Precondition`] and becomes
//! `412 Precondition Failed`.

use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};

/// Result type used across the crate's handler surface and backend port.
pub type ProxyResult<T> = Result<T, ProxyError>;

/// An error from a storage-proxy operation.
///
/// Each variant maps to a fixed HTTP status via [`IntoResponse`]. The response
/// body is a small JSON envelope (`{ "error": <message> }`); the wasm client
/// keys off the status code, not the body.
#[derive(Debug, thiserror::Error)]
pub enum ProxyError {
    /// The requested object or securable does not exist. → 404.
    #[error("not found: {0}")]
    NotFound(String),

    /// The caller is not authenticated. → 401.
    #[error("unauthenticated: {0}")]
    Unauthenticated(String),

    /// The caller is authenticated but not permitted for this verb/securable. → 403.
    #[error("permission denied: {0}")]
    PermissionDenied(String),

    /// The request addresses a key outside the securable's authorized scope
    /// (confused-deputy guard). → 403.
    #[error("out of scope: {0}")]
    OutOfScope(String),

    /// The request is malformed (bad securable, unparsable header, `If-Match: *`). → 400.
    #[error("invalid request: {0}")]
    InvalidArgument(String),

    /// A conditional write's `If-Match` did not match the current object. → 412.
    #[error("precondition failed: {0}")]
    PreconditionFailed(String),

    /// The requested byte range cannot be satisfied. → 416.
    #[error("range not satisfiable: {0}")]
    RangeNotSatisfiable(String),

    /// A `PUT` body exceeded the in-proxy buffering cap for a conditional write. → 413.
    #[error("payload too large: {0}")]
    PayloadTooLarge(String),

    /// The requested functionality is not implemented (e.g. GCP credential
    /// vending in the local arm). → 501.
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    /// An outcome from the underlying object store. Mapped to a status by
    /// matching the [`object_store::Error`] variant (see [`ProxyError::storage_parts`]).
    #[error(transparent)]
    Storage(#[from] object_store::Error),

    /// An unexpected internal failure. → 500.
    #[error("internal error: {0}")]
    Internal(String),
}

impl ProxyError {
    /// The status for a wrapped [`object_store::Error`]. This is the single relay
    /// point for cloud-storage semantics — notably `Precondition` → 412, the
    /// server-enforced half of the `If-Match` conditional-write contract.
    fn storage_parts(err: &object_store::Error) -> StatusCode {
        match err {
            object_store::Error::NotFound { .. } => StatusCode::NOT_FOUND,
            object_store::Error::Precondition { .. } => StatusCode::PRECONDITION_FAILED,
            object_store::Error::NotModified { .. } => StatusCode::NOT_MODIFIED,
            object_store::Error::AlreadyExists { .. } => StatusCode::CONFLICT,
            object_store::Error::PermissionDenied { .. } => StatusCode::FORBIDDEN,
            object_store::Error::Unauthenticated { .. } => StatusCode::UNAUTHORIZED,
            object_store::Error::NotSupported { .. }
            | object_store::Error::NotImplemented { .. } => StatusCode::NOT_IMPLEMENTED,
            object_store::Error::InvalidPath { .. } => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// The HTTP status this error maps to.
    fn status(&self) -> StatusCode {
        match self {
            ProxyError::NotFound(_) => StatusCode::NOT_FOUND,
            ProxyError::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
            ProxyError::PermissionDenied(_) | ProxyError::OutOfScope(_) => StatusCode::FORBIDDEN,
            ProxyError::InvalidArgument(_) => StatusCode::BAD_REQUEST,
            ProxyError::PreconditionFailed(_) => StatusCode::PRECONDITION_FAILED,
            ProxyError::RangeNotSatisfiable(_) => StatusCode::RANGE_NOT_SATISFIABLE,
            ProxyError::PayloadTooLarge(_) => StatusCode::PAYLOAD_TOO_LARGE,
            ProxyError::NotImplemented(_) => StatusCode::NOT_IMPLEMENTED,
            ProxyError::Storage(e) => Self::storage_parts(e),
            ProxyError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ProxyError {
    fn into_response(self) -> Response {
        let status = self.status();
        let message = self.to_string();
        // A minimal JSON envelope. The wasm client keys off the status; the body
        // is for humans/logs. `Content-Type` is set explicitly so a bare string
        // body isn't served as `text/plain`.
        let body = format!("{{\"error\":{}}}", serde_json_string(&message));
        (status, [(header::CONTENT_TYPE, "application/json")], body).into_response()
    }
}

/// Serialize a string as a JSON string literal (escaping quotes/backslashes/control
/// chars) without pulling `serde_json` into this crate's dependency set.
fn serde_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn statuses_match_contract() {
        assert_eq!(
            ProxyError::OutOfScope("x".into()).status(),
            StatusCode::FORBIDDEN
        );
        assert_eq!(
            ProxyError::PreconditionFailed("x".into()).status(),
            StatusCode::PRECONDITION_FAILED
        );
        assert_eq!(
            ProxyError::RangeNotSatisfiable("x".into()).status(),
            StatusCode::RANGE_NOT_SATISFIABLE
        );
        assert_eq!(
            ProxyError::NotImplemented("gcp").status(),
            StatusCode::NOT_IMPLEMENTED
        );
    }

    #[test]
    fn object_store_precondition_maps_to_412() {
        let err = object_store::Error::Precondition {
            path: "p".into(),
            source: "etag mismatch".into(),
        };
        assert_eq!(
            ProxyError::Storage(err).status(),
            StatusCode::PRECONDITION_FAILED
        );
    }

    #[test]
    fn object_store_not_found_maps_to_404() {
        let err = object_store::Error::NotFound {
            path: "p".into(),
            source: "missing".into(),
        };
        assert_eq!(ProxyError::Storage(err).status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn json_escaping() {
        assert_eq!(serde_json_string("a\"b\\c"), r#""a\"b\\c""#);
        assert_eq!(serde_json_string("line\nbreak"), r#""line\nbreak""#);
    }
}
