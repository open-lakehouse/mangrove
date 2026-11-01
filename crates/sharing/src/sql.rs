//! What the SQL backends share: error mapping.

use crate::{Error, Result};

/// Maps a database error onto the port's errors: a unique violation is a taken
/// name, a foreign-key violation a missing recipient.
pub(crate) fn db(e: sqlx::Error) -> Error {
    match &e {
        sqlx::Error::RowNotFound => Error::NotFound,
        sqlx::Error::Database(db_err) => match db_err.kind() {
            sqlx::error::ErrorKind::UniqueViolation => Error::AlreadyExists,
            sqlx::error::ErrorKind::ForeignKeyViolation => Error::NotFound,
            _ => Error::generic(format!("database error: {e}")),
        },
        _ => Error::generic(format!("database error: {e}")),
    }
}

/// `NotFound` unless a write touched a row.
pub(crate) fn affected(rows: u64) -> Result<()> {
    if rows == 0 {
        Err(Error::NotFound)
    } else {
        Ok(())
    }
}
