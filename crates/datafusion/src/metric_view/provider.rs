//! [`MetricViewTableProvider`]: a DataFusion table provider for a metric view.
//!
//! The provider exposes the metric view's full relational schema (every
//! dimension and measure) and returns a [`MetricViewPlaceholder`] from
//! [`TableProvider::get_logical_plan`]. The SQL planner inlines that placeholder
//! in place of the table scan, and [`super::analyzer::ResolveMetricView`] later
//! rewrites it into a concrete aggregation. Because resolution happens during
//! analysis, [`TableProvider::scan`] is never reached for a metric view; it
//! errors if somehow called.

use std::borrow::Cow;
use std::sync::Arc;

use datafusion::arrow::datatypes::SchemaRef;
use datafusion::catalog::{Session, TableProvider};
use datafusion::common::not_impl_err;
use datafusion::error::Result;
use datafusion::execution::session_state::SessionState;
use datafusion::logical_expr::{Expr, LogicalPlan, TableType};
use datafusion::physical_plan::ExecutionPlan;

use super::lower::build_placeholder;
use super::model::MetricView;
use super::placeholder::MetricViewPlaceholder;

/// A [`TableProvider`] backing a Unity Catalog metric view.
#[derive(Debug)]
pub struct MetricViewTableProvider {
    /// The placeholder logical node, wrapped as a [`LogicalPlan::Extension`].
    plan: LogicalPlan,
    /// The view's relational schema (all dimensions + all measures).
    schema: SchemaRef,
}

impl MetricViewTableProvider {
    /// Build a provider from a parsed metric view and its resolved source plan.
    ///
    /// `state` supplies the SQL expression planner used to parse the view's
    /// dimension/measure/filter strings against the `source` schema.
    pub fn try_new(state: &SessionState, view: &MetricView, source: LogicalPlan) -> Result<Self> {
        let placeholder = build_placeholder(state, view, source)?;
        Self::from_placeholder(placeholder)
    }

    /// Build a provider directly from an already-constructed placeholder.
    pub fn from_placeholder(placeholder: MetricViewPlaceholder) -> Result<Self> {
        use datafusion::logical_expr::Extension;
        let schema = Arc::clone(
            datafusion::logical_expr::UserDefinedLogicalNodeCore::schema(&placeholder).inner(),
        );
        let plan = LogicalPlan::Extension(Extension {
            node: Arc::new(placeholder),
        });
        Ok(Self { plan, schema })
    }
}

#[async_trait::async_trait]
impl TableProvider for MetricViewTableProvider {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }

    fn table_type(&self) -> TableType {
        TableType::View
    }

    /// Hand the planner the placeholder logical plan so it inlines the metric
    /// view in place of a table scan. [`super::analyzer::ResolveMetricView`]
    /// then resolves it.
    fn get_logical_plan(&self) -> Option<Cow<'_, LogicalPlan>> {
        Some(Cow::Borrowed(&self.plan))
    }

    async fn scan(
        &self,
        _state: &dyn Session,
        _projection: Option<&Vec<usize>>,
        _filters: &[Expr],
        _limit: Option<usize>,
    ) -> Result<Arc<dyn ExecutionPlan>> {
        // The planner inlines `get_logical_plan` instead of scanning, and the
        // analyzer rewrites the placeholder before execution. Reaching here
        // means the placeholder survived planning unresolved.
        not_impl_err!(
            "metric view was not resolved before execution; ensure the \
             ResolveMetricView analyzer rule is registered on the SessionContext"
        )
    }
}
