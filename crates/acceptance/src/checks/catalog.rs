//! Catalog and schema coverage — the portable baseline (all UC implementations).

use futures::StreamExt;

use super::{unique, with_cleanup};
use crate::{AcceptanceResult, JourneyContext};

/// Catalog create → verify → list → get → update comment → delete.
pub async fn catalog_crud(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_cat");
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;

            // Present in the listing.
            let found = ctx
                .client()
                .list_catalogs()
                .with_max_results(200)
                .into_stream()
                .any(|c| {
                    let catalog = catalog.clone();
                    async move { c.map(|c| c.name == catalog).unwrap_or(false) }
                })
                .await;
            assert!(found, "created catalog not found in listing");

            // Get by name.
            let fetched = ctx.client().catalog(&catalog).get().await?;
            assert_eq!(fetched.name, catalog);

            // Update comment (exercises PATCH).
            ctx.client()
                .catalog(&catalog)
                .update()
                .with_comment("updated comment".to_string())
                .await?;
            Ok(())
        },
        || async {
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// Catalog + several schemas → list → verify each present → delete all.
pub async fn catalog_hierarchy(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_hier_cat");
    let schemas = ["alpha", "beta", "gamma"];
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            for s in schemas {
                ctx.client().create_schema(s, &catalog).await?;
            }

            let listed: Vec<_> = ctx
                .client()
                .list_schemas(&catalog)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            for s in schemas {
                assert!(
                    listed.iter().any(|sc| sc.name == s),
                    "schema {s} missing from listing"
                );
            }
            Ok(())
        },
        || async {
            for s in schemas {
                let _ = ctx.client().schema(&catalog, s).delete().await;
            }
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// Schema create → get → list → update comment (plus a catalog update en route).
pub async fn schema_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_sc_cat");
    let schema = unique("conf_sc");
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client()
                .catalog(&catalog)
                .update()
                .with_comment("catalog comment".to_string())
                .await?;

            let created = ctx
                .client()
                .create_schema(&schema, &catalog)
                .with_comment("conformance schema".to_string())
                .await?;
            assert_eq!(created.name, schema);
            assert_eq!(created.catalog_name, catalog);

            let fetched = ctx.client().schema(&catalog, &schema).get().await?;
            assert_eq!(fetched.name, schema);

            let listed: Vec<_> = ctx
                .client()
                .list_schemas(&catalog)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                listed.iter().any(|s| s.name == schema),
                "created schema not found in listing"
            );

            ctx.client()
                .schema(&catalog, &schema)
                .update()
                .with_comment("updated comment".to_string())
                .await?;
            Ok(())
        },
        || async {
            let _ = ctx.client().schema(&catalog, &schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}
