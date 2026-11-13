//! The `/delta/v1` error contract, decoupled from any server's internal error.
//!
//! Two error types live here:
//!
//! - [`DeltaBackendError`] — what the backend [port](crate::backend::DeltaBackend)
//!   returns. It is a *pre-image* of the Delta error envelope: each variant maps
//!   to a fixed `(StatusCode, DeltaErrorType)`. Each server's adapter converts its
//!   own internal error into this enum, so the crate never sees a server error
//!   type.
//! - [`DeltaApiError`] — the error half of every handler `Result`. It wraps a
//!   [`DeltaBackendError`] plus the crate's own logic errors (contract validation,
//!   commit arbitration), and its [`IntoResponse`] emits the Delta envelope
//!   (`{ "error": { message, type, code } }`) with the status codes
//!   `delta.yaml`'s `DeltaErrorType` assigns.
//!
//! # Server errors are redacted
//!
//! For 4xx errors the variant's message is the client-facing explanation and is
//! returned as-is. 5xx errors ([`Internal`](DeltaBackendError::Internal),
//! [`Backend`](DeltaBackendError::Backend), …) usually carry driver or storage
//! detail that must not reach a caller: the response carries a generic message
//! plus a reference id, and the full error (with its source chain) is logged at
//! `error` level under the same id.

use std::borrow::Cow;

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::coordinator::CommitError;
use crate::models::{DeltaErrorModel, DeltaErrorResponse, DeltaErrorType};

