//! What the SQL backends share: error mapping, sealing and pagination.

use olai_store::encryption::EnvelopeEncryptor;

use crate::{CredentialSecrets, Error, Result};

/// Maps a database error onto the port's errors: a unique violation is a taken
/// name, a foreign-key violation a missing credential.
pub(crate) fn db(e: sqlx::Error) -> Error {
    match &e {
        sqlx::Error::RowNotFound => Error::NotFound,
        sqlx::Error::Database(db_err) => match db_err.kind() {
            sqlx::error::ErrorKind::UniqueViolation => Error::AlreadyExists,
            sqlx::error::ErrorKind::ForeignKeyViolation => Error::NotFound,
            _ => Error::generic(e.to_string()),
        },
        _ => Error::generic(e.to_string()),
    }
}

fn sealing(e: olai_store::Error) -> Error {
    Error::generic(format!("sealing credential secrets: {e}"))
}

/// The envelope AAD of a credential's secrets.
fn secret_aad(id: &str) -> String {
    format!("credential/{id}")
}

/// Seals the secrets of the credential with id `id`.
pub(crate) async fn seal(
    encryptor: &EnvelopeEncryptor,
    id: &str,
    secrets: &CredentialSecrets,
) -> Result<Vec<u8>> {
    encryptor
        .seal(&secret_aad(id), &secrets.to_bytes()?)
        .await
        .map_err(sealing)
}

/// Opens the secrets of the credential with id `id`.
pub(crate) async fn open(
    encryptor: &EnvelopeEncryptor,
    id: &str,
    sealed: &[u8],
) -> Result<CredentialSecrets> {
    let plaintext = encryptor
        .open(&secret_aad(id), sealed)
        .await
        .map_err(sealing)?;
    CredentialSecrets::from_bytes(&plaintext)
}

/// `sealed` re-wrapped under the active key, or `None` if it already is.
pub(crate) async fn rewrap(
    encryptor: &EnvelopeEncryptor,
    sealed: &[u8],
) -> Result<Option<Vec<u8>>> {
    encryptor.rewrap(sealed).await.map_err(sealing)
}

/// Splits the extra row fetched past `max` off `rows`, returning its token.
pub(crate) fn paginate<R, T: From<R>>(
    mut rows: Vec<R>,
    max: Option<usize>,
    token: impl Fn(&R) -> String,
) -> (Vec<T>, Option<String>) {
    let next = match max {
        Some(max) if rows.len() > max => {
            rows.truncate(max);
            rows.last().map(token)
        }
        _ => None,
    };
    (rows.into_iter().map(Into::into).collect(), next)
}
