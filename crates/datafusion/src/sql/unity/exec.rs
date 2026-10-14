//! Physical execution for Unity Catalog DDL.
//!
//! A [`ExecuteUnityCatalogPlanNode`] (the logical `Extension` node produced for
//! `CREATE`/`DROP CATALOG`/`SCHEMA`) is lowered by the [`UnityCatalogPlanner`]
//! to a [`StreamingTableExec`] over a [`UnityCatalogPartitionStream`], which
//! runs the statement against a live [`UnityCatalogClient`] and yields a single
//! result `RecordBatch`. Reusing DataFusion's generic streaming node keeps us
//! from re-implementing the `ExecutionPlan` boilerplate.
//!
//! The client is resolved at planning time from a [`UnityClientExtension`] set
//! on the session config; if it is absent the planner errors rather than
//! silently dropping the DDL.

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::arrow::datatypes::SchemaRef;
use datafusion::catalog::Session;
use datafusion::common::{exec_datafusion_err, internal_err};
use datafusion::error::Result;
use datafusion::execution::{SendableRecordBatchStream, TaskContext};
use datafusion::logical_expr::physical_planning_context::PhysicalPlanningContext;
use datafusion::logical_expr::{LogicalPlan, UserDefinedLogicalNode};
use datafusion::physical_plan::ExecutionPlan;
use datafusion::physical_plan::stream::RecordBatchStreamAdapter;
use datafusion::physical_plan::streaming::{PartitionStream, StreamingTableExec};
use datafusion::physical_planner::{ExtensionPlanner, PhysicalPlanner};
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_object_store::UnityObjectStoreFactory;

use super::{ExecutableUnityCatalogStatement, ExecuteUnityCatalogPlanNode, UnityCatalogStatement};

/// Session-config extension carrying the [`UnityCatalogClient`] used to execute
/// Unity Catalog DDL. Hydrofoil sets this on the session state when Unity
/// Catalog is wired; without it, UC DDL cannot be planned.
#[derive(Clone)]
pub struct UnityClientExtension(pub UnityCatalogClient);

/// Session-config extension carrying the [`UnityObjectStoreFactory`] used by the
/// managed-table write paths (bulk ingest append and managed `CREATE TABLE`).
/// Unlike [`UnityClientExtension`] (which only exposes the catalog client), the
/// factory also vends fresh per-table object-store credentials, which the kernel
/// committer needs to stage and publish commits. Set on the session state
/// alongside [`UnityClientExtension`] when Unity Catalog is wired.
#[derive(Clone)]
pub struct UnityFactoryExt(pub Arc<UnityObjectStoreFactory>);

/// [`ExtensionPlanner`] that lowers [`ExecuteUnityCatalogPlanNode`] to a
/// [`StreamingTableExec`] over a [`UnityCatalogPartitionStream`].
#[derive(Debug, Default)]
pub struct UnityCatalogPlanner;

#[async_trait]
impl ExtensionPlanner for UnityCatalogPlanner {
    async fn plan_extension(
        &self,
        _planner: &dyn PhysicalPlanner,
        node: &dyn UserDefinedLogicalNode,
        logical_inputs: &[&LogicalPlan],
        physical_inputs: &[Arc<dyn ExecutionPlan>],
        session_state: &dyn Session,
        _planning_ctx: &PhysicalPlanningContext,
    ) -> Result<Option<Arc<dyn ExecutionPlan>>> {
        let Some(node) = node.as_any().downcast_ref::<ExecuteUnityCatalogPlanNode>() else {
            return Ok(None);
        };
        if !logical_inputs.is_empty() || !physical_inputs.is_empty() {
            return internal_err!("ExecuteUnityCatalogPlanNode does not take inputs");
        }
        let client = session_state
            .config()
            .get_extension::<UnityClientExtension>()
            .ok_or_else(|| {
                exec_datafusion_err!(
                    "Unity Catalog is not configured; cannot execute Unity Catalog DDL"
                )
            })?;

        let schema: SchemaRef = Arc::new(node.statement.return_schema().as_arrow().clone());
        let partition = Arc::new(UnityCatalogPartitionStream {
            statement: node.statement.clone(),
            client: client.0.clone(),
            schema: schema.clone(),
        });
        let exec = StreamingTableExec::try_new(
            schema,
            vec![partition],
            None,  // no projection
            None,  // no output ordering
            false, // not infinite
            None,  // no limit
        )?;
        Ok(Some(Arc::new(exec)))
    }
}

/// A single-partition [`PartitionStream`] that executes one
/// [`UnityCatalogStatement`] against Unity Catalog, yielding one result batch.
struct UnityCatalogPartitionStream {
    statement: UnityCatalogStatement,
    client: UnityCatalogClient,
    schema: SchemaRef,
}

impl std::fmt::Debug for UnityCatalogPartitionStream {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnityCatalogPartitionStream")
            .field("statement", &self.statement.command_name())
            .finish_non_exhaustive()
    }
}

impl PartitionStream for UnityCatalogPartitionStream {
    fn schema(&self) -> &SchemaRef {
        &self.schema
    }

    fn execute(&self, _ctx: Arc<TaskContext>) -> SendableRecordBatchStream {
        let statement = self.statement.clone();
        let client = self.client.clone();
        let schema = self.schema.clone();
        let fut = async move { statement.execute(client).await };
        Box::pin(RecordBatchStreamAdapter::new(
            schema,
            futures::stream::once(fut),
        ))
    }
}