/// A boxed error, the source carried by [`DeltaBackendError::Backend`].
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// Error returned by a [`DeltaBackend`](crate::backend::DeltaBackend) operation.
///
/// Each variant has a fixed `(StatusCode, DeltaErrorType)` target, applied by
/// [`DeltaApiError`]'s [`IntoResponse`]. A server adapter converts its internal
/// error into one of these variants, preserving the response semantics without
/// exposing its own error type to the crate.
///
/// `#[non_exhaustive]`: new variants may be added in minor releases, so match
/// with a wildcard arm.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum DeltaBackendError {
    /// The requested resource does not exist. → 404 `NoSuchTableException`.
    #[error("{0}")]
    NotFound(String),

    /// The table is not a Delta table (or not one this endpoint serves); clients
    /// should fall back to the UC REST API. → 400 `UnsupportedTableFormatException`.
    #[error("{0}")]
    UnsupportedTableFormat(String),

    /// The request is malformed (unparsable body, bad path or query parameter).
    /// → 400 `BadRequestException`.
    #[error("bad request: {0}")]
    BadRequest(String),

    /// The requested catalog does not exist. → 404 `NoSuchCatalogException`.
    #[error("{0}")]
    NoSuchCatalog(String),

    /// A backend failure with no recognized semantics, surfaced as a generic
    /// not-found. → 404 `NotFoundException`.
    #[error("{0}")]
    NotFoundGeneric(String),

    /// A resource with the same identity already exists. → 409 `AlreadyExistsException`.
    #[error("{0}")]
    AlreadyExists(String),

    /// The caller is authenticated but not permitted. → 403 `PermissionDeniedException`.
    #[error("{0}")]
    PermissionDenied(String),

    /// The caller is not authenticated. → 401 `NotAuthorizedException`.
    #[error("{0}")]
    Unauthenticated(String),

    /// The request is malformed or a parameter is invalid. → 400 `InvalidParameterValueException`.
    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    /// A commit lost the version race: another commit holds that version.
    /// → 409 `CommitVersionConflictException`.
    #[error("commit version conflict: {0}")]
    CommitVersionConflict(String),

    /// The `add-commit` is a retry of a commit that already landed (same version,
    /// same file name).
    ///
    /// Not a client-facing error: the handler answers a recognized retry with the
    /// current table state (200), as the spec requires. A backend's
    /// [`apply_update`](crate::backend::DeltaBackend::apply_update) returns this
    /// after rolling back everything the retry applied. If it ever reaches the
    /// response layer it is reported as 500 `CommitStateUnknownException`, which
    /// tells the client to check the table rather than rebase.
    #[error("commit already accepted")]
    CommitAlreadyAccepted,

    /// The outcome of an `add-commit` cannot be determined (e.g. a retry of a
    /// version whose catalog record has been purged). → 500
    /// `CommitStateUnknownException`: the client must check the table state
    /// before retrying, never rebase blindly.
    #[error("commit state unknown: {0}")]
    CommitStateUnknown(String),

    /// An `assert-etag` / `assert-table-uuid` requirement was not met. → 409 `UpdateRequirementConflictException`.
    #[error("update requirement conflict: {0}")]
    UpdateRequirementConflict(String),

    /// The request was throttled or hit a resource limit. → 429 `ResourceExhaustedException`.
    #[error("resource exhausted: {0}")]
    ResourceExhausted(String),

    /// The requested functionality is not implemented. → 501 `NotImplementedException`.
    #[error("not implemented: {0}")]
    NotImplemented(Cow<'static, str>),

    /// An unexpected backend failure. → 500 `InternalServerErrorException`.
    /// The message is logged, not returned to the client.
    #[error("internal error: {0}")]
    Internal(String),

    /// An unexpected backend failure carrying its source error. → 500
    /// `InternalServerErrorException`. The source chain is logged, not returned
    /// to the client.
    #[error("backend error")]
    Backend(#[source] BoxError),
}

impl DeltaBackendError {
    /// Wrap any error as an unexpected backend failure ([`Backend`](Self::Backend)),
    /// keeping it as the error source.
    pub fn backend(source: impl Into<BoxError>) -> Self {
        Self::Backend(source.into())
    }
}

impl From<CommitError> for DeltaBackendError {
    fn from(err: CommitError) -> Self {
        match err {
            CommitError::VersionConflict(m) => DeltaBackendError::CommitVersionConflict(m),
            CommitError::AlreadyAccepted => DeltaBackendError::CommitAlreadyAccepted,
            CommitError::StateUnknown(m) => DeltaBackendError::CommitStateUnknown(m),
            CommitError::InvalidArgument(m) => DeltaBackendError::InvalidArgument(m),
            CommitError::ResourceExhausted(m) => DeltaBackendError::ResourceExhausted(m),
            CommitError::Backend(m) => DeltaBackendError::Internal(m),
        }
    }
}

/// The error half of every Delta handler `Result`.
///
/// Wraps a [`DeltaBackendError`]; the crate's own logic errors are constructed via
/// the helper constructors ([`invalid_argument`](Self::invalid_argument), etc.).
/// Its [`IntoResponse`] serializes the Delta API error envelope.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DeltaApiError(#[from] pub DeltaBackendError);

impl From<CommitError> for DeltaApiError {
    fn from(err: CommitError) -> Self {
        DeltaApiError(err.into())
    }
}

impl DeltaApiError {
    /// A 400 `InvalidParameterValueException` from crate logic (e.g. contract validation).
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        DeltaApiError(DeltaBackendError::InvalidArgument(message.into()))
    }

    /// A 400 `BadRequestException` for a malformed request.
    pub fn bad_request(message: impl Into<String>) -> Self {
        DeltaApiError(DeltaBackendError::BadRequest(message.into()))
    }

    /// A 500 `InternalServerErrorException` for a server-side invariant violation.
    pub fn internal(message: impl Into<String>) -> Self {
        DeltaApiError(DeltaBackendError::Internal(message.into()))
    }

    /// A 403 `PermissionDeniedException`.
    pub fn permission_denied(message: impl Into<String>) -> Self {
        DeltaApiError(DeltaBackendError::PermissionDenied(message.into()))
    }

    /// A 404 `NoSuchTableException`.
    pub fn not_found(message: impl Into<String>) -> Self {
        DeltaApiError(DeltaBackendError::NotFound(message.into()))
    }

    /// A 501 `NotImplementedException`.
    pub fn not_implemented(what: impl Into<Cow<'static, str>>) -> Self {
        DeltaApiError(DeltaBackendError::NotImplemented(what.into()))
    }

    /// A 401 `NotAuthorizedException`.
    pub fn unauthenticated(message: impl Into<String>) -> Self {
        DeltaApiError(DeltaBackendError::Unauthenticated(message.into()))
    }

    /// The wrapped backend error.
    pub fn kind(&self) -> &DeltaBackendError {
        &self.0
    }

    /// The `(status, error-type)` pair for the wrapped error.
    pub fn parts(&self) -> (StatusCode, DeltaErrorType) {
        use DeltaErrorType::*;
        match &self.0 {
            DeltaBackendError::NotFound(_) => (StatusCode::NOT_FOUND, NoSuchTableException),
            DeltaBackendError::NoSuchCatalog(_) => (StatusCode::NOT_FOUND, NoSuchCatalogException),
            DeltaBackendError::NotFoundGeneric(_) => (StatusCode::NOT_FOUND, NotFoundException),
            DeltaBackendError::AlreadyExists(_) => (StatusCode::CONFLICT, AlreadyExistsException),
            DeltaBackendError::PermissionDenied(_) => {
                (StatusCode::FORBIDDEN, PermissionDeniedException)
            }
            DeltaBackendError::Unauthenticated(_) => {
                (StatusCode::UNAUTHORIZED, NotAuthorizedException)
            }
            DeltaBackendError::InvalidArgument(_) => {
                (StatusCode::BAD_REQUEST, InvalidParameterValueException)
            }
            DeltaBackendError::BadRequest(_) => (StatusCode::BAD_REQUEST, BadRequestException),
            DeltaBackendError::UnsupportedTableFormat(_) => {
                (StatusCode::BAD_REQUEST, UnsupportedTableFormatException)
            }
            DeltaBackendError::CommitVersionConflict(_) => {
                (StatusCode::CONFLICT, CommitVersionConflictException)
            }
            DeltaBackendError::CommitAlreadyAccepted | DeltaBackendError::CommitStateUnknown(_) => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    CommitStateUnknownException,
                )
            }
            DeltaBackendError::UpdateRequirementConflict(_) => {
                (StatusCode::CONFLICT, UpdateRequirementConflictException)
            }
            DeltaBackendError::ResourceExhausted(_) => {
                (StatusCode::TOO_MANY_REQUESTS, ResourceExhaustedException)
            }
            DeltaBackendError::NotImplemented(_) => {
                (StatusCode::NOT_IMPLEMENTED, NotImplementedException)
            }
            DeltaBackendError::Internal(_) | DeltaBackendError::Backend(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                InternalServerErrorException,
            ),
        }
    }

    /// The message the client sees. 5xx detail is redacted (see the module docs);
    /// `reference` ties the generic message to the logged error.
    fn client_message(&self, status: StatusCode, reference: &str) -> String {
        match &self.0 {
            // Static, crate- or adapter-authored text: safe to return.
            DeltaBackendError::NotImplemented(_) => self.0.to_string(),
            DeltaBackendError::CommitAlreadyAccepted | DeltaBackendError::CommitStateUnknown(_) => {
                format!(
                    "the commit outcome is unknown; reload the table before retrying \
                     (reference {reference})"
                )
            }
            _ if status.is_server_error() => {
                format!("internal server error (reference {reference})")
            }
            _ => self.0.to_string(),
        }
    }
}

