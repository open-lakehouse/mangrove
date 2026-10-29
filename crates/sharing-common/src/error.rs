pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Errors from parsing or converting the wire models.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Malformed response: {source}")]
    MalformedResponse {
        #[from]
        source: serde_json::Error,
    },

    #[error("Invalid Argument: {0}")]
    InvalidArgument(String),
}
