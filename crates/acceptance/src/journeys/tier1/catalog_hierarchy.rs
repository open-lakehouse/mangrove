//! Catalog Hierarchy Journey
//!
//! This journey tests the hierarchical structure of Unity Catalog by creating
//! a catalog, then creating schemas within it, and verifying the relationships.
//! It demonstrates full lifecycle management of the catalog hierarchy.

use async_trait::async_trait;
use futures::StreamExt;

use crate::execution::{
    ImplementationTag, JourneyContext, JourneyLogger, JourneyMetadata, JourneyState, JourneyTier,
    ResourceTag, UserJourney, cleanup_step,
};
use crate::init_journey;
use crate::reporting::ReportingConfig;
use crate::{AcceptanceError, AcceptanceResult};

/// Catalog hierarchy journey testing catalog and schema creation
pub struct CatalogHierarchyJourney {
    catalog_name: String,
    schema_names: Vec<String>,
    logger: Option<JourneyLogger>,
}

impl CatalogHierarchyJourney {
    /// Create a new catalog hierarchy journey
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let catalog_name = format!("hierarchy_catalog_{}", timestamp);
        let schema_names = vec![
            format!("test_schema_{}", timestamp),
            format!("analytics_schema_{}", timestamp),
            format!("staging_schema_{}", timestamp),
        ];

        Self {
            catalog_name,
            schema_names,
            logger: None,
        }
    }

    /// Create with a specific catalog name and schema names
    #[allow(dead_code)] // retained as journey API surface
    pub fn with_names(catalog_name: impl Into<String>, schema_names: Vec<String>) -> Self {
        Self {
            catalog_name: catalog_name.into(),
            schema_names,
            logger: None,
        }
    }

    /// Create with custom configuration
    #[allow(dead_code)] // retained as journey API surface
    pub fn with_config(
        catalog_name: impl Into<String>,
        schema_names: Vec<String>,
        config: ReportingConfig,
    ) -> Self {
        let logger = JourneyLogger::with_config("catalog_hierarchy", config);

        Self {
            catalog_name: catalog_name.into(),
            schema_names,
            logger: Some(logger),
        }
    }

    /// Get the catalog name for this journey
    #[allow(dead_code)] // retained as journey API surface
    pub fn catalog_name(&self) -> &str {
        &self.catalog_name
    }

    /// Get the schema names for this journey
    #[allow(dead_code)] // retained as journey API surface
    pub fn schema_names(&self) -> &[String] {
        &self.schema_names
    }
}

#[async_trait]
impl UserJourney for CatalogHierarchyJourney {
    fn name(&self) -> &str {
        "catalog_hierarchy"
    }

