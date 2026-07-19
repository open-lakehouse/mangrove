//! Error type for the in-browser query engine.
//!
//! The one distinction that matters to callers is **unsupported vs. failed**:
//! [`Error::Unsupported`] means this table/query is outside the wasm engine's
//! v1 envelope (non-Delta, deletion vectors, unbackfilled commit tail, AWS/R2
//! storage, …) and the caller should fall back to another runner, while every
//! other variant is a genuine failure worth surfacing. The wasm bindings expose
//! this as a machine-readable discriminant on the thrown error.

/// Result alias for this crate.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error raised while resolving or querying a table.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The table or query is outside the wasm engine's supported envelope; the
    /// caller should fall back to another runner. The message states the exact
    /// gate that fired.
    #[error("unsupported by the in-browser engine: {0}")]
    Unsupported(String),

    /// A Unity Catalog REST call failed (network, auth, or an error response).
    #[error("unity catalog request failed: {0}")]
    UnityCatalog(String),

    /// Storage access failed (fetching log or data files).
    #[error(transparent)]
    ObjectStore(#[from] object_store::Error),

    /// Delta table resolution or scan failed.
    #[error(transparent)]
    Delta(#[from] deltalake_core::DeltaTableError),

    /// Query planning or execution failed.
    #[error(transparent)]
    DataFusion(#[from] datafusion::error::DataFusionError),

    /// Arrow IPC encoding failed.
    #[error(transparent)]
    Arrow(#[from] arrow_schema::ArrowError),

    /// A URL failed to parse or was structurally invalid.
    #[error("invalid url: {0}")]
    InvalidUrl(String),

    /// A server response failed to deserialize.
    #[error("invalid response: {0}")]
    InvalidResponse(String),
}

/// Map a canonical `olai-uc-client` error into a query-wasm [`Error`].
///
/// A `reqwest`-transport failure (the wasm Fetch backend) is tagged with the
/// `network/CORS:` marker so [`crate::bindings::classify`] surfaces it as
/// `NETWORK` — the same discriminant the hand-rolled REST client produced. Every
/// other client error is a genuine catalog/protocol failure.
impl From<unitycatalog_client::Error> for Error {
    fn from(err: unitycatalog_client::Error) -> Self {
        match err {
            unitycatalog_client::Error::RequestError(e) => {
                Error::UnityCatalog(format!("network/CORS: {e}"))
            }
            other => Error::UnityCatalog(other.to_string()),
        }
    }
}

/// Map a canonical `olai-uc-object-store` error into a query-wasm [`Error`].
///
/// Credential-vending goes through the same wasm Fetch transport, so a client
/// transport failure nested inside is likewise tagged `network/CORS:`. Storage
/// I/O errors flow through as [`Error::ObjectStore`] (its own `From` already
/// covers the raw `object_store::Error` path from `get`/`list`).
impl From<unitycatalog_object_store::Error> for Error {
    fn from(err: unitycatalog_object_store::Error) -> Self {
        match err {
            unitycatalog_object_store::Error::ClientError { source } => source.into(),
            unitycatalog_object_store::Error::UnityCatalogError { source } => {
                Error::UnityCatalog(source.to_string())
            }
            other => Error::UnityCatalog(other.to_string()),
        }
    }
}

impl Error {
    /// Map a raw `object_store::Error` from a storage op (list / get / head),
    /// re-tagging a browser transport failure with the `network/CORS:` marker.
    ///
    /// Unlike credential vending — which flows through `olai-uc-client` and gets
    /// its `network/CORS:` tag from the [`From<unitycatalog_client::Error>`] impl
    /// above — a storage-IO failure surfaces as `object_store::Error::Generic`
    /// (e.g. `Generic MicrosoftAzure error: error sending request …`) with no
    /// `network/cors` substring, so [`crate::bindings::classify`] would otherwise
    /// misclassify a blocked read/list as `FAILED` instead of the `NETWORK`
    /// fallback code. Detect the wasm Fetch transport failure by its message and
    /// re-tag; every other storage error flows through as [`Error::ObjectStore`].
    ///
    /// This is scoped to the volume files path; the table read path keeps the
    /// blanket `#[from] object_store::Error` and shares this latent gap (a
    /// separate follow-up).
    pub fn from_object_store(err: object_store::Error) -> Self {
        let text = err.to_string().to_ascii_lowercase();
        // reqwest / browser-Fetch transport failure strings. `error sending
        // request` is reqwest's connect/send failure; the others cover the
        // browser's `TypeError: Failed to fetch` (opaque CORS) and DNS/connect.
        let is_transport = text.contains("error sending request")
            || text.contains("failed to fetch")
            || text.contains("cors")
            || text.contains("network")
            || text.contains("connect");
        if is_transport {
            Error::UnityCatalog(format!("network/CORS: {err}"))
        } else {
            Error::ObjectStore(err)
        }
    }

    /// Shorthand for an [`Error::Unsupported`] with a formatted reason.
    pub fn unsupported(reason: impl Into<String>) -> Self {
        Self::Unsupported(reason.into())
    }

    /// True when the caller should fall back to another runner rather than
    /// surface this as a failure.
    pub fn is_unsupported(&self) -> bool {
        matches!(self, Self::Unsupported(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `Generic` store error whose message reads like a reqwest transport
    /// failure is re-tagged with the `network/CORS:` marker so the bindings
    /// classify it as `NETWORK`.
    #[test]
    fn from_object_store_retags_transport_failure() {
        let raw = object_store::Error::Generic {
            store: "MicrosoftAzure",
            source: "error sending request for url (https://acct.blob.core.windows.net/…)".into(),
        };
        let mapped = Error::from_object_store(raw);
        match mapped {
            Error::UnityCatalog(msg) => {
                assert!(
                    msg.to_ascii_lowercase().contains("network/cors"),
                    "expected a network/CORS tag, got: {msg}"
                );
            }
            other => panic!("expected UnityCatalog(network/CORS), got {other:?}"),
        }
    }

    /// A genuine storage error (e.g. a missing object) is NOT re-tagged — it
    /// flows through as `ObjectStore` so it classifies as `FAILED`.
    #[test]
    fn from_object_store_passes_through_not_found() {
        let raw = object_store::Error::NotFound {
            path: "some/object".to_string(),
            source: "blob not found".into(),
        };
        let mapped = Error::from_object_store(raw);
        assert!(
            matches!(mapped, Error::ObjectStore(_)),
            "a NotFound must stay ObjectStore, got {mapped:?}"
        );
    }
}
