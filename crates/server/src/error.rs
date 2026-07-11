use axum::extract::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
#[cfg(feature = "grpc")]
use tonic::Status;
use tracing::error;
use unitycatalog_common::ErrorResponse;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Delta Kernel Error: {source}")]
    DeltaKernel {
        #[from]
        source: delta_kernel::Error,
    },

    #[error("Common Error: {source}")]
    Common {
        #[from]
        source: unitycatalog_common::Error,
    },

    #[error("Sharing Error: {source}")]
    Sharing {
        #[from]
        source: unitycatalog_sharing_client::Error,
    },

    #[error("Object Store Error: {source}")]
    ObjectStore {
        #[from]
        source: object_store::Error,
    },

    #[error("Serialization Error: {source}")]
    SerDe {
        #[from]
        source: serde_json::Error,
    },

    #[error("Entity not found.")]
    NotFound,

    #[error("No or invalid token provided.")]
    Unauthenticated,

    #[error("Recipient is not allowed to read the entity.")]
    NotAllowed,

    #[error("Already exists")]
    AlreadyExists,

    #[error("Commit version conflict: {0}")]
    CommitVersionConflict(String),

    #[error("Update requirement conflict: {0}")]
    UpdateRequirementConflict(String),

    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    #[error("Invalid argument")]
    InvalidArgument(String),

    #[error("Invalid identifier: {0}")]
    InvalidIdentifier(#[from] uuid::Error),

    #[error("Missing recipient")]
    MissingRecipient,

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("Not implemented: {0}")]
    NotImplemented(&'static str),

    #[error("Cloud credential error: {source}")]
    CloudCredential {
        #[from]
        source: olai_http::Error,
    },

    #[error("Resource store error: {source}")]
    ResourceStore {
        #[from]
        source: olai_store::Error,
    },
}

impl Error {
    pub fn generic(message: impl ToString) -> Self {
        Error::Generic(message.to_string())
    }

    pub fn invalid_argument(message: impl ToString) -> Self {
        Error::InvalidArgument(message.to_string())
    }