    fn description(&self) -> &str {
        "Catalog hierarchy testing: create catalog, create multiple schemas, verify relationships, cleanup"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![ResourceTag::Catalogs, ResourceTag::Schemas],
            implementations: vec![ImplementationTag::All],
            tier: JourneyTier::Tier1Crud,
            requires_external_storage: false,
        }
    }

    fn save_state(&self) -> AcceptanceResult<JourneyState> {
        let mut state = JourneyState::empty();
        state.set_string("catalog_name", self.catalog_name.clone());

        // Store schema names as a JSON array
        let schema_names_json = serde_json::to_string(&self.schema_names).map_err(|e| {
            AcceptanceError::JourneyValidation(format!("Failed to serialize schema names: {}", e))
        })?;
        state.set_string("schema_names", schema_names_json);

        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(catalog_name) = state.get_string("catalog_name") {
            self.catalog_name = catalog_name;
        }
        if let Some(schema_names_json) = state.get_string("schema_names") {
            self.schema_names = serde_json::from_str(&schema_names_json).map_err(|e| {
                AcceptanceError::JourneyValidation(format!(
                    "Failed to deserialize schema names: {}",
                    e
                ))
            })?;
        }
        Ok(())
    }

    async fn execute(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        // Create logger directly since we can't mutate self
        let logger = if let Some(ref logger) = self.logger {
            logger.clone()
        } else {
            init_journey!(
                "catalog_hierarchy",
                &format!("Catalog hierarchy testing for '{}'", self.catalog_name)
            )
        };

        // Step 1: Create the parent catalog
        logger.info("📁 Creating parent catalog")?;
        let created_catalog = logger
            .step("create_catalog", async {
                ctx.client()
                    .create_catalog(&self.catalog_name)
                    .with_storage_root(ctx.storage_root.clone())
                    .with_comment(Some(
                        "Hierarchy test catalog for schema management".to_string(),
                    ))
                    .await
                    .map_err(|e| {
                        AcceptanceError::UnityCatalog(format!("Failed to create catalog: {}", e))
                    })
            })
            .await?;

        logger.info(&format!("✅ Created catalog: {}", created_catalog.name))?;

        // Step 2: Verify catalog exists and is accessible
        logger.info("🔍 Verifying catalog creation")?;
        logger
            .step("verify_catalog", async {
                let catalog_info = ctx
                    .client()
                    .catalog(&self.catalog_name)
                    .get()
                    .await
                    .map_err(|e| {
                        AcceptanceError::UnityCatalog(format!("Failed to get catalog: {}", e))
                    })?;

                if catalog_info.name != self.catalog_name {
                    return Err(AcceptanceError::JourneyValidation(format!(
                        "Catalog name mismatch: expected '{}', got '{}'",
                        self.catalog_name, catalog_info.name
                    )));
                }

                logger.info(&format!(
                    "Catalog ID: {}",
                    catalog_info.id.unwrap_or_else(|| "N/A".to_string())
                ))?;
                Ok(())
            })
            .await?;

        // Step 3: Create schemas within the catalog
        logger.info("📂 Creating schemas within catalog")?;
        let mut created_schemas = Vec::new();

        for (index, schema_name) in self.schema_names.iter().enumerate() {
            let step_id = format!("create_schema_{}", index + 1);
            let created_schema = logger
                .step(&step_id, async {
                    ctx.client()
                        .catalog(&self.catalog_name)
                        .create_schema(schema_name)
                        .with_comment(Some(format!(
                            "Test schema {} for hierarchy validation",
                            index + 1
                        )))
                        .await
                        .map_err(|e| {
                            AcceptanceError::UnityCatalog(format!(
                                "Failed to create schema '{}': {}",
                                schema_name, e
                            ))
                        })
                })
                .await?;

            logger.info(&format!("✅ Created schema: {}", created_schema.full_name))?;
            created_schemas.push(created_schema);
        }

        // Step 4: List schemas in the catalog and verify they exist
        logger.info("📋 Listing schemas in catalog")?;
        logger
            .step("list_schemas", async {
                let mut schemas_stream = ctx
                    .client()
                    .list_schemas(&self.catalog_name)
                    .with_max_results(50)
                    .with_include_browse(false)
                    .into_stream();
                let mut found_schemas = Vec::new();

                while let Some(schema_result) = schemas_stream.next().await {
                    let schema = schema_result.map_err(|e| {
                        AcceptanceError::UnityCatalog(format!("Failed to list schemas: {}", e))
                    })?;
                    found_schemas.push(schema);
                }

                logger.info(&format!("Found {} schemas in catalog", found_schemas.len()))?;

                // Verify all our schemas are present
                for schema_name in &self.schema_names {
                    let found = found_schemas.iter().any(|s| s.name == *schema_name);
                    if !found {
                        return Err(AcceptanceError::JourneyValidation(format!(
                            "Schema '{}' not found in catalog listing",
                            schema_name
                        )));
                    }
                    logger.info(&format!("  ✓ Found schema: {}", schema_name))?;
                }

                Ok(found_schemas.len())
            })
            .await?;

        // Step 5: Verify schema details and hierarchy relationships
        logger.info("🔍 Verifying schema details and relationships")?;
        for (index, schema_name) in self.schema_names.iter().enumerate() {
            let step_id = format!("verify_schema_{}", index + 1);

            logger
                .step(&step_id, async {
                    let schema_info = ctx
                        .client()
                        .catalog(&self.catalog_name)
                        .schema(schema_name)
                        .get()
                        .await
                        .map_err(|e| {
                            AcceptanceError::UnityCatalog(format!(
                                "Failed to get schema '{}': {}",
                                schema_name, e
                            ))
                        })?;

                    // Verify schema belongs to our catalog
                    if schema_info.catalog_name != self.catalog_name {
                        return Err(AcceptanceError::JourneyValidation(format!(
                            "Schema '{}' catalog mismatch: expected '{}', got '{}'",
                            schema_name, self.catalog_name, schema_info.catalog_name
                        )));
                    }

                    // Verify schema name
                    if schema_info.name != *schema_name {
                        return Err(AcceptanceError::JourneyValidation(format!(
                            "Schema name mismatch: expected '{}', got '{}'",
                            schema_name, schema_info.name
                        )));
                    }

                    // Verify full name format
                    let expected_full_name = format!("{}.{}", self.catalog_name, schema_name);
                    if schema_info.full_name != expected_full_name {
                        return Err(AcceptanceError::JourneyValidation(format!(
                            "Schema full name mismatch: expected '{}', got '{}'",
                            expected_full_name, schema_info.full_name
                        )));
                    }

                    logger.info(&format!(
                        "  ✓ Schema '{}' verified - Catalog: {}, Full name: {}",
                        schema_name, schema_info.catalog_name, schema_info.full_name
                    ))?;

                    Ok(())
                })
                .await?;
        }

        // Step 6: Test schema access patterns
        logger.info("🔄 Testing schema access patterns")?;
        logger
            .step("test_access_patterns", async {
                for schema_name in &self.schema_names {
                    // Test direct schema access
                    let _schema_direct = ctx
                        .client()
                        .catalog(&self.catalog_name)
                        .schema(schema_name)
                        .get()
                        .await
                        .map_err(|e| {
                            AcceptanceError::UnityCatalog(format!(
                                "Direct schema access failed for '{}': {}",
                                schema_name, e
                            ))
                        })?;

                    logger.info(&format!(
                        "  ✓ Direct access to schema '{}' successful",
                        schema_name
                    ))?;
                }

                Ok::<(), AcceptanceError>(())
            })
            .await?;

        logger.info("🎉 Hierarchy validation completed successfully")?;

        // Step 7: Cleanup - Delete schemas first, then catalog
        logger.info("🧹 Cleaning up hierarchy (schemas first, then catalog)")?;

        // Delete schemas in reverse order
        for (index, schema_name) in self.schema_names.iter().enumerate().rev() {
            let step_id = format!("cleanup_schema_{}", index + 1);

            logger
                .step(&step_id, async {
                    ctx.client()
                        .catalog(&self.catalog_name)
                        .schema(schema_name)
                        .delete()
                        .await
                        .map_err(|e| {
                            AcceptanceError::UnityCatalog(format!(
                                "Failed to delete schema '{}': {}",
                                schema_name, e
                            ))
                        })
                })
                .await?;

            logger.info(&format!("🗑️ Deleted schema: {}", schema_name))?;
        }

        // Delete the catalog
        logger
            .step("cleanup_catalog", async {
                ctx.client()
                    .catalog(&self.catalog_name)
                    .delete()
                    .with_force(true)
                    .await
                    .map_err(|e| {
                        AcceptanceError::UnityCatalog(format!("Failed to delete catalog: {}", e))
                    })
            })
            .await?;

        logger.info(&format!("🗑️ Deleted catalog: {}", self.catalog_name))?;

        logger.info("✨ Catalog hierarchy journey completed successfully")?;
        logger.finish(true)?;

        Ok(())
    }

    async fn cleanup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        if let Some(logger) = &self.logger {
            logger.info("🧹 Starting emergency cleanup operations")?;

            // Try to delete schemas first
            for schema_name in &self.schema_names {
                cleanup_step(
                    logger,
                    &format!("emergency_cleanup_schema_{}", schema_name),
                    ctx.client()
                        .catalog(&self.catalog_name)
                        .schema(schema_name)
                        .delete()
                        .into_future(),
                )
                .await?;
            }

            // Try to delete the catalog
            cleanup_step(
                logger,
                "emergency_cleanup_catalog",
                ctx.client()
                    .catalog(&self.catalog_name)
                    .delete()
                    .with_force(true)
                    .into_future(),
            )
            .await?;

            logger.info("Emergency cleanup completed")?;
        }
        Ok(())
    }
}

