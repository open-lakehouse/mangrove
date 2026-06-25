//! Detecting a metric view on a Unity Catalog [`Table`].
//!
//! A metric view is a first-class table type: [`TableType::MetricView`] with the
//! definition carried in the table's `view_definition` field. Per the Unity
//! Catalog Tables API, `view_definition` is a single text field whose format
//! depends on the table type — SQL for plain views, **YAML for metric views** —
//! so this module parses it as YAML only when the type is `METRIC_VIEW`.

use crate::models::tables::v1::{Table, TableType};

use super::model::MetricView;

/// If `table` is a metric view, parse and return its definition.
///
/// Returns `Ok(None)` for any non-metric-view table, `Err` if it is a metric
/// view but its `view_definition` is missing or malformed.
pub fn metric_view_of(table: &Table) -> Result<Option<MetricView>, MetricViewDetectError> {
    if table.table_type != ::buffa::EnumValue::Known(TableType::MetricView) {
        return Ok(None);
    }

    let yaml = table
        .view_definition
        .as_deref()
        .ok_or(MetricViewDetectError::MissingDefinition)?;
    let view = MetricView::from_yaml(yaml).map_err(MetricViewDetectError::Parse)?;
    Ok(Some(view))
}

/// Failure detecting/parsing a metric view from a [`Table`].
#[derive(Debug, thiserror::Error)]
pub enum MetricViewDetectError {
    #[error("table is a METRIC_VIEW but has no view_definition")]
    MissingDefinition,
    #[error("invalid metric-view YAML: {0}")]
    Parse(#[source] serde_yml::Error),
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::models::tables::v1::{Table, TableType};

    use super::*;

    fn table(table_type: TableType, view_definition: Option<&str>) -> Table {
        Table {
            name: "orders_metrics".to_string(),
            catalog_name: "main".to_string(),
            schema_name: "sales".to_string(),
            table_type: ::buffa::EnumValue::Known(table_type),
            view_definition: view_definition.map(str::to_string),
            properties: HashMap::new(),
            ..Default::default()
        }
    }

    const VALID_YAML: &str = r#"
version: "1.1"
source: main.sales.orders
dimensions:
  - name: order_date
    expr: o_orderdate
measures:
  - name: total_revenue
    expr: SUM(o_totalprice)
"#;

    #[test]
    fn detects_metric_view_from_field() {
        let view = metric_view_of(&table(TableType::MetricView, Some(VALID_YAML)))
            .expect("detect")
            .expect("is a metric view");
        assert_eq!(view.source, "main.sales.orders");
        assert_eq!(view.dimensions.len(), 1);
        assert_eq!(view.measures.len(), 1);
    }

    #[test]
    fn non_metric_view_is_none() {
        // A managed table, even if it carries a view_definition, is not detected.
        assert!(
            metric_view_of(&table(TableType::Managed, Some(VALID_YAML)))
                .unwrap()
                .is_none()
        );
        assert!(
            metric_view_of(&table(TableType::External, None))
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn metric_view_without_definition_errors() {
        let err = metric_view_of(&table(TableType::MetricView, None)).unwrap_err();
        assert!(matches!(err, MetricViewDetectError::MissingDefinition));
    }

    #[test]
    fn metric_view_with_bad_yaml_errors() {
        let err = metric_view_of(&table(TableType::MetricView, Some("just: [a, b"))).unwrap_err();
        assert!(matches!(err, MetricViewDetectError::Parse(_)));
    }
}
