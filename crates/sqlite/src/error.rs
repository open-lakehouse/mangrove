use unitycatalog_common::Error as CommonError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Connection(sqlx::Error),

    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Generic error: {0}")]
    Generic(String),

    #[error("Entity not found: '{0}'")]
    EntityNotFound(String),

    #[error("Already exists: '{0}'")]
    AlreadyExists(String),
}

impl Error {
    pub fn entity_not_found(msg: impl Into<String>) -> Self {
        Error::EntityNotFound(msg.into())
    }

    pub fn generic(msg: impl Into<String>) -> Self {
        Error::Generic(msg.into())
    }
}

impl From<sqlx::Error> for Error {
    fn from(e: sqlx::Error) -> Self {
        match &e {
            sqlx::Error::RowNotFound => Error::EntityNotFound("Row not found".to_string()),
            sqlx::Error::Database(db_err) => {
                // SQLite surfaces constraint violations via `kind()`; the extended result
                // codes (`SQLITE_CONSTRAINT_UNIQUE` = 2067, `SQLITE_CONSTRAINT_FOREIGNKEY`
                // = 787) are exposed through the driver's `code()` string.
                match db_err.kind() {
                    sqlx::error::ErrorKind::UniqueViolation => {
                        Error::AlreadyExists("Unique violation".to_string())
                    }
                    sqlx::error::ErrorKind::ForeignKeyViolation => {
                        Error::EntityNotFound("Foreign key violation".to_string())
                    }
                    _ => Error::Connection(e),
                }
            }
            _ => Error::Connection(e),
        }
    }
}

impl From<Error> for olai_store::Error {
    fn from(e: Error) -> Self {
        match e {
            Error::EntityNotFound(_) => olai_store::Error::NotFound,
            Error::AlreadyExists(_) => olai_store::Error::AlreadyExists,
            other => olai_store::Error::Generic(other.to_string()),
        }
    }
}

impl From<olai_store::Error> for Error {
    fn from(e: olai_store::Error) -> Self {
        match e {
            olai_store::Error::NotFound => Error::EntityNotFound("not found".to_string()),
            olai_store::Error::AlreadyExists => Error::AlreadyExists("already exists".to_string()),
            olai_store::Error::InvalidArgument(m) => Error::Generic(m),
            other => Error::Generic(other.to_string()),
        }
    }
}

// Note: the previous `DecodePageToken`/base64 variant was dropped when the
// generic `olai_store::SqlStore` took over object/association pagination.

impl From<Error> for CommonError {
    fn from(e: Error) -> Self {
        match e {
            Error::Connection(e) => CommonError::generic(e.to_string()),
            Error::Migration(e) => CommonError::generic(e.to_string()),
            Error::Generic(e) => CommonError::Generic(e),
            Error::EntityNotFound(_) => CommonError::NotFound,
            Error::AlreadyExists(_) => CommonError::AlreadyExists,
        }
    }
}