/// Render an error and its source chain as `outer: cause: root`.
fn error_chain(err: &dyn std::error::Error) -> String {
    let mut out = err.to_string();
    let mut source = err.source();
    while let Some(cause) = source {
        out.push_str(": ");
        out.push_str(&cause.to_string());
        source = cause.source();
    }
    out
}

impl IntoResponse for DeltaApiError {
    fn into_response(self) -> Response {
        let (status, error_type) = self.parts();
        let message = if status.is_server_error() {
            let reference = uuid::Uuid::now_v7().to_string();
            tracing::error!(
                reference = %reference,
                status = status.as_u16(),
                error = %error_chain(&self.0),
                "delta api request failed"
            );
            self.client_message(status, &reference)
        } else {
            self.client_message(status, "")
        };
        let body = DeltaErrorResponse {
            error: DeltaErrorModel {
                message,
                error_type,
                code: status.as_u16(),
                stack: None,
            },
        };
        (status, Json(body)).into_response()
    }
}

/// Result type used across the crate's Delta logic and handler surface.
pub type DeltaApiResult<T> = Result<T, DeltaApiError>;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    async fn body_of(err: DeltaApiError) -> (StatusCode, serde_json::Value) {
        let resp = err.into_response();
        let status = resp.status();
        let bytes = to_bytes(resp.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn internal_detail_is_redacted() {
        let (status, json) = body_of(DeltaApiError::internal(
            "query on internal table delta_commits_shard_7 failed",
        ))
        .await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        let msg = json["error"]["message"].as_str().unwrap();
        assert!(!msg.contains("delta_commits_shard_7"), "{msg}");
        assert!(msg.contains("reference"), "{msg}");
        assert_eq!(json["error"]["type"], "InternalServerErrorException");
    }

    #[tokio::test]
    async fn backend_source_is_redacted() {
        let source = std::io::Error::other("disk /var/secret full");
        let (status, json) = body_of(DeltaApiError(DeltaBackendError::backend(source))).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert!(
            !json["error"]["message"]
                .as_str()
                .unwrap()
                .contains("/var/secret")
        );
    }

    #[tokio::test]
    async fn client_errors_keep_their_message() {
        let (status, json) = body_of(DeltaApiError::invalid_argument("bad thing")).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["error"]["message"], "invalid argument: bad thing");
    }

    #[tokio::test]
    async fn leaked_already_accepted_is_commit_state_unknown() {
        let (status, json) = body_of(DeltaApiError(DeltaBackendError::CommitAlreadyAccepted)).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(json["error"]["type"], "CommitStateUnknownException");
    }

    #[test]
    fn error_chain_includes_sources() {
        let err = DeltaBackendError::backend(std::io::Error::other("root cause"));
        assert_eq!(error_chain(&err), "backend error: root cause");
    }
}
