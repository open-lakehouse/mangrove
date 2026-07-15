//! Registered model + model version coverage.
//!
//! `registered_model_lifecycle` and `model_version_lifecycle` are portable
//! baseline checks (UC OSS Java implements the `/models` surface).

use futures::StreamExt;
use unitycatalog_common::models::model_versions::v1::{CreateModelVersion, ModelVersionStatus};
use unitycatalog_common::models::registered_models::v1::CreateRegisteredModel;

use super::{unique, with_cleanup};
use crate::{AcceptanceResult, JourneyContext};

/// Registered model: create → get → list → delete. Portable baseline.
pub async fn registered_model_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_rm_cat");
    let schema = "s";
    let model = "m";
    with_cleanup(
        || async {
            // A registered model allocates a managed storage location, so it needs a
            // catalog with a managed storage root (like a managed volume).
            ctx.create_catalog_for_managed_volume(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;

            let created = ctx
                .client()
                .create_registered_model(CreateRegisteredModel {
                    name: model.to_string(),
                    catalog_name: catalog.clone(),
                    schema_name: schema.to_string(),
                    comment: Some("conformance registered model".to_string()),
                    ..Default::default()
                })
                .await?;
            assert_eq!(created.name, model);
            assert_eq!(created.full_name, format!("{catalog}.{schema}.{model}"));

            let full_name = format!("{catalog}.{schema}.{model}");
            let fetched = ctx
                .client()
                .registered_model_from_full_name(&full_name)
                .get()
                .await?;
            assert_eq!(fetched.name, model);

            let models: Vec<_> = ctx
                .client()
                .list_registered_models()
                .with_catalog_name(catalog.clone())
                .with_schema_name(schema.to_string())
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                models.iter().any(|m| m.name == model),
                "registered model missing from listing"
            );
            Ok(())
        },
        || async {
            let full_name = format!("{catalog}.{schema}.{model}");
            let _ = ctx
                .client()
                .registered_model_from_full_name(&full_name)
                .delete()
                .with_force(true)
                .await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// Model version: create (PENDING) → get → finalize (READY) → list → delete.
/// Portable baseline.
pub async fn model_version_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_mv_cat");
    let schema = "s";
    let model = "m";
    let full_name = format!("{catalog}.{schema}.{model}");
    with_cleanup(
        || async {
            ctx.create_catalog_for_managed_volume(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;
            ctx.client()
                .create_registered_model(CreateRegisteredModel {
                    name: model.to_string(),
                    catalog_name: catalog.clone(),
                    schema_name: schema.to_string(),
                    ..Default::default()
                })
                .await?;

            let created = ctx
                .client()
                .create_model_version(CreateModelVersion {
                    model_name: model.to_string(),
                    catalog_name: catalog.clone(),
                    schema_name: schema.to_string(),
                    source: "s3://source/model".to_string(),
                    ..Default::default()
                })
                .await?;
            assert_eq!(created.version, 1);
            assert_eq!(
                created.status.as_known(),
                Some(ModelVersionStatus::PendingRegistration)
            );

            let fetched = ctx.client().get_model_version(&full_name, 1).await?;
            assert_eq!(fetched.version, 1);

            let finalized = ctx.client().finalize_model_version(&full_name, 1).await?;
            assert_eq!(finalized.status.as_known(), Some(ModelVersionStatus::Ready));

            let versions: Vec<_> = ctx
                .client()
                .list_model_versions(&full_name)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                versions.iter().any(|v| v.version == 1),
                "model version missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx.client().delete_model_version(&full_name, 1).await;
            let _ = ctx
                .client()
                .registered_model_from_full_name(&full_name)
                .delete()
                .with_force(true)
                .await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}
