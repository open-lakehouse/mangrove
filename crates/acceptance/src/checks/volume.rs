//! Volume coverage: managed (baseline) and external (extended, needs storage).

use futures::StreamExt;
use unitycatalog_common::credentials::v1::Purpose;
use unitycatalog_common::models::volumes::v1::VolumeType;

use super::{unique, with_cleanup};
use crate::{AcceptanceResult, JourneyContext};

/// Managed volume: create → get → list → delete. Portable baseline.
pub async fn managed_volume_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_mvol_cat");
    let schema = "s";
    let volume = "v";
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;

            let created = ctx
                .client()
                .create_volume(&catalog, schema, volume, VolumeType::Managed)
                .with_comment("conformance managed volume".to_string())
                .await?;
            assert_eq!(created.name, volume);

            let fetched = ctx.client().volume(&catalog, schema, volume).get().await?;
            assert_eq!(fetched.name, volume);

            let volumes: Vec<_> = ctx
                .client()
                .list_volumes(&catalog, schema)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                volumes.iter().any(|v| v.name == volume),
                "volume missing from listing"
            );
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

/// External volume over an external location. Self-skips without cloud storage.
pub async fn external_volume_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    if !ctx.storage_root.starts_with("s3://")
        && !ctx.storage_root.starts_with("abfss://")
        && !ctx.storage_root.starts_with("gs://")
    {
        return Err(crate::conformance::skip(
            "external volume needs a cloud storage root (UC_INTEGRATION_STORAGE_ROOT)",
        ));
    }

    let catalog = unique("conf_evol_cat");
    let schema = "s";
    let volume = "v";
    let credential = unique("conf_evol_cred");
    let ext_loc = unique("conf_evol_loc");
    let storage_path = format!(
        "{}/volumes/{volume}/",
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
                .create_external_location(&ext_loc, &storage_path, &credential)
                .await?;

            let created = ctx
                .client()
                .create_volume(&catalog, schema, volume, VolumeType::External)
                .with_storage_location(storage_path.clone())
                .await?;
            assert_eq!(created.volume_type, VolumeType::External);

            let fetched = ctx.client().volume(&catalog, schema, volume).get().await?;
            assert_eq!(fetched.volume_type, VolumeType::External);
            Ok(())
        },
        || async {
            let _ = ctx.client().volume(&catalog, schema, volume).delete().await;
            let _ = ctx.client().external_location(&ext_loc).delete().await;
            let _ = ctx.client().credential(&credential).delete().await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}
