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
