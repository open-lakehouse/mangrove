//! Delta Sharing management coverage: shares, recipients, providers (extended).
//!
//! These are our-server / managed-Databricks resources; UC OSS does not implement
//! a sharing server, so on the OSS target they resolve to `Skipped`/quarantined.

use futures::StreamExt;
use unitycatalog_common::providers::v1::ProviderAuthenticationType;
use unitycatalog_common::recipients::v1::AuthenticationType;
use unitycatalog_common::tables::v1::{DataSourceFormat, TableType};

use super::{unique, with_cleanup};
use crate::{AcceptanceResult, JourneyContext};

/// Share: create (with a backing managed table) → get → list → delete.
pub async fn share_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_share_cat");
    let schema = "s";
    let table = "t";
    let share = unique("conf_share");
    let table_full = format!("{catalog}.{schema}.{table}");
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

            let created = ctx
                .client()
                .create_share(&share)
                .with_comment("conformance share".to_string())
                .await?;
            assert_eq!(created.name, share);

            let fetched = ctx.client().share(&share).get().await?;
            assert_eq!(fetched.name, share);

            let listed: Vec<_> = ctx
                .client()
                .list_shares()
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                listed.iter().any(|s| s.name == share),
                "share missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx.client().share(&share).delete().await;
            let _ = ctx
                .client()
                .table_from_full_name(&table_full)
                .delete()
                .await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// Recipient (TOKEN auth): create → get → list → delete.
pub async fn recipient_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let recipient = unique("conf_recipient");
    let owner =
        std::env::var("UC_TEST_RECIPIENT_OWNER").unwrap_or_else(|_| "account users".to_string());
    with_cleanup(
        || async {
            let created = ctx
                .client()
                .create_recipient(&recipient, AuthenticationType::Token, owner)
                .with_comment("conformance recipient".to_string())
                .await?;
            assert_eq!(created.name, recipient);

            let fetched = ctx.client().recipient(&recipient).get().await?;
            assert_eq!(fetched.name, recipient);

            let listed: Vec<_> = ctx
                .client()
                .list_recipients()
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                listed.iter().any(|r| r.name == recipient),
                "recipient missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx.client().recipient(&recipient).delete().await;
        },
    )
    .await
}

/// Provider (TOKEN auth): create → get → list → update comment → delete.
pub async fn provider_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let provider = unique("conf_provider");
    with_cleanup(
        || async {
            let created = ctx
                .client()
                .create_provider(&provider, ProviderAuthenticationType::Token)
                .with_comment("conformance provider".to_string())
                .await?;
            assert_eq!(created.name, provider);

            let fetched = ctx.client().provider(&provider).get().await?;
            assert_eq!(fetched.name, provider);

            let listed: Vec<_> = ctx
                .client()
                .list_providers()
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                listed.iter().any(|p| p.name == provider),
                "provider missing from listing"
            );

            ctx.client()
                .provider(&provider)
                .update()
                .with_comment("updated comment".to_string())
                .await?;
            Ok(())
        },
        || async {
            let _ = ctx.client().provider(&provider).delete().await;
        },
    )
    .await
}
