//! Temporary credential vending: table, path, and volume scopes (extended).

use unitycatalog_client::{PathOperation, TableOperation, VolumeOperation};
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};
use unitycatalog_common::volumes::v1::VolumeType;

use super::{unique, with_cleanup};
use crate::conformance::skip;
use crate::{AcceptanceResult, JourneyContext};

/// Managed table → generate read and read-write temporary table credentials.
pub async fn table_credentials(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_ttc_cat");
    let schema = "s";
    let table = "t";
    let full_name = format!("{catalog}.{schema}.{table}");
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;
            ctx.client()
                .create_table(
                    table,
                    schema,
                    &catalog,
                    TableType::Managed,
                    DataSourceFormat::Delta,
                )
                .await?;

            for op in [TableOperation::Read, TableOperation::ReadWrite] {
                let (cred, _id) = ctx
                    .client()
                    .temporary_credentials()
                    .temporary_table_credential(full_name.clone(), op)
                    .await?;
                assert!(
                    cred.credentials.is_some(),
                    "no temporary table credentials returned"
                );
            }
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

/// Generate read and read-write temporary path credentials for the storage root.
/// Self-skips without a cloud storage root.
pub async fn path_credentials(ctx: &JourneyContext) -> AcceptanceResult<()> {
    if !ctx.storage_root.starts_with("s3://")
        && !ctx.storage_root.starts_with("abfss://")
        && !ctx.storage_root.starts_with("gs://")
    {
        return Err(skip(
            "path credentials need a cloud storage root (UC_INTEGRATION_STORAGE_ROOT)",
        ));
    }
    for op in [PathOperation::Read, PathOperation::ReadWrite] {
        let (cred, _url) = ctx
            .client()
            .temporary_credentials()
            .temporary_path_credential(ctx.storage_root.as_str(), op, None)
            .await?;
        assert!(
            cred.credentials.is_some(),
            "no temporary path credentials returned"
        );
    }
    Ok(())
}

/// Managed volume → generate read and read-write temporary volume credentials.
pub async fn volume_credentials(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_tvc_cat");
    let schema = "s";
    let volume = "v";
    let full_name = format!("{catalog}.{schema}.{volume}");
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;
            ctx.client()
                .create_volume(&catalog, schema, volume, VolumeType::Managed)
                .await?;

            for op in [VolumeOperation::Read, VolumeOperation::ReadWrite] {
                let (cred, _id) = ctx
                    .client()
                    .temporary_credentials()
                    .temporary_volume_credential(full_name.clone(), op)
                    .await?;
                assert!(
                    cred.credentials.is_some(),
                    "no temporary volume credentials returned"
                );
            }
            Ok(())
        },
        || async {
            let _ = ctx.client().volume(&catalog, schema, volume).delete().await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}
