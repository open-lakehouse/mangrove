//! Hand-written ergonomic constructors for the generated function models.
//!
//! `CreateFunctionRequest` wraps its payload in a `function_info` envelope (a
//! [`CreateFunction`]) to match the Unity Catalog wire contract, so the generated
//! `CreateFunctionBuilder` only exposes a single `function_info` setter. These helpers
//! restore fluent construction of the payload; they complement the `with_*` setters
//! buffa-codegen already emits on `CreateFunction` (`with_routine_definition`,
//! `with_routine_body_language`, `with_comment`) by adding a required-field constructor
//! and setters for the message- and map-typed fields codegen omits.

use super::functions::v1::{
    CreateFunction, FunctionParameterInfos, ParameterStyle, RoutineBody, SecurityType,
    SqlDataAccess,
};

impl CreateFunction {
    /// Create a `CreateFunction` payload from the required function attributes.
    ///
    /// Optional fields (input params, routine definition, comment, properties, …) are
    /// left unset and can be added with the `with_*` methods.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        catalog_name: impl Into<String>,
        schema_name: impl Into<String>,
        data_type: impl Into<String>,
        full_data_type: impl Into<String>,
        parameter_style: ParameterStyle,
        is_deterministic: bool,
        sql_data_access: SqlDataAccess,
        is_null_call: bool,
        security_type: SecurityType,
        routine_body: RoutineBody,
    ) -> Self {
        Self {
            name: name.into(),
            catalog_name: catalog_name.into(),
            schema_name: schema_name.into(),
            data_type: data_type.into(),
            full_data_type: full_data_type.into(),
            parameter_style: buffa::EnumValue::Known(parameter_style),
            is_deterministic,
            sql_data_access: buffa::EnumValue::Known(sql_data_access),
            is_null_call,
            security_type: buffa::EnumValue::Known(security_type),
            routine_body: buffa::EnumValue::Known(routine_body),
            ..Default::default()
        }
    }

    /// Set the array of function parameter infos.
    #[must_use = "with_* setters return `self` by value; assign or chain the result"]
    pub fn with_input_params(
        mut self,
        input_params: impl Into<Option<FunctionParameterInfos>>,
    ) -> Self {
        self.input_params = buffa::MessageField::from(input_params.into());
        self
    }

    /// Set the key-value properties attached to the securable.
    #[must_use = "with_* setters return `self` by value; assign or chain the result"]
    pub fn with_properties<I, K, V>(mut self, properties: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.properties = properties
            .into_iter()
            .map(|(k, v)| (k.into(), v.into()))
            .collect();
        self
    }
}
