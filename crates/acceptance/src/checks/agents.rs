//! NEW coverage for mangrove-specific agent securables: agents and agent skills.
//!
//! These do not exist on UC OSS or managed Databricks, so they are expected to
//! `Skipped`/quarantine on those targets. This session writes the checks to
//! *attempt* coverage against our Rust server; whatever fails live is flagged.

use unitycatalog_common::models::agent_skills::v0alpha1::AgentSkillType;
use unitycatalog_common::models::agents::v0alpha1::InvocationProtocol;

use super::{unique, with_cleanup};
use crate::{AcceptanceResult, JourneyContext};

/// Agent: create (MCP protocol) → get → list → delete.
pub async fn agent_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_agent_cat");
    let schema = "s";
    let agent = "a";
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;

            let created = ctx
                .client()
                .create_agent(
                    &catalog,
                    schema,
                    agent,
                    InvocationProtocol::Mcp,
                    "https://example.invalid/agent",
                )
                .await?;
            assert_eq!(created.name, agent);

            let fetched = ctx.client().agent(&catalog, schema, agent).get().await?;
            assert_eq!(fetched.name, agent);

            let listed = ctx.client().list_agents(&catalog, schema).await?.agents;
            assert!(
                listed.iter().any(|a| a.name == agent),
                "agent missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx.client().agent(&catalog, schema, agent).delete().await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// Agent skill: create (MANAGED) → get → list → delete.
pub async fn agent_skill_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_skill_cat");
    let schema = "s";
    let skill = "sk";
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;

            let created = ctx
                .client()
                .create_agent_skill(&catalog, schema, skill, AgentSkillType::Managed)
                .await?;
            assert_eq!(created.name, skill);

            let fetched = ctx
                .client()
                .agent_skill(&catalog, schema, skill)
                .get()
                .await?;
            assert_eq!(fetched.name, skill);

            let listed = ctx
                .client()
                .list_agent_skills(&catalog, schema)
                .await?
                .agent_skills;
            assert!(
                listed.iter().any(|s| s.name == skill),
                "agent skill missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx
                .client()
                .agent_skill(&catalog, schema, skill)
                .delete()
                .await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}
