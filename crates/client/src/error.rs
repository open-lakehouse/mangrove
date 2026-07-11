use unitycatalog_delta_api::models::DeltaErrorModel;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Common Error: {source}")]
    Common {
        #[from]
        source: unitycatalog_common::Error,
    },

    #[error("Delta API error {}: [{:?}] {}", .0.code, .0.error_type, .0.message)]
    Delta(DeltaErrorModel),

    #[error("Client Error: {source}")]
    ClientError {
        #[from]
        source: olai_http::Error,
    },

    #[error("Malformed response: {source}")]
    MalformedResponse {
        #[from]
        source: serde_json::Error,
    },

    #[error("Malformed url: {source}")]
    MalformedUrl {
        #[from]
        source: url::ParseError,
    },

    #[error("Reqwuest error: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("API error: {0}")]
    Api(#[from] UcApiError),

    #[error("Generic error: {0}")]
    Generic(String),
}

impl Error {
    pub fn generic(message: impl ToString) -> Self {
        Error::Generic(message.to_string())
    }

    pub fn is_not_found(&self) -> bool {
        match self {
            Error::Api(UcApiError::NotFound { .. }) => true,
            Error::Delta(model) => model.error_type.is_not_found(),
            _ => false,
        }
    }

    pub fn is_already_exists(&self) -> bool {
        match self {
            Error::Api(UcApiError::AlreadyExists { .. }) => true,
            Error::Delta(model) => model.error_type.is_already_exists(),
            _ => false,
        }
    }

    pub fn is_permission_denied(&self) -> bool {
        match self {
            Error::Api(UcApiError::PermissionDenied { .. }) => true,
            Error::Delta(model) => {
                matches!(
                    model.error_type,
                    unitycatalog_delta_api::models::DeltaErrorType::PermissionDeniedException
                )
            }
            _ => false,
        }
    }

    pub fn is_unauthenticated(&self) -> bool {
        match self {
            Error::Api(UcApiError::Unauthenticated { .. }) => true,
            Error::Delta(model) => {
                matches!(
                    model.error_type,
                    unitycatalog_delta_api::models::DeltaErrorType::NotAuthorizedException
                )
            }
            _ => false,
        }
    }

    /// Whether this is a Delta `CommitVersionConflictException` (409): a
    /// concurrent commit ratified the proposed version first. The caller should
    /// rebuild its snapshot and retry the commit.
    pub fn is_commit_conflict(&self) -> bool {
        matches!(self, Error::Delta(model) if model.error_type.is_commit_conflict())
    }

    /// Whether this is a Delta `UpdateRequirementConflictException` (409): an
    /// `assert-etag`/`assert-table-uuid` requirement was not met. The caller
    /// should reload the table and retry.
    pub fn is_update_requirement_conflict(&self) -> bool {
        matches!(self, Error::Delta(model) if model.error_type.is_update_requirement_conflict())
    }

    /// Whether this is a Delta `ResourceExhaustedException` / `TooManyRequestsException`
    /// (429): the request was throttled or hit the unbackfilled-commit limit. The
    /// caller should back off (and backfill pending commits) before retrying.
    pub fn is_resource_exhausted(&self) -> bool {
        matches!(self, Error::Delta(model) if model.error_type.is_resource_exhausted())
    }

    /// Whether this is a Delta `CommitStateUnknownException` (500): the commit
    /// outcome is unknown. The caller must check table state before retrying to
    /// avoid duplicate commits.
    pub fn is_commit_state_unknown(&self) -> bool {
        matches!(self, Error::Delta(model) if model.error_type.is_commit_state_unknown())
    }

    /// Whether this is a Delta `UnsupportedTableFormatException` (400): the table
    /// is not Delta, or is a Delta table this `/delta/v1` endpoint does not
    /// support. The caller should fall back to the legacy UC table API.
    pub fn is_unsupported_table_format(&self) -> bool {
        matches!(self, Error::Delta(model) if model.error_type.is_unsupported_table_format())
    }

    /// Whether this is a Delta `NotImplementedException` (501): the server does
    /// not implement this `/delta/v1` functionality. The caller should fall back
    /// to the legacy UC table API.
    pub fn is_not_implemented(&self) -> bool {
        matches!(self, Error::Delta(model) if model.error_type.is_not_implemented())
    }

