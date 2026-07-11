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
//!   (`{ "error": { message, type, code } }`) with the exact status codes the
//!   reference `DeltaApiExceptionHandler` uses.
//!
//! [`DeltaApiError`]'s [`IntoResponse`] implementation reproduces the previous
//! server-side `DeltaError` variant → status mapping exactly, so response bodies
//! and status codes are unchanged by the extraction.

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::models::{DeltaErrorModel, DeltaErrorResponse, DeltaErrorType};

/// Error returned by a [`DeltaBackend`](crate::backend::DeltaBackend) operation.
///
/// Each variant has a fixed `(StatusCode, DeltaErrorType)` target, applied by
/// [`DeltaApiError`]'s [`IntoResponse`]. A server adapter converts its internal
/// error into one of these variants, preserving the response semantics without
/// exposing its own error type to the crate.
#[derive(Debug, thiserror::Error)]
pub enum DeltaBackendError {
    /// The requested resource does not exist. → 404 `NoSuchTableException`.
    #[error("{0}")]
    NotFound(String),

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

    /// A commit lost the version race / was already accepted. → 409 `CommitVersionConflictException`.
    #[error("commit version conflict: {0}")]
    CommitVersionConflict(String),

    /// An `assert-etag` / `assert-table-uuid` requirement was not met. → 409 `UpdateRequirementConflictException`.
    #[error("update requirement conflict: {0}")]
    UpdateRequirementConflict(String),

    /// The request was throttled or hit a resource limit. → 429 `ResourceExhaustedException`.
    #[error("resource exhausted: {0}")]
    ResourceExhausted(String),

    /// The requested functionality is not implemented. → 501 `NotImplementedException`.
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    /// An unexpected backend failure. → 500 `InternalServerErrorException`.
    #[error("internal error: {0}")]
    Internal(String),
}

/// The error half of every Delta handler `Result`.
///
/// Wraps a [`DeltaBackendError`]; the crate's own logic errors are constructed via
/// the helper constructors ([`invalid_argument`](Self::invalid_argument), etc.).
/// Its [`IntoResponse`] serializes the Delta API error envelope.
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct DeltaApiError(#[from] pub DeltaBackendError);

impl DeltaApiError {
    /// A 400 `InvalidParameterValueException` from crate logic (e.g. contract validation).
    pub fn invalid_argument(message: impl Into<String>) -> Self {
        DeltaApiError(DeltaBackendError::InvalidArgument(message.into()))
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
    pub fn not_implemented(what: &'static str) -> Self {
        DeltaApiError(DeltaBackendError::NotImplemented(what))
    }

    /// The `(status, error-type)` pair for the wrapped error. Reproduces the
    /// previous server-side `DeltaError::parts` mapping exactly.
    fn parts(&self) -> (StatusCode, DeltaErrorType) {
        use DeltaErrorType::*;
        match &self.0 {
            DeltaBackendError::NotFound(_) => (StatusCode::NOT_FOUND, NoSuchTableException),
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
            DeltaBackendError::CommitVersionConflict(_) => {
                (StatusCode::CONFLICT, CommitVersionConflictException)
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
            DeltaBackendError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                InternalServerErrorException,
            ),
        }
    }
}

impl IntoResponse for DeltaApiError {
    fn into_response(self) -> Response {
        let (status, error_type) = self.parts();
        let body = DeltaErrorResponse {
            error: DeltaErrorModel {
                message: self.0.to_string(),
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
