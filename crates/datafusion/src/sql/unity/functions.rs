use datafusion::arrow::array::RecordBatch;
use datafusion::common::{DataFusionError, Result};
use datafusion::sql::sqlparser::ast::{DataType as SqlDataType, ObjectName};
use unitycatalog_client::UnityCatalogClient;
use unitycatalog_common::models::functions::v1::{
    FunctionParameterInfo, FunctionParameterInfos, ParameterMode, ParameterStyle, RoutineBody,
    SecurityType, SqlDataAccess,
};

use crate::sql::unity::{create_response_to_batch, drop_response_to_batch};

/// The language a Unity Catalog function body is written in.
///
/// Only [`FunctionLanguage::Sql`] is executable today; the other variants exist
/// so the host parser can faithfully carry what the user wrote and this crate
/// can reject it with a precise error rather than creating it incorrectly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum FunctionLanguage {
    Sql,
    Python,
    External,
}

/// The SQL data-access characteristic of a function (`CONTAINS SQL` /
/// `READS SQL DATA` / `NO SQL`). Defaults to `CONTAINS SQL` for SQL UDFs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Hash)]
pub enum SqlDataAccessKind {
    ContainsSql,
    ReadsSqlData,
    NoSql,
}

impl From<SqlDataAccessKind> for SqlDataAccess {
    fn from(value: SqlDataAccessKind) -> Self {
        match value {
            SqlDataAccessKind::ContainsSql => SqlDataAccess::ContainsSql,
            SqlDataAccessKind::ReadsSqlData => SqlDataAccess::ReadsSqlData,
            SqlDataAccessKind::NoSql => SqlDataAccess::NoSql,
        }
    }
}

/// A single `CREATE FUNCTION` parameter: `name data_type [DEFAULT expr] [COMMENT str]`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct FunctionParam {
    pub name: String,
    pub data_type: SqlDataType,
    pub default: Option<String>,
    pub comment: Option<String>,
}

/// `CREATE [OR REPLACE] FUNCTION [IF NOT EXISTS] <catalog>.<schema>.<fn>(<params>)
/// RETURNS <type> [characteristics] RETURN <expr>` — a Unity Catalog scalar SQL
/// UDF. Lowered to an `Extension` node like the other UC DDL so it rides through
/// the SQL DDL gate and is authorized by Cedar.
///
/// Only scalar SQL UDFs are supported: `RETURNS TABLE` (UDTFs) and non-SQL
/// languages are rejected at execution with a clear `NotImplemented` error.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct CreateFunctionStatement {
    pub name: ObjectName,
    /// `OR REPLACE` — carried for the Cedar/explain contract; the UC
    /// `CreateFunction` API has no replace flag, so it has no effect yet.
    pub or_replace: bool,
    /// `IF NOT EXISTS` — carried for the contract; no API field yet.
    pub if_not_exists: bool,
    pub params: Vec<FunctionParam>,
    /// Scalar return type from the `RETURNS` clause.
    pub returns: SqlDataType,
    pub language: FunctionLanguage,
    /// `DETERMINISTIC` (true) / `NOT DETERMINISTIC` (false); SQL UDFs default true.
    pub deterministic: bool,
    pub data_access: SqlDataAccessKind,
    /// The raw `RETURN <expr>` body text.
    pub routine_definition: String,
    pub comment: Option<String>,
}

