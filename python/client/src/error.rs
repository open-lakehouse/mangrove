//! How a client error reaches the generated exception classes.
//!
//! The generated `codegen/error.rs` raises a client error as `UnityCatalogError` or the
//! subclass of its HTTP status (`NotFoundError`, `ConflictError`, …), reading the server's
//! error answer through [`ApiErrorAnswer`]. The client keeps its own error type
//! ([`unitycatalog_client::Error`]), so the answer is read from it here.

use unitycatalog_client::{Error, UcApiError};

use crate::codegen::error::{ApiErrorAnswer, ApiErrorParts};

impl ApiErrorAnswer for Error {
    fn into_api_error(self) -> Result<ApiErrorParts, Self> {
        match self {
            Error::Api(api) => Ok(api_parts(api)),
            Error::Delta(model) => Ok(ApiErrorParts {
                code: serde_json::to_value(model.error_type)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_string)),
                message: model.message,
                status: model.code,
                details: Vec::new(),
            }),
            // The generated clients surface an error status as a transport error wrapping
            // the retry layer's error, which still carries the status and body.
            Error::ClientError { source } => match retry_answer(&source) {
                Some(parts) => Ok(parts),
                None => Err(Error::ClientError { source }),
            },
            other => Err(other),
        }
    }
}

fn api_parts(api: UcApiError) -> ApiErrorParts {
    let status = api.http_status();
    let code = Some(api.error_code().to_string()).filter(|code| !code.is_empty());
    let message = match api {
        UcApiError::InvalidParameter { message }
        | UcApiError::Unauthenticated { message }
        | UcApiError::PermissionDenied { message }
        | UcApiError::NotFound { message }
        | UcApiError::AlreadyExists { message }
        | UcApiError::RequestLimitExceeded { message }
        | UcApiError::InternalError { message }
        | UcApiError::TemporarilyUnavailable { message }
        | UcApiError::Other { message, .. } => message,
        other => other.to_string(),
    };
    ApiErrorParts {
        message,
        code,
        status,
        details: Vec::new(),
    }
}

/// The error answer inside an `olai_http` error, read like a UC error body
/// (`{"error_code": …, "message": …, "details": […]}`).
fn retry_answer(err: &olai_http::Error) -> Option<ApiErrorParts> {
    use olai_http::Error as Http;
    let retry = match err {
        Http::NotFound { source }
        | Http::AlreadyExists { source }
        | Http::Precondition { source }
        | Http::NotModified { source }
        | Http::PermissionDenied { source }
        | Http::Unauthenticated { source }
        | Http::Generic { source } => source.downcast_ref::<olai_http::RetryError>()?,
        _ => return None,
    };
    let status = retry.status()?.as_u16();
    let body = retry.body().unwrap_or_default();
    let json = serde_json::from_str::<serde_json::Value>(body).ok();
    let text = |keys: &[&str]| {
        keys.iter()
            .find_map(|key| json.as_ref()?.get(key)?.as_str().map(str::to_string))
    };
    Some(ApiErrorParts {
        code: text(&["error_code", "errorCode"]),
        message: text(&["message"]).unwrap_or_else(|| body.trim().to_string()),
        status,
        details: json
            .as_ref()
            .and_then(|v| v.get("details")?.as_array().cloned())
            .unwrap_or_default(),
    })
}
