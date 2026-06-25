//! External Table Lifecycle Journey
//!
//! Tests the lifecycle of an external Delta table backed by cloud storage:
//! create credential → external location → create EXTERNAL table → get → delete
//!
//! NOTE: Pending recording against managed Databricks UC.
//! Run with UC_INTEGRATION_RECORD=true to record.

use async_trait::async_trait;
use unitycatalog_common::credentials::v1::Purpose;
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};

use crate::execution::{
    ImplementationTag, JourneyContext, JourneyMetadata, JourneyState, JourneyTier, ResourceTag,
    UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct TableExternalLifecycleJourney {
    catalog_name: String,
    schema_name: String,
    table_name: String,
    credential_name: String,
    external_location_name: String,
}

impl TableExternalLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("ext_tbl_catalog_{}", timestamp),
            schema_name: format!("ext_tbl_schema_{}", timestamp),
            table_name: format!("ext_tbl_{}", timestamp),
            credential_name: format!("ext_tbl_cred_{}", timestamp),
            external_location_name: format!("ext_tbl_loc_{}", timestamp),
        }
    }
}

impl Default for TableExternalLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for TableExternalLifecycleJourney {
    fn name(&self) -> &str {
        "table_external_lifecycle"
    }

    fn description(&self) -> &str {
        "External table lifecycle: credential + external location → create EXTERNAL DELTA table → get → delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Tables,
                ResourceTag::ExternalLocations,
                ResourceTag::Credentials,
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
            ],
            implementations: vec![ImplementationTag::ManagedDatabricks],
            tier: JourneyTier::Tier2Governance,
            requires_external_storage: true,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());
        state.set_string("schema_name", self.schema_name.clone());
        state.set_string("table_name", self.table_name.clone());
        state.set_string("credential_name", self.credential_name.clone());
        state.set_string(
            "external_location_name",
            self.external_location_name.clone(),
        );
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("catalog_name") {
            self.catalog_name = v;
        }
        if let Some(v) = state.get_string("schema_name") {
            self.schema_name = v;
        }
        if let Some(v) = state.get_string("table_name") {
            self.table_name = v;
        }
        if let Some(v) = state.get_string("credential_name") {
            self.credential_name = v;
        }
        if let Some(v) = state.get_string("external_location_name") {
            self.external_location_name = v;
        }
        Ok(())
    }

    async fn execute(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        let full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.table_name
        );
        let table_location = format!("{}/tables/{}/", ctx.storage_root, self.table_name);

        // Step 1: Create catalog and schema
        ctx.client()
            .create_catalog(&self.catalog_name)
            .with_storage_root(ctx.storage_root.clone())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create catalog: {}", e))
            })?;
        ctx.client()
            .create_schema(&self.schema_name, &self.catalog_name)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create schema: {}", e))
            })?;

        // Step 2: Create credential + external location
        ctx.client()
            .create_credential(&self.credential_name, Purpose::Storage)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create credential: {}", e))
            })?;
        ctx.client()
            .create_external_location(
                &self.external_location_name,
                &table_location,
                &self.credential_name,
            )
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!(
                    "Failed to create external location: {}",
                    e
                ))
            })?;

        // Step 3: Create external Delta table
        println!("  🗃️  Creating external table '{}'", full_name);
        let table = ctx
            .client()
            .create_table(
                &self.table_name,
                &self.schema_name,
                &self.catalog_name,
                TableType::External,
                DataSourceFormat::Delta,
            )
            .with_storage_location(table_location.clone())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create external table: {}", e))
            })?;
        assert_eq!(table.name, self.table_name);
        println!("  ✓ External table created: {}", table.full_name);

        // Step 4: Get table and verify it is external
        let fetched = ctx
            .client()
            .table_from_full_name(&full_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get table: {}", e))
            })?;
        assert_eq!(
            fetched.table_type,
            ::buffa::EnumValue::Known(TableType::External)
        );
        println!("  ✓ Table type confirmed: External");

        Ok(())
    }

    async fn cleanup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        let full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.table_name
        );
        let _ = ctx.client().table_from_full_name(&full_name).delete().await;
        let _ = ctx
            .client()
            .schema(&self.catalog_name, &self.schema_name)
            .delete()
            .await;
        let _ = ctx.client().catalog(&self.catalog_name).delete().await;
        let _ = ctx
            .client()
            .external_location(&self.external_location_name)
            .delete()
            .await;
        let _ = ctx
            .client()
            .credential(&self.credential_name)
            .delete()
            .await;
        Ok(())
    }
}
