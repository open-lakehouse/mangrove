pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Model Error: {source}")]
    Model {
        #[from]
        source: open_sharing_common::Error,
    },

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

    #[error("Generic error: {0}")]
    Generic(String),
}

/// Consume an error HTTP response and turn it into an [`Error`].
///
/// Used by the generated clients on non-success statuses. The sharing client's
/// error type is coarse, so the status and raw body are folded into
/// [`Error::Generic`] for diagnostics.
pub(crate) async fn parse_error_response(response: reqwest::Response) -> Error {
    let status = response.status();
    match response.bytes().await {
        Ok(body) => Error::Generic(format!(
            "request failed with status {status}: {}",
            String::from_utf8_lossy(&body)
        )),
        Err(e) => Error::RequestError(e),
    }
}

impl Error {
    pub fn generic(message: impl ToString) -> Self {
        Error::Generic(message.to_string())
    }
}