impl CreateFunctionStatement {
    pub(crate) async fn execute(&self, client: UnityCatalogClient) -> Result<RecordBatch> {
        let (catalog, schema, func) = split_function_name(&self.name)?;

        if self.language != FunctionLanguage::Sql {
            return Err(DataFusionError::NotImplemented(
                "only LANGUAGE SQL functions are supported; \
                 Python and external functions are not yet implemented"
                    .to_string(),
            ));
        }

        // We keep the parsed SQL type text for both `data_type` and
        // `full_data_type`; `type_name` (the ColumnTypeName enum) is left
        // unspecified, mirroring a minimal create payload.
        let data_type_text = self.returns.to_string();

        let params = FunctionParameterInfos {
            parameters: self
                .params
                .iter()
                .enumerate()
                .map(|(i, p)| FunctionParameterInfo {
                    name: p.name.clone(),
                    type_text: p.data_type.to_string(),
                    position: Some(i as i32),
                    parameter_mode: ::buffa::EnumValue::Known(ParameterMode::In),
                    parameter_type: ::buffa::EnumValue::Known(
                        unitycatalog_common::models::functions::v1::FunctionParameterType::Param,
                    ),
                    parameter_default: p.default.clone(),
                    comment: p.comment.clone(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        };

        let info = client
            .create_function(
                func,
                catalog,
                schema,
                data_type_text.clone(),
                data_type_text,
                ParameterStyle::S,
                self.deterministic,
                self.data_access.into(),
                // Null-calling is not a writable DDL clause; default to false.
                false,
                SecurityType::Definer,
                RoutineBody::Sql,
            )
            .with_input_params(Some(params))
            .with_routine_definition(Some(self.routine_definition.clone()))
            .with_routine_body_language(Some("SQL".to_string()))
            .with_comment(self.comment.clone())
            .await
            .map_err(|e| DataFusionError::External(Box::new(e)))?;

        create_response_to_batch(self.name.to_string(), "Function", info)
    }
}

/// `DROP FUNCTION [IF EXISTS] <catalog>.<schema>.<fn>`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub struct DropFunctionStatement {
    pub name: ObjectName,
    pub if_exists: bool,
}

impl DropFunctionStatement {
    pub(crate) async fn execute(&self, client: UnityCatalogClient) -> Result<RecordBatch> {
        let (catalog, schema, func) = split_function_name(&self.name)?;
        // Functions have no cascade concept; always a plain delete.
        client
            .function(catalog, schema, func)
            .delete()
            .with_force(false)
            .await
            .map_err(|e| DataFusionError::External(Box::new(e)))?;
        drop_response_to_batch(self.name.to_string(), "Function", "success")
    }
}

/// Split a function [`ObjectName`] into `(catalog, schema, function)`.
///
/// Functions must be fully qualified — hydrofoil has no "current catalog" to
/// resolve a shorter name against (matching [`super::tables`]' three-part
/// requirement for managed tables).
fn split_function_name(name: &ObjectName) -> Result<(String, String, String)> {
    match name.0.as_slice() {
        [catalog, schema, function] => Ok((
            catalog.to_string(),
            schema.to_string(),
            function.to_string(),
        )),
        _ => Err(DataFusionError::Execution(format!(
            "Function name '{name}' must be a three-part identifier (<catalog>.<schema>.<function>)"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use datafusion::sql::sqlparser::ast::Ident;

    use super::*;

    fn name(parts: &[&str]) -> ObjectName {
        parts
            .iter()
            .map(|p| Ident::new(*p))
            .collect::<Vec<_>>()
            .into()
    }

    #[test]
    fn split_function_name_requires_three_parts() {
        assert!(split_function_name(&name(&["c", "s", "f"])).is_ok());
        let (catalog, schema, func) = split_function_name(&name(&["c", "s", "f"])).unwrap();
        assert_eq!(
            (catalog.as_str(), schema.as_str(), func.as_str()),
            ("c", "s", "f")
        );

        for bad in [name(&["f"]), name(&["s", "f"]), name(&["a", "b", "c", "d"])] {
            assert!(
                split_function_name(&bad).is_err(),
                "{bad} should be rejected"
            );
        }
    }

    #[test]
    fn sql_data_access_maps_to_proto_enum() {
        assert_eq!(
            SqlDataAccess::from(SqlDataAccessKind::ContainsSql),
            SqlDataAccess::ContainsSql
        );
        assert_eq!(
            SqlDataAccess::from(SqlDataAccessKind::ReadsSqlData),
            SqlDataAccess::ReadsSqlData
        );
        assert_eq!(
            SqlDataAccess::from(SqlDataAccessKind::NoSql),
            SqlDataAccess::NoSql
        );
    }

    /// A scalar SQL UDF statement we can reuse across mapping assertions.
    fn sample_create() -> CreateFunctionStatement {
        CreateFunctionStatement {
            name: name(&["c", "s", "add"]),
            or_replace: false,
            if_not_exists: false,
            params: vec![
                FunctionParam {
                    name: "x".to_string(),
                    data_type: SqlDataType::Int(None),
                    default: None,
                    comment: Some("first".to_string()),
                },
                FunctionParam {
                    name: "y".to_string(),
                    data_type: SqlDataType::Int(None),
                    default: Some("1".to_string()),
                    comment: None,
                },
            ],
            returns: SqlDataType::Int(None),
            language: FunctionLanguage::Sql,
            deterministic: true,
            data_access: SqlDataAccessKind::ContainsSql,
            routine_definition: "x + y".to_string(),
            comment: None,
        }
    }

    /// Reproduce the `input_params` construction from `execute` so we can assert
    /// the SQL-clause → `FunctionParameterInfo` mapping without a live client.
    fn build_params(stmt: &CreateFunctionStatement) -> FunctionParameterInfos {
        use unitycatalog_common::models::functions::v1::FunctionParameterType;
        FunctionParameterInfos {
            parameters: stmt
                .params
                .iter()
                .enumerate()
                .map(|(i, p)| FunctionParameterInfo {
                    name: p.name.clone(),
                    type_text: p.data_type.to_string(),
                    position: Some(i as i32),
                    parameter_mode: ::buffa::EnumValue::Known(ParameterMode::In),
                    parameter_type: ::buffa::EnumValue::Known(FunctionParameterType::Param),
                    parameter_default: p.default.clone(),
                    comment: p.comment.clone(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    #[test]
    fn params_map_with_zero_based_positions_and_metadata() {
        use unitycatalog_common::models::functions::v1::FunctionParameterType;
        let params = build_params(&sample_create());
        assert_eq!(params.parameters.len(), 2);

        let x = &params.parameters[0];
        assert_eq!(x.name, "x");
        assert_eq!(x.position, Some(0));
        assert_eq!(
            x.parameter_mode,
            ::buffa::EnumValue::Known(ParameterMode::In)
        );
        assert_eq!(
            x.parameter_type,
            ::buffa::EnumValue::Known(FunctionParameterType::Param)
        );
        assert_eq!(x.parameter_default, None);
        assert_eq!(x.comment.as_deref(), Some("first"));

        let y = &params.parameters[1];
        assert_eq!(y.position, Some(1));
        assert_eq!(y.parameter_default.as_deref(), Some("1"));
        assert_eq!(y.comment, None);
    }
}
