//! Deriving a metric view's [`DependencyList`] from its definition.
//!
//! `view_dependencies` is the set of tables (and functions) a metric view reads.
//! The definition is the single source of truth, so the server *derives* this
//! rather than trusting a client-supplied list. Derivation is **strict**: every
//! relation named by the view's `source` and its `joins` must resolve to a
//! concrete three-part `catalog.schema.table` name, or derivation fails — we
//! never store a partial dependency list.
//!
//! Two source shapes are supported:
//!
//! * a bare three-part name (`catalog.schema.table`) — the common case, resolved
//!   structurally; and
//! * an inline SQL query — parsed with [`sqlparser`], walking every `FROM` /
//!   `JOIN` relation and collecting the tables it reads.

use sqlparser::ast::{ObjectName, Query, SetExpr, Statement, TableFactor};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

use super::model::MetricView;
use crate::models::tables::v1::{Dependency, DependencyList, TableDependency, dependency};

/// Failure deriving dependencies from a metric-view definition.
#[derive(Debug, thiserror::Error)]
pub enum DependencyError {
    /// A relation reference could not be resolved to a `catalog.schema.table`
    /// name (e.g. a one- or two-part name, or an unsupported source shape).
    #[error(
        "cannot resolve metric-view dependency '{0}': expected a three-part catalog.schema.table name"
    )]
    UnresolvedRelation(String),

    /// An inline-SQL source could not be parsed.
    #[error("invalid inline SQL in metric-view source: {0}")]
    Sql(#[source] sqlparser::parser::ParserError),

    /// The inline SQL parsed to something other than a single query.
    #[error("metric-view source SQL must be a single SELECT query")]
    NotAQuery,
}

/// Derive the [`DependencyList`] a metric view reads from its `source` and
/// `joins`. The result is deduplicated and ordered by first appearance.
///
/// Returns an error if any relation cannot be resolved (strict policy).
pub fn dependencies(view: &MetricView) -> Result<DependencyList, DependencyError> {
    let mut tables: Vec<String> = Vec::new();
    collect_source(&view.source, &mut tables)?;
    for join in &view.joins {
        collect_source(&join.source, &mut tables)?;
    }

    let dependencies = tables
        .into_iter()
        .map(|table_full_name| Dependency {
            dependency: Some(dependency::Dependency::Table(Box::new(TableDependency {
                table_full_name,
                ..Default::default()
            }))),
            ..Default::default()
        })
        .collect();
    Ok(DependencyList {
        dependencies,
        ..Default::default()
    })
}

/// Resolve a single metric-view `source` string into table dependencies,
/// appending each `catalog.schema.table` name to `out` (deduplicated).
fn collect_source(source: &str, out: &mut Vec<String>) -> Result<(), DependencyError> {
    // Fast path: a bare three-part name is by far the common case and needs no
    // SQL parser.
    if let Some(name) = three_part_name(source) {
        push_unique(out, name);
        return Ok(());
    }

    // A bare dotted identifier path that isn't exactly three parts (e.g.
    // `schema.table`) is a name, not a query — report it as unresolvable rather
    // than feeding it to the SQL parser, where it would surface as a confusing
    // parse error.
    if is_bare_name(source) {
        return Err(DependencyError::UnresolvedRelation(
            source.trim().to_string(),
        ));
    }

    // Otherwise treat the source as inline SQL and collect its base relations.
    let dialect = GenericDialect {};
    let statements = Parser::parse_sql(&dialect, source).map_err(DependencyError::Sql)?;
    let [Statement::Query(query)] = statements.as_slice() else {
        return Err(DependencyError::NotAQuery);
    };
    let mut names = Vec::new();
    collect_query_relations(query, &mut names);
    if names.is_empty() {
        return Err(DependencyError::UnresolvedRelation(source.to_string()));
    }
    for name in names {
        let full = object_name_three_part(&name)
            .ok_or_else(|| DependencyError::UnresolvedRelation(name.to_string()))?;
        push_unique(out, full);
    }
    Ok(())
}

/// If `s` is a dotted three-part `catalog.schema.table` identifier (no SQL
/// syntax, no quoting), return it normalized; otherwise `None`.
fn three_part_name(s: &str) -> Option<String> {
    let s = s.trim();
    let parts: Vec<&str> = s.split('.').collect();
    if parts.len() == 3 && parts.iter().all(|p| is_plain_ident(p)) {
        Some(parts.join("."))
    } else {
        None
    }
}

