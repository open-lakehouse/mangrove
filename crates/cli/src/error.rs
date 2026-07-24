// A convenience type for declaring Results in the Delta Sharing libraries.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Common error: {0}")]
    Common(#[from] unitycatalog_common::Error),

    #[error("Client error: {0}")]
    Client(#[from] unitycatalog_client::Error),

    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// A usage error the caller can fix (bad argument, malformed URL, …).
    #[error("{0}")]
    Usage(String),
}

/// A stable, machine-readable classification of a [`Error`].
///
/// Agents branch on `kind` (in the structured error body) and on the process
/// [`Error::exit_code`] without parsing human-facing prose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    NotFound,
    Auth,
    AlreadyExists,
    Usage,
    Other,
}

impl ErrorKind {
    /// The snake_case token emitted in the structured error body.
    pub fn as_str(self) -> &'static str {
        match self {
            ErrorKind::NotFound => "not_found",
            ErrorKind::Auth => "auth",
            ErrorKind::AlreadyExists => "already_exists",
            ErrorKind::Usage => "usage",
            ErrorKind::Other => "other",
        }
    }
}

impl Error {
    /// Classify this error for agents. Delegates to the client's typed error
    /// predicates ([`is_not_found`](unitycatalog_client::Error::is_not_found),
    /// `is_unauthenticated`, `is_permission_denied`, `is_already_exists`).
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::Client(e) if e.is_not_found() => ErrorKind::NotFound,
            Error::Client(e) if e.is_unauthenticated() || e.is_permission_denied() => {
                ErrorKind::Auth
            }
            Error::Client(e) if e.is_already_exists() => ErrorKind::AlreadyExists,
            Error::Usage(_) => ErrorKind::Usage,
            _ => ErrorKind::Other,
        }
    }

    /// The process exit code, so a caller can branch on outcome without parsing
    /// output: `0` ok / `2` usage / `3` not-found / `4` auth / `1` other.
    pub fn exit_code(&self) -> u8 {
        match self.kind() {
            ErrorKind::Usage => 2,
            ErrorKind::NotFound => 3,
            ErrorKind::Auth => 4,
            ErrorKind::AlreadyExists | ErrorKind::Other => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_error_maps_to_kind_and_exit_code() {
        let err = Error::Usage("bad url".into());
        assert_eq!(err.kind(), ErrorKind::Usage);
        assert_eq!(err.kind().as_str(), "usage");
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn io_error_is_other() {
        let err = Error::Io(std::io::Error::other("boom"));
        assert_eq!(err.kind(), ErrorKind::Other);
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn error_kind_strings_are_stable() {
        assert_eq!(ErrorKind::NotFound.as_str(), "not_found");
        assert_eq!(ErrorKind::Auth.as_str(), "auth");
        assert_eq!(ErrorKind::AlreadyExists.as_str(), "already_exists");
        assert_eq!(ErrorKind::Other.as_str(), "other");
    }
}
