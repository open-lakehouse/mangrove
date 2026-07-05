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

impl Error {
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