impl Default for CatalogHierarchyJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reporting::ReportingConfig;

    #[test]
    fn test_default_journey() {
        let journey = CatalogHierarchyJourney::default();
        assert!(journey.catalog_name.starts_with("hierarchy_catalog_"));
        assert_eq!(journey.schema_names.len(), 3);
        assert!(journey.schema_names[0].starts_with("test_schema_"));
        assert!(journey.schema_names[1].starts_with("analytics_schema_"));
        assert!(journey.schema_names[2].starts_with("staging_schema_"));
    }

    #[test]
    fn test_with_custom_names() {
        let schema_names = vec!["custom_schema_1".to_string(), "custom_schema_2".to_string()];
        let journey = CatalogHierarchyJourney::with_names("custom_catalog", schema_names.clone());

        assert_eq!(journey.catalog_name, "custom_catalog");
        assert_eq!(journey.schema_names, schema_names);
    }

    #[test]
    fn test_with_config() {
        let config = ReportingConfig {
            verbosity: 2,
            ..ReportingConfig::default()
        };
        let schema_names = vec!["test_schema".to_string()];

        let journey =
            CatalogHierarchyJourney::with_config("test_catalog", schema_names.clone(), config);
        assert_eq!(journey.catalog_name, "test_catalog");
        assert_eq!(journey.schema_names, schema_names);
        assert!(journey.logger.is_some());
    }

    #[test]
    fn test_state_management() {
        let schema_names = vec!["schema1".to_string(), "schema2".to_string()];
        let journey = CatalogHierarchyJourney::with_names("test_catalog", schema_names.clone());

        // Save state
        let state = journey.save_state().unwrap();
        assert!(state.get_string("catalog_name").is_some());
        assert!(state.get_string("schema_names").is_some());

        // Create new journey and load state
        let mut new_journey = CatalogHierarchyJourney::with_names(
            "different_catalog",
            vec!["different_schema".to_string()],
        );
        new_journey.load_state(&state).unwrap();

        assert_eq!(new_journey.catalog_name, journey.catalog_name);
        assert_eq!(new_journey.schema_names, journey.schema_names);
    }

    #[test]
    fn test_journey_properties() {
        let journey = CatalogHierarchyJourney::new();
        assert_eq!(journey.name(), "catalog_hierarchy");
        assert!(journey.description().contains("hierarchy"));
        assert!(journey.description().contains("catalog"));
        assert!(journey.description().contains("schema"));
    }

    #[test]
    fn test_getters() {
        let schema_names = vec!["getter_test_schema".to_string()];
        let journey =
            CatalogHierarchyJourney::with_names("getter_test_catalog", schema_names.clone());

        assert_eq!(journey.catalog_name(), "getter_test_catalog");
        assert_eq!(journey.schema_names(), &schema_names);
    }
}