/// A plain SQL identifier segment: non-empty, alphanumeric/underscore only.
/// Rejects anything that hints at SQL syntax (spaces, parens, quotes), forcing
/// it down the inline-SQL path instead.
fn is_plain_ident(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Whether `s` is a bare dotted identifier path (`a`, `a.b`, `a.b.c.d`) with no
/// SQL syntax — i.e. a relation name rather than an inline query.
fn is_bare_name(s: &str) -> bool {
    let s = s.trim();
    !s.is_empty() && s.split('.').all(is_plain_ident)
}

/// Walk a parsed query and collect every base relation referenced in `FROM` /
/// `JOIN` clauses, recursing through subqueries and set operations.
fn collect_query_relations(query: &Query, out: &mut Vec<ObjectName>) {
    collect_set_expr(&query.body, out);
}

fn collect_set_expr(set_expr: &SetExpr, out: &mut Vec<ObjectName>) {
    match set_expr {
        SetExpr::Select(select) => {
            for twj in &select.from {
                collect_table_factor(&twj.relation, out);
                for join in &twj.joins {
                    collect_table_factor(&join.relation, out);
                }
            }
        }
        SetExpr::Query(q) => collect_query_relations(q, out),
        SetExpr::SetOperation { left, right, .. } => {
            collect_set_expr(left, out);
            collect_set_expr(right, out);
        }
        // Values / Insert / Update / Table reference no source relation we track.
        _ => {}
    }
}

fn collect_table_factor(factor: &TableFactor, out: &mut Vec<ObjectName>) {
    match factor {
        TableFactor::Table { name, .. } => out.push(name.clone()),
        TableFactor::Derived { subquery, .. } => collect_query_relations(subquery, out),
        TableFactor::NestedJoin {
            table_with_joins, ..
        } => {
            collect_table_factor(&table_with_joins.relation, out);
            for join in &table_with_joins.joins {
                collect_table_factor(&join.relation, out);
            }
        }
        // Table functions, UNNEST, JSON_TABLE, etc. are not catalog relations.
        _ => {}
    }
}

/// Render an [`ObjectName`] as a three-part `catalog.schema.table` name, or
/// `None` if it is not exactly three identifier parts.
fn object_name_three_part(name: &ObjectName) -> Option<String> {
    let idents: Vec<String> = name
        .0
        .iter()
        .map(|part| match part.as_ident() {
            Some(ident) => ident.value.clone(),
            None => part.to_string(),
        })
        .collect();
    if idents.len() == 3 {
        Some(idents.join("."))
    } else {
        None
    }
}

fn push_unique(out: &mut Vec<String>, name: String) {
    if !out.contains(&name) {
        out.push(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(source: &str, join_sources: &[&str]) -> MetricView {
        MetricView {
            version: None,
            source: source.to_string(),
            filter: None,
            joins: join_sources
                .iter()
                .enumerate()
                .map(|(i, s)| super::super::model::Join {
                    name: format!("j{i}"),
                    source: s.to_string(),
                    on: None,
                    using: vec![],
                    cardinality: None,
                })
                .collect(),
            dimensions: vec![],
            measures: vec![],
        }
    }

    fn table_names(list: &DependencyList) -> Vec<String> {
        list.dependencies
            .iter()
            .filter_map(|d| match &d.dependency {
                Some(dependency::Dependency::Table(t)) => Some(t.table_full_name.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn three_part_source() {
        let deps = dependencies(&view("main.sales.orders", &[])).unwrap();
        assert_eq!(table_names(&deps), vec!["main.sales.orders"]);
    }

    #[test]
    fn source_plus_joins_dedup() {
        let deps = dependencies(&view(
            "main.sales.orders",
            &["main.sales.customers", "main.sales.orders"],
        ))
        .unwrap();
        assert_eq!(
            table_names(&deps),
            vec!["main.sales.orders", "main.sales.customers"]
        );
    }

    #[test]
    fn inline_sql_source_resolved() {
        let deps = dependencies(&view(
            "SELECT * FROM main.sales.orders o JOIN main.sales.customers c ON o.c_id = c.id",
            &[],
        ))
        .unwrap();
        assert_eq!(
            table_names(&deps),
            vec!["main.sales.orders", "main.sales.customers"]
        );
    }

    #[test]
    fn two_part_source_is_unresolvable() {
        let err = dependencies(&view("sales.orders", &[])).unwrap_err();
        assert!(matches!(err, DependencyError::UnresolvedRelation(_)));
    }

    #[test]
    fn inline_sql_with_two_part_relation_is_unresolvable() {
        let err = dependencies(&view("SELECT * FROM sales.orders", &[])).unwrap_err();
        assert!(matches!(err, DependencyError::UnresolvedRelation(_)));
    }

    #[test]
    fn malformed_inline_sql_errors() {
        let err = dependencies(&view("SELECT * FROM (", &[])).unwrap_err();
        assert!(matches!(err, DependencyError::Sql(_)));
    }
}
