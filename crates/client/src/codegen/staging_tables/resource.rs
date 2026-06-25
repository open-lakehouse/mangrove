// @generated — do not edit by hand.
use super::builders::*;
use super::client::StagingTableServiceClient;
use unitycatalog_common::models::staging_tables::v1::*;
/// A client scoped to a single `staging_table`.
#[derive(Clone)]
pub struct StagingTableClient {
    pub(crate) staging_table_name: String,
    pub(crate) client: StagingTableServiceClient,
}
impl StagingTableClient {
    /// Create a client bound to the resource's name components.
    pub fn new(staging_table_name: impl Into<String>, client: StagingTableServiceClient) -> Self {
        Self {
            staging_table_name: staging_table_name.into(),
            client,
        }
    }
    /// This resource's own name (the leaf component).
    pub fn name(&self) -> &str {
        &self.staging_table_name
    }
    /// The fully-qualified name of this resource.
    pub fn full_name(&self) -> String {
        self.staging_table_name.clone()
    }
}
