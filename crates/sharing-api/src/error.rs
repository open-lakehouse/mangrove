//! The sharing API error contract, decoupled from any server's internal error.
//!
//! [`Error`] is the error half of every handler `Result` in this crate: the
//! generated `SharingHandler` / `SharingVolumeHandler` / `SharingSkillHandler`
//! traits return `crate::Result<T>`, and the hand-written NDJSON query path does
//! too. Its [`IntoResponse`] emits the Unity Catalog error envelope
//! (`{ "errorCode", "message" }`) with the status codes the sharing surface uses,
//! reproducing `unitycatalog_sharing_client`'s server-side mapping — plus a
//! [`NotImplemented`](Error::NotImplemented) variant (→ 501) for the
//! not-yet-implemented protocol additions (CDF, async queries, delta format).

use unitycatalog_common::Error as CommonError;

/// Result type used across the crate's sharing handlers + router.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Alias matching the [`crate::SharingApiResult`] re-export.
pub type SharingApiResult<T> = Result<T>;

/// The error half of every sharing handler `Result`.
///
/// A server adapter converts its own internal error into one of these variants
/// (typically via `From`), so the crate never sees a server error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A delta-kernel failure encountered while serving the query path. → 500.
    #[error("Delta Kernel Error: {source}")]
    DeltaKernel {
        #[from]
        source: delta_kernel::Error,
    },

    /// A Unity Catalog common error, carrying its own status mapping. Surfaced
    /// with the status/code the common error dictates.
    #[error("Common Error: {source}")]
    Common {
        #[from]
        source: CommonError,
    },

    /// A response (de)serialization failure. → 500.
    #[error("Malformed response: {source}")]
    MalformedResponse {
        #[from]
        source: serde_json::Error,
    },

    /// A malformed URL. → 500.
    #[error("Malformed url: {source}")]
    MalformedUrl {
        #[from]
        source: url::ParseError,
    },

    /// The requested resource does not exist. → 404.
    #[error("not found")]
    NotFound,

    /// The caller is not permitted. → 403.
    #[error("permission denied")]
    NotAllowed,

    /// A malformed request or invalid parameter. → 400.
    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),

    /// A malformed predicate hint. → 400.
    #[error("Invalid predicate: {0}")]
    InvalidPredicate(String),

    /// A protocol feature whose type/route exists but whose serving path is not
    /// implemented (CDF, async queries, `responseformat=delta`). → 501.
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    /// An unexpected failure. → 500.
    #[error("Generic error: {0}")]
    Generic(String),
}

impl Error {
    pub fn generic(message: impl ToString) -> Self {
        Error::Generic(message.to_string())
    }

    pub fn invalid_argument(message: impl ToString) -> Self {
        Error::InvalidArgument(message.to_string())
    }

    pub fn invalid_predicate(msg: impl ToString) -> Self {
        Self::InvalidPredicate(msg.to_string())
    }

    /// A 501 `NotImplemented` for a not-yet-served protocol feature.
    pub fn not_implemented(what: &'static str) -> Self {
        Error::NotImplemented(what)
    }

    /// Returns a machine-readable error code matching the UC API spec.
    pub fn error_code(&self) -> &str {
        match self {
            Error::InvalidArgument(_) | Error::InvalidPredicate(_) | Error::MalformedUrl { .. } => {
                "INVALID_PARAMETER_VALUE"
            }
            Error::NotFound => "NOT_FOUND",
            Error::NotAllowed => "PERMISSION_DENIED",
            Error::NotImplemented(_) => "NOT_IMPLEMENTED",
            Error::Common { source } => source.error_code(),
            _ => "INTERNAL_ERROR",
        }
    }
}

/// Bridge from the shared `unitycatalog_sharing_client` error (produced by the
/// reused protocol conversions + capability parsing) into this crate's error.
impl From<unitycatalog_sharing_client::error::Error> for Error {
    fn from(err: unitycatalog_sharing_client::error::Error) -> Self {
        use unitycatalog_sharing_client::error::Error as Sc;
        match err {
            Sc::DeltaKernel { source } => Error::DeltaKernel { source },
            Sc::Common { source } => Error::Common { source },
            Sc::MalformedResponse { source } => Error::MalformedResponse { source },
            Sc::MalformedUrl { source } => Error::MalformedUrl { source },
            Sc::InvalidArgument(m) => Error::InvalidArgument(m),
            Sc::InvalidPredicate(m) => Error::InvalidPredicate(m),
            other => Error::Generic(other.to_string()),
        }
    }
}

#[cfg(feature = "axum")]
mod server {
    use axum::extract::Json;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use tracing::error;
    use unitycatalog_common::ErrorResponse;

    use super::Error;

    const INTERNAL_ERROR: (StatusCode, &str) = (
        StatusCode::INTERNAL_SERVER_ERROR,
        "The request is not handled correctly due to a server error.",
    );

    const INVALID_ARGUMENT: (StatusCode, &str) = (
        StatusCode::BAD_REQUEST,
        "Invalid argument provided in the request.",
    );

    impl IntoResponse for Error {
        fn into_response(self) -> Response {
            let error_code = self.error_code().to_string();
            let (status, message): (StatusCode, &str) = match self {
                Error::Common { source } => {
                    let (status, message) = source.response_parts();
                    return (
                        status,
                        Json(ErrorResponse {
                            error_code,
                            message: message.to_string(),
                        }),
                    )
                        .into_response();
                }
                Error::DeltaKernel { source } => {
                    error!("Delta Kernel error: {}", source);
                    INTERNAL_ERROR
                }
                Error::InvalidArgument(ref message) => {
                    error!("Invalid argument: {}", message);
                    INVALID_ARGUMENT
                }
                Error::MalformedUrl { source } => {
                    error!("Malformed URL: {}", source);
                    INTERNAL_ERROR
                }
                Error::MalformedResponse { source } => {
                    error!("Malformed response: {}", source);
                    INTERNAL_ERROR
                }
                Error::InvalidPredicate(ref msg) => {
                    error!("Invalid predicate: {}", msg);
                    (
                        StatusCode::BAD_REQUEST,
                        "Invalid predicate provided in the request.",
                    )
                }
                Error::NotFound => (
                    StatusCode::NOT_FOUND,
                    "The requested resource was not found.",
                ),
                Error::NotAllowed => (
                    StatusCode::FORBIDDEN,
                    "The caller is not permitted to perform this action.",
                ),
                Error::NotImplemented(what) => {
                    error!("Not implemented: {}", what);
                    (StatusCode::NOT_IMPLEMENTED, what)
                }
                Error::Generic(ref message) => {
                    error!("Generic error: {}", message);
                    INTERNAL_ERROR
                }
            };

            (
                status,
                Json(ErrorResponse {
                    error_code,
                    message: message.to_string(),
                }),
            )
                .into_response()
        }
    }
}
