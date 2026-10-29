//! The sharing API error contract, decoupled from any server's internal error.
//!
//! [`Error`] is the error half of every handler `Result` in this crate: the
//! generated `SharingHandler` / `SharingVolumeHandler` / `SharingSkillHandler`
//! traits return `crate::Result<T>`, and the hand-written NDJSON query path does
//! too. Its [`IntoResponse`] emits the [`ErrorResponse`] envelope
//! (`{ "errorCode", "message" }`, the shape Delta Sharing and Unity Catalog
//! servers share) with the status codes the sharing surface uses, plus a
//! [`NotImplemented`](Error::NotImplemented) variant (→ 501) for the
//! not-yet-implemented protocol additions (CDF, async queries, delta format).
//!
//! The crate depends on no catalog: a host maps its own errors onto these
//! variants.

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

    /// The request carries no valid recipient credential. → 401.
    #[error("unauthenticated")]
    Unauthenticated,

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
            Error::Unauthenticated => "UNAUTHENTICATED",
            Error::NotImplemented(_) => "NOT_IMPLEMENTED",
            _ => "INTERNAL_ERROR",
        }
    }
}

/// The error body every sharing endpoint returns.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    /// A machine-readable code, e.g. `NOT_FOUND` (see [`Error::error_code`]).
    pub error_code: String,
    pub message: String,
}

/// Bridge from the shared wire-model error (produced by the protocol
/// conversions + capability parsing in `open_sharing_common`).
impl From<open_sharing_common::Error> for Error {
    fn from(err: open_sharing_common::Error) -> Self {
        use open_sharing_common::Error as Sc;
        match err {
            Sc::MalformedResponse { source } => Error::MalformedResponse { source },
            Sc::InvalidArgument(m) => Error::InvalidArgument(m),
        }
    }
}

#[cfg(feature = "axum")]
mod server {
    use axum::extract::Json;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Response};
    use tracing::error;

    use super::{Error, ErrorResponse};

    fn internal() -> (StatusCode, String) {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "The request is not handled correctly due to a server error.".to_string(),
        )
    }

    impl IntoResponse for Error {
        fn into_response(self) -> Response {
            let error_code = self.error_code().to_string();
            let (status, message): (StatusCode, String) = match self {
                Error::DeltaKernel { source } => {
                    error!("Delta Kernel error: {}", source);
                    internal()
                }
                // Invalid-argument messages are written for the caller (a version
                // past the end of the table, an unshared history, …), so they are
                // returned rather than replaced by a generic message.
                Error::InvalidArgument(message) => (StatusCode::BAD_REQUEST, message),
                Error::MalformedUrl { source } => {
                    error!("Malformed URL: {}", source);
                    internal()
                }
                Error::MalformedResponse { source } => {
                    error!("Malformed response: {}", source);
                    internal()
                }
                Error::InvalidPredicate(ref msg) => {
                    error!("Invalid predicate: {}", msg);
                    (
                        StatusCode::BAD_REQUEST,
                        "Invalid predicate provided in the request.".to_string(),
                    )
                }
                Error::NotFound => (
                    StatusCode::NOT_FOUND,
                    "The requested resource was not found.".to_string(),
                ),
                Error::NotAllowed => (
                    StatusCode::FORBIDDEN,
                    "The caller is not permitted to perform this action.".to_string(),
                ),
                Error::Unauthenticated => (
                    StatusCode::UNAUTHORIZED,
                    "No or invalid recipient token provided.".to_string(),
                ),
                Error::NotImplemented(what) => {
                    error!("Not implemented: {}", what);
                    (StatusCode::NOT_IMPLEMENTED, what.to_string())
                }
                Error::Generic(ref message) => {
                    error!("Generic error: {}", message);
                    internal()
                }
            };

            (
                status,
                Json(ErrorResponse {
                    error_code,
                    message,
                }),
            )
                .into_response()
        }
    }
}
