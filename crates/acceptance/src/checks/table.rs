//! Table coverage: managed tables, metric views, and external tables.

use futures::StreamExt;
use unitycatalog_common::credentials::v1::Purpose;
use unitycatalog_common::tables::v1::{
    DataSourceFormat, GetTableExistsRequest, TableType, dependency,
};

use super::{managed_delta, unique, with_cleanup};
use crate::{AcceptanceResult, JourneyContext};

const METRIC_VIEW_YAML: &str = "version: \"1.1\"\nsource: cat.sch.orders\n\
                                measures:\n  - name: revenue\n    expr: SUM(price)\n";

/// Managed Delta table, driven through the real `/delta/v1` staging flow:
/// createStagingTable → write `_delta_log/0.json` → createTable → commit a data
/// file → `SELECT *` (3 rows), then get → list → summaries → exists → delete.
///
/// Managed tables cannot be created with a bare `create_table` — the server
/// requires the staging flow so the catalog owns the commit log. The write path is
/// driven locally against the vended `file://` location (no cloud credentials); the
/// check self-skips if the target vends non-`file://` storage.
pub async fn managed_table_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_mtbl_cat");
    let schema = "s";
    let table = "t";
    let full_name = format!("{catalog}.{schema}.{table}");
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;

            // Create + commit + read back one data file through the Delta staging
            // flow. This both creates the managed table (registering it at v0) and
            // proves the client can drive the catalog-managed write+read contract.
            let rows =
                managed_delta::create_commit_read(ctx.client(), &catalog, schema, table).await?;
            assert_eq!(
                rows, 3,
                "expected 3 rows read back through the managed snapshot"
            );

            let fetched = ctx.client().table_from_full_name(&full_name).get().await?;
            assert_eq!(fetched.name, table);

            let tables: Vec<_> = ctx
                .client()
                .list_tables(&catalog, schema)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                tables.iter().any(|t| t.name == table),
                "table missing from listing"
            );

            let summaries = ctx
                .client()
                .list_table_summaries(&catalog)
                .with_schema_name_pattern(schema.to_string())
                .await?
                .tables;
            assert!(
                summaries.iter().any(|s| s.full_name == full_name),
                "table missing from summaries"
            );

            let exists = ctx
                .client()
                .tables_client()
                .get_table_exists(&GetTableExistsRequest {
                    full_name: full_name.clone(),
                    ..Default::default()
                })
                .await?;
            assert!(exists.table_exists, "table reported as not existing");
            Ok(())
        },
        || async {
            let _ = ctx.client().table_from_full_name(&full_name).delete().await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// Metric view: create with a YAML `view_definition`, then verify it round-trips
/// through create and get (and, when the server derives them, `view_dependencies`).
pub async fn metric_view_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_mv_cat");
    let schema = "s";
    let view = "mv";
    let full_name = format!("{catalog}.{schema}.{view}");
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;

            let created = ctx
                .client()
                .create_table(
                    view,
                    schema,
                    &catalog,
                    TableType::MetricView,
                    DataSourceFormat::Delta,
                )
                .with_view_definition(Some(METRIC_VIEW_YAML.to_string()))
                .with_comment("conformance metric view".to_string())
                .await?;
            assert_eq!(
                created.table_type,
                TableType::MetricView,
                "not a METRIC_VIEW"
            );
            assert_eq!(
                created.view_definition.as_deref(),
                Some(METRIC_VIEW_YAML),
                "view_definition not preserved on create"
            );

            let fetched = ctx.client().table_from_full_name(&full_name).get().await?;
            assert_eq!(fetched.table_type, TableType::MetricView);
            assert_eq!(
                fetched.view_definition.as_deref(),
                Some(METRIC_VIEW_YAML),
                "view_definition not preserved through get"
            );
            // Only servers that derive dependencies populate this; assert when present.
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

            let tables: Vec<_> = ctx
                .client()
                .list_tables(&catalog, schema)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                tables.iter().any(|t| t.name == view),
                "metric view missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx.client().table_from_full_name(&full_name).delete().await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// External table over an external location backed by a storage credential.
/// Self-skips without a configured external storage root.
pub async fn external_table_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    if !ctx.storage_root.starts_with("s3://")
        && !ctx.storage_root.starts_with("abfss://")
        && !ctx.storage_root.starts_with("gs://")
    {
        return Err(crate::conformance::skip(
            "external table needs a cloud storage root (UC_INTEGRATION_STORAGE_ROOT)",
        ));
    }

    let catalog = unique("conf_extbl_cat");
    let schema = "s";
    let table = "t";
    let credential = unique("conf_extbl_cred");
    let ext_loc = unique("conf_extbl_loc");
    let full_name = format!("{catalog}.{schema}.{table}");
    let table_location = format!(
        "{}/tables/{full_name}/",
        ctx.storage_root.trim_end_matches('/')
    );

    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;
            ctx.client()
                .create_credential(&credential, Purpose::Storage)
                .await?;
            ctx.client()
                .create_external_location(&ext_loc, &table_location, &credential)
                .await?;

            let created = ctx
                .client()
                .create_table(
                    table,
                    schema,
                    &catalog,
                    TableType::External,
                    DataSourceFormat::Delta,
                )
                .with_storage_location(table_location.clone())
                .await?;
            assert_eq!(created.table_type, TableType::External);

            let fetched = ctx.client().table_from_full_name(&full_name).get().await?;
            assert_eq!(fetched.table_type, TableType::External);
            Ok(())
        },
        || async {
            let _ = ctx.client().table_from_full_name(&full_name).delete().await;
            let _ = ctx.client().external_location(&ext_loc).delete().await;
            let _ = ctx.client().credential(&credential).delete().await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}
