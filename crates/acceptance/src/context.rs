//! Execution context handed to every conformance check.
//!
//! A [`JourneyContext`] bundles a live [`UnityCatalogClient`] with the storage root
//! the active target expects, so checks can create catalogs with an explicit
//! `MANAGED LOCATION` (required by managed Databricks) without hardcoding a bucket.

use unitycatalog_client::UnityCatalogClient;
use url::Url;

use crate::{AcceptanceError, AcceptanceResult};
use olai_http::CloudClient;

/// The Unity Catalog REST base path every target mounts the catalog API under.
const UC_API_BASE: &str = "/api/2.1/unity-catalog";

/// Execution context for a conformance check.
///
/// Bundles the [`UnityCatalogClient`] driving the system under test with the
/// active target's `storage_root` (e.g. `s3://bucket/uc-test/` for Databricks,
/// `file:///tmp/uc-test/` for the OSS servers).
///
/// `explicit_catalog_storage_root` captures a behavioral difference between
/// targets: managed Databricks requires each managed catalog to carry its own
/// `storage_root`, whereas our Rust server rejects a client-supplied `file://`
/// root unless it is covered by a registered external location — but accepts a
/// catalog that inherits the server's metastore-level managed root. So on the
/// OSS targets checks create catalogs *without* a storage root (inherit), and on
/// managed Databricks they pass one explicitly. Use
/// [`create_catalog`](Self::create_catalog) rather than calling the client
/// directly so this is handled uniformly.
#[derive(Clone)]
pub struct JourneyContext {
    client: UnityCatalogClient,
    /// Cloud/file storage root for the active target. May be empty when the
    /// target inherits a server-configured managed root.
    pub storage_root: String,
    /// Whether managed catalogs must be created with an explicit `storage_root`.
    explicit_catalog_storage_root: bool,
}

impl JourneyContext {
    /// Create a context from an already-built client and storage root. Catalogs
    /// inherit the server's managed root (no explicit `storage_root` on create);
    /// see [`with_explicit_catalog_storage_root`](Self::with_explicit_catalog_storage_root).
    pub fn new(client: UnityCatalogClient, storage_root: impl Into<String>) -> Self {
        Self {
            client,
            storage_root: storage_root.into(),
            explicit_catalog_storage_root: false,
        }
    }

    /// Set whether managed catalogs are created with an explicit `storage_root`
    /// (managed Databricks) or inherit the server's managed root (OSS targets).
    pub fn with_explicit_catalog_storage_root(mut self, explicit: bool) -> Self {
        self.explicit_catalog_storage_root = explicit;
        self
    }

    /// Build a context targeting an unauthenticated live server (the OSS servers).
    /// Catalogs inherit the server's configured managed storage root.
    ///
    /// `server_url` is the server origin (e.g. `http://localhost:8080`); the UC
    /// API base path is appended automatically.
    pub fn live(server_url: &str, storage_root: impl Into<String>) -> AcceptanceResult<Self> {
        Self::build(server_url, None, storage_root, false)
    }

    /// Build a context targeting a token-authenticated live server (managed
    /// Databricks). Catalogs are created with an explicit `storage_root`.
    pub fn live_with_token(
        server_url: &str,
        token: &str,
        storage_root: impl Into<String>,
    ) -> AcceptanceResult<Self> {
        Self::build(server_url, Some(token), storage_root, true)
    }

    fn build(
        server_url: &str,
        token: Option<&str>,
        storage_root: impl Into<String>,
        explicit_catalog_storage_root: bool,
    ) -> AcceptanceResult<Self> {
        let origin: Url = server_url.parse().map_err(|e| {
            AcceptanceError::JourneyValidation(format!("invalid server URL {server_url:?}: {e}"))
        })?;
        let base_url = origin.join(UC_API_BASE).map_err(|e| {
            AcceptanceError::JourneyValidation(format!("could not join UC API base path: {e}"))
        })?;

        let cloud = match token {
            Some(token) => CloudClient::new_with_token(token),
            None => CloudClient::new_unauthenticated(),
        };

        Ok(
            Self::new(UnityCatalogClient::new(cloud, base_url), storage_root)
                .with_explicit_catalog_storage_root(explicit_catalog_storage_root),
        )
    }

    /// The Unity Catalog client for the active target.
    pub fn client(&self) -> &UnityCatalogClient {
        &self.client
    }

    /// Create a managed catalog the way the active target expects: with an
    /// explicit `storage_root` on managed Databricks, or inheriting the server's
    /// managed root on the OSS targets. Checks should use this instead of calling
    /// `client().create_catalog(...)` directly.
    pub async fn create_catalog(&self, name: &str) -> AcceptanceResult<()> {
        let builder = self.client.create_catalog(name);
        if self.explicit_catalog_storage_root {
            builder.with_storage_root(self.storage_root.clone()).await?;
        } else {
            builder.await?;
        }
        Ok(())
    }
}
