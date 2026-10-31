//! Async Rust client for the Unity Catalog REST API.
//!
//! [`UnityCatalogClient`] is the entry point: construct it from a base URL and an
//! auth token, then reach a resource through the accessor for that resource —
//! `catalog(name)`, `list_catalogs()`, `tables_client()`, and so on. Each accessor
//! returns a scoped sub-client or request builder; list builders implement
//! `IntoFuture` (so you can `.await` them directly) and `into_stream()` for
//! auto-paginated iteration.
//!
//! Two surfaces are hand-written rather than generated from the API spec and have
//! their own accessors: [`UnityCatalogClient::delta_v1`] for the `/delta/v1` Delta
//! REST API and [`UnityCatalogClient::temporary_credentials`] for credential
//! vending with name → UUID resolution.
//!
//! ```no_run
//! use unitycatalog_client::UnityCatalogClient;
//! use url::Url;
//!
//! # async fn run() -> unitycatalog_client::Result<()> {
//! let client = UnityCatalogClient::new_with_token(
//!     Url::parse("https://example.com/api/2.1/unity-catalog/").unwrap(),
//!     "dapi...",
//! );
//!
//! // Fetch one catalog by name.
//! let catalog = client.catalog("main").get().await?;
//! # Ok(())
//! # }
//! ```
//!
//! Fallible calls return [`Result`], whose error type [`Error`] distinguishes UC
//! API errors ([`UcApiError`]) from Delta error envelopes and offers predicates
//! like [`Error::is_not_found`] for control flow.

pub use codegen::UnityCatalogClient;
pub use codegen::agent_skills::AgentSkillClient;
pub use codegen::agents::AgentClient;
pub use codegen::catalogs::CatalogClient;
pub use codegen::credentials::CredentialClient;
pub use codegen::external_locations::ExternalLocationClient;
pub use codegen::functions::FunctionClient;
pub use codegen::policies::PolicyClient;
pub use codegen::providers::ProviderClient;
pub use codegen::recipients::RecipientClient;
pub use codegen::registered_models::RegisteredModelClient;
pub use codegen::schemas::SchemaClient;
pub use codegen::shares::ShareClient;
pub use codegen::staging_tables::StagingTableClient;
pub use codegen::tables::TableClient;
pub use codegen::tag_policies::TagPolicyClient;
pub use codegen::volumes::VolumeClient;
pub use delta_v1::DeltaV1Client;
pub use error::*;
pub use temporary_credentials::*;

pub mod codegen;
mod delta_v1;
pub mod error;
mod temporary_credentials;

/// The HTTP transport the client is built on, selected per target. Native builds
/// use `olai_http::CloudClient`; `wasm32` builds use `olai_http_wasm::WasmClient`
/// (the browser Fetch transport). This mirrors the alias the generated code emits
/// (see `codegen/client.rs`) so the hand-written [`delta_v1`] and
/// [`temporary_credentials`] surfaces hold the same transport type.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) use olai_http::CloudClient as Transport;
#[cfg(target_arch = "wasm32")]
pub(crate) use olai_http_wasm::WasmClient as Transport;

impl UnityCatalogClient {
    /// Ergonomic accessor for the temporary credential vending client.
    ///
    /// Wraps the generated low-level client with the hand-written name → UUID resolving helpers
    /// (`temporary_table_credential`, `temporary_volume_credential`, `temporary_path_credential`).
    pub fn temporary_credentials(&self) -> TemporaryCredentialClient {
        TemporaryCredentialClient::new(self.temporary_credentials_client())
    }

    /// Ergonomic accessor for the hand-written `/delta/v1/` Delta REST API client.
    ///
    /// Reuses a generated low-level client's cloud client and base URL (both carry
    /// the same auth + endpoint), so the Delta v1 client shares the aggregate
    /// client's configuration without touching generated code.
    pub fn delta_v1(&self) -> DeltaV1Client {
        let base = self.tables_client();
        DeltaV1Client::new(base.client, base.base_url)
    }
}