    /// Returns a machine-readable error code matching the UC API spec.
    pub fn error_code(&self) -> &str {
        match self {
            Error::NotFound => "RESOURCE_NOT_FOUND",
            Error::AlreadyExists => "RESOURCE_ALREADY_EXISTS",
            Error::CommitVersionConflict(_) => "COMMIT_VERSION_CONFLICT",
            Error::UpdateRequirementConflict(_) => "UPDATE_REQUIREMENT_CONFLICT",
            Error::ResourceExhausted(_) => "RESOURCE_EXHAUSTED",
            Error::NotAllowed => "PERMISSION_DENIED",
            Error::Unauthenticated => "UNAUTHENTICATED",
            Error::InvalidArgument(_) => "INVALID_PARAMETER_VALUE",
            Error::InvalidIdentifier(_) => "INVALID_PARAMETER_VALUE",
            Error::MissingRecipient => "INVALID_PARAMETER_VALUE",
            Error::Common { source } => source.error_code(),
            Error::DeltaKernel { .. } => "INTERNAL_ERROR",
            Error::Sharing { .. } => "INTERNAL_ERROR",
            Error::ObjectStore { .. } => "INTERNAL_ERROR",
            Error::SerDe { .. } => "INTERNAL_ERROR",
            Error::Generic(_) => "INTERNAL_ERROR",
            Error::NotImplemented(_) => "NOT_IMPLEMENTED",
            Error::CloudCredential { .. } => "INTERNAL_ERROR",
            Error::ResourceStore { source } => match source {
                olai_store::Error::NotFound => "RESOURCE_NOT_FOUND",
                olai_store::Error::AlreadyExists => "RESOURCE_ALREADY_EXISTS",
                olai_store::Error::InvalidArgument(_) => "INVALID_PARAMETER_VALUE",
                olai_store::Error::InvalidIdentifier(_) => "INVALID_PARAMETER_VALUE",
                _ => "INTERNAL_ERROR",
            },
        }
    }
}

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
        let (status, message) = match self {
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
            // EXTERNAL ERRORS
            Error::DeltaKernel { source } => {
                error!("Delta Kernel error: {}", source);
                INTERNAL_ERROR
            }
            Error::Sharing { source } => {
                error!("Delta Sharing error: {}", source);
                INTERNAL_ERROR
            }
            Error::ObjectStore { source } => match &source {
                object_store::Error::NotFound { .. } => (
                    StatusCode::NOT_FOUND,
                    "The requested resource does not exist.",
                ),
                object_store::Error::AlreadyExists { .. } => {
                    (StatusCode::CONFLICT, "The resource already exists.")
                }
                _ => {
                    error!("Object store error: {}", source);
                    INTERNAL_ERROR
                }
            },
            Error::SerDe { source } => {
                error!("Serialization error: {}", source);
                INTERNAL_ERROR
            }
            Error::NotFound => (
                StatusCode::NOT_FOUND,
                "The requested resource does not exist.",
            ),
            Error::NotAllowed => (
                StatusCode::FORBIDDEN,
                "The request is forbidden from being fulfilled.",
            ),
            Error::AlreadyExists => (StatusCode::CONFLICT, "The resource already exists."),
            Error::CommitVersionConflict(message) => {
                error!("Commit version conflict: {}", message);
                (
                    StatusCode::CONFLICT,
                    "The commit version was already accepted by another writer.",
                )
            }
            Error::UpdateRequirementConflict(message) => {
                error!("Update requirement conflict: {}", message);
                (
                    StatusCode::CONFLICT,
                    "An update requirement (assert-table-uuid / assert-etag) was not met.",
                )
            }
            Error::ResourceExhausted(message) => {
                error!("Resource exhausted: {}", message);
                (
                    StatusCode::TOO_MANY_REQUESTS,
                    "The maximum number of unbackfilled commits for this table was reached.",
                )
            }
            Error::Unauthenticated => (
                StatusCode::UNAUTHORIZED,
                "The request is unauthenticated. The bearer token is missing or incorrect.",
            ),
            Error::InvalidArgument(message) => {
                error!("Invalid argument: {}", message);
                INVALID_ARGUMENT
            }
            Error::InvalidIdentifier(e) => {
                error!("Invalid identifier: {}", e);
                INVALID_ARGUMENT
            }
            Error::Generic(message) => {
                error!("Generic error: {}", message);
                INTERNAL_ERROR
            }
            Error::NotImplemented(what) => {
                error!("Not implemented: {}", what);
                (StatusCode::NOT_IMPLEMENTED, "Endpoint not implemented yet.")
            }
            Error::CloudCredential { source } => {
                error!("Cloud credential error: {}", source);
                INTERNAL_ERROR
            }
            Error::MissingRecipient => {
                error!("Failed to extract recipient from request");
                (
                    StatusCode::BAD_REQUEST,
                    "Failed to extract recipient from request",
                )
            }
            Error::ResourceStore { source } => match source {
                olai_store::Error::NotFound => (
                    StatusCode::NOT_FOUND,
                    "The requested resource does not exist.",
                ),
                olai_store::Error::AlreadyExists => {
                    (StatusCode::CONFLICT, "The resource already exists.")
                }
                olai_store::Error::InvalidArgument(msg) => {
                    error!("Invalid argument: {}", msg);
                    INVALID_ARGUMENT
                }
                _ => {
                    error!("Resource store error: {}", source);
                    INTERNAL_ERROR
                }
            },
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

#[cfg(feature = "grpc")]
impl From<Error> for Status {
    fn from(e: Error) -> Self {
        match e {
            Error::Common { source } => match source {
                unitycatalog_common::Error::NotFound => {
                    Status::not_found("The requested resource does not exist.")
                }
                unitycatalog_common::Error::InvalidArgument(msg) => Status::invalid_argument(msg),
                unitycatalog_common::Error::InvalidIdentifier(e) => {
                    Status::invalid_argument(e.to_string())
                }
                unitycatalog_common::Error::InvalidTableLocation(loc) => {
                    Status::invalid_argument(format!("Invalid table location: {loc}"))
                }
                unitycatalog_common::Error::InvalidUrl(e) => {
                    Status::invalid_argument(format!("Invalid URL: {e}"))
                }
                _ => Status::internal(format!("Common Error: {source}")),
            },
            Error::Sharing { source } => Status::internal(format!("Sharing Error: {}", source)),
            Error::NotFound => Status::not_found("The requested resource does not exist."),
            Error::NotAllowed => {
                Status::permission_denied("The request is forbidden from being fulfilled.")
            }
            Error::Unauthenticated => Status::unauthenticated(
                "The request is unauthenticated. The bearer token is missing or incorrect.",
            ),
            Error::DeltaKernel { source } => {
                Status::internal(format!("Delta Kernel Error: {}", source))
            }
            Error::SerDe { source } => {
                Status::internal(format!("Serialization/Deserialization Error: {}", source))
            }
            // Error::InvalidTableLocation(location) => {
            //     Status::internal(format!("Invalid table location: {}", location))
            // }
            Error::MissingRecipient => {
                Status::invalid_argument("Failed to extract recipient from request")
            }
            // Error::DataFusion(error) => Status::internal(error.to_string()),
            // Error::Arrow(error) => Status::internal(error.to_string()),
            // Error::InvalidPredicate(msg) => Status::invalid_argument(msg),
            Error::AlreadyExists => Status::already_exists("The resource already exists."),
            Error::CommitVersionConflict(message) => Status::aborted(message),
            Error::UpdateRequirementConflict(message) => Status::aborted(message),
            Error::ResourceExhausted(message) => Status::resource_exhausted(message),
            Error::InvalidIdentifier(e) => Status::invalid_argument(e.to_string()),
            Error::InvalidArgument(message) => Status::invalid_argument(message),
            Error::Generic(message) => Status::internal(message),
            Error::NotImplemented(what) => {
                Status::unimplemented(format!("Not implemented: {what}"))
            }
            // Error::Client(error) => Status::internal(error.to_string()),
            // Error::InvalidUrl(_) => Status::internal("Invalid url"),
            Error::ObjectStore { source } => {
                Status::internal(format!("ObjectStore error: {}", source))
            }
            Error::CloudCredential { source } => {
                Status::internal(format!("Cloud credential error: {}", source))
            }
            Error::ResourceStore { source } => match source {
                olai_store::Error::NotFound => {
                    Status::not_found("The requested resource does not exist.")
                }
                olai_store::Error::AlreadyExists => {
                    Status::already_exists("The resource already exists.")
                }
                olai_store::Error::InvalidArgument(msg) => Status::invalid_argument(msg),
                _ => Status::internal(format!("Resource store error: {source}")),
            },
        }
    }
}
