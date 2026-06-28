//! The `MEASURE()` marker aggregate function.
//!
//! A query against a metric view references a measure by wrapping the measure's
//! name in `MEASURE(...)`, e.g.
//!
//! ```sql
//! SELECT order_date, MEASURE(total_revenue)
//! FROM mv
//! GROUP BY order_date
//! ```
//!
//! `MEASURE` is registered as an aggregate UDF purely so that such a query
//! *plans* — the SQL planner accepts it in the select list alongside a
//! `GROUP BY`, and `total_revenue` resolves against the metric view's declared
//! schema. It is a **marker only**: [`ResolveMetricView`] rewrites every
//! `MEASURE(measure_col)` into the measure's real aggregate expression before
//! execution, so the accumulator below is never actually run. This mirrors
//! Spark's `Measure` placeholder resolved by `ResolveMetricView`.
//!
//! [`ResolveMetricView`]: super::analyzer::ResolveMetricView

use std::sync::{Arc, LazyLock};

use datafusion::arrow::datatypes::DataType;
use datafusion::common::{exec_err, plan_err};
use datafusion::error::Result;
use datafusion::logical_expr::function::{AccumulatorArgs, StateFieldsArgs};
use datafusion::logical_expr::{
    Accumulator, AggregateUDF, AggregateUDFImpl, Signature, Volatility,
};

/// Name of the marker aggregate function, as written in queries.
pub const MEASURE_UDF_NAME: &str = "measure";

/// The `MEASURE()` marker aggregate UDF. See module docs.
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct MeasureMarkerUdf {
    signature: Signature,
}

impl Default for MeasureMarkerUdf {
    fn default() -> Self {
        Self {
            // One argument of any type, immutable. The arg is a reference to the
            // metric view's measure column; its concrete type is whatever the
            // measure declares.
            signature: Signature::any(1, Volatility::Immutable),
        }
    }
}

impl AggregateUDFImpl for MeasureMarkerUdf {
    fn name(&self) -> &str {
        MEASURE_UDF_NAME
    }

    fn signature(&self) -> &Signature {
        &self.signature
    }

    /// Identity: `MEASURE(col)` types as `col` so the wrapping expression
    /// type-checks against the metric view's declared measure type.
    fn return_type(&self, arg_types: &[DataType]) -> Result<DataType> {
        match arg_types {
            [ty] => Ok(ty.clone()),
            _ => plan_err!("MEASURE() takes exactly one argument"),
        }
    }

    /// Never invoked: `ResolveMetricView` rewrites `MEASURE(...)` away before the
    /// plan reaches execution. Reaching here means the analyzer rule was not
    /// registered or did not fire.
    fn accumulator(&self, _args: AccumulatorArgs) -> Result<Box<dyn Accumulator>> {
        exec_err!(
            "MEASURE() is a metric-view marker and must be resolved by the \
             ResolveMetricView analyzer rule before execution; it cannot be \
             evaluated directly"
        )
    }

    fn state_fields(
        &self,
        _args: StateFieldsArgs,
    ) -> Result<Vec<datafusion::arrow::datatypes::FieldRef>> {
        exec_err!("MEASURE() has no execution state; it must be resolved before execution")
    }
}

/// The shared [`AggregateUDF`] handle for `MEASURE()`.
pub fn measure_udf() -> Arc<AggregateUDF> {
    static UDF: LazyLock<Arc<AggregateUDF>> =
        LazyLock::new(|| Arc::new(AggregateUDF::from(MeasureMarkerUdf::default())));
    UDF.clone()
}
