/// A convenience type for declaring Results in the credentials crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors raised while resolving or vending storage credentials.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// No registered location or credential covers the requested location.
    #[error("not found")]
    NotFound,

    /// A malformed storage URL or an unusable credential.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// An unexpected failure.
    #[error("Generic error: {0}")]
    Generic(String),

    /// A credential could not be (de)serialized.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// The cloud provider rejected or failed a token / SAS / STS exchange.
    #[cfg(feature = "vending")]
    #[error("Cloud credential error: {0}")]
    CloudCredential(#[from] olai_http::Error),
}

impl Error {
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Generic(msg.into())
    }

    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::InvalidArgument(msg.into())
    }
}

impl From<url::ParseError> for Error {
    fn from(e: url::ParseError) -> Self {
        Self::InvalidArgument(format!("invalid url: {e}"))
    }
}