    /// Whether this is a `404` that is **not** a recognizable Delta error envelope
    /// — i.e. the `/delta/v1` route itself is absent (no `UnsupportedTableFormat`
    /// / `NoSuchTable` envelope was returned), as on a UC deployment that does not
    /// serve `/delta/v1` at all.
    ///
    /// This is deliberately distinct from [`Error::is_not_found`]: an *enveloped*
    /// `NoSuchTableException` (a genuinely missing table) is an [`Error::Delta`]
    /// and returns `false` here, so callers can fall back on a missing route
    /// without masking a missing table.
    pub fn is_route_missing(&self) -> bool {
        matches!(self, Error::Api(UcApiError::Other { status: 404, .. }))
    }

    /// Whether the caller should react to this `/delta/v1` loadTable error by
    /// falling back to the legacy UC table API (filesystem snapshot): an
    /// unsupported table format, an unimplemented endpoint, or an entirely missing
    /// route. A genuine `NoSuchTable`/auth/other error returns `false` and must be
    /// propagated.
    pub fn should_fall_back_to_legacy(&self) -> bool {
        self.is_unsupported_table_format() || self.is_not_implemented() || self.is_route_missing()
    }
}

/// Typed error variants mapped to the Databricks Unity Catalog API error code spec.
#[derive(Debug, thiserror::Error, PartialEq, Eq, Clone)]
#[non_exhaustive]
pub enum UcApiError {
    #[error("Invalid parameter: {message}")]
    InvalidParameter { message: String },

    #[error("Unauthenticated: {message}")]
    Unauthenticated { message: String },

    #[error("Permission denied: {message}")]
    PermissionDenied { message: String },

    #[error("Resource not found: {message}")]
    NotFound { message: String },

    #[error("Resource already exists: {message}")]
    AlreadyExists { message: String },

    #[error("Request limit exceeded: {message}")]
    RequestLimitExceeded { message: String },

    #[error("Internal server error: {message}")]
    InternalError { message: String },

    #[error("Temporarily unavailable: {message}")]
    TemporarilyUnavailable { message: String },

    #[error("API error {status}: [{error_code}] {message}")]
    Other {
        status: u16,
        error_code: String,
        message: String,
    },
}

impl UcApiError {
    /// Returns the UC API error code string.
    pub fn error_code(&self) -> &str {
        match self {
            UcApiError::InvalidParameter { .. } => "INVALID_PARAMETER_VALUE",
            UcApiError::Unauthenticated { .. } => "UNAUTHENTICATED",
            UcApiError::PermissionDenied { .. } => "PERMISSION_DENIED",
            UcApiError::NotFound { .. } => "RESOURCE_NOT_FOUND",
            UcApiError::AlreadyExists { .. } => "RESOURCE_ALREADY_EXISTS",
            UcApiError::RequestLimitExceeded { .. } => "REQUEST_LIMIT_EXCEEDED",
            UcApiError::InternalError { .. } => "INTERNAL_ERROR",
            UcApiError::TemporarilyUnavailable { .. } => "TEMPORARILY_UNAVAILABLE",
            UcApiError::Other { error_code, .. } => error_code,
        }
    }

    /// Returns the HTTP status code associated with this error.
    pub fn http_status(&self) -> u16 {
        match self {
            UcApiError::InvalidParameter { .. } => 400,
            UcApiError::Unauthenticated { .. } => 401,
            UcApiError::PermissionDenied { .. } => 403,
            UcApiError::NotFound { .. } => 404,
            UcApiError::AlreadyExists { .. } => 409,
            UcApiError::RequestLimitExceeded { .. } => 429,
            UcApiError::InternalError { .. } => 500,
            UcApiError::TemporarilyUnavailable { .. } => 503,
            UcApiError::Other { status, .. } => *status,
        }
    }

    /// Construct from an API response with status code, error code string, and message.
    pub fn from_api_response(status: u16, error_code: &str, message: String) -> Self {
        match error_code {
            "INVALID_PARAMETER_VALUE" => UcApiError::InvalidParameter { message },
            "UNAUTHENTICATED" => UcApiError::Unauthenticated { message },
            "PERMISSION_DENIED" => UcApiError::PermissionDenied { message },
            "RESOURCE_NOT_FOUND" => UcApiError::NotFound { message },
            "RESOURCE_ALREADY_EXISTS" => UcApiError::AlreadyExists { message },
            "REQUEST_LIMIT_EXCEEDED" => UcApiError::RequestLimitExceeded { message },
            "INTERNAL_ERROR" => UcApiError::InternalError { message },
            "TEMPORARILY_UNAVAILABLE" => UcApiError::TemporarilyUnavailable { message },
            other => UcApiError::Other {
                status,
                error_code: other.to_string(),
                message,
            },
        }
    }
}

