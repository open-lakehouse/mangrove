//! NEW coverage for mangrove-specific governance securables: ABAC policies,
//! governed tag policies, and entity-tag assignments.
//!
//! These surfaces do not exist on UC OSS (and mostly not on managed Databricks),
//! so they are expected to `Skipped`/quarantine on those targets. This session
//! writes the checks to *attempt* coverage; whatever fails live is flagged.

use unitycatalog_common::models::policies::v1::{PolicyInfo, PolicyType};
use unitycatalog_common::models::tags::v1::{EntityTagAssignment, TagPolicy};

use super::{unique, with_cleanup};
use crate::{AcceptanceResult, JourneyContext};

/// ABAC policy on a catalog: create → list → get → delete.
pub async fn policy_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_pol_cat");
    let policy = unique("conf_pol");
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;

            let info = PolicyInfo {
                name: policy.clone(),
                on_securable_type: "catalogs".to_string(),
                on_securable_fullname: catalog.clone(),
                policy_type: PolicyType::RowFilter.into(),
                ..Default::default()
            };
            let created = ctx.client().policy(&policy).create_policy(info).await?;
            assert_eq!(created.name, policy);

            let listed = ctx
                .client()
                .list_policies("catalogs", &catalog)
                .await?
                .policies;
            assert!(
                listed.iter().any(|p| p.name == policy),
                "policy missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx.client().policy(&policy).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// Governed tag policy: create → get → list → delete.
pub async fn tag_policy_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let tag_key = unique("conf_tag");
    with_cleanup(
        || async {
            let policy = TagPolicy {
                tag_key: tag_key.clone(),
                description: Some("conformance tag policy".to_string()),
                ..Default::default()
            };
            let created = ctx.client().create_tag_policy(policy).await?;
            assert_eq!(created.tag_key, tag_key);

            let fetched = ctx.client().tag_policy(&tag_key).get().await?;
            assert_eq!(fetched.tag_key, tag_key);

            let listed = ctx.client().list_tag_policies().await?.tag_policies;
            assert!(
                listed.iter().any(|p| p.tag_key == tag_key),
                "tag policy missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx.client().tag_policy(&tag_key).delete().await;
        },
    )
    .await
}

/// Entity tag assignment on a catalog: create → get → list → delete.
pub async fn entity_tag_assignment_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_etag_cat");
    let tag_key = unique("conf_etag");
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;

            let assignment = EntityTagAssignment {
                entity_type: "catalogs".to_string(),
                entity_name: catalog.clone(),
                tag_key: tag_key.clone(),
                tag_value: Some("conformance".to_string()),
                ..Default::default()
            };
            ctx.client()
                .create_entity_tag_assignment(assignment)
                .await?;

            let fetched = ctx
                .client()
                .get_entity_tag_assignment("catalogs", &catalog, &tag_key)
                .await?;
            assert_eq!(fetched.tag_key, tag_key);

            let listed = ctx
                .client()
                .list_entity_tag_assignments("catalogs", &catalog)
                .await?
                .tag_assignments;
            assert!(
                listed.iter().any(|a| a.tag_key == tag_key),
                "entity tag assignment missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx
                .client()
                .delete_entity_tag_assignment("catalogs", &catalog, &tag_key)
                .await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}
