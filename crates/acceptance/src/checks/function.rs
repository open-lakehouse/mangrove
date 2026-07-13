//! Function (SQL UDF) coverage. `function_lifecycle` is portable; `function_update`
//! is split out because UC OSS has no function-update operation.

use futures::StreamExt;
use unitycatalog_common::models::functions::v1::{
    ParameterStyle, RoutineBody, SecurityType, SqlDataAccess,
};

use super::{unique, with_cleanup};
use crate::{AcceptanceResult, JourneyContext};

/// Create a catalog+schema and a scalar SQL UDF; get; list. Portable baseline.
pub async fn function_lifecycle(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_fn_cat");
    let schema = "s";
    let function = "add_one";
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;

            let created = ctx
                .client()
                .create_function(
                    function,
                    &catalog,
                    schema,
                    "INT",
                    "INT",
                    ParameterStyle::S,
                    true,
                    SqlDataAccess::ContainsSql,
                    true,
                    SecurityType::Definer,
                    RoutineBody::Sql,
                )
                .with_routine_definition("SELECT 42".to_string())
                .with_comment("conformance UDF".to_string())
                .await?;
            assert_eq!(created.name, function);

            let fetched = ctx
                .client()
                .function(&catalog, schema, function)
                .get()
                .await?;
            assert_eq!(fetched.name, function);

            let functions: Vec<_> = ctx
                .client()
                .list_functions(&catalog, schema)
                .into_stream()
                .collect::<Vec<_>>()
                .await
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            assert!(
                functions.iter().any(|f| f.name == function),
                "function missing from listing"
            );
            Ok(())
        },
        || async {
            let _ = ctx
                .client()
                .function(&catalog, schema, function)
                .delete()
                .await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}

/// Function update (owner change). Split from the lifecycle because UC OSS v0.5.0
/// does not implement function update — quarantined for the oss_java target.
pub async fn function_update(ctx: &JourneyContext) -> AcceptanceResult<()> {
    let catalog = unique("conf_fnup_cat");
    let schema = "s";
    let function = "upd_fn";
    with_cleanup(
        || async {
            ctx.create_catalog(&catalog).await?;
            ctx.client().create_schema(schema, &catalog).await?;
            ctx.client()
                .create_function(
                    function,
                    &catalog,
                    schema,
                    "INT",
                    "INT",
                    ParameterStyle::S,
                    true,
                    SqlDataAccess::ContainsSql,
                    true,
                    SecurityType::Definer,
                    RoutineBody::Sql,
                )
                .with_routine_definition("SELECT 1".to_string())
                .await?;

            let updated = ctx
                .client()
                .function(&catalog, schema, function)
                .update()
                .with_owner("conformance_owner".to_string())
                .await?;
            assert_eq!(updated.owner.as_deref(), Some("conformance_owner"));
            Ok(())
        },
        || async {
            let _ = ctx
                .client()
                .function(&catalog, schema, function)
                .delete()
                .await;
            let _ = ctx.client().schema(&catalog, schema).delete().await;
            let _ = ctx.client().catalog(&catalog).delete().await;
        },
    )
    .await
}