/// Serde helper for parsing the UC API error body.
#[derive(serde::Deserialize)]
struct ApiErrorBody {
    #[serde(alias = "errorCode")]
    error_code: String,
    message: String,
}

/// Read an error HTTP response, parse the UC API JSON body, and return a typed [`Error`].
pub(crate) async fn parse_error_response(response: reqwest::Response) -> Error {
    let status = response.status().as_u16();
    match response.bytes().await {
        Ok(body) => match serde_json::from_slice::<ApiErrorBody>(&body) {
            Ok(api_err) => {
                UcApiError::from_api_response(status, &api_err.error_code, api_err.message).into()
            }
            Err(_) => UcApiError::Other {
                status,
                error_code: String::new(),
                message: String::from_utf8_lossy(&body).into_owned(),
            }
            .into(),
        },
        Err(e) => Error::RequestError(e),
    }
}

/// Read an error HTTP response from a `/delta/v1` endpoint, parse the Delta API
/// error envelope (`{ "error": { message, type, code, stack? } }`), and return a
/// typed [`Error::Delta`].
///
/// Falls back to [`UcApiError::Other`] when the body is not a recognizable Delta
/// error envelope (e.g. a bare proxy 502 or a truncated body), preserving the raw
/// bytes for diagnostics.
pub(crate) async fn parse_delta_error_response(response: reqwest::Response) -> Error {
    use unitycatalog_delta_api::models::DeltaErrorResponse;

    let status = response.status().as_u16();
    match response.bytes().await {
        Ok(body) => match serde_json::from_slice::<DeltaErrorResponse>(&body) {
            Ok(envelope) => Error::Delta(envelope.error),
            Err(_) => UcApiError::Other {
                status,
                error_code: String::new(),
                message: String::from_utf8_lossy(&body).into_owned(),
            }
            .into(),
        },
        Err(e) => Error::RequestError(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_response(status: u16, body: &'static str) -> reqwest::Response {
        http::Response::builder()
            .status(status)
            .header("content-type", "application/json")
            .body(bytes::Bytes::from_static(body.as_bytes()))
            .map(reqwest::Response::from)
            .unwrap()
    }

    #[tokio::test]
    async fn test_parse_error_resource_not_found() {
        let resp = make_response(
            404,
            r#"{"error_code":"RESOURCE_NOT_FOUND","message":"catalog 'foo' not found"}"#,
        );
        let err = parse_error_response(resp).await;
        assert!(err.is_not_found());
        assert!(matches!(
            err,
            Error::Api(UcApiError::NotFound { ref message }) if message == "catalog 'foo' not found"
        ));
    }

    #[tokio::test]
    async fn test_parse_error_camel_case_alias() {
        let resp = make_response(
            404,
            r#"{"errorCode":"RESOURCE_NOT_FOUND","message":"not found"}"#,
        );
        let err = parse_error_response(resp).await;
        assert!(err.is_not_found());
    }

    #[tokio::test]
    async fn test_parse_error_non_json_body() {
        let resp = make_response(500, "Internal Server Error");
        let err = parse_error_response(resp).await;
        assert!(matches!(
            err,
            Error::Api(UcApiError::Other {
                status: 500,
                ref message,
                ..
            }) if message == "Internal Server Error"
        ));
    }

    #[tokio::test]
    async fn test_parse_error_already_exists() {
        let resp = make_response(
            409,
            r#"{"error_code":"RESOURCE_ALREADY_EXISTS","message":"already exists"}"#,
        );
        let err = parse_error_response(resp).await;
        assert!(err.is_already_exists());
    }

    #[tokio::test]
    async fn test_parse_delta_error_not_found() {
        let resp = make_response(
            404,
            r#"{"error":{"message":"table 'x' not found","type":"NoSuchTableException","code":404}}"#,
        );
        let err = parse_delta_error_response(resp).await;
        assert!(err.is_not_found());
        assert!(!err.is_already_exists());
        assert!(matches!(
            err,
            Error::Delta(ref m) if m.message == "table 'x' not found" && m.code == 404
        ));
    }

    #[tokio::test]
    async fn test_parse_delta_error_already_exists() {
        let resp = make_response(
            409,
            r#"{"error":{"message":"exists","type":"AlreadyExistsException","code":409}}"#,
        );
        let err = parse_delta_error_response(resp).await;
        assert!(err.is_already_exists());
        assert!(!err.is_not_found());
    }

    #[tokio::test]
    async fn test_parse_delta_error_commit_conflict() {
        let resp = make_response(
            409,
            r#"{"error":{"message":"conflict","type":"CommitVersionConflictException","code":409}}"#,
        );
        let err = parse_delta_error_response(resp).await;
        assert!(err.is_commit_conflict());
        assert!(!err.is_update_requirement_conflict());
        assert!(!err.is_already_exists());
    }

    #[tokio::test]
    async fn test_parse_delta_error_update_requirement_conflict() {
        let resp = make_response(
            409,
            r#"{"error":{"message":"etag mismatch","type":"UpdateRequirementConflictException","code":409}}"#,
        );
        let err = parse_delta_error_response(resp).await;
        assert!(err.is_update_requirement_conflict());
        assert!(!err.is_commit_conflict());
    }

    #[tokio::test]
    async fn test_parse_delta_error_resource_exhausted() {
        for ty in ["ResourceExhaustedException", "TooManyRequestsException"] {
            let body = format!(r#"{{"error":{{"message":"slow down","type":"{ty}","code":429}}}}"#);
            // make_response needs a 'static str; build the response inline instead.
            let resp = http::Response::builder()
                .status(429)
                .header("content-type", "application/json")
                .body(bytes::Bytes::from(body))
                .map(reqwest::Response::from)
                .unwrap();
            let err = parse_delta_error_response(resp).await;
            assert!(err.is_resource_exhausted(), "type {ty} should be exhausted");
        }
    }

    #[tokio::test]
    async fn test_parse_delta_error_commit_state_unknown() {
        let resp = make_response(
            500,
            r#"{"error":{"message":"unknown","type":"CommitStateUnknownException","code":500}}"#,
        );
        let err = parse_delta_error_response(resp).await;
        assert!(err.is_commit_state_unknown());
    }

    #[tokio::test]
    async fn test_parse_delta_error_with_stack() {
        let resp = make_response(
            500,
            r#"{"error":{"message":"boom","type":"InternalServerErrorException","code":500,"stack":["a","b"]}}"#,
        );
        let err = parse_delta_error_response(resp).await;
        assert!(matches!(
            err,
            Error::Delta(ref m) if m.stack.as_deref() == Some(&["a".to_string(), "b".to_string()][..])
        ));
    }

    #[tokio::test]
    async fn test_parse_delta_error_non_envelope_body() {
        let resp = make_response(502, "Bad Gateway");
        let err = parse_delta_error_response(resp).await;
        assert!(matches!(
            err,
            Error::Api(UcApiError::Other { status: 502, ref message, .. }) if message == "Bad Gateway"
        ));
    }

    #[tokio::test]
    async fn test_parse_delta_error_unsupported_table_format() {
        // A 400 with the typed envelope: the table is not Delta / not supported by
        // /delta/v1. Should trigger the legacy-API fallback but is not a not-found.
        let resp = make_response(
            400,
            r#"{"error":{"message":"not a delta table","type":"UnsupportedTableFormatException","code":400}}"#,
        );
        let err = parse_delta_error_response(resp).await;
        assert!(err.is_unsupported_table_format());
        assert!(err.should_fall_back_to_legacy());
        assert!(!err.is_not_found());
        assert!(!err.is_route_missing());
    }

    #[tokio::test]
    async fn test_parse_delta_error_not_implemented() {
        let resp = make_response(
            501,
            r#"{"error":{"message":"nope","type":"NotImplementedException","code":501}}"#,
        );
        let err = parse_delta_error_response(resp).await;
        assert!(err.is_not_implemented());
        assert!(err.should_fall_back_to_legacy());
    }

    #[tokio::test]
    async fn test_route_missing_is_non_envelope_404() {
        // A 404 with no Delta error envelope = the /delta/v1 route is absent. This
        // must fall back to the legacy API, distinct from an enveloped
        // NoSuchTableException (a genuinely missing table), which must propagate.
        let resp = make_response(404, "Not Found");
        let err = parse_delta_error_response(resp).await;
        assert!(err.is_route_missing());
        assert!(err.should_fall_back_to_legacy());
        // is_not_found also reports true for the missing route via the Api arm, but
        // the enveloped not-found below must NOT be treated as a missing route.
        let enveloped = parse_delta_error_response(make_response(
            404,
            r#"{"error":{"message":"no table","type":"NoSuchTableException","code":404}}"#,
        ))
        .await;
        assert!(enveloped.is_not_found());
        assert!(!enveloped.is_route_missing());
        assert!(!enveloped.should_fall_back_to_legacy());
    }
}
