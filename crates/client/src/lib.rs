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
//! # Features
//!
//! Each service is gated behind a cargo feature named after its model package in
//! `olai-uc-common` (`catalogs`, `tables`, `tags`, …); `all-resources` (the default)
//! enables them all. Disable default features and list only the resources you call
//! to keep the rest of the model packages out of the build.
//!
//! With the `handlers` feature (native only), each per-service client also
//! implements its `olai-uc-handlers` trait — `CatalogServiceClient` is a
//! `CatalogHandler`, and so on — forwarding every call to the server it points
//! at. A service can then back individual resources with an upstream Unity
//! Catalog. Client errors convert into the handler error for the same condition.
//!
//! Fallible calls return [`Result`], whose error type [`Error`] distinguishes UC
//! API errors ([`UcApiError`]) from Delta error envelopes and offers predicates
//! like [`Error::is_not_found`] for control flow.

pub use codegen::UnityCatalogClient;
#[cfg(feature = "agent_skills")]
pub use codegen::agent_skills::AgentSkillClient;
#[cfg(feature = "agents")]
pub use codegen::agents::AgentClient;
#[cfg(feature = "catalogs")]
pub use codegen::catalogs::CatalogClient;
#[cfg(feature = "credentials")]
pub use codegen::credentials::CredentialClient;
#[cfg(feature = "external_locations")]
pub use codegen::external_locations::ExternalLocationClient;
#[cfg(feature = "functions")]
pub use codegen::functions::FunctionClient;
#[cfg(feature = "policies")]
pub use codegen::policies::PolicyClient;
#[cfg(feature = "providers")]
pub use codegen::providers::ProviderClient;
#[cfg(feature = "recipients")]
pub use codegen::recipients::RecipientClient;
#[cfg(feature = "registered_models")]
pub use codegen::registered_models::RegisteredModelClient;
#[cfg(feature = "schemas")]
pub use codegen::schemas::SchemaClient;
#[cfg(feature = "shares")]
pub use codegen::shares::ShareClient;
#[cfg(feature = "staging_tables")]
pub use codegen::staging_tables::StagingTableClient;
#[cfg(feature = "tables")]
pub use codegen::tables::TableClient;
#[cfg(feature = "tags")]
pub use codegen::tag_policies::TagPolicyClient;
#[cfg(feature = "volumes")]
pub use codegen::volumes::VolumeClient;
pub use delta_v1::DeltaV1Client;
pub use error::*;
#[cfg(feature = "temporary_credentials")]
pub use temporary_credentials::*;

pub mod codegen;
mod delta_v1;
pub mod error;
#[cfg(all(feature = "handlers", not(target_arch = "wasm32")))]
mod handlers;
#[cfg(feature = "temporary_credentials")]
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
    #[cfg(feature = "temporary_credentials")]
    /// Ergonomic accessor for the temporary credential vending client.
    ///
    /// Wraps the generated low-level client with the hand-written name → UUID resolving helpers
    /// (`temporary_table_credential`, `temporary_volume_credential`, `temporary_path_credential`).
    pub fn temporary_credentials(&self) -> TemporaryCredentialClient {
        TemporaryCredentialClient::new(self.temporary_credentials_client())
    }

    #[cfg(feature = "tables")]
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
