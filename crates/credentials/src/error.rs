/// A convenience type for declaring Results in the credentials crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors raised while resolving or vending storage credentials.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A shared Unity Catalog error (invalid argument, not found, serialization, …).
    #[error(transparent)]
    Common(#[from] unitycatalog_common::Error),

    /// The cloud provider rejected or failed a token / SAS / STS exchange.
    #[error("Cloud credential error: {0}")]
    CloudCredential(#[from] olai_http::Error),
}

impl Error {
    pub fn generic(msg: impl Into<String>) -> Self {
        Self::Common(unitycatalog_common::Error::generic(msg))
    }

    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::Common(unitycatalog_common::Error::invalid_argument(msg))
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Self::Common(e.into())
    }
}
