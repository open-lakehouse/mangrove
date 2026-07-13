//! Cross-resource coverage: multi-securable workflows.

use futures::StreamExt;
use unitycatalog_common::credentials::v1::Purpose;
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};
use unitycatalog_common::volumes::v1::VolumeType;

use super::{unique, with_cleanup};
use crate::conformance::skip;
use crate::{AcceptanceResult, JourneyContext};

/// A catalog with several schemas, each holding a managed table + volume; then
/// verify the full hierarchy lists back correctly.
pub async fn lakehouse_hierarchy(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_lake_cat");
    let schemas = ["bronze", "silver"];
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            for schema in schemas {
                ctx.client().create_schema(schema, &catalog).await?;
                ctx.client()
                    .create_table(
                        format!("{schema}_events"),
                        schema,
                        &catalog,
                        TableType::Managed,
                        DataSourceFormat::Delta,
                    )
                    .await?;
                ctx.client()
                    .create_volume(
                        &catalog,
                        schema,
                        format!("{schema}_files"),
                        VolumeType::Managed,
                    )
                    .await?;
            }

            let listed: Vec<_> = ctx
                .client()
                .list_schemas(&catalog)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert_eq!(listed.len(), schemas.len(), "unexpected schema count");

            for schema in schemas {
                let tables: Vec<_> = ctx
                    .client()
                    .list_tables(&catalog, schema)
                    .into_stream()
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;
                assert_eq!(tables.len(), 1, "expected 1 table in {schema}");
                let volumes: Vec<_> = ctx
                    .client()
                    .list_volumes(&catalog, schema)
                    .into_stream()
                    .collect::<Vec<_>>()
                    .await
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()?;
                assert_eq!(volumes.len(), 1, "expected 1 volume in {schema}");
            }
            Ok(())
        },
        || async {
            for schema in schemas {
                let _ = ctx
                    .client()
                    .table_from_full_name(format!("{catalog}.{schema}.{schema}_events"))
                    .delete()
                    .await;
                let _ = ctx
                    .client()
                    .volume(&catalog, schema, format!("{schema}_files"))
                    .delete()
                    .await;
                let _ = ctx.client().schema(&catalog, schema).delete().await;
            }
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// Full governance chain: catalog → schema → credential → external location →
/// external table. Self-skips without cloud storage.
pub async fn governance_setup(ctx: &JourneyContext) -> AcceptanceResult<()> {
    if !ctx.storage_root.starts_with("s3://")
        && !ctx.storage_root.starts_with("abfss://")
        && !ctx.storage_root.starts_with("gs://")
    {
        return Err(skip(
            "governance chain needs a cloud storage root (UC_INTEGRATION_STORAGE_ROOT)",
        ));
    }

    let catalog = unique("conf_gov_cat");
    let schema = "s";
    let table = "t";
    let credential = unique("conf_gov_cred");
    let ext_loc = unique("conf_gov_loc");
    let full_name = format!("{catalog}.{schema}.{table}");
    let location = format!(
        "{}/gov/{full_name}/",
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
                .create_external_location(&ext_loc, &location, &credential)
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
                .with_storage_location(location.clone())
                .await?;
            assert_eq!(created.table_type, TableType::External);
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
