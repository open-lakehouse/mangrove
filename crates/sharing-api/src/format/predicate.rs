//! Translate Delta Sharing JSON predicate hints into kernel predicates.
//!
//! Hints are advisory: the server may use them to skip files, the recipient still
//! filters. So the translation is **best-effort but sound** — it never produces a
//! predicate that would skip a file the original predicate could match:
//!
//! - an unsupported child of an `and` is dropped (the conjunction only gets
//!   weaker, so it skips fewer files);
//! - an `or` needs every child, and a `not` needs an *exact* translation of its
//!   child (negating a weakened predicate would make it stronger);
//! - anything else unsupported drops the whole hint.
//!
//! Supported ops: `and`, `or`, `not`, `isNull`, `equal`, `lessThan`,
//! `lessThanOrEqual`, `greaterThan`, `greaterThanOrEqual` over `column` and
//! `literal` leaves of type `boolean`, `int`, `long`, `float`, `double`, `string`
//! and `date`.

use chrono::NaiveDate;
use delta_kernel::expressions::{Expression, Predicate, Scalar};

use open_sharing_common::models::open_sharing::v1::JsonPredicate;

/// Translate a JSON predicate hint, or `None` if no sound pruning predicate can be
/// derived from it.
pub fn to_kernel_predicate(hint: &JsonPredicate) -> Option<Predicate> {
    predicate(hint, false)
}

/// Translate `node` as a predicate. With `exact`, only a translation equivalent to
/// `node` is acceptable; otherwise a weaker one (implied by `node`) is too.
fn predicate(node: &JsonPredicate, exact: bool) -> Option<Predicate> {
    match node.op.as_str() {
        "and" => {
            let children: Vec<_> = node
                .children
                .iter()
                .map(|child| predicate(child, exact))
                .collect();
            if exact && children.iter().any(Option::is_none) {
                return None;
            }
            let children: Vec<_> = children.into_iter().flatten().collect();
            (!children.is_empty()).then(|| Predicate::and_from(children))
        }
        "or" => {
            let children: Option<Vec<_>> = node
                .children
                .iter()
                .map(|child| predicate(child, exact))
                .collect();
            children.filter(|c| !c.is_empty()).map(Predicate::or_from)
        }
        "not" => match node.children.as_slice() {
            [child] => predicate(child, true).map(Predicate::not),
            _ => None,
        },
        "isNull" => match node.children.as_slice() {
            [child] => expression(child).map(Expression::is_null),
            _ => None,
        },
        op @ ("equal" | "lessThan" | "lessThanOrEqual" | "greaterThan" | "greaterThanOrEqual") => {
            let [left, right] = node.children.as_slice() else {
                return None;
            };
            let (left, right) = (expression(left)?, expression(right)?);
            Some(match op {
                "equal" => left.eq(right),
                "lessThan" => left.lt(right),
                "lessThanOrEqual" => left.le(right),
                "greaterThan" => left.gt(right),
                _ => left.ge(right),
            })
        }
        _ => None,
    }
}

/// Translate a `column` / `literal` leaf.
fn expression(node: &JsonPredicate) -> Option<Expression> {
    match node.op.as_str() {
        "column" => {
            let name = node.name.as_deref().filter(|n| !n.is_empty())?;
            Some(Expression::column([name]))
        }
        "literal" => {
            let value = node.value.as_deref()?;
            let scalar = match node.value_type.as_deref()? {
                "bool" | "boolean" => Scalar::Boolean(value.parse().ok()?),
                "int" => Scalar::Integer(value.parse().ok()?),
                "long" => Scalar::Long(value.parse().ok()?),
                "float" => Scalar::Float(value.parse().ok()?),
                "double" => Scalar::Double(value.parse().ok()?),
                "string" => Scalar::String(value.to_string()),
                "date" => {
                    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()?;
                    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1)?;
                    Scalar::Date(i32::try_from((date - epoch).num_days()).ok()?)
                }
                _ => return None,
            };
            Some(Expression::literal(scalar))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(json: serde_json::Value) -> JsonPredicate {
        serde_json::from_value(json).unwrap()
    }

    fn column(name: &str, ty: &str) -> serde_json::Value {
        serde_json::json!({"op": "column", "name": name, "valueType": ty})
    }

    fn literal(value: &str, ty: &str) -> serde_json::Value {
        serde_json::json!({"op": "literal", "value": value, "valueType": ty})
    }

    fn unsupported() -> serde_json::Value {
        serde_json::json!({"op": "like", "children": []})
    }

    #[test]
    fn translates_comparisons() {
        let hint = parse(serde_json::json!({
            "op": "equal",
            "children": [column("letter", "string"), literal("a", "string")]
        }));
        let expected = Expression::column(["letter"]).eq(Expression::literal("a"));
        assert_eq!(to_kernel_predicate(&hint), Some(expected));
    }

    #[test]
    fn translates_dates_to_days_since_epoch() {
        let hint = parse(serde_json::json!({
            "op": "greaterThanOrEqual",
            "children": [column("d", "date"), literal("1970-01-11", "date")]
        }));
        let expected = Expression::column(["d"]).ge(Expression::literal(Scalar::Date(10)));
        assert_eq!(to_kernel_predicate(&hint), Some(expected));
    }

    #[test]
    fn and_drops_unsupported_children() {
        let hint = parse(serde_json::json!({
            "op": "and",
            "children": [
                {"op": "equal", "children": [column("letter", "string"), literal("a", "string")]},
                unsupported(),
            ]
        }));
        let expected =
            Predicate::and_from([Expression::column(["letter"]).eq(Expression::literal("a"))]);
        assert_eq!(to_kernel_predicate(&hint), Some(expected));
    }

    #[test]
    fn or_needs_every_child() {
        let hint = parse(serde_json::json!({
            "op": "or",
            "children": [
                {"op": "equal", "children": [column("letter", "string"), literal("a", "string")]},
                unsupported(),
            ]
        }));
        assert_eq!(to_kernel_predicate(&hint), None);
    }

    #[test]
    fn not_needs_an_exact_child() {
        // NOT(a AND <unsupported>) must not become NOT(a): that would skip files
        // where `a` holds but the unsupported conjunct does not.
        let hint = parse(serde_json::json!({
            "op": "not",
            "children": [{
                "op": "and",
                "children": [
                    {"op": "equal", "children": [column("letter", "string"), literal("a", "string")]},
                    unsupported(),
                ]
            }]
        }));
        assert_eq!(to_kernel_predicate(&hint), None);
    }

    #[test]
    fn rejects_malformed_literals() {
        let hint = parse(serde_json::json!({
            "op": "equal",
            "children": [column("n", "int"), literal("not-a-number", "int")]
        }));
        assert_eq!(to_kernel_predicate(&hint), None);
    }
}
