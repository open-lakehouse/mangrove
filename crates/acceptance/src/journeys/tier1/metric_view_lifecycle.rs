//! Metric View Lifecycle Journey
//!
//! Tests the full lifecycle of a metric view (a first-class `METRIC_VIEW` table
//! type whose `view_definition` carries a YAML semantic-layer definition):
//! create catalog → create schema → create metric view → get → list → delete.
//!
//! The core assertion is that `table_type` and `view_definition` round-trip
//! through create and get. When the server derives `view_dependencies` from the
//! definition (uc-rs), the journey also checks the derived dependency list.

use async_trait::async_trait;
use futures::StreamExt;
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType, dependency};

use crate::execution::{
    ImplementationTag, JourneyContext, JourneyMetadata, JourneyState, JourneyTier, ResourceTag,
    UserJourney,
};
use crate::{AcceptanceError, AcceptanceResult};

/// A minimal valid UC-1.1 metric-view definition (matches the shape used by the
/// server-side create_table test fixture).
const METRIC_VIEW_YAML: &str = "version: \"1.1\"\nsource: cat.sch.orders\n\
                                measures:\n  - name: revenue\n    expr: SUM(price)\n";

pub struct MetricViewLifecycleJourney {
    catalog_name: String,
    schema_name: String,
    view_name: String,
}

impl MetricViewLifecycleJourney {
    pub fn new() -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        Self {
            catalog_name: format!("metric_view_catalog_{}", timestamp),
            schema_name: format!("metric_view_schema_{}", timestamp),
            view_name: format!("metric_view_{}", timestamp),
        }
    }
}

impl Default for MetricViewLifecycleJourney {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl UserJourney for MetricViewLifecycleJourney {
    fn name(&self) -> &str {
        "metric_view_lifecycle"
    }

    fn description(&self) -> &str {
        "Metric view lifecycle: create catalog+schema, create METRIC_VIEW with a YAML view_definition, get, list, delete"
    }

    fn metadata(&self) -> JourneyMetadata {
        JourneyMetadata {
            resources: vec![
                ResourceTag::Catalogs,
                ResourceTag::Schemas,
                ResourceTag::Tables,
            ],
            // The pinned OSS Java image (v0.4.1) has neither the METRIC_VIEW
            // table type nor the view_definition field (both arrived upstream in
            // v0.5.0), so it is excluded here. Adding `ImplementationTag::OssJava`
            // once a v0.5.0+ image is available is a one-line change.
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
        state.set_string("view_name", self.view_name.clone());
        Ok(state)
    }

    fn load_state(&mut self, state: &JourneyState) -> AcceptanceResult<()> {
        if let Some(v) = state.get_string("catalog_name") {
            self.catalog_name = v;
        }
        if let Some(v) = state.get_string("schema_name") {
            self.schema_name = v;
        }
        if let Some(v) = state.get_string("view_name") {
            self.view_name = v;
        }
        Ok(())
    }

    async fn execute(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        let full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.view_name
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

        // Step 3: Create metric view. A metric view has no storage of its own;
        // the YAML definition travels in `view_definition`.
        println!("  📐 Creating metric view '{}'", full_name);
        let view = ctx
            .client()
            .create_table(
                &self.view_name,
                &self.schema_name,
                &self.catalog_name,
                TableType::MetricView,
                DataSourceFormat::Delta,
            )
            .with_view_definition(Some(METRIC_VIEW_YAML.to_string()))
            .with_comment("Metric view lifecycle test".to_string())
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to create metric view: {}", e))
            })?;

        assert_eq!(view.name, self.view_name);
        assert_eq!(
            view.table_type,
            TableType::MetricView,
            "created table is not a METRIC_VIEW"
        );
        assert_eq!(
            view.view_definition.as_deref(),
            Some(METRIC_VIEW_YAML),
            "view_definition not preserved on create"
        );
        println!("  ✓ Metric view created: {}", view.full_name);

        // Step 4: Get the metric view — the core round-trip assertion.
        let fetched = ctx
            .client()
            .table_from_full_name(&full_name)
            .get()
            .await
            .map_err(|e| {
                AcceptanceError::JourneyExecution(format!("Failed to get metric view: {}", e))
            })?;
        assert_eq!(fetched.name, self.view_name);
        assert_eq!(
            fetched.table_type,
            TableType::MetricView,
            "fetched table is not a METRIC_VIEW"
        );
        assert_eq!(
            fetched.view_definition.as_deref(),
            Some(METRIC_VIEW_YAML),
            "view_definition not preserved through get"
        );
        // A server that derives dependencies (uc-rs) populates `view_dependencies`
        // from the definition's `source`. Recordings predating that derivation may
        // omit the field, so assert only when it is present.
        if let Some(deps) = fetched.view_dependencies.as_option() {
            let names: Vec<_> = deps
                .dependencies
                .iter()
                .filter_map(|d| match &d.dependency {
                    Some(dependency::Dependency::Table(t)) => Some(t.table_full_name.as_str()),
                    _ => None,
                })
                .collect();
            assert_eq!(
                names,
                vec!["cat.sch.orders"],
                "view_dependencies not derived from the definition's source"
            );
        }
        println!("  ✓ Metric view fetched with view_definition intact");

        // Step 5: List tables — the metric view must appear.
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
            tables.iter().any(|t| t.name == self.view_name),
            "Created metric view not found in list"
        );
        println!("  ✓ Listed {} table(s)", tables.len());

        Ok(())
    }

    async fn cleanup(&self, ctx: &JourneyContext) -> AcceptanceResult<()> {
        let full_name = format!(
            "{}.{}.{}",
            self.catalog_name, self.schema_name, self.view_name
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
