//! Table Managed Lifecycle Journey
//!
//! Tests the full lifecycle of a managed Delta table:
//! create catalog → create schema → create managed DELTA table → get → list → delete

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_common::tables::v1::{DataSourceFormat, GetTableExistsRequest, TableType};

use crate::execution::{
    ImplementationTag, JourneyContext, JourneyMetadata, JourneyState, JourneyTier, ResourceTag,
    UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

pub struct TableManagedLifecycleJourney {
    catalog_name: String,
    schema_name: String,
    table_name: String,
}

impl TableManagedLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("table_lc_catalog_{}", timestamp),
            schema_name: format!("table_lc_schema_{}", timestamp),
            table_name: format!("table_lc_{}", timestamp),
        }
    }
}

impl Default for TableManagedLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for TableManagedLifecycleJourney {
    fn name(&self) -> &str {
        "table_managed_lifecycle"
    }

    fn description(&self) -> &str {
        "Managed table lifecycle: create catalog+schema, create MANAGED DELTA table, get, list, list summaries, exists, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
                ResourceTag::Tables,
            ],
            // The Java OSS server rejects managed-table creation with a 500
            // ("stagingLocation is null") unless backed by configured cloud
            // storage, so it is excluded here.
            implementations: vec![
                ImplementationTag::OssRust,
                ImplementationTag::ManagedDatabricks,
            ],
            tier: JourneyTier::Tier1Crud,
            requires_external_storage: false,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());
        state.set_string("schema_name", self.schema_name.clone());
        state.set_string("table_name", self.table_name.clone());
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
        Ok(())
    }

    async fn execute(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        let full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.table_name
        );

        // Step 1: Create catalog
        println!("  📁 Creating catalog '{}'", self.catalog_name);
        ctx.client()
            .create_catalog(&self.catalog_name)
            .with_storage_root(ctx.storage_root.clone())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create catalog: {}", e))
            })?;

        // Step 2: Create schema
        println!(
            "  📂 Creating schema '{}.{}'",
            self.catalog_name, self.schema_name
        );
        ctx.client()
            .create_schema(&self.schema_name, &self.catalog_name)
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create schema: {}", e))
            })?;

        // Step 3: Create managed Delta table
        println!("  🗃️  Creating managed table '{}'", full_name);
        let table = ctx
            .client()
            .create_table(
                &self.table_name,
                &self.schema_name,
                &self.catalog_name,
                TableType::Managed,
                DataSourceFormat::Delta,
            )
            .with_comment("Managed table lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create table: {}", e))
            })?;

        assert_eq!(table.name, self.table_name);
        println!("  ✓ Table created: {}", table.full_name);

        // Step 4: Get table
        let fetched = ctx
            .client()
            .table_from_full_name(&full_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get table: {}", e))
            })?;
        assert_eq!(fetched.name, self.table_name);
        println!("  ✓ Table fetched: {}", fetched.full_name);

        // Step 5: List tables
        let tables: Vec<_> = ctx
            .client()
            .list_tables(&self.catalog_name, &self.schema_name)
            .into_stream()
            .collect::<Vec<_>>()
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list tables: {}", e))
            })?;
        assert!(
            tables.iter().any(|t| t.name == self.table_name),
            "Created table not found in list"
        );
        println!("  ✓ Listed {} table(s)", tables.len());

        // Step 6: List table summaries (catalog-scoped read path; OSS Java lacks this)
        let summaries = ctx
            .client()
            .list_table_summaries(&self.catalog_name)
            .with_schema_name_pattern(self.schema_name.clone())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to list table summaries: {}", e))
            })?
            .tables;
        assert!(
            summaries.iter().any(|s| s.full_name == full_name),
            "Created table not found in summaries"
        );
        println!("  ✓ Listed {} table summary(ies)", summaries.len());

        // Step 7: Check table existence (GET /tables/{full_name}/exists)
        let exists = ctx
            .client()
            .tables_client()
            .get_table_exists(&GetTableExistsRequest {
                full_name: full_name.clone(),
                ..Default::default()
            })
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to check table exists: {}", e))
            })?;
        assert!(exists.table_exists, "Table reported as not existing");
        println!("  ✓ Table existence confirmed");

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

        Ok(())
    }
}
