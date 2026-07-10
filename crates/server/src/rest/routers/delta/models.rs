//! Server-side glue for the UC Delta REST API (`/delta/v1/...`).
//!
//! The wire DTOs are hand-maintained and shared with the client, so they live in
//! [`unitycatalog_delta_api::models`] — re-exported here so the router and
//! the API trait can keep referring to them as `models::*`. What stays server-only
//! is [`DeltaError`]: the mapping from the server's internal [`Error`] onto the
//! Delta API error envelope, plus its axum [`IntoResponse`].

use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::Error;

pub use unitycatalog_delta_api::models::*;

/// Error wrapper used as the error half of every Delta handler's `Result`. It maps
/// the server's internal [`Error`] onto the Delta API envelope
/// (`{ "error": { message, type, code } }`) so `/delta/v1/` responses match the
/// reference `DeltaApiExceptionHandler`, distinct from the server's standard
/// `{errorCode, message}` envelope.
#[derive(Debug)]
pub struct DeltaError(pub Error);

impl From<Error> for DeltaError {
    fn from(e: Error) -> Self {
        DeltaError(e)
    }
}

impl DeltaError {
    fn parts(&self) -> (StatusCode, DeltaErrorType) {
        use DeltaErrorType::*;
        match &self.0 {
            Error::NotFound | Error::ResourceStore { .. } => {
                (StatusCode::NOT_FOUND, NoSuchTableException)
            }
            // The wrapped common error carries its own semantics; dispatch on its
            // machine-readable code so e.g. an "already exists" doesn't surface as 404.
            Error::Common { source } => match source.error_code() {
                "RESOURCE_ALREADY_EXISTS" => (StatusCode::CONFLICT, AlreadyExistsException),
                "INVALID_PARAMETER_VALUE" => {
                    (StatusCode::BAD_REQUEST, InvalidParameterValueException)
                }
                "PERMISSION_DENIED" => (StatusCode::FORBIDDEN, PermissionDeniedException),
                "COMMIT_VERSION_CONFLICT" => (StatusCode::CONFLICT, CommitVersionConflictException),
                "RESOURCE_EXHAUSTED" => (StatusCode::TOO_MANY_REQUESTS, ResourceExhaustedException),
                _ => (StatusCode::NOT_FOUND, NotFoundException),
            },
            Error::NotAllowed => (StatusCode::FORBIDDEN, PermissionDeniedException),
            Error::Unauthenticated => (StatusCode::UNAUTHORIZED, NotAuthorizedException),
            Error::AlreadyExists => (StatusCode::CONFLICT, AlreadyExistsException),
            Error::CommitVersionConflict(_) => {
                (StatusCode::CONFLICT, CommitVersionConflictException)
            }
            Error::UpdateRequirementConflict(_) => {
                (StatusCode::CONFLICT, UpdateRequirementConflictException)
            }
            Error::ResourceExhausted(_) => {
                (StatusCode::TOO_MANY_REQUESTS, ResourceExhaustedException)
            }
            Error::InvalidArgument(_) | Error::InvalidIdentifier(_) | Error::MissingRecipient => {
                (StatusCode::BAD_REQUEST, InvalidParameterValueException)
            }
            Error::NotImplemented(_) => (StatusCode::NOT_IMPLEMENTED, NotImplementedException),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                InternalServerErrorException,
            ),
        }
    }
}

impl IntoResponse for DeltaError {
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
