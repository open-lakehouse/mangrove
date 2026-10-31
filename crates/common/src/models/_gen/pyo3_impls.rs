// @generated — do not edit by hand.
/// A member of `unitycatalog_client._enums.AgentSkillType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyAgentSkillType(pub super::agent_skills::v0alpha1::AgentSkillType);
impl PyAgentSkillType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::agent_skills::v0alpha1::AgentSkillType as ::buffa::Enumeration>::to_i32(
            &self.0,
        ) {
            1i32 => ::core::option::Option::Some("EXTERNAL"),
            2i32 => ::core::option::Option::Some("MANAGED"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::agent_skills::v0alpha1::AgentSkillType> for PyAgentSkillType {
    fn from(value: super::agent_skills::v0alpha1::AgentSkillType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAgentSkillType> for super::agent_skills::v0alpha1::AgentSkillType {
    fn from(value: PyAgentSkillType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyAgentSkillType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "AgentSkillType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "EXTERNAL" => 1i32,
            "MANAGED" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid AgentSkillType: {:?}; expected one of: EXTERNAL, MANAGED",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::agent_skills::v0alpha1::AgentSkillType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyAgentSkillType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "AgentSkillType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "AgentSkillType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "AgentSkillType value {} has no Python member",
                <super::agent_skills::v0alpha1::AgentSkillType as ::buffa::Enumeration>::to_i32(
                    &self.0
                )
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.InvocationProtocol`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyInvocationProtocol(pub super::agents::v0alpha1::InvocationProtocol);
impl PyInvocationProtocol {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::agents::v0alpha1::InvocationProtocol as ::buffa::Enumeration>::to_i32(&self.0)
        {
            1i32 => ::core::option::Option::Some("MCP"),
            2i32 => ::core::option::Option::Some("A2A"),
            3i32 => ::core::option::Option::Some("OPENAI"),
            4i32 => ::core::option::Option::Some("ANTHROPIC"),
            5i32 => ::core::option::Option::Some("REST"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::agents::v0alpha1::InvocationProtocol> for PyInvocationProtocol {
    fn from(value: super::agents::v0alpha1::InvocationProtocol) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyInvocationProtocol> for super::agents::v0alpha1::InvocationProtocol {
    fn from(value: PyInvocationProtocol) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyInvocationProtocol {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "InvocationProtocol"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "MCP" => 1i32,
            "A2A" => 2i32,
            "OPENAI" => 3i32,
            "ANTHROPIC" => 4i32,
            "REST" => 5i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid InvocationProtocol: {:?}; expected one of: MCP, A2A, OPENAI, ANTHROPIC, REST",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::agents::v0alpha1::InvocationProtocol as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyInvocationProtocol {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "InvocationProtocol");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "InvocationProtocol")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "InvocationProtocol value {} has no Python member",
                <super::agents::v0alpha1::InvocationProtocol as ::buffa::Enumeration>::to_i32(
                    &self.0
                )
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.CatalogType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyCatalogType(pub super::catalogs::v1::CatalogType);
impl PyCatalogType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::catalogs::v1::CatalogType as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("MANAGED_CATALOG"),
            2i32 => ::core::option::Option::Some("DELTASHARING_CATALOG"),
            3i32 => ::core::option::Option::Some("SYSTEM_CATALOG"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::catalogs::v1::CatalogType> for PyCatalogType {
    fn from(value: super::catalogs::v1::CatalogType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyCatalogType> for super::catalogs::v1::CatalogType {
    fn from(value: PyCatalogType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyCatalogType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "CatalogType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "MANAGED_CATALOG" => 1i32,
            "DELTASHARING_CATALOG" => 2i32,
            "SYSTEM_CATALOG" => 3i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid CatalogType: {:?}; expected one of: MANAGED_CATALOG, DELTASHARING_CATALOG, SYSTEM_CATALOG",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::catalogs::v1::CatalogType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyCatalogType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "CatalogType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "CatalogType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "CatalogType value {} has no Python member",
                <super::catalogs::v1::CatalogType as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.Purpose`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyPurpose(pub super::credentials::v1::Purpose);
impl PyPurpose {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::credentials::v1::Purpose as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("STORAGE"),
            2i32 => ::core::option::Option::Some("SERVICE"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::credentials::v1::Purpose> for PyPurpose {
    fn from(value: super::credentials::v1::Purpose) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyPurpose> for super::credentials::v1::Purpose {
    fn from(value: PyPurpose) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyPurpose {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "Purpose"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "STORAGE" => 1i32,
            "SERVICE" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid Purpose: {:?}; expected one of: STORAGE, SERVICE",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::credentials::v1::Purpose as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyPurpose {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "Purpose");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "Purpose")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "Purpose value {} has no Python member",
                <super::credentials::v1::Purpose as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.FunctionParameterType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyFunctionParameterType(pub super::functions::v1::FunctionParameterType);
impl PyFunctionParameterType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::functions::v1::FunctionParameterType as ::buffa::Enumeration>::to_i32(&self.0)
        {
            1i32 => ::core::option::Option::Some("COLUMN"),
            2i32 => ::core::option::Option::Some("PARAM"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::functions::v1::FunctionParameterType>
    for PyFunctionParameterType
{
    fn from(value: super::functions::v1::FunctionParameterType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyFunctionParameterType>
    for super::functions::v1::FunctionParameterType
{
    fn from(value: PyFunctionParameterType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyFunctionParameterType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "FunctionParameterType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "COLUMN" => 1i32,
            "PARAM" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid FunctionParameterType: {:?}; expected one of: COLUMN, PARAM",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::functions::v1::FunctionParameterType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyFunctionParameterType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "FunctionParameterType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "FunctionParameterType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "FunctionParameterType value {} has no Python member",
                <super::functions::v1::FunctionParameterType as ::buffa::Enumeration>::to_i32(
                    &self.0
                )
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.ParameterMode`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyParameterMode(pub super::functions::v1::ParameterMode);
impl PyParameterMode {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::functions::v1::ParameterMode as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("IN"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::functions::v1::ParameterMode> for PyParameterMode {
    fn from(value: super::functions::v1::ParameterMode) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyParameterMode> for super::functions::v1::ParameterMode {
    fn from(value: PyParameterMode) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyParameterMode {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "ParameterMode"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "IN" => 1i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!("invalid ParameterMode: {:?}; expected one of: IN", other),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::functions::v1::ParameterMode as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyParameterMode {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "ParameterMode");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "ParameterMode")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "ParameterMode value {} has no Python member",
                <super::functions::v1::ParameterMode as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.ParameterStyle`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyParameterStyle(pub super::functions::v1::ParameterStyle);
impl PyParameterStyle {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::functions::v1::ParameterStyle as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("S"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::functions::v1::ParameterStyle> for PyParameterStyle {
    fn from(value: super::functions::v1::ParameterStyle) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyParameterStyle> for super::functions::v1::ParameterStyle {
    fn from(value: PyParameterStyle) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyParameterStyle {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "ParameterStyle"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "S" => 1i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!("invalid ParameterStyle: {:?}; expected one of: S", other),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::functions::v1::ParameterStyle as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyParameterStyle {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "ParameterStyle");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "ParameterStyle")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "ParameterStyle value {} has no Python member",
                <super::functions::v1::ParameterStyle as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.RoutineBody`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyRoutineBody(pub super::functions::v1::RoutineBody);
impl PyRoutineBody {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::functions::v1::RoutineBody as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("SQL"),
            2i32 => ::core::option::Option::Some("EXTERNAL"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::functions::v1::RoutineBody> for PyRoutineBody {
    fn from(value: super::functions::v1::RoutineBody) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyRoutineBody> for super::functions::v1::RoutineBody {
    fn from(value: PyRoutineBody) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyRoutineBody {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "RoutineBody"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "SQL" => 1i32,
            "EXTERNAL" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid RoutineBody: {:?}; expected one of: SQL, EXTERNAL",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::functions::v1::RoutineBody as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyRoutineBody {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "RoutineBody");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "RoutineBody")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "RoutineBody value {} has no Python member",
                <super::functions::v1::RoutineBody as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.SecurityType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PySecurityType(pub super::functions::v1::SecurityType);
impl PySecurityType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::functions::v1::SecurityType as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("DEFINER"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::functions::v1::SecurityType> for PySecurityType {
    fn from(value: super::functions::v1::SecurityType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PySecurityType> for super::functions::v1::SecurityType {
    fn from(value: PySecurityType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PySecurityType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "SecurityType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "DEFINER" => 1i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid SecurityType: {:?}; expected one of: DEFINER",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::functions::v1::SecurityType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PySecurityType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "SecurityType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "SecurityType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "SecurityType value {} has no Python member",
                <super::functions::v1::SecurityType as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.SqlDataAccess`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PySqlDataAccess(pub super::functions::v1::SqlDataAccess);
impl PySqlDataAccess {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::functions::v1::SqlDataAccess as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("CONTAINS_SQL"),
            2i32 => ::core::option::Option::Some("READS_SQL_DATA"),
            3i32 => ::core::option::Option::Some("NO_SQL"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::functions::v1::SqlDataAccess> for PySqlDataAccess {
    fn from(value: super::functions::v1::SqlDataAccess) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PySqlDataAccess> for super::functions::v1::SqlDataAccess {
    fn from(value: PySqlDataAccess) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PySqlDataAccess {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "SqlDataAccess"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "CONTAINS_SQL" => 1i32,
            "READS_SQL_DATA" => 2i32,
            "NO_SQL" => 3i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid SqlDataAccess: {:?}; expected one of: CONTAINS_SQL, READS_SQL_DATA, NO_SQL",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::functions::v1::SqlDataAccess as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PySqlDataAccess {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "SqlDataAccess");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "SqlDataAccess")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "SqlDataAccess value {} has no Python member",
                <super::functions::v1::SqlDataAccess as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.ModelVersionStatus`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyModelVersionStatus(pub super::model_versions::v1::ModelVersionStatus);
impl PyModelVersionStatus {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::model_versions::v1::ModelVersionStatus as ::buffa::Enumeration>::to_i32(
            &self.0,
        ) {
            1i32 => ::core::option::Option::Some("PENDING_REGISTRATION"),
            2i32 => ::core::option::Option::Some("FAILED_REGISTRATION"),
            3i32 => ::core::option::Option::Some("READY"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::model_versions::v1::ModelVersionStatus> for PyModelVersionStatus {
    fn from(value: super::model_versions::v1::ModelVersionStatus) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyModelVersionStatus> for super::model_versions::v1::ModelVersionStatus {
    fn from(value: PyModelVersionStatus) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyModelVersionStatus {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "ModelVersionStatus"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "PENDING_REGISTRATION" => 1i32,
            "FAILED_REGISTRATION" => 2i32,
            "READY" => 3i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid ModelVersionStatus: {:?}; expected one of: PENDING_REGISTRATION, FAILED_REGISTRATION, READY",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::model_versions::v1::ModelVersionStatus as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyModelVersionStatus {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "ModelVersionStatus");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "ModelVersionStatus")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "ModelVersionStatus value {} has no Python member",
                <super::model_versions::v1::ModelVersionStatus as ::buffa::Enumeration>::to_i32(
                    &self.0
                )
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.PolicyType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyPolicyType(pub super::policies::v1::PolicyType);
impl PyPolicyType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::policies::v1::PolicyType as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("POLICY_TYPE_ROW_FILTER"),
            2i32 => ::core::option::Option::Some("POLICY_TYPE_COLUMN_MASK"),
            3i32 => ::core::option::Option::Some("POLICY_TYPE_DENY"),
            4i32 => ::core::option::Option::Some("POLICY_TYPE_GRANT"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::policies::v1::PolicyType> for PyPolicyType {
    fn from(value: super::policies::v1::PolicyType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyPolicyType> for super::policies::v1::PolicyType {
    fn from(value: PyPolicyType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyPolicyType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "PolicyType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "POLICY_TYPE_ROW_FILTER" => 1i32,
            "POLICY_TYPE_COLUMN_MASK" => 2i32,
            "POLICY_TYPE_DENY" => 3i32,
            "POLICY_TYPE_GRANT" => 4i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid PolicyType: {:?}; expected one of: POLICY_TYPE_ROW_FILTER, POLICY_TYPE_COLUMN_MASK, POLICY_TYPE_DENY, POLICY_TYPE_GRANT",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::policies::v1::PolicyType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyPolicyType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "PolicyType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "PolicyType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "PolicyType value {} has no Python member",
                <super::policies::v1::PolicyType as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.ProviderAuthenticationType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyProviderAuthenticationType(pub super::providers::v1::ProviderAuthenticationType);
impl PyProviderAuthenticationType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::providers::v1::ProviderAuthenticationType as ::buffa::Enumeration>::to_i32(
            &self.0,
        ) {
            1i32 => ::core::option::Option::Some("TOKEN"),
            2i32 => ::core::option::Option::Some("OAUTH_CLIENT_CREDENTIALS"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::providers::v1::ProviderAuthenticationType>
    for PyProviderAuthenticationType
{
    fn from(value: super::providers::v1::ProviderAuthenticationType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyProviderAuthenticationType>
    for super::providers::v1::ProviderAuthenticationType
{
    fn from(value: PyProviderAuthenticationType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyProviderAuthenticationType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!(
                "unitycatalog_client._enums",
                "ProviderAuthenticationType"
            ),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "TOKEN" => 1i32,
            "OAUTH_CLIENT_CREDENTIALS" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid ProviderAuthenticationType: {:?}; expected one of: TOKEN, OAUTH_CLIENT_CREDENTIALS",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::providers::v1::ProviderAuthenticationType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyProviderAuthenticationType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "ProviderAuthenticationType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(
            py,
            "unitycatalog_client._enums",
            "ProviderAuthenticationType",
        )?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "ProviderAuthenticationType value {} has no Python member",
                <super::providers::v1::ProviderAuthenticationType as ::buffa::Enumeration>::to_i32(
                    &self.0
                )
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.AuthenticationType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyAuthenticationType(pub super::recipients::v1::AuthenticationType);
impl PyAuthenticationType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::recipients::v1::AuthenticationType as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("TOKEN"),
            2i32 => ::core::option::Option::Some("OAUTH_CLIENT_CREDENTIALS"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::recipients::v1::AuthenticationType> for PyAuthenticationType {
    fn from(value: super::recipients::v1::AuthenticationType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAuthenticationType> for super::recipients::v1::AuthenticationType {
    fn from(value: PyAuthenticationType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyAuthenticationType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "AuthenticationType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "TOKEN" => 1i32,
            "OAUTH_CLIENT_CREDENTIALS" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid AuthenticationType: {:?}; expected one of: TOKEN, OAUTH_CLIENT_CREDENTIALS",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::recipients::v1::AuthenticationType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyAuthenticationType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "AuthenticationType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "AuthenticationType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "AuthenticationType value {} has no Python member",
                <super::recipients::v1::AuthenticationType as ::buffa::Enumeration>::to_i32(
                    &self.0
                )
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.Action`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyAction(pub super::shares::v1::Action);
impl PyAction {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::shares::v1::Action as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("ADD"),
            2i32 => ::core::option::Option::Some("REMOVE"),
            3i32 => ::core::option::Option::Some("UPDATE"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::shares::v1::Action> for PyAction {
    fn from(value: super::shares::v1::Action) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAction> for super::shares::v1::Action {
    fn from(value: PyAction) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyAction {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "Action"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "ADD" => 1i32,
            "REMOVE" => 2i32,
            "UPDATE" => 3i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid Action: {:?}; expected one of: ADD, REMOVE, UPDATE",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::shares::v1::Action as ::buffa::Enumeration>::from_i32(n).unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyAction {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "Action");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "Action")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "Action value {} has no Python member",
                <super::shares::v1::Action as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.DataObjectStatus`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyDataObjectStatus(pub super::shares::v1::DataObjectStatus);
impl PyDataObjectStatus {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::shares::v1::DataObjectStatus as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("ACTIVE"),
            2i32 => ::core::option::Option::Some("PERMISSION_DENIED"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::shares::v1::DataObjectStatus> for PyDataObjectStatus {
    fn from(value: super::shares::v1::DataObjectStatus) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyDataObjectStatus> for super::shares::v1::DataObjectStatus {
    fn from(value: PyDataObjectStatus) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyDataObjectStatus {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "DataObjectStatus"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "ACTIVE" => 1i32,
            "PERMISSION_DENIED" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid DataObjectStatus: {:?}; expected one of: ACTIVE, PERMISSION_DENIED",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::shares::v1::DataObjectStatus as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyDataObjectStatus {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "DataObjectStatus");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "DataObjectStatus")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "DataObjectStatus value {} has no Python member",
                <super::shares::v1::DataObjectStatus as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.DataObjectType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyDataObjectType(pub super::shares::v1::DataObjectType);
impl PyDataObjectType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::shares::v1::DataObjectType as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("TABLE"),
            2i32 => ::core::option::Option::Some("SCHEMA"),
            3i32 => ::core::option::Option::Some("VIEW"),
            4i32 => ::core::option::Option::Some("MATERIALIZED_VIEW"),
            5i32 => ::core::option::Option::Some("STREAMING_TABLE"),
            6i32 => ::core::option::Option::Some("MODEL"),
            8i32 => ::core::option::Option::Some("FUNCTION"),
            10i32 => ::core::option::Option::Some("VOLUME"),
            11i32 => ::core::option::Option::Some("AGENT_SKILL"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::shares::v1::DataObjectType> for PyDataObjectType {
    fn from(value: super::shares::v1::DataObjectType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyDataObjectType> for super::shares::v1::DataObjectType {
    fn from(value: PyDataObjectType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyDataObjectType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "DataObjectType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "TABLE" => 1i32,
            "SCHEMA" => 2i32,
            "VIEW" => 3i32,
            "MATERIALIZED_VIEW" => 4i32,
            "STREAMING_TABLE" => 5i32,
            "MODEL" => 6i32,
            "FUNCTION" => 8i32,
            "VOLUME" => 10i32,
            "AGENT_SKILL" => 11i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid DataObjectType: {:?}; expected one of: TABLE, SCHEMA, VIEW, MATERIALIZED_VIEW, STREAMING_TABLE, MODEL, FUNCTION, VOLUME, AGENT_SKILL",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::shares::v1::DataObjectType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyDataObjectType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "DataObjectType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "DataObjectType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "DataObjectType value {} has no Python member",
                <super::shares::v1::DataObjectType as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.HistoryStatus`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyHistoryStatus(pub super::shares::v1::HistoryStatus);
impl PyHistoryStatus {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::shares::v1::HistoryStatus as ::buffa::Enumeration>::to_i32(&self.0) {
            0i32 => ::core::option::Option::Some("DISABLED"),
            1i32 => ::core::option::Option::Some("ENABLED"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::shares::v1::HistoryStatus> for PyHistoryStatus {
    fn from(value: super::shares::v1::HistoryStatus) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyHistoryStatus> for super::shares::v1::HistoryStatus {
    fn from(value: PyHistoryStatus) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyHistoryStatus {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "HistoryStatus"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "DISABLED" => 0i32,
            "ENABLED" => 1i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid HistoryStatus: {:?}; expected one of: DISABLED, ENABLED",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::shares::v1::HistoryStatus as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyHistoryStatus {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "HistoryStatus");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "HistoryStatus")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "HistoryStatus value {} has no Python member",
                <super::shares::v1::HistoryStatus as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.PartitionValueOp`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyPartitionValueOp(pub super::shares::v1::PartitionValueOp);
impl PyPartitionValueOp {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::shares::v1::PartitionValueOp as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("EQUAL"),
            2i32 => ::core::option::Option::Some("LIKE"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::shares::v1::PartitionValueOp> for PyPartitionValueOp {
    fn from(value: super::shares::v1::PartitionValueOp) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyPartitionValueOp> for super::shares::v1::PartitionValueOp {
    fn from(value: PyPartitionValueOp) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyPartitionValueOp {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "PartitionValueOp"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "EQUAL" => 1i32,
            "LIKE" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid PartitionValueOp: {:?}; expected one of: EQUAL, LIKE",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::shares::v1::PartitionValueOp as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyPartitionValueOp {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "PartitionValueOp");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "PartitionValueOp")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "PartitionValueOp value {} has no Python member",
                <super::shares::v1::PartitionValueOp as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.ColumnTypeName`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyColumnTypeName(pub super::tables::v1::ColumnTypeName);
impl PyColumnTypeName {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::tables::v1::ColumnTypeName as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("BOOLEAN"),
            2i32 => ::core::option::Option::Some("BYTE"),
            3i32 => ::core::option::Option::Some("SHORT"),
            4i32 => ::core::option::Option::Some("INT"),
            5i32 => ::core::option::Option::Some("LONG"),
            6i32 => ::core::option::Option::Some("FLOAT"),
            7i32 => ::core::option::Option::Some("DOUBLE"),
            8i32 => ::core::option::Option::Some("DATE"),
            9i32 => ::core::option::Option::Some("TIMESTAMP"),
            10i32 => ::core::option::Option::Some("STRING"),
            11i32 => ::core::option::Option::Some("BINARY"),
            12i32 => ::core::option::Option::Some("DECIMAL"),
            13i32 => ::core::option::Option::Some("INTERVAL"),
            14i32 => ::core::option::Option::Some("ARRAY"),
            15i32 => ::core::option::Option::Some("STRUCT"),
            16i32 => ::core::option::Option::Some("MAP"),
            17i32 => ::core::option::Option::Some("CHAR"),
            18i32 => ::core::option::Option::Some("NULL"),
            19i32 => ::core::option::Option::Some("USER_DEFINED_TYPE"),
            20i32 => ::core::option::Option::Some("TIMESTAMP_NTZ"),
            21i32 => ::core::option::Option::Some("VARIANT"),
            22i32 => ::core::option::Option::Some("TABLE_TYPE"),
            23i32 => ::core::option::Option::Some("GEOMETRY"),
            24i32 => ::core::option::Option::Some("GEOGRAPHY"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::tables::v1::ColumnTypeName> for PyColumnTypeName {
    fn from(value: super::tables::v1::ColumnTypeName) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyColumnTypeName> for super::tables::v1::ColumnTypeName {
    fn from(value: PyColumnTypeName) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyColumnTypeName {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "ColumnTypeName"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "BOOLEAN" => 1i32,
            "BYTE" => 2i32,
            "SHORT" => 3i32,
            "INT" => 4i32,
            "LONG" => 5i32,
            "FLOAT" => 6i32,
            "DOUBLE" => 7i32,
            "DATE" => 8i32,
            "TIMESTAMP" => 9i32,
            "STRING" => 10i32,
            "BINARY" => 11i32,
            "DECIMAL" => 12i32,
            "INTERVAL" => 13i32,
            "ARRAY" => 14i32,
            "STRUCT" => 15i32,
            "MAP" => 16i32,
            "CHAR" => 17i32,
            "NULL" => 18i32,
            "USER_DEFINED_TYPE" => 19i32,
            "TIMESTAMP_NTZ" => 20i32,
            "VARIANT" => 21i32,
            "TABLE_TYPE" => 22i32,
            "GEOMETRY" => 23i32,
            "GEOGRAPHY" => 24i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid ColumnTypeName: {:?}; expected one of: BOOLEAN, BYTE, SHORT, INT, LONG, FLOAT, DOUBLE, DATE, TIMESTAMP, STRING, BINARY, DECIMAL, INTERVAL, ARRAY, STRUCT, MAP, CHAR, NULL, USER_DEFINED_TYPE, TIMESTAMP_NTZ, VARIANT, TABLE_TYPE, GEOMETRY, GEOGRAPHY",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::tables::v1::ColumnTypeName as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyColumnTypeName {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "ColumnTypeName");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "ColumnTypeName")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "ColumnTypeName value {} has no Python member",
                <super::tables::v1::ColumnTypeName as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.DataSourceFormat`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyDataSourceFormat(pub super::tables::v1::DataSourceFormat);
impl PyDataSourceFormat {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::tables::v1::DataSourceFormat as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("DELTA"),
            2i32 => ::core::option::Option::Some("ICEBERG"),
            3i32 => ::core::option::Option::Some("HUDI"),
            4i32 => ::core::option::Option::Some("PARQUET"),
            5i32 => ::core::option::Option::Some("CSV"),
            6i32 => ::core::option::Option::Some("JSON"),
            7i32 => ::core::option::Option::Some("ORC"),
            8i32 => ::core::option::Option::Some("AVRO"),
            9i32 => ::core::option::Option::Some("TEXT"),
            10i32 => ::core::option::Option::Some("UNITY_CATALOG"),
            11i32 => ::core::option::Option::Some("DELTASHARING"),
            12i32 => ::core::option::Option::Some("DELTA_UNIFORM_ICEBERG"),
            13i32 => ::core::option::Option::Some("DELTA_UNIFORM_HUDI"),
            14i32 => ::core::option::Option::Some("HIVE"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::tables::v1::DataSourceFormat> for PyDataSourceFormat {
    fn from(value: super::tables::v1::DataSourceFormat) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyDataSourceFormat> for super::tables::v1::DataSourceFormat {
    fn from(value: PyDataSourceFormat) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyDataSourceFormat {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "DataSourceFormat"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "DELTA" => 1i32,
            "ICEBERG" => 2i32,
            "HUDI" => 3i32,
            "PARQUET" => 4i32,
            "CSV" => 5i32,
            "JSON" => 6i32,
            "ORC" => 7i32,
            "AVRO" => 8i32,
            "TEXT" => 9i32,
            "UNITY_CATALOG" => 10i32,
            "DELTASHARING" => 11i32,
            "DELTA_UNIFORM_ICEBERG" => 12i32,
            "DELTA_UNIFORM_HUDI" => 13i32,
            "HIVE" => 14i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid DataSourceFormat: {:?}; expected one of: DELTA, ICEBERG, HUDI, PARQUET, CSV, JSON, ORC, AVRO, TEXT, UNITY_CATALOG, DELTASHARING, DELTA_UNIFORM_ICEBERG, DELTA_UNIFORM_HUDI, HIVE",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::tables::v1::DataSourceFormat as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyDataSourceFormat {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "DataSourceFormat");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "DataSourceFormat")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "DataSourceFormat value {} has no Python member",
                <super::tables::v1::DataSourceFormat as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.TableType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyTableType(pub super::tables::v1::TableType);
impl PyTableType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::tables::v1::TableType as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("MANAGED"),
            2i32 => ::core::option::Option::Some("EXTERNAL"),
            3i32 => ::core::option::Option::Some("VIEW"),
            4i32 => ::core::option::Option::Some("MATERIALIZED_VIEW"),
            5i32 => ::core::option::Option::Some("STREAMING_TABLE"),
            6i32 => ::core::option::Option::Some("MANAGED_SHALLOW_CLONE"),
            8i32 => ::core::option::Option::Some("EXTERNAL_SHALLOW_CLONE"),
            9i32 => ::core::option::Option::Some("METRIC_VIEW"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::tables::v1::TableType> for PyTableType {
    fn from(value: super::tables::v1::TableType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyTableType> for super::tables::v1::TableType {
    fn from(value: PyTableType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyTableType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "TableType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "MANAGED" => 1i32,
            "EXTERNAL" => 2i32,
            "VIEW" => 3i32,
            "MATERIALIZED_VIEW" => 4i32,
            "STREAMING_TABLE" => 5i32,
            "MANAGED_SHALLOW_CLONE" => 6i32,
            "EXTERNAL_SHALLOW_CLONE" => 8i32,
            "METRIC_VIEW" => 9i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid TableType: {:?}; expected one of: MANAGED, EXTERNAL, VIEW, MATERIALIZED_VIEW, STREAMING_TABLE, MANAGED_SHALLOW_CLONE, EXTERNAL_SHALLOW_CLONE, METRIC_VIEW",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::tables::v1::TableType as ::buffa::Enumeration>::from_i32(n).unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyTableType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "TableType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "TableType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "TableType value {} has no Python member",
                <super::tables::v1::TableType as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.TagAssignmentSourceType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyTagAssignmentSourceType(pub super::tags::v1::TagAssignmentSourceType);
impl PyTagAssignmentSourceType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::tags::v1::TagAssignmentSourceType as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some(
                "TAG_ASSIGNMENT_SOURCE_TYPE_SYSTEM_DATA_CLASSIFICATION",
            ),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::tags::v1::TagAssignmentSourceType> for PyTagAssignmentSourceType {
    fn from(value: super::tags::v1::TagAssignmentSourceType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyTagAssignmentSourceType> for super::tags::v1::TagAssignmentSourceType {
    fn from(value: PyTagAssignmentSourceType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyTagAssignmentSourceType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "TagAssignmentSourceType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "TAG_ASSIGNMENT_SOURCE_TYPE_SYSTEM_DATA_CLASSIFICATION" => 1i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid TagAssignmentSourceType: {:?}; expected one of: TAG_ASSIGNMENT_SOURCE_TYPE_SYSTEM_DATA_CLASSIFICATION",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::tags::v1::TagAssignmentSourceType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyTagAssignmentSourceType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "TagAssignmentSourceType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "TagAssignmentSourceType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "TagAssignmentSourceType value {} has no Python member",
                <super::tags::v1::TagAssignmentSourceType as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.GenerateTemporaryModelVersionCredentialsRequestOperation`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyGenerateTemporaryModelVersionCredentialsRequestOperation(
    pub super::temporary_credentials::v1::generate_temporary_model_version_credentials_request::Operation,
);
impl PyGenerateTemporaryModelVersionCredentialsRequestOperation {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::temporary_credentials::v1::generate_temporary_model_version_credentials_request::Operation as ::buffa::Enumeration>::to_i32(
            &self.0,
        ) {
            0i32 => ::core::option::Option::Some("UNSPECIFIED"),
            1i32 => ::core::option::Option::Some("READ_MODEL_VERSION"),
            2i32 => ::core::option::Option::Some("READ_WRITE_MODEL_VERSION"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<
    super::temporary_credentials::v1::generate_temporary_model_version_credentials_request::Operation,
> for PyGenerateTemporaryModelVersionCredentialsRequestOperation {
    fn from(
        value: super::temporary_credentials::v1::generate_temporary_model_version_credentials_request::Operation,
    ) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyGenerateTemporaryModelVersionCredentialsRequestOperation>
for super::temporary_credentials::v1::generate_temporary_model_version_credentials_request::Operation {
    fn from(value: PyGenerateTemporaryModelVersionCredentialsRequestOperation) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py>
    for PyGenerateTemporaryModelVersionCredentialsRequestOperation
{
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!(
                "unitycatalog_client._enums",
                "GenerateTemporaryModelVersionCredentialsRequestOperation"
            ),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "UNSPECIFIED" => 0i32,
            "READ_MODEL_VERSION" => 1i32,
            "READ_WRITE_MODEL_VERSION" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid GenerateTemporaryModelVersionCredentialsRequestOperation: {:?}; expected one of: UNSPECIFIED, READ_MODEL_VERSION, READ_WRITE_MODEL_VERSION",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(
            Self(
                <super::temporary_credentials::v1::generate_temporary_model_version_credentials_request::Operation as ::buffa::Enumeration>::from_i32(
                        n,
                    )
                    .unwrap_or_default(),
            ),
        )
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyGenerateTemporaryModelVersionCredentialsRequestOperation {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr = ::pyo3::type_hint_identifier!(
        "unitycatalog_client._enums",
        "GenerateTemporaryModelVersionCredentialsRequestOperation"
    );
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(
            py,
            "unitycatalog_client._enums",
            "GenerateTemporaryModelVersionCredentialsRequestOperation",
        )?;
        let name = self
            .__name()
            .ok_or_else(|| {
                ::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "GenerateTemporaryModelVersionCredentialsRequestOperation value {} has no Python member",
                        <
                        super::temporary_credentials::v1::generate_temporary_model_version_credentials_request::Operation
                        as ::buffa::Enumeration > ::to_i32(& self.0)
                    ),
                )
            })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.GenerateTemporaryPathCredentialsRequestOperation`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyGenerateTemporaryPathCredentialsRequestOperation(
    pub super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation,
);
impl PyGenerateTemporaryPathCredentialsRequestOperation {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation as ::buffa::Enumeration>::to_i32(
            &self.0,
        ) {
            0i32 => ::core::option::Option::Some("UNSPECIFIED"),
            1i32 => ::core::option::Option::Some("PATH_READ"),
            2i32 => ::core::option::Option::Some("PATH_READ_WRITE"),
            3i32 => ::core::option::Option::Some("PATH_CREATE_TABLE"),
            _ => ::core::option::Option::None,
        }
    }
}

impl
    ::core::convert::From<
        super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation,
    > for PyGenerateTemporaryPathCredentialsRequestOperation
{
    fn from(
        value: super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation,
    ) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyGenerateTemporaryPathCredentialsRequestOperation>
    for super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation
{
    fn from(value: PyGenerateTemporaryPathCredentialsRequestOperation) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyGenerateTemporaryPathCredentialsRequestOperation {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!(
                "unitycatalog_client._enums",
                "GenerateTemporaryPathCredentialsRequestOperation"
            ),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "UNSPECIFIED" => 0i32,
            "PATH_READ" => 1i32,
            "PATH_READ_WRITE" => 2i32,
            "PATH_CREATE_TABLE" => 3i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid GenerateTemporaryPathCredentialsRequestOperation: {:?}; expected one of: UNSPECIFIED, PATH_READ, PATH_READ_WRITE, PATH_CREATE_TABLE",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(
            Self(
                <super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation as ::buffa::Enumeration>::from_i32(
                        n,
                    )
                    .unwrap_or_default(),
            ),
        )
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyGenerateTemporaryPathCredentialsRequestOperation {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr = ::pyo3::type_hint_identifier!(
        "unitycatalog_client._enums",
        "GenerateTemporaryPathCredentialsRequestOperation"
    );
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(
            py,
            "unitycatalog_client._enums",
            "GenerateTemporaryPathCredentialsRequestOperation",
        )?;
        let name = self
            .__name()
            .ok_or_else(|| {
                ::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "GenerateTemporaryPathCredentialsRequestOperation value {} has no Python member",
                        <
                        super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation
                        as ::buffa::Enumeration > ::to_i32(& self.0)
                    ),
                )
            })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.GenerateTemporaryTableCredentialsRequestOperation`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyGenerateTemporaryTableCredentialsRequestOperation(
    pub super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation,
);
impl PyGenerateTemporaryTableCredentialsRequestOperation {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation as ::buffa::Enumeration>::to_i32(
            &self.0,
        ) {
            0i32 => ::core::option::Option::Some("UNSPECIFIED"),
            1i32 => ::core::option::Option::Some("READ"),
            2i32 => ::core::option::Option::Some("READ_WRITE"),
            _ => ::core::option::Option::None,
        }
    }
}

impl
    ::core::convert::From<
        super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation,
    > for PyGenerateTemporaryTableCredentialsRequestOperation
{
    fn from(
        value: super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation,
    ) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyGenerateTemporaryTableCredentialsRequestOperation>
    for super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation
{
    fn from(value: PyGenerateTemporaryTableCredentialsRequestOperation) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py>
    for PyGenerateTemporaryTableCredentialsRequestOperation
{
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!(
                "unitycatalog_client._enums",
                "GenerateTemporaryTableCredentialsRequestOperation"
            ),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "UNSPECIFIED" => 0i32,
            "READ" => 1i32,
            "READ_WRITE" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid GenerateTemporaryTableCredentialsRequestOperation: {:?}; expected one of: UNSPECIFIED, READ, READ_WRITE",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(
            Self(
                <super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation as ::buffa::Enumeration>::from_i32(
                        n,
                    )
                    .unwrap_or_default(),
            ),
        )
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyGenerateTemporaryTableCredentialsRequestOperation {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr = ::pyo3::type_hint_identifier!(
        "unitycatalog_client._enums",
        "GenerateTemporaryTableCredentialsRequestOperation"
    );
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(
            py,
            "unitycatalog_client._enums",
            "GenerateTemporaryTableCredentialsRequestOperation",
        )?;
        let name = self
            .__name()
            .ok_or_else(|| {
                ::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "GenerateTemporaryTableCredentialsRequestOperation value {} has no Python member",
                        <
                        super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation
                        as ::buffa::Enumeration > ::to_i32(& self.0)
                    ),
                )
            })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.GenerateTemporaryVolumeCredentialsRequestOperation`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyGenerateTemporaryVolumeCredentialsRequestOperation(
    pub super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation,
);
impl PyGenerateTemporaryVolumeCredentialsRequestOperation {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation as ::buffa::Enumeration>::to_i32(
            &self.0,
        ) {
            0i32 => ::core::option::Option::Some("UNSPECIFIED"),
            1i32 => ::core::option::Option::Some("READ_VOLUME"),
            2i32 => ::core::option::Option::Some("WRITE_VOLUME"),
            _ => ::core::option::Option::None,
        }
    }
}

impl
    ::core::convert::From<
        super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation,
    > for PyGenerateTemporaryVolumeCredentialsRequestOperation
{
    fn from(
        value: super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation,
    ) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyGenerateTemporaryVolumeCredentialsRequestOperation>
    for super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation
{
    fn from(value: PyGenerateTemporaryVolumeCredentialsRequestOperation) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py>
    for PyGenerateTemporaryVolumeCredentialsRequestOperation
{
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!(
                "unitycatalog_client._enums",
                "GenerateTemporaryVolumeCredentialsRequestOperation"
            ),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "UNSPECIFIED" => 0i32,
            "READ_VOLUME" => 1i32,
            "WRITE_VOLUME" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid GenerateTemporaryVolumeCredentialsRequestOperation: {:?}; expected one of: UNSPECIFIED, READ_VOLUME, WRITE_VOLUME",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(
            Self(
                <super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation as ::buffa::Enumeration>::from_i32(
                        n,
                    )
                    .unwrap_or_default(),
            ),
        )
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyGenerateTemporaryVolumeCredentialsRequestOperation {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr = ::pyo3::type_hint_identifier!(
        "unitycatalog_client._enums",
        "GenerateTemporaryVolumeCredentialsRequestOperation"
    );
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(
            py,
            "unitycatalog_client._enums",
            "GenerateTemporaryVolumeCredentialsRequestOperation",
        )?;
        let name = self
            .__name()
            .ok_or_else(|| {
                ::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "GenerateTemporaryVolumeCredentialsRequestOperation value {} has no Python member",
                        <
                        super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation
                        as ::buffa::Enumeration > ::to_i32(& self.0)
                    ),
                )
            })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// A member of `unitycatalog_client._enums.VolumeType`, or its name.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PyVolumeType(pub super::volumes::v1::VolumeType);
impl PyVolumeType {
    /// The name of the value, or `None` for a value without a Python member.
    fn __name(&self) -> ::core::option::Option<&'static str> {
        match <super::volumes::v1::VolumeType as ::buffa::Enumeration>::to_i32(&self.0) {
            1i32 => ::core::option::Option::Some("EXTERNAL"),
            2i32 => ::core::option::Option::Some("MANAGED"),
            _ => ::core::option::Option::None,
        }
    }
}

impl ::core::convert::From<super::volumes::v1::VolumeType> for PyVolumeType {
    fn from(value: super::volumes::v1::VolumeType) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyVolumeType> for super::volumes::v1::VolumeType {
    fn from(value: PyVolumeType) -> Self {
        value.0
    }
}

impl<'a, 'py> ::pyo3::FromPyObject<'a, 'py> for PyVolumeType {
    type Error = ::pyo3::PyErr;
    const INPUT_TYPE: ::pyo3::inspect::PyStaticExpr = {
        use ::pyo3::type_hint_union;
        type_hint_union!(
            ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "VolumeType"),
            ::pyo3::type_hint_identifier!("builtins", "str")
        )
    };

    fn extract(obj: ::pyo3::Borrowed<'a, 'py, ::pyo3::PyAny>) -> ::pyo3::PyResult<Self> {
        let name: ::pyo3::pybacked::PyBackedStr = ::pyo3::types::PyAnyMethods::extract(&*obj)?;
        let n: i32 = match &*name {
            "EXTERNAL" => 1i32,
            "MANAGED" => 2i32,
            other => {
                return ::core::result::Result::Err(::pyo3::exceptions::PyValueError::new_err(
                    ::std::format!(
                        "invalid VolumeType: {:?}; expected one of: EXTERNAL, MANAGED",
                        other
                    ),
                ));
            }
        };
        ::core::result::Result::Ok(Self(
            <super::volumes::v1::VolumeType as ::buffa::Enumeration>::from_i32(n)
                .unwrap_or_default(),
        ))
    }
}

impl<'py> ::pyo3::IntoPyObject<'py> for PyVolumeType {
    type Target = ::pyo3::PyAny;
    type Output = ::pyo3::Bound<'py, ::pyo3::PyAny>;
    type Error = ::pyo3::PyErr;
    const OUTPUT_TYPE: ::pyo3::inspect::PyStaticExpr =
        ::pyo3::type_hint_identifier!("unitycatalog_client._enums", "VolumeType");
    fn into_pyobject(self, py: ::pyo3::Python<'py>) -> ::pyo3::PyResult<Self::Output> {
        static CLASS: ::pyo3::sync::PyOnceLock<::pyo3::Py<::pyo3::types::PyType>> =
            ::pyo3::sync::PyOnceLock::new();
        let class = CLASS.import(py, "unitycatalog_client._enums", "VolumeType")?;
        let name = self.__name().ok_or_else(|| {
            ::pyo3::exceptions::PyValueError::new_err(::std::format!(
                "VolumeType value {} has no Python member",
                <super::volumes::v1::VolumeType as ::buffa::Enumeration>::to_i32(&self.0)
            ))
        })?;
        ::pyo3::types::PyAnyMethods::getattr(class.as_any(), name)
    }
}

/// An agent skill registered in Unity Catalog.
#[::pyo3::pyclass(
    name = "AgentSkill",
    eq,
    frozen,
    module = "unitycatalog_client._client.agent_skills",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAgentSkill(pub super::agent_skills::v0alpha1::AgentSkill);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAgentSkill {
    /// Name of the agent skill, relative to parent schema.
    ///
    /// Per the Agent Skills spec a skill name is lowercase letters, numbers, and
    /// hyphens, but Unity Catalog object names disallow hyphens, so UC stores the
    /// catalog object name with underscores; the spec-shaped name lives in the
    /// skill's own SKILL.md.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Name of parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Name of parent schema.
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }

    /// The three-level (fully qualified) name of the agent skill.
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }

    /// The storage location of the skill's directory (the root that holds
    /// `SKILL.md`).
    #[getter]
    fn storage_location(&self) -> ::std::string::String {
        self.0.storage_location.clone()
    }

    /// The unique identifier of the agent skill.
    #[getter]
    fn agent_skill_id(&self) -> ::std::string::String {
        self.0.agent_skill_id.clone()
    }

    /// How the storage location is provisioned (external or managed).
    #[getter]
    fn agent_skill_type(&self) -> ::core::option::Option<PyAgentSkillType> {
        self.0
            .agent_skill_type
            .as_known()
            .filter(|e| {
                <super::agent_skills::v0alpha1::AgentSkillType as ::buffa::Enumeration>::to_i32(e)
                    != 0
            })
            .map(PyAgentSkillType::from)
    }

    /// A human-readable description of what the skill does and when to use it.
    ///
    /// Mirrors the Agent Skills spec `description` frontmatter field.
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }

    /// SPDX license identifier or free-form license text for the skill.
    #[getter]
    fn license(&self) -> ::core::option::Option<::std::string::String> {
        self.0.license.clone()
    }

    /// The tools the skill is permitted to use, as declared in its SKILL.md.
    #[getter]
    fn allowed_tools(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.allowed_tools.clone()
    }

    /// Arbitrary additional metadata declared by the skill.
    #[getter]
    fn metadata(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.metadata.clone()
    }

    /// Username of current owner of the agent skill.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Time at which this agent skill was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of agent skill creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this agent skill was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified the agent skill.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// The unique identifier of the metastore.
    #[getter]
    fn metastore_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.metastore_id.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("catalog_name", false),
                ("schema_name", false),
                ("full_name", false),
                ("storage_location", false),
                ("agent_skill_id", false),
                ("agent_skill_type", false),
                ("description", false),
                ("license", false),
                ("allowed_tools", false),
                ("metadata", false),
                ("owner", false),
                ("comment", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("metastore_id", false),
            ],
        )
    }
}

impl ::core::convert::From<super::agent_skills::v0alpha1::AgentSkill> for PyAgentSkill {
    fn from(value: super::agent_skills::v0alpha1::AgentSkill) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAgentSkill> for super::agent_skills::v0alpha1::AgentSkill {
    fn from(value: PyAgentSkill) -> Self {
        value.0
    }
}

/// List agent skills response.
#[::pyo3::pyclass(
    name = "ListAgentSkillsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.agent_skills",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListAgentSkillsResponse(pub super::agent_skills::v0alpha1::ListAgentSkillsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListAgentSkillsResponse {
    /// The agent skills returned.
    #[getter]
    fn agent_skills(&self) -> ::std::vec::Vec<PyAgentSkill> {
        self.0
            .agent_skills
            .iter()
            .cloned()
            .map(PyAgentSkill::from)
            .collect()
    }

    /// Opaque token to retrieve the next page of results.
    ///
    /// Absent if there are no more pages. page_token should be set to
    /// this value for the next request to retrieve the next page of results.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("agent_skills", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::agent_skills::v0alpha1::ListAgentSkillsResponse>
    for PyListAgentSkillsResponse
{
    fn from(value: super::agent_skills::v0alpha1::ListAgentSkillsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListAgentSkillsResponse>
    for super::agent_skills::v0alpha1::ListAgentSkillsResponse
{
    fn from(value: PyListAgentSkillsResponse) -> Self {
        value.0
    }
}

/// An agent registered in Unity Catalog.
#[::pyo3::pyclass(
    name = "Agent",
    eq,
    frozen,
    module = "unitycatalog_client._client.agents",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAgent(pub super::agents::v0alpha1::Agent);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAgent {
    /// Name of the agent, relative to parent schema.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Name of parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Name of parent schema.
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }

    /// The three-level (fully qualified) name of the agent.
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }

    /// The unique identifier of the agent.
    #[getter]
    fn agent_id(&self) -> ::std::string::String {
        self.0.agent_id.clone()
    }

    /// The protocol a recipient uses to invoke the agent.
    #[getter]
    fn invocation_protocol(&self) -> ::core::option::Option<PyInvocationProtocol> {
        self.0
            .invocation_protocol
            .as_known()
            .filter(|e| {
                <super::agents::v0alpha1::InvocationProtocol as ::buffa::Enumeration>::to_i32(e)
                    != 0
            })
            .map(PyInvocationProtocol::from)
    }

    /// The agent's invocation endpoint URL.
    #[getter]
    fn endpoint(&self) -> ::std::string::String {
        self.0.endpoint.clone()
    }

    /// An LLM-readable description of what the agent does and the inputs it
    /// expects.
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }

    /// Capability identifiers advertised by the agent (e.g. `sql_query`,
    /// `document_search`, `code_execution`).
    #[getter]
    fn capabilities(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.capabilities.clone()
    }

    /// A JSON Schema (encoded as a JSON string) describing the agent's expected
    /// input payload. Kept as an opaque string so the schema can evolve without
    /// changing this message.
    #[getter]
    fn input_schema(&self) -> ::core::option::Option<::std::string::String> {
        self.0.input_schema.clone()
    }

    /// Username of current owner of the agent.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Time at which this agent was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of agent creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this agent was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified the agent.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// The unique identifier of the metastore.
    #[getter]
    fn metastore_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.metastore_id.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("catalog_name", false),
                ("schema_name", false),
                ("full_name", false),
                ("agent_id", false),
                ("invocation_protocol", false),
                ("endpoint", false),
                ("description", false),
                ("capabilities", false),
                ("input_schema", false),
                ("owner", false),
                ("comment", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("metastore_id", false),
            ],
        )
    }
}

impl ::core::convert::From<super::agents::v0alpha1::Agent> for PyAgent {
    fn from(value: super::agents::v0alpha1::Agent) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAgent> for super::agents::v0alpha1::Agent {
    fn from(value: PyAgent) -> Self {
        value.0
    }
}

/// List agents response.
#[::pyo3::pyclass(
    name = "ListAgentsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.agents",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListAgentsResponse(pub super::agents::v0alpha1::ListAgentsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListAgentsResponse {
    /// The agents returned.
    #[getter]
    fn agents(&self) -> ::std::vec::Vec<PyAgent> {
        self.0.agents.iter().cloned().map(PyAgent::from).collect()
    }

    /// Opaque token to retrieve the next page of results.
    ///
    /// Absent if there are no more pages. page_token should be set to
    /// this value for the next request to retrieve the next page of results.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("agents", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::agents::v0alpha1::ListAgentsResponse> for PyListAgentsResponse {
    fn from(value: super::agents::v0alpha1::ListAgentsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListAgentsResponse> for super::agents::v0alpha1::ListAgentsResponse {
    fn from(value: PyListAgentsResponse) -> Self {
        value.0
    }
}

/// A catalog is a root-level namespace that contains schemas.
#[::pyo3::pyclass(
    name = "Catalog",
    eq,
    frozen,
    module = "unitycatalog_client._client.catalogs",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyCatalog(pub super::catalogs::v1::Catalog);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCatalog {
    /// Name of catalog.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Unique identifier for the catalog.
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }

    /// Username of current owner of catalog.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// A map of key-value properties attached to the securable.
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }

    /// Storage root URL for managed tables within catalog.
    #[getter]
    fn storage_root(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_root.clone()
    }

    /// The name of delta sharing provider.
    ///
    /// A Delta Sharing catalog is a catalog that is based on a Delta share on a remote sharing server.
    #[getter]
    fn provider_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.provider_name.clone()
    }

    /// The name of the share under the share provider.
    #[getter]
    fn share_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.share_name.clone()
    }

    /// The type of the catalog.
    #[getter]
    fn catalog_type(&self) -> ::core::option::Option<PyCatalogType> {
        self.0
            .catalog_type
            .as_ref()
            .and_then(|e| e.as_known())
            .filter(|e| <super::catalogs::v1::CatalogType as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyCatalogType::from)
    }

    /// Storage location URL (full path) for managed storage of the catalog.
    ///
    /// A unique path under `storage_root`, used as the managed-storage parent for
    /// managed tables/volumes whose schema has no storage location of its own.
    /// Absent when the catalog has no `storage_root`.
    /// Example: `s3://bucket/ucroot/__unitystorage/catalogs/{catalog_id}`.
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }

    /// Time at which this catalog was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of catalog creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this catalog was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified catalog.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// Indicates whether the principal is limited to retrieving metadata
    /// for the associated object through the BROWSE privilege when include_browse
    /// is enabled in the request.
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("id", false),
                ("owner", false),
                ("comment", false),
                ("properties", false),
                ("storage_root", false),
                ("provider_name", false),
                ("share_name", false),
                ("catalog_type", false),
                ("storage_location", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("browse_only", false),
            ],
        )
    }
}

impl ::core::convert::From<super::catalogs::v1::Catalog> for PyCatalog {
    fn from(value: super::catalogs::v1::Catalog) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyCatalog> for super::catalogs::v1::Catalog {
    fn from(value: PyCatalog) -> Self {
        value.0
    }
}

/// List catalogs response.
#[::pyo3::pyclass(
    name = "ListCatalogsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.catalogs",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListCatalogsResponse(pub super::catalogs::v1::ListCatalogsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListCatalogsResponse {
    /// The catalogs returned.
    #[getter]
    fn catalogs(&self) -> ::std::vec::Vec<PyCatalog> {
        self.0
            .catalogs
            .iter()
            .cloned()
            .map(PyCatalog::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("catalogs", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::catalogs::v1::ListCatalogsResponse> for PyListCatalogsResponse {
    fn from(value: super::catalogs::v1::ListCatalogsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListCatalogsResponse> for super::catalogs::v1::ListCatalogsResponse {
    fn from(value: PyListCatalogsResponse) -> Self {
        value.0
    }
}

/// The AWS IAM role configuration used server-side to call STS AssumeRole.
///
/// This is an internal configuration type, not exposed in the public API response.
/// It stores the static credentials (or ambient credential fallback) used to
/// authorize the STS:AssumeRole call when vending temporary credentials.
#[::pyo3::pyclass(
    name = "AwsIamRoleConfig",
    eq,
    frozen,
    module = "unitycatalog_client._client.credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAwsIamRoleConfig(pub super::credentials::v1::AwsIamRoleConfig);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAwsIamRoleConfig {
    #[new]
    #[pyo3(
        signature = (
            role_arn = None,
            region = None,
            access_key_id = None,
            secret_access_key = None,
            session_token = None
        )
    )]
    fn new(
        role_arn: ::core::option::Option<::std::string::String>,
        region: ::core::option::Option<::std::string::String>,
        access_key_id: ::core::option::Option<::std::string::String>,
        secret_access_key: ::core::option::Option<::std::string::String>,
        session_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::credentials::v1::AwsIamRoleConfig as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = role_arn {
            inner.role_arn = value;
        }
        {
            let value = region;
            inner.region = value;
        }
        {
            let value = access_key_id;
            inner.access_key_id = value;
        }
        {
            let value = secret_access_key;
            inner.secret_access_key = value;
        }
        {
            let value = session_token;
            inner.session_token = value;
        }
        Self(inner)
    }

    /// The Amazon Resource Name (ARN) of the AWS IAM role used to vend temporary credentials.
    #[getter]
    fn role_arn(&self) -> ::std::string::String {
        self.0.role_arn.clone()
    }

    /// The AWS region where STS calls are made. Defaults to "us-east-1" if absent.
    #[getter]
    fn region(&self) -> ::core::option::Option<::std::string::String> {
        self.0.region.clone()
    }

    /// AWS access key ID used to authorize the STS AssumeRole call.
    ///
    /// When set together with secret_access_key, these static credentials are used
    /// as the base identity to call STS:AssumeRole. When omitted, the server falls
    /// back to ambient credentials (instance profile, ECS task role, WebIdentity, etc.).
    #[getter]
    fn access_key_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.access_key_id.clone()
    }

    /// AWS secret access key paired with access_key_id.
    #[getter]
    fn secret_access_key(&self) -> ::core::option::Option<::std::string::String> {
        self.0.secret_access_key.clone()
    }

    /// Optional AWS session token for temporary base credentials (e.g. when the
    /// caller itself holds STS-vended credentials as the base identity).
    #[getter]
    fn session_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.session_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("role_arn", false),
                ("region", false),
                ("access_key_id", false),
                ("secret_access_key", true),
                ("session_token", true),
            ],
        )
    }
}

impl ::core::convert::From<super::credentials::v1::AwsIamRoleConfig> for PyAwsIamRoleConfig {
    fn from(value: super::credentials::v1::AwsIamRoleConfig) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAwsIamRoleConfig> for super::credentials::v1::AwsIamRoleConfig {
    fn from(value: PyAwsIamRoleConfig) -> Self {
        value.0
    }
}

/// The Azure managed identity configuration.
#[::pyo3::pyclass(
    name = "AzureManagedIdentity",
    eq,
    frozen,
    module = "unitycatalog_client._client.credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAzureManagedIdentity(pub super::credentials::v1::AzureManagedIdentity);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAzureManagedIdentity {
    #[new]
    #[pyo3(
        signature = (
            access_connector_id = None,
            credential_id = None,
            managed_identity_id = None
        )
    )]
    fn new(
        access_connector_id: ::core::option::Option<::std::string::String>,
        credential_id: ::core::option::Option<::std::string::String>,
        managed_identity_id: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::credentials::v1::AzureManagedIdentity as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = access_connector_id {
            inner.access_connector_id = value;
        }
        {
            let value = credential_id;
            inner.credential_id = value;
        }
        {
            let value = managed_identity_id;
            inner.managed_identity_id = value;
        }
        Self(inner)
    }

    /// The Azure resource ID of the Azure Databricks Access Connector.
    /// Use the format /subscriptions/{guid}/resourceGroups/{rg-name}/providers/
    /// Microsoft.Databricks/accessConnectors/{connector-name}.
    #[getter]
    fn access_connector_id(&self) -> ::std::string::String {
        self.0.access_connector_id.clone()
    }

    /// The Databricks internal ID that represents this managed identity.
    #[getter]
    fn credential_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.credential_id.clone()
    }

    /// The Azure resource ID of the managed identity.
    /// Use the format /subscriptions/{guid}/resourceGroups/{rg-name}/providers/
    /// Microsoft.ManagedIdentity/userAssignedIdentities/{identity-name}.
    /// This is only available for user-assigned identities. For system-assigned
    /// identities, the access_connector_id is used to identify the identity.
    /// If this field is not provided, then the system-assigned identity of the
    /// Access Connector is used.
    #[getter]
    fn managed_identity_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.managed_identity_id.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("access_connector_id", false),
                ("credential_id", false),
                ("managed_identity_id", false),
            ],
        )
    }
}

impl ::core::convert::From<super::credentials::v1::AzureManagedIdentity>
    for PyAzureManagedIdentity
{
    fn from(value: super::credentials::v1::AzureManagedIdentity) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAzureManagedIdentity>
    for super::credentials::v1::AzureManagedIdentity
{
    fn from(value: PyAzureManagedIdentity) -> Self {
        value.0
    }
}

/// The Azure service principal configuration.
#[::pyo3::pyclass(
    name = "AzureServicePrincipal",
    eq,
    frozen,
    module = "unitycatalog_client._client.credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAzureServicePrincipal(pub super::credentials::v1::AzureServicePrincipal);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAzureServicePrincipal {
    #[new]
    #[pyo3(
        signature = (
            directory_id = None,
            application_id = None,
            client_secret = None,
            federated_token_file = None
        )
    )]
    fn new(
        directory_id: ::core::option::Option<::std::string::String>,
        application_id: ::core::option::Option<::std::string::String>,
        client_secret: ::core::option::Option<::std::string::String>,
        federated_token_file: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::credentials::v1::AzureServicePrincipal as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = directory_id {
            inner.directory_id = value;
        }
        if let ::core::option::Option::Some(value) = application_id {
            inner.application_id = value;
        }
        if let ::core::option::Option::Some(value) = client_secret {
            inner.credential = ::core::option::Option::Some(
                super::credentials::v1::azure_service_principal::Credential::ClientSecret(value),
            );
        }
        if let ::core::option::Option::Some(value) = federated_token_file {
            inner.credential = ::core::option::Option::Some(
                super::credentials::v1::azure_service_principal::Credential::FederatedTokenFile(
                    value,
                ),
            );
        }
        Self(inner)
    }

    /// The directory ID corresponding to the Azure Active Directory (AAD) tenant of the application.
    #[getter]
    fn directory_id(&self) -> ::std::string::String {
        self.0.directory_id.clone()
    }

    /// The application ID of the application registration within the referenced AAD tenant.
    #[getter]
    fn application_id(&self) -> ::std::string::String {
        self.0.application_id.clone()
    }

    ///The client secret generated for the above app ID in AAD.
    #[getter]
    fn client_secret(&self) -> ::core::option::Option<::std::string::String> {
        match &self.0.credential {
            ::core::option::Option::Some(
                super::credentials::v1::azure_service_principal::Credential::ClientSecret(value),
            ) => ::core::option::Option::Some(::core::clone::Clone::clone(value)),
            _ => ::core::option::Option::None,
        }
    }

    ///Location of the file containing a federated token, for workload identity federation.
    #[getter]
    fn federated_token_file(&self) -> ::core::option::Option<::std::string::String> {
        match &self.0.credential {
            ::core::option::Option::Some(
                super::credentials::v1::azure_service_principal::Credential::FederatedTokenFile(
                    value,
                ),
            ) => ::core::option::Option::Some(::core::clone::Clone::clone(value)),
            _ => ::core::option::Option::None,
        }
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("directory_id", false),
                ("application_id", false),
                ("client_secret", false),
                ("federated_token_file", false),
            ],
        )
    }
}

impl ::core::convert::From<super::credentials::v1::AzureServicePrincipal>
    for PyAzureServicePrincipal
{
    fn from(value: super::credentials::v1::AzureServicePrincipal) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAzureServicePrincipal>
    for super::credentials::v1::AzureServicePrincipal
{
    fn from(value: PyAzureServicePrincipal) -> Self {
        value.0
    }
}

/// The Azure storage key configuration.
#[::pyo3::pyclass(
    name = "AzureStorageKey",
    eq,
    frozen,
    module = "unitycatalog_client._client.credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAzureStorageKey(pub super::credentials::v1::AzureStorageKey);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAzureStorageKey {
    #[new]
    #[pyo3(signature = (account_name = None, account_key = None))]
    fn new(
        account_name: ::core::option::Option<::std::string::String>,
        account_key: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::credentials::v1::AzureStorageKey as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = account_name {
            inner.account_name = value;
        }
        if let ::core::option::Option::Some(value) = account_key {
            inner.account_key = value;
        }
        Self(inner)
    }

    /// The name of the storage account.
    #[getter]
    fn account_name(&self) -> ::std::string::String {
        self.0.account_name.clone()
    }

    /// The account key of the storage account.
    #[getter]
    fn account_key(&self) -> ::std::string::String {
        self.0.account_key.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("account_name", false), ("account_key", true)],
        )
    }
}

impl ::core::convert::From<super::credentials::v1::AzureStorageKey> for PyAzureStorageKey {
    fn from(value: super::credentials::v1::AzureStorageKey) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAzureStorageKey> for super::credentials::v1::AzureStorageKey {
    fn from(value: PyAzureStorageKey) -> Self {
        value.0
    }
}

/// A credential used to access external data sources or services.
#[::pyo3::pyclass(
    name = "Credential",
    eq,
    frozen,
    module = "unitycatalog_client._client.credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyCredential(pub super::credentials::v1::Credential);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCredential {
    /// The credential name.
    ///
    /// The name must be unique among storage and service credentials within the metastore.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// The unique identifier of the credential.
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }

    /// Indicates the purpose of the credential.
    #[getter]
    fn purpose(&self) -> ::core::option::Option<PyPurpose> {
        self.0
            .purpose
            .as_known()
            .filter(|e| <super::credentials::v1::Purpose as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyPurpose::from)
    }

    /// Whether the credential is usable only for read operations.
    ///
    /// Only applicable when purpose is STORAGE.
    #[getter]
    fn read_only(&self) -> bool {
        self.0.read_only
    }

    /// Comment associated with the credential.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Username of current owner of credential.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// Time at which this credential was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of credential creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this credential was last modified, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified the credential.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// Whether this credential is the current metastore's root storage credential.
    ///
    /// Only applicable when purpose is STORAGE.
    #[getter]
    fn used_for_managed_storage(&self) -> bool {
        self.0.used_for_managed_storage
    }

    /// The full name of the credential.
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }

    /// The Azure service principal configuration.
    ///
    /// Sealed into the object's inline sensitive blob (envelope-encrypted, redacted
    /// from ordinary reads) rather than stored in the searchable properties.
    #[getter]
    fn azure_service_principal(&self) -> ::core::option::Option<PyAzureServicePrincipal> {
        self.0
            .azure_service_principal
            .clone()
            .into_option()
            .map(PyAzureServicePrincipal::from)
    }

    /// The Azure managed identity configuration.
    #[getter]
    fn azure_managed_identity(&self) -> ::core::option::Option<PyAzureManagedIdentity> {
        self.0
            .azure_managed_identity
            .clone()
            .into_option()
            .map(PyAzureManagedIdentity::from)
    }

    /// The Azure storage key configuration.
    #[getter]
    fn azure_storage_key(&self) -> ::core::option::Option<PyAzureStorageKey> {
        self.0
            .azure_storage_key
            .clone()
            .into_option()
            .map(PyAzureStorageKey::from)
    }

    /// The AWS IAM role configuration.
    #[getter]
    fn aws_iam_role(&self) -> ::core::option::Option<PyAwsIamRoleConfig> {
        self.0
            .aws_iam_role
            .clone()
            .into_option()
            .map(PyAwsIamRoleConfig::from)
    }

    /// The Databricks managed GCP service account configuration.
    #[getter]
    fn databricks_gcp_service_account(
        &self,
    ) -> ::core::option::Option<PyDatabricksGcpServiceAccount> {
        self.0
            .databricks_gcp_service_account
            .clone()
            .into_option()
            .map(PyDatabricksGcpServiceAccount::from)
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("id", false),
                ("purpose", false),
                ("read_only", false),
                ("comment", false),
                ("owner", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("used_for_managed_storage", false),
                ("full_name", false),
                ("azure_service_principal", true),
                ("azure_managed_identity", true),
                ("azure_storage_key", true),
                ("aws_iam_role", true),
                ("databricks_gcp_service_account", true),
            ],
        )
    }
}

impl ::core::convert::From<super::credentials::v1::Credential> for PyCredential {
    fn from(value: super::credentials::v1::Credential) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyCredential> for super::credentials::v1::Credential {
    fn from(value: PyCredential) -> Self {
        value.0
    }
}

/// The Databricks managed GCP service account configuration.
#[::pyo3::pyclass(
    name = "DatabricksGcpServiceAccount",
    eq,
    frozen,
    module = "unitycatalog_client._client.credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyDatabricksGcpServiceAccount(pub super::credentials::v1::DatabricksGcpServiceAccount);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDatabricksGcpServiceAccount {
    #[new]
    #[pyo3(signature = (credential_id = None, email = None, private_key_id = None))]
    fn new(
        credential_id: ::core::option::Option<::std::string::String>,
        email: ::core::option::Option<::std::string::String>,
        private_key_id: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::credentials::v1::DatabricksGcpServiceAccount as ::core::default::Default>::default();
        {
            let value = credential_id;
            inner.credential_id = value;
        }
        {
            let value = email;
            inner.email = value;
        }
        {
            let value = private_key_id;
            inner.private_key_id = value;
        }
        Self(inner)
    }

    /// The Databricks internal ID that represents this managed identity.
    #[getter]
    fn credential_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.credential_id.clone()
    }

    /// The email of the service account.
    #[getter]
    fn email(&self) -> ::core::option::Option<::std::string::String> {
        self.0.email.clone()
    }

    /// The ID that represents the private key for this Service Account.
    #[getter]
    fn private_key_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.private_key_id.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("credential_id", false),
                ("email", false),
                ("private_key_id", false),
            ],
        )
    }
}

impl ::core::convert::From<super::credentials::v1::DatabricksGcpServiceAccount>
    for PyDatabricksGcpServiceAccount
{
    fn from(value: super::credentials::v1::DatabricksGcpServiceAccount) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyDatabricksGcpServiceAccount>
    for super::credentials::v1::DatabricksGcpServiceAccount
{
    fn from(value: PyDatabricksGcpServiceAccount) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "ListCredentialsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListCredentialsResponse(pub super::credentials::v1::ListCredentialsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListCredentialsResponse {
    /// The credentials returned.
    #[getter]
    fn credentials(&self) -> ::std::vec::Vec<PyCredential> {
        self.0
            .credentials
            .iter()
            .cloned()
            .map(PyCredential::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("credentials", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::credentials::v1::ListCredentialsResponse>
    for PyListCredentialsResponse
{
    fn from(value: super::credentials::v1::ListCredentialsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListCredentialsResponse>
    for super::credentials::v1::ListCredentialsResponse
{
    fn from(value: PyListCredentialsResponse) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "ExternalLocation",
    eq,
    frozen,
    module = "unitycatalog_client._client.external_locations",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyExternalLocation(pub super::external_locations::v1::ExternalLocation);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyExternalLocation {
    /// Name of the external location.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Path URL of the external location.
    #[getter]
    fn url(&self) -> ::std::string::String {
        self.0.url.clone()
    }

    /// Name of the storage credential used with this location.
    #[getter]
    fn credential_name(&self) -> ::std::string::String {
        self.0.credential_name.clone()
    }

    /// Indicates whether the external location is read-only.
    #[getter]
    fn read_only(&self) -> bool {
        self.0.read_only
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// The owner of the external location.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// Unique ID of the location's storage credential.
    #[getter]
    fn credential_id(&self) -> ::std::string::String {
        self.0.credential_id.clone()
    }

    /// Time at which this catalog was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of catalog creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this catalog was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified catalog.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// Indicates whether the principal is limited to retrieving metadata
    /// for the associated object through the BROWSE privilege when include_browse is enabled in the request.
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }

    #[getter]
    fn external_location_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.external_location_id.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("url", false),
                ("credential_name", false),
                ("read_only", false),
                ("comment", false),
                ("owner", false),
                ("credential_id", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("browse_only", false),
                ("external_location_id", false),
            ],
        )
    }
}

impl ::core::convert::From<super::external_locations::v1::ExternalLocation> for PyExternalLocation {
    fn from(value: super::external_locations::v1::ExternalLocation) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyExternalLocation> for super::external_locations::v1::ExternalLocation {
    fn from(value: PyExternalLocation) -> Self {
        value.0
    }
}

/// List external locations response.
#[::pyo3::pyclass(
    name = "ListExternalLocationsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.external_locations",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListExternalLocationsResponse(
    pub super::external_locations::v1::ListExternalLocationsResponse,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListExternalLocationsResponse {
    /// The external locations returned.
    #[getter]
    fn external_locations(&self) -> ::std::vec::Vec<PyExternalLocation> {
        self.0
            .external_locations
            .iter()
            .cloned()
            .map(PyExternalLocation::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("external_locations", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::external_locations::v1::ListExternalLocationsResponse>
    for PyListExternalLocationsResponse
{
    fn from(value: super::external_locations::v1::ListExternalLocationsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListExternalLocationsResponse>
    for super::external_locations::v1::ListExternalLocationsResponse
{
    fn from(value: PyListExternalLocationsResponse) -> Self {
        value.0
    }
}

/// The payload for creating a new function.
///
/// Wrapped in the `function_info` envelope of a `CreateFunctionRequest`, matching
/// the Unity Catalog `POST /functions` wire contract.
#[::pyo3::pyclass(
    name = "CreateFunction",
    eq,
    frozen,
    module = "unitycatalog_client._client.functions",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyCreateFunction(pub super::functions::v1::CreateFunction);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateFunction {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            catalog_name = None,
            schema_name = None,
            data_type = None,
            full_data_type = None,
            input_params = None,
            parameter_style = None,
            is_deterministic = None,
            sql_data_access = None,
            is_null_call = None,
            security_type = None,
            routine_body = None,
            routine_definition = None,
            external_language = None,
            comment = None,
            properties = None,
            return_params = None,
            routine_dependencies = None,
            specific_name = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        data_type: ::core::option::Option<::std::string::String>,
        full_data_type: ::core::option::Option<::std::string::String>,
        input_params: ::core::option::Option<PyFunctionParameterInfos>,
        parameter_style: ::core::option::Option<PyParameterStyle>,
        is_deterministic: ::core::option::Option<bool>,
        sql_data_access: ::core::option::Option<PySqlDataAccess>,
        is_null_call: ::core::option::Option<bool>,
        security_type: ::core::option::Option<PySecurityType>,
        routine_body: ::core::option::Option<PyRoutineBody>,
        routine_definition: ::core::option::Option<::std::string::String>,
        external_language: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<::std::string::String>,
        return_params: ::core::option::Option<PyFunctionParameterInfos>,
        routine_dependencies: ::core::option::Option<PyDependencyList>,
        specific_name: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::functions::v1::CreateFunction as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = data_type {
            inner.data_type = value;
        }
        if let ::core::option::Option::Some(value) = full_data_type {
            inner.full_data_type = value;
        }
        {
            let value = input_params;
            inner.input_params = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = parameter_style;
            inner.parameter_style = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::functions::v1::ParameterStyle as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        if let ::core::option::Option::Some(value) = is_deterministic {
            inner.is_deterministic = value;
        }
        {
            let value = sql_data_access;
            inner.sql_data_access = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::functions::v1::SqlDataAccess as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        if let ::core::option::Option::Some(value) = is_null_call {
            inner.is_null_call = value;
        }
        {
            let value = security_type;
            inner.security_type = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::functions::v1::SecurityType as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        {
            let value = routine_body;
            inner.routine_body = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::functions::v1::RoutineBody as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        {
            let value = routine_definition;
            inner.routine_definition = value;
        }
        {
            let value = external_language;
            inner.external_language = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = properties;
            inner.properties = value;
        }
        {
            let value = return_params;
            inner.return_params = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = routine_dependencies;
            inner.routine_dependencies = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = specific_name;
            inner.specific_name = value;
        }
        Self(inner)
    }

    /// Name of function, relative to parent schema.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Name of parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Name of parent schema.
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }

    /// Full data type specification of the return type of the function.
    #[getter]
    fn data_type(&self) -> ::std::string::String {
        self.0.data_type.clone()
    }

    /// Full data type specification as SQL/catalogString text.
    #[getter]
    fn full_data_type(&self) -> ::std::string::String {
        self.0.full_data_type.clone()
    }

    /// The array of function parameter infos.
    #[getter]
    fn input_params(&self) -> ::core::option::Option<PyFunctionParameterInfos> {
        self.0
            .input_params
            .clone()
            .into_option()
            .map(PyFunctionParameterInfos::from)
    }

    /// The parameter-passing style.
    #[getter]
    fn parameter_style(&self) -> ::core::option::Option<PyParameterStyle> {
        self.0
            .parameter_style
            .as_known()
            .filter(|e| {
                <super::functions::v1::ParameterStyle as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PyParameterStyle::from)
    }

    /// Indicates whether the function is deterministic.
    #[getter]
    fn is_deterministic(&self) -> bool {
        self.0.is_deterministic
    }

    /// SQL data access information.
    #[getter]
    fn sql_data_access(&self) -> ::core::option::Option<PySqlDataAccess> {
        self.0
            .sql_data_access
            .as_known()
            .filter(|e| {
                <super::functions::v1::SqlDataAccess as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PySqlDataAccess::from)
    }

    /// Indicates whether the function is null-calling.
    #[getter]
    fn is_null_call(&self) -> bool {
        self.0.is_null_call
    }

    /// The security type of the function.
    #[getter]
    fn security_type(&self) -> ::core::option::Option<PySecurityType> {
        self.0
            .security_type
            .as_known()
            .filter(|e| {
                <super::functions::v1::SecurityType as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PySecurityType::from)
    }

    /// The routine body.
    #[getter]
    fn routine_body(&self) -> ::core::option::Option<PyRoutineBody> {
        self.0
            .routine_body
            .as_known()
            .filter(|e| <super::functions::v1::RoutineBody as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyRoutineBody::from)
    }

    /// Function body.
    #[getter]
    fn routine_definition(&self) -> ::core::option::Option<::std::string::String> {
        self.0.routine_definition.clone()
    }

    /// External language of the function.
    ///
    /// Set when routine_body is EXTERNAL.
    #[getter]
    fn external_language(&self) -> ::core::option::Option<::std::string::String> {
        self.0.external_language.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// JSON-serialized key-value pair map, encoded (escaped) as a string.
    #[getter]
    fn properties(&self) -> ::core::option::Option<::std::string::String> {
        self.0.properties.clone()
    }

    /// The return parameters of a table-valued function.
    #[getter]
    fn return_params(&self) -> ::core::option::Option<PyFunctionParameterInfos> {
        self.0
            .return_params
            .clone()
            .into_option()
            .map(PyFunctionParameterInfos::from)
    }

    /// The tables and functions the function depends on.
    #[getter]
    fn routine_dependencies(&self) -> ::core::option::Option<PyDependencyList> {
        self.0
            .routine_dependencies
            .clone()
            .into_option()
            .map(PyDependencyList::from)
    }

    /// Specific name of the function; reserved for future use.
    #[getter]
    fn specific_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.specific_name.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("catalog_name", false),
                ("schema_name", false),
                ("data_type", false),
                ("full_data_type", false),
                ("input_params", false),
                ("parameter_style", false),
                ("is_deterministic", false),
                ("sql_data_access", false),
                ("is_null_call", false),
                ("security_type", false),
                ("routine_body", false),
                ("routine_definition", false),
                ("external_language", false),
                ("comment", false),
                ("properties", false),
                ("return_params", false),
                ("routine_dependencies", false),
                ("specific_name", false),
            ],
        )
    }
}

impl ::core::convert::From<super::functions::v1::CreateFunction> for PyCreateFunction {
    fn from(value: super::functions::v1::CreateFunction) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyCreateFunction> for super::functions::v1::CreateFunction {
    fn from(value: PyCreateFunction) -> Self {
        value.0
    }
}

/// A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
#[::pyo3::pyclass(
    name = "Function",
    eq,
    frozen,
    module = "unitycatalog_client._client.functions",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyFunction(pub super::functions::v1::Function);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyFunction {
    /// Name of function, relative to parent schema.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Name of parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Name of parent schema.
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }

    /// The three-level (fully qualified) name of the function.
    ///
    /// Format: catalog_name.schema_name.function_name
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }

    /// Full data type specification of the return type of the function.
    #[getter]
    fn data_type(&self) -> ::std::string::String {
        self.0.data_type.clone()
    }

    /// Full data type specification as SQL/catalogString text.
    #[getter]
    fn full_data_type(&self) -> ::std::string::String {
        self.0.full_data_type.clone()
    }

    /// The array of function parameter infos.
    #[getter]
    fn input_params(&self) -> ::core::option::Option<PyFunctionParameterInfos> {
        self.0
            .input_params
            .clone()
            .into_option()
            .map(PyFunctionParameterInfos::from)
    }

    /// The return parameters of a table-valued function.
    #[getter]
    fn return_params(&self) -> ::core::option::Option<PyFunctionParameterInfos> {
        self.0
            .return_params
            .clone()
            .into_option()
            .map(PyFunctionParameterInfos::from)
    }

    /// External language of the function.
    ///
    /// Set when routine_body is EXTERNAL.
    #[getter]
    fn external_language(&self) -> ::core::option::Option<::std::string::String> {
        self.0.external_language.clone()
    }

    /// Function body.
    #[getter]
    fn routine_definition(&self) -> ::core::option::Option<::std::string::String> {
        self.0.routine_definition.clone()
    }

    /// The tables and functions the function depends on.
    #[getter]
    fn routine_dependencies(&self) -> ::core::option::Option<PyDependencyList> {
        self.0
            .routine_dependencies
            .clone()
            .into_option()
            .map(PyDependencyList::from)
    }

    /// The parameter-passing style.
    #[getter]
    fn parameter_style(&self) -> ::core::option::Option<PyParameterStyle> {
        self.0
            .parameter_style
            .as_known()
            .filter(|e| {
                <super::functions::v1::ParameterStyle as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PyParameterStyle::from)
    }

    /// Indicates whether the function is deterministic.
    #[getter]
    fn is_deterministic(&self) -> bool {
        self.0.is_deterministic
    }

    /// SQL data access information.
    #[getter]
    fn sql_data_access(&self) -> ::core::option::Option<PySqlDataAccess> {
        self.0
            .sql_data_access
            .as_known()
            .filter(|e| {
                <super::functions::v1::SqlDataAccess as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PySqlDataAccess::from)
    }

    /// Indicates whether the function is null-calling.
    #[getter]
    fn is_null_call(&self) -> bool {
        self.0.is_null_call
    }

    /// The security type of the function.
    #[getter]
    fn security_type(&self) -> ::core::option::Option<PySecurityType> {
        self.0
            .security_type
            .as_known()
            .filter(|e| {
                <super::functions::v1::SecurityType as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PySecurityType::from)
    }

    /// Specific name of the function; reserved for future use.
    #[getter]
    fn specific_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.specific_name.clone()
    }

    /// The routine body.
    #[getter]
    fn routine_body(&self) -> ::core::option::Option<PyRoutineBody> {
        self.0
            .routine_body
            .as_known()
            .filter(|e| <super::functions::v1::RoutineBody as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyRoutineBody::from)
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// JSON-serialized key-value pair map, encoded (escaped) as a string.
    #[getter]
    fn properties(&self) -> ::core::option::Option<::std::string::String> {
        self.0.properties.clone()
    }

    /// Username of current owner of the function.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// Unique identifier for the function.
    #[getter]
    fn function_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.function_id.clone()
    }

    /// Time at which this function was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of function creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this function was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified the function.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// Indicates whether the principal is limited to retrieving metadata
    /// for the associated object through the BROWSE privilege when include_browse
    /// is enabled in the request.
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("catalog_name", false),
                ("schema_name", false),
                ("full_name", false),
                ("data_type", false),
                ("full_data_type", false),
                ("input_params", false),
                ("return_params", false),
                ("external_language", false),
                ("routine_definition", false),
                ("routine_dependencies", false),
                ("parameter_style", false),
                ("is_deterministic", false),
                ("sql_data_access", false),
                ("is_null_call", false),
                ("security_type", false),
                ("specific_name", false),
                ("routine_body", false),
                ("comment", false),
                ("properties", false),
                ("owner", false),
                ("function_id", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("browse_only", false),
            ],
        )
    }
}

impl ::core::convert::From<super::functions::v1::Function> for PyFunction {
    fn from(value: super::functions::v1::Function) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyFunction> for super::functions::v1::Function {
    fn from(value: PyFunction) -> Self {
        value.0
    }
}

/// Information about a single function parameter.
#[::pyo3::pyclass(
    name = "FunctionParameterInfo",
    eq,
    frozen,
    module = "unitycatalog_client._client.functions",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyFunctionParameterInfo(pub super::functions::v1::FunctionParameterInfo);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyFunctionParameterInfo {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            type_text = None,
            type_json = None,
            type_name = None,
            type_precision = None,
            type_scale = None,
            type_interval_type = None,
            position = None,
            parameter_mode = None,
            parameter_type = None,
            parameter_default = None,
            comment = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        type_text: ::core::option::Option<::std::string::String>,
        type_json: ::core::option::Option<::std::string::String>,
        type_name: ::core::option::Option<PyColumnTypeName>,
        type_precision: ::core::option::Option<i32>,
        type_scale: ::core::option::Option<i32>,
        type_interval_type: ::core::option::Option<::std::string::String>,
        position: ::core::option::Option<i32>,
        parameter_mode: ::core::option::Option<PyParameterMode>,
        parameter_type: ::core::option::Option<PyFunctionParameterType>,
        parameter_default: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::functions::v1::FunctionParameterInfo as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = type_text {
            inner.type_text = value;
        }
        {
            let value = type_json;
            inner.type_json = value;
        }
        {
            let value = type_name;
            inner.type_name = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::tables::v1::ColumnTypeName as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        {
            let value = type_precision;
            inner.type_precision = value;
        }
        {
            let value = type_scale;
            inner.type_scale = value;
        }
        {
            let value = type_interval_type;
            inner.type_interval_type = value;
        }
        {
            let value = position;
            inner.position = value;
        }
        {
            let value = parameter_mode;
            inner.parameter_mode = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::functions::v1::ParameterMode as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        {
            let value = parameter_type;
            inner.parameter_type = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                    <super::functions::v1::FunctionParameterType as ::core::convert::From<
                        _,
                    >>::from(e),
                )
                })
                .unwrap_or_default();
        }
        {
            let value = parameter_default;
            inner.parameter_default = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        Self(inner)
    }

    /// Name of parameter.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Full data type specification as SQL/catalogString text.
    #[getter]
    fn type_text(&self) -> ::std::string::String {
        self.0.type_text.clone()
    }

    /// Full data type specification, JSON-serialized.
    #[getter]
    fn type_json(&self) -> ::core::option::Option<::std::string::String> {
        self.0.type_json.clone()
    }

    /// Data type name.
    #[getter]
    fn type_name(&self) -> ::core::option::Option<PyColumnTypeName> {
        self.0
            .type_name
            .as_known()
            .filter(|e| <super::tables::v1::ColumnTypeName as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyColumnTypeName::from)
    }

    /// Digits of precision; required for DecimalTypes.
    #[getter]
    fn type_precision(&self) -> ::core::option::Option<i32> {
        self.0.type_precision
    }

    /// Digits to right of decimal; required for DecimalTypes.
    #[getter]
    fn type_scale(&self) -> ::core::option::Option<i32> {
        self.0.type_scale
    }

    /// Format of IntervalType.
    #[getter]
    fn type_interval_type(&self) -> ::core::option::Option<::std::string::String> {
        self.0.type_interval_type.clone()
    }

    /// Ordinal position of column (starting at position 0).
    #[getter]
    fn position(&self) -> ::core::option::Option<i32> {
        self.0.position
    }

    /// The mode of the function parameter.
    #[getter]
    fn parameter_mode(&self) -> ::core::option::Option<PyParameterMode> {
        self.0
            .parameter_mode
            .as_known()
            .filter(|e| {
                <super::functions::v1::ParameterMode as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PyParameterMode::from)
    }

    /// The type of function parameter.
    #[getter]
    fn parameter_type(&self) -> ::core::option::Option<PyFunctionParameterType> {
        self.0
            .parameter_type
            .as_known()
            .filter(|e| {
                <super::functions::v1::FunctionParameterType as ::buffa::Enumeration>::to_i32(e)
                    != 0
            })
            .map(PyFunctionParameterType::from)
    }

    /// Default value of the parameter.
    #[getter]
    fn parameter_default(&self) -> ::core::option::Option<::std::string::String> {
        self.0.parameter_default.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("type_text", false),
                ("type_json", false),
                ("type_name", false),
                ("type_precision", false),
                ("type_scale", false),
                ("type_interval_type", false),
                ("position", false),
                ("parameter_mode", false),
                ("parameter_type", false),
                ("parameter_default", false),
                ("comment", false),
            ],
        )
    }
}

impl ::core::convert::From<super::functions::v1::FunctionParameterInfo>
    for PyFunctionParameterInfo
{
    fn from(value: super::functions::v1::FunctionParameterInfo) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyFunctionParameterInfo>
    for super::functions::v1::FunctionParameterInfo
{
    fn from(value: PyFunctionParameterInfo) -> Self {
        value.0
    }
}

/// A collection of function parameters.
#[::pyo3::pyclass(
    name = "FunctionParameterInfos",
    eq,
    frozen,
    module = "unitycatalog_client._client.functions",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyFunctionParameterInfos(pub super::functions::v1::FunctionParameterInfos);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyFunctionParameterInfos {
    #[new]
    #[pyo3(signature = (parameters = None))]
    fn new(parameters: ::core::option::Option<::std::vec::Vec<PyFunctionParameterInfo>>) -> Self {
        let mut inner =
            <super::functions::v1::FunctionParameterInfos as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = parameters {
            inner.parameters = value.into_iter().map(::core::convert::Into::into).collect();
        }
        Self(inner)
    }

    /// The parameters of the function.
    #[getter]
    fn parameters(&self) -> ::std::vec::Vec<PyFunctionParameterInfo> {
        self.0
            .parameters
            .iter()
            .cloned()
            .map(PyFunctionParameterInfo::from)
            .collect()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("parameters", false)])
    }
}

impl ::core::convert::From<super::functions::v1::FunctionParameterInfos>
    for PyFunctionParameterInfos
{
    fn from(value: super::functions::v1::FunctionParameterInfos) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyFunctionParameterInfos>
    for super::functions::v1::FunctionParameterInfos
{
    fn from(value: PyFunctionParameterInfos) -> Self {
        value.0
    }
}

/// List functions response.
#[::pyo3::pyclass(
    name = "ListFunctionsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.functions",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListFunctionsResponse(pub super::functions::v1::ListFunctionsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListFunctionsResponse {
    /// The functions returned.
    #[getter]
    fn functions(&self) -> ::std::vec::Vec<PyFunction> {
        self.0
            .functions
            .iter()
            .cloned()
            .map(PyFunction::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("functions", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::functions::v1::ListFunctionsResponse>
    for PyListFunctionsResponse
{
    fn from(value: super::functions::v1::ListFunctionsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListFunctionsResponse>
    for super::functions::v1::ListFunctionsResponse
{
    fn from(value: PyListFunctionsResponse) -> Self {
        value.0
    }
}

/// List model versions response.
#[::pyo3::pyclass(
    name = "ListModelVersionsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.model_versions",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListModelVersionsResponse(pub super::model_versions::v1::ListModelVersionsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListModelVersionsResponse {
    /// The model versions returned.
    #[getter]
    fn model_versions(&self) -> ::std::vec::Vec<PyModelVersion> {
        self.0
            .model_versions
            .iter()
            .cloned()
            .map(PyModelVersion::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("model_versions", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::model_versions::v1::ListModelVersionsResponse>
    for PyListModelVersionsResponse
{
    fn from(value: super::model_versions::v1::ListModelVersionsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListModelVersionsResponse>
    for super::model_versions::v1::ListModelVersionsResponse
{
    fn from(value: PyListModelVersionsResponse) -> Self {
        value.0
    }
}

/// A model version under a registered model.
///
/// A version is identified by its parent model's three-level name plus an
/// integer `version` number assigned by the server. It carries its own artifact
/// storage location and moves through the PENDING_REGISTRATION -> READY lifecycle
/// via the finalize operation.
#[::pyo3::pyclass(
    name = "ModelVersion",
    eq,
    frozen,
    module = "unitycatalog_client._client.model_versions",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyModelVersion(pub super::model_versions::v1::ModelVersion);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyModelVersion {
    /// Name of the parent registered model, relative to parent schema.
    #[getter]
    fn model_name(&self) -> ::std::string::String {
        self.0.model_name.clone()
    }

    /// Name of parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Name of parent schema.
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }

    /// Integer model version number, used to reference the model version in API
    /// requests. Assigned by the server, monotonically increasing per model.
    #[getter]
    fn version(&self) -> i64 {
        self.0.version
    }

    /// URI indicating the location of the source artifacts used to create the model
    /// version.
    #[getter]
    fn source(&self) -> ::std::string::String {
        self.0.source.clone()
    }

    /// The run id used by the ML package that generated this model.
    #[getter]
    fn run_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.run_id.clone()
    }

    /// Current status of the model version.
    #[getter]
    fn status(&self) -> ::core::option::Option<PyModelVersionStatus> {
        self.0
            .status
            .as_known()
            .filter(|e| {
                <super::model_versions::v1::ModelVersionStatus as ::buffa::Enumeration>::to_i32(e)
                    != 0
            })
            .map(PyModelVersionStatus::from)
    }

    /// The storage location under which the model version's artifacts are stored.
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Unique identifier for the model version.
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }

    /// Time at which this model version was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of model version creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this model version was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified the model version.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// The unique identifier of the metastore.
    #[getter]
    fn metastore_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.metastore_id.clone()
    }

    /// Indicates whether the principal is limited to retrieving metadata for the
    /// associated object through the BROWSE privilege when include_browse is
    /// enabled in the request.
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("model_name", false),
                ("catalog_name", false),
                ("schema_name", false),
                ("version", false),
                ("source", false),
                ("run_id", false),
                ("status", false),
                ("storage_location", false),
                ("comment", false),
                ("id", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("metastore_id", false),
                ("browse_only", false),
            ],
        )
    }
}

impl ::core::convert::From<super::model_versions::v1::ModelVersion> for PyModelVersion {
    fn from(value: super::model_versions::v1::ModelVersion) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyModelVersion> for super::model_versions::v1::ModelVersion {
    fn from(value: PyModelVersion) -> Self {
        value.0
    }
}

/// An argument passed to a row-filter or column-mask function.
#[::pyo3::pyclass(
    name = "FunctionArg",
    eq,
    frozen,
    module = "unitycatalog_client._client.policies",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyFunctionArg(pub super::policies::v1::FunctionArg);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyFunctionArg {
    #[new]
    #[pyo3(signature = (alias = None, constant = None))]
    fn new(
        alias: ::core::option::Option<::std::string::String>,
        constant: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::policies::v1::FunctionArg as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = alias {
            inner.value = ::core::option::Option::Some(
                super::policies::v1::function_arg::Value::Alias(value),
            );
        }
        if let ::core::option::Option::Some(value) = constant {
            inner.value = ::core::option::Option::Some(
                super::policies::v1::function_arg::Value::Constant(value),
            );
        }
        Self(inner)
    }

    ///References a column via its `MatchColumn.alias`.
    #[getter]
    fn alias(&self) -> ::core::option::Option<::std::string::String> {
        match &self.0.value {
            ::core::option::Option::Some(super::policies::v1::function_arg::Value::Alias(
                value,
            )) => ::core::option::Option::Some(::core::clone::Clone::clone(value)),
            _ => ::core::option::Option::None,
        }
    }

    ///A literal constant value, passed as-is to the function.
    #[getter]
    fn constant(&self) -> ::core::option::Option<::std::string::String> {
        match &self.0.value {
            ::core::option::Option::Some(super::policies::v1::function_arg::Value::Constant(
                value,
            )) => ::core::option::Option::Some(::core::clone::Clone::clone(value)),
            _ => ::core::option::Option::None,
        }
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("alias", false), ("constant", false)])
    }
}

impl ::core::convert::From<super::policies::v1::FunctionArg> for PyFunctionArg {
    fn from(value: super::policies::v1::FunctionArg) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyFunctionArg> for super::policies::v1::FunctionArg {
    fn from(value: PyFunctionArg) -> Self {
        value.0
    }
}

/// A reference to the SQL function that implements a row-filter or column-mask policy,
/// together with the arguments it is invoked with.
#[::pyo3::pyclass(
    name = "FunctionRef",
    eq,
    frozen,
    module = "unitycatalog_client._client.policies",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyFunctionRef(pub super::policies::v1::FunctionRef);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyFunctionRef {
    #[new]
    #[pyo3(signature = (function_name = None, using = None, on_column = None))]
    fn new(
        function_name: ::core::option::Option<::std::string::String>,
        using: ::core::option::Option<::std::vec::Vec<PyFunctionArg>>,
        on_column: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::policies::v1::FunctionRef as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = function_name {
            inner.function_name = value;
        }
        if let ::core::option::Option::Some(value) = using {
            inner.using = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = on_column;
            inner.on_column = value;
        }
        Self(inner)
    }

    /// Fully qualified name of the function (catalog.schema.function) to invoke.
    #[getter]
    fn function_name(&self) -> ::std::string::String {
        self.0.function_name.clone()
    }

    /// The arguments passed to the function, in declaration order.
    #[getter]
    fn using(&self) -> ::std::vec::Vec<PyFunctionArg> {
        self.0
            .using
            .iter()
            .cloned()
            .map(PyFunctionArg::from)
            .collect()
    }

    /// The alias of the column to be masked; it must refer to one of the policy's match_columns.
    ///
    /// Required for column masks: the column value is passed to the mask function as its
    /// first argument. Not used for row filters.
    #[getter]
    fn on_column(&self) -> ::core::option::Option<::std::string::String> {
        self.0.on_column.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("function_name", false),
                ("using", false),
                ("on_column", false),
            ],
        )
    }
}

impl ::core::convert::From<super::policies::v1::FunctionRef> for PyFunctionRef {
    fn from(value: super::policies::v1::FunctionRef) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyFunctionRef> for super::policies::v1::FunctionRef {
    fn from(value: PyFunctionRef) -> Self {
        value.0
    }
}

/// List policies response.
#[::pyo3::pyclass(
    name = "ListPoliciesResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.policies",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListPoliciesResponse(pub super::policies::v1::ListPoliciesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListPoliciesResponse {
    /// The policies returned.
    #[getter]
    fn policies(&self) -> ::std::vec::Vec<PyPolicyInfo> {
        self.0
            .policies
            .iter()
            .cloned()
            .map(PyPolicyInfo::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("policies", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::policies::v1::ListPoliciesResponse> for PyListPoliciesResponse {
    fn from(value: super::policies::v1::ListPoliciesResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListPoliciesResponse> for super::policies::v1::ListPoliciesResponse {
    fn from(value: PyListPoliciesResponse) -> Self {
        value.0
    }
}

/// A column referenced by a policy, together with the alias it is bound to in the
/// row-filter or column-mask function invocation.
#[::pyo3::pyclass(
    name = "MatchColumn",
    eq,
    frozen,
    module = "unitycatalog_client._client.policies",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyMatchColumn(pub super::policies::v1::MatchColumn);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyMatchColumn {
    #[new]
    #[pyo3(signature = (alias = None, condition = None))]
    fn new(
        alias: ::core::option::Option<::std::string::String>,
        condition: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::policies::v1::MatchColumn as ::core::default::Default>::default();
        {
            let value = alias;
            inner.alias = value;
        }
        if let ::core::option::Option::Some(value) = condition {
            inner.condition = value;
        }
        Self(inner)
    }

    /// The name of the alias the column is bound to when invoking the row-filter or
    /// column-mask function.
    #[getter]
    fn alias(&self) -> ::core::option::Option<::std::string::String> {
        self.0.alias.clone()
    }

    /// The condition on the column, e.g. its name, that must match for this binding to apply.
    #[getter]
    fn condition(&self) -> ::std::string::String {
        self.0.condition.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("alias", false), ("condition", false)])
    }
}

impl ::core::convert::From<super::policies::v1::MatchColumn> for PyMatchColumn {
    fn from(value: super::policies::v1::MatchColumn) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyMatchColumn> for super::policies::v1::MatchColumn {
    fn from(value: PyMatchColumn) -> Self {
        value.0
    }
}

/// A row-filter or column-mask policy bound to principals and scoped to a securable.
///
/// Policies are read as whole documents; enforcement is performed by the query engine,
/// not by this service. `when_condition` is stored and returned opaque — this service does
/// not parse or evaluate it.
#[::pyo3::pyclass(
    name = "PolicyInfo",
    eq,
    frozen,
    module = "unitycatalog_client._client.policies",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyPolicyInfo(pub super::policies::v1::PolicyInfo);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyPolicyInfo {
    /// The name of the policy. Unique within the securable it is defined on.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Server-assigned unique identifier for the policy.
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }

    /// The type of securable the policy is defined on.
    ///
    /// Supported values: CATALOG, SCHEMA, TABLE.
    #[getter]
    fn on_securable_type(&self) -> ::std::string::String {
        self.0.on_securable_type.clone()
    }

    /// The fully qualified name of the securable the policy is defined on.
    #[getter]
    fn on_securable_fullname(&self) -> ::std::string::String {
        self.0.on_securable_fullname.clone()
    }

    /// The kind of enforcement this policy applies.
    #[getter]
    fn policy_type(&self) -> ::core::option::Option<PyPolicyType> {
        self.0
            .policy_type
            .as_known()
            .filter(|e| <super::policies::v1::PolicyType as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyPolicyType::from)
    }

    /// The principals the policy applies to. Empty means "all principals".
    #[getter]
    fn to_principals(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.to_principals.clone()
    }

    /// The principals explicitly excluded from the policy, even if matched by to_principals.
    #[getter]
    fn except_principals(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.except_principals.clone()
    }

    /// An opaque condition expression, e.g. "hasTagValue('env','prod')", evaluated by the
    /// query engine at enforcement time. Stored and returned as-is; not parsed server-side.
    #[getter]
    fn when_condition(&self) -> ::core::option::Option<::std::string::String> {
        self.0.when_condition.clone()
    }

    /// The columns referenced by the row-filter or column-mask function, and the aliases
    /// they are bound to.
    ///
    /// Only valid when for_securable_type is TABLE; the policy then applies only to tables
    /// whose columns satisfy all match conditions.
    #[getter]
    fn match_columns(&self) -> ::std::vec::Vec<PyMatchColumn> {
        self.0
            .match_columns
            .iter()
            .cloned()
            .map(PyMatchColumn::from)
            .collect()
    }

    /// User-provided free-form text description of the policy.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Time at which this policy was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Time at which this policy was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of the user who created the policy.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Username of the user who last modified the policy.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// The type of securables the policy takes effect on, e.g. TABLE for a policy defined
    /// on a schema that applies to the tables within it.
    #[getter]
    fn for_securable_type(&self) -> ::core::option::Option<::std::string::String> {
        self.0.for_securable_type.clone()
    }

    ///The row-filter function to invoke. Set only when policy_type is POLICY_TYPE_ROW_FILTER.
    #[getter]
    fn row_filter(&self) -> ::core::option::Option<PyFunctionRef> {
        match &self.0.options {
            ::core::option::Option::Some(super::policies::v1::policy_info::Options::RowFilter(
                value,
            )) => ::core::option::Option::Some(<PyFunctionRef>::from((**value).clone())),
            _ => ::core::option::Option::None,
        }
    }

    ///The column-mask function to invoke. Set only when policy_type is POLICY_TYPE_COLUMN_MASK.
    #[getter]
    fn column_mask(&self) -> ::core::option::Option<PyFunctionRef> {
        match &self.0.options {
            ::core::option::Option::Some(
                super::policies::v1::policy_info::Options::ColumnMask(value),
            ) => ::core::option::Option::Some(<PyFunctionRef>::from((**value).clone())),
            _ => ::core::option::Option::None,
        }
    }

    ///The privileges to deny. Set only when policy_type is POLICY_TYPE_DENY.
    #[getter]
    fn deny(&self) -> ::core::option::Option<PyPrivilegeOptions> {
        match &self.0.options {
            ::core::option::Option::Some(super::policies::v1::policy_info::Options::Deny(
                value,
            )) => ::core::option::Option::Some(<PyPrivilegeOptions>::from((**value).clone())),
            _ => ::core::option::Option::None,
        }
    }

    ///The privileges to grant. Set only when policy_type is POLICY_TYPE_GRANT.
    #[getter]
    fn grant(&self) -> ::core::option::Option<PyPrivilegeOptions> {
        match &self.0.options {
            ::core::option::Option::Some(super::policies::v1::policy_info::Options::Grant(
                value,
            )) => ::core::option::Option::Some(<PyPrivilegeOptions>::from((**value).clone())),
            _ => ::core::option::Option::None,
        }
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("id", false),
                ("on_securable_type", false),
                ("on_securable_fullname", false),
                ("policy_type", false),
                ("to_principals", false),
                ("except_principals", false),
                ("when_condition", false),
                ("match_columns", false),
                ("comment", false),
                ("created_at", false),
                ("updated_at", false),
                ("created_by", false),
                ("updated_by", false),
                ("for_securable_type", false),
                ("row_filter", false),
                ("column_mask", false),
                ("deny", false),
                ("grant", false),
            ],
        )
    }
}

impl ::core::convert::From<super::policies::v1::PolicyInfo> for PyPolicyInfo {
    fn from(value: super::policies::v1::PolicyInfo) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyPolicyInfo> for super::policies::v1::PolicyInfo {
    fn from(value: PyPolicyInfo) -> Self {
        value.0
    }
}

/// The privileges a deny or grant policy applies to.
#[::pyo3::pyclass(
    name = "PrivilegeOptions",
    eq,
    frozen,
    module = "unitycatalog_client._client.policies",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyPrivilegeOptions(pub super::policies::v1::PrivilegeOptions);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyPrivilegeOptions {
    #[new]
    #[pyo3(signature = (privileges = None))]
    fn new(privileges: ::core::option::Option<::std::vec::Vec<::std::string::String>>) -> Self {
        let mut inner =
            <super::policies::v1::PrivilegeOptions as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = privileges {
            inner.privileges = value;
        }
        Self(inner)
    }

    /// The privileges to deny or grant when the principal and condition match.
    #[getter]
    fn privileges(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.privileges.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("privileges", false)])
    }
}

impl ::core::convert::From<super::policies::v1::PrivilegeOptions> for PyPrivilegeOptions {
    fn from(value: super::policies::v1::PrivilegeOptions) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyPrivilegeOptions> for super::policies::v1::PrivilegeOptions {
    fn from(value: PyPrivilegeOptions) -> Self {
        value.0
    }
}

/// Response to list providers.
#[::pyo3::pyclass(
    name = "ListProvidersResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.providers",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListProvidersResponse(pub super::providers::v1::ListProvidersResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListProvidersResponse {
    /// List of providers.
    #[getter]
    fn providers(&self) -> ::std::vec::Vec<PyProvider> {
        self.0
            .providers
            .iter()
            .cloned()
            .map(PyProvider::from)
            .collect()
    }

    /// Opaque pagination token to go to next page based on previous query.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("providers", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::providers::v1::ListProvidersResponse>
    for PyListProvidersResponse
{
    fn from(value: super::providers::v1::ListProvidersResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListProvidersResponse>
    for super::providers::v1::ListProvidersResponse
{
    fn from(value: PyListProvidersResponse) -> Self {
        value.0
    }
}

/// A provider represents an organization sharing data with this metastore.
///
/// A provider is the inbound counterpart of a recipient: it is registered from a
/// share activation/credential file and is used to access shares offered by an
/// upstream Delta Sharing server.
#[::pyo3::pyclass(
    name = "Provider",
    eq,
    frozen,
    module = "unitycatalog_client._client.providers",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyProvider(pub super::providers::v1::Provider);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyProvider {
    /// Unique ID of the provider.
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }

    /// The name of the provider.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// The delta sharing authentication type.
    #[getter]
    fn authentication_type(&self) -> ::core::option::Option<PyProviderAuthenticationType> {
        self.0
            .authentication_type
            .as_known()
            .filter(|e| {
                <super::providers::v1::ProviderAuthenticationType as ::buffa::Enumeration>::to_i32(
                    e,
                ) != 0
            })
            .map(PyProviderAuthenticationType::from)
    }

    /// Username of the provider owner.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// Description about the provider.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// The recipient profile (credential file contents) used to connect to the
    /// sharing server. Present for TOKEN and OAUTH_CLIENT_CREDENTIALS authentication.
    #[getter]
    fn recipient_profile_str(&self) -> ::core::option::Option<::std::string::String> {
        self.0.recipient_profile_str.clone()
    }

    /// A map of key-value properties attached to the provider.
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }

    /// Time at which this provider was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of the creator of the provider.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this provider was updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of the provider updater.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("id", false),
                ("name", false),
                ("authentication_type", false),
                ("owner", false),
                ("comment", false),
                ("recipient_profile_str", false),
                ("properties", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
            ],
        )
    }
}

impl ::core::convert::From<super::providers::v1::Provider> for PyProvider {
    fn from(value: super::providers::v1::Provider) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyProvider> for super::providers::v1::Provider {
    fn from(value: PyProvider) -> Self {
        value.0
    }
}

/// Response to list recipients.
#[::pyo3::pyclass(
    name = "ListRecipientsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.recipients",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListRecipientsResponse(pub super::recipients::v1::ListRecipientsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListRecipientsResponse {
    /// List of recipients.
    #[getter]
    fn recipients(&self) -> ::std::vec::Vec<PyRecipient> {
        self.0
            .recipients
            .iter()
            .cloned()
            .map(PyRecipient::from)
            .collect()
    }

    /// Opaque pagination token to go to next page based on previous query.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("recipients", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::recipients::v1::ListRecipientsResponse>
    for PyListRecipientsResponse
{
    fn from(value: super::recipients::v1::ListRecipientsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListRecipientsResponse>
    for super::recipients::v1::ListRecipientsResponse
{
    fn from(value: PyListRecipientsResponse) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "Recipient",
    eq,
    frozen,
    module = "unitycatalog_client._client.recipients",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyRecipient(pub super::recipients::v1::Recipient);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyRecipient {
    /// Unique ID of the recipient.
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }

    /// The name of the recipient.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// The delta sharing authentication type.
    #[getter]
    fn authentication_type(&self) -> ::core::option::Option<PyAuthenticationType> {
        self.0
            .authentication_type
            .as_known()
            .filter(|e| {
                <super::recipients::v1::AuthenticationType as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PyAuthenticationType::from)
    }

    /// Username of the recipient owner.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// Description about the recipient.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Recipient properties as map of string key-value pairs.
    #[getter]
    fn properties_kvpairs(&self) -> ::core::option::Option<PySecurablePropertiesKvPairs> {
        self.0
            .properties_kvpairs
            .clone()
            .into_option()
            .map(PySecurablePropertiesKvPairs::from)
    }

    /// Time at which this recipient was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of recipient creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// This field is only present when the authentication_type is TOKEN.
    #[getter]
    fn tokens(&self) -> ::std::vec::Vec<PyRecipientToken> {
        self.0
            .tokens
            .iter()
            .cloned()
            .map(PyRecipientToken::from)
            .collect()
    }

    /// Time at which this recipient was updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of recipient updater.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("id", false),
                ("name", false),
                ("authentication_type", false),
                ("owner", false),
                ("comment", false),
                ("properties_kvpairs", false),
                ("created_at", false),
                ("created_by", false),
                ("tokens", false),
                ("updated_at", false),
                ("updated_by", false),
            ],
        )
    }
}

impl ::core::convert::From<super::recipients::v1::Recipient> for PyRecipient {
    fn from(value: super::recipients::v1::Recipient) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyRecipient> for super::recipients::v1::Recipient {
    fn from(value: PyRecipient) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "RecipientToken",
    eq,
    frozen,
    module = "unitycatalog_client._client.recipients",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyRecipientToken(pub super::recipients::v1::RecipientToken);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyRecipientToken {
    /// Unique ID of the recipient token.
    #[getter]
    fn id(&self) -> ::std::string::String {
        self.0.id.clone()
    }

    /// Time at which this recipient token was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> i64 {
        self.0.created_at
    }

    /// Username of recipient token creator.
    #[getter]
    fn created_by(&self) -> ::std::string::String {
        self.0.created_by.clone()
    }

    /// Full activation URL to retrieve the access token. It will be empty if the token is already retrieved.
    #[getter]
    fn activation_url(&self) -> ::std::string::String {
        self.0.activation_url.clone()
    }

    /// Expiration timestamp of the token in epoch milliseconds.
    #[getter]
    fn expiration_time(&self) -> i64 {
        self.0.expiration_time
    }

    /// Time at which this recipient token was updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> i64 {
        self.0.updated_at
    }

    /// Username of recipient token updater.
    #[getter]
    fn updated_by(&self) -> ::std::string::String {
        self.0.updated_by.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("id", false),
                ("created_at", false),
                ("created_by", false),
                ("activation_url", false),
                ("expiration_time", false),
                ("updated_at", false),
                ("updated_by", false),
            ],
        )
    }
}

impl ::core::convert::From<super::recipients::v1::RecipientToken> for PyRecipientToken {
    fn from(value: super::recipients::v1::RecipientToken) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyRecipientToken> for super::recipients::v1::RecipientToken {
    fn from(value: PyRecipientToken) -> Self {
        value.0
    }
}

/// An object with the properties of a securable as key-value pairs.
#[::pyo3::pyclass(
    name = "SecurablePropertiesKvPairs",
    eq,
    frozen,
    module = "unitycatalog_client._client.recipients",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PySecurablePropertiesKvPairs(pub super::recipients::v1::SecurablePropertiesKvPairs);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PySecurablePropertiesKvPairs {
    #[new]
    #[pyo3(signature = (properties = None))]
    fn new(
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
    ) -> Self {
        let mut inner = <super::recipients::v1::SecurablePropertiesKvPairs as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        Self(inner)
    }

    /// A map of key-value properties attached to the securable.
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("properties", false)])
    }
}

impl ::core::convert::From<super::recipients::v1::SecurablePropertiesKvPairs>
    for PySecurablePropertiesKvPairs
{
    fn from(value: super::recipients::v1::SecurablePropertiesKvPairs) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PySecurablePropertiesKvPairs>
    for super::recipients::v1::SecurablePropertiesKvPairs
{
    fn from(value: PySecurablePropertiesKvPairs) -> Self {
        value.0
    }
}

/// List registered models response.
#[::pyo3::pyclass(
    name = "ListRegisteredModelsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.registered_models",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListRegisteredModelsResponse(
    pub super::registered_models::v1::ListRegisteredModelsResponse,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListRegisteredModelsResponse {
    /// The registered models returned.
    #[getter]
    fn registered_models(&self) -> ::std::vec::Vec<PyRegisteredModel> {
        self.0
            .registered_models
            .iter()
            .cloned()
            .map(PyRegisteredModel::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("registered_models", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::registered_models::v1::ListRegisteredModelsResponse>
    for PyListRegisteredModelsResponse
{
    fn from(value: super::registered_models::v1::ListRegisteredModelsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListRegisteredModelsResponse>
    for super::registered_models::v1::ListRegisteredModelsResponse
{
    fn from(value: PyListRegisteredModelsResponse) -> Self {
        value.0
    }
}

/// A registered model within the three-level namespace (catalog.schema.model).
///
/// Registered models group a collection of model versions. Individual versions
/// carry their own artifact storage location and are created, finalized, and
/// governed independently.
#[::pyo3::pyclass(
    name = "RegisteredModel",
    eq,
    frozen,
    module = "unitycatalog_client._client.registered_models",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyRegisteredModel(pub super::registered_models::v1::RegisteredModel);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyRegisteredModel {
    /// Name of registered model, relative to parent schema.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Name of parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Name of parent schema.
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }

    /// The three-level (fully qualified) name of the registered model.
    ///
    /// Format: catalog_name.schema_name.model_name
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }

    /// The storage location under which model version data files are stored.
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }

    /// Username of current owner of the registered model.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Unique identifier for the registered model.
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }

    /// Time at which this registered model was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of registered model creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this registered model was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified the registered model.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// The unique identifier of the metastore.
    #[getter]
    fn metastore_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.metastore_id.clone()
    }

    /// Indicates whether the principal is limited to retrieving metadata for the
    /// associated object through the BROWSE privilege when include_browse is
    /// enabled in the request.
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("catalog_name", false),
                ("schema_name", false),
                ("full_name", false),
                ("storage_location", false),
                ("owner", false),
                ("comment", false),
                ("id", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("metastore_id", false),
                ("browse_only", false),
            ],
        )
    }
}

impl ::core::convert::From<super::registered_models::v1::RegisteredModel> for PyRegisteredModel {
    fn from(value: super::registered_models::v1::RegisteredModel) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyRegisteredModel> for super::registered_models::v1::RegisteredModel {
    fn from(value: PyRegisteredModel) -> Self {
        value.0
    }
}

/// List Schemas response.
#[::pyo3::pyclass(
    name = "ListSchemasResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.schemas",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListSchemasResponse(pub super::schemas::v1::ListSchemasResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListSchemasResponse {
    /// The schemas returned.
    #[getter]
    fn schemas(&self) -> ::std::vec::Vec<PySchema> {
        self.0.schemas.iter().cloned().map(PySchema::from).collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("schemas", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::schemas::v1::ListSchemasResponse> for PyListSchemasResponse {
    fn from(value: super::schemas::v1::ListSchemasResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListSchemasResponse> for super::schemas::v1::ListSchemasResponse {
    fn from(value: PyListSchemasResponse) -> Self {
        value.0
    }
}

/// A schema is a namespace within a catalog that contains tables.
#[::pyo3::pyclass(
    name = "Schema",
    eq,
    frozen,
    module = "unitycatalog_client._client.schemas",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PySchema(pub super::schemas::v1::Schema);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PySchema {
    /// Name of schema, relative to parent catalog.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Name of parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Full name of schema, in form of catalog_name.schema_name.
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// A map of key-value properties attached to the securable.
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }

    /// Username of current owner of schema.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// Time at which this schema was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of schema creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this schema was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified schema.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// Unique identifier for the schema.
    #[getter]
    fn schema_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.schema_id.clone()
    }

    /// Storage root URL for managed storage location of the schema.
    ///
    /// Can be set when creating a schema. When set, managed tables/volumes created
    /// in this schema are rooted here; otherwise the parent catalog's storage
    /// location is used. Example: `s3://bucket/ucroot`.
    #[getter]
    fn storage_root(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_root.clone()
    }

    /// Storage location URL (full path) for managed storage of the schema.
    ///
    /// A unique path under `storage_root`. Absent when the schema has no
    /// `storage_root`, in which case managed securables fall back to the parent
    /// catalog's storage location.
    /// Example: `s3://bucket/ucroot/__unitystorage/schemas/{schema_id}`.
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }

    /// Indicates whether the principal is limited to retrieving metadata
    /// for the associated object through the BROWSE privilege when include_browse
    /// is enabled in the request.
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("catalog_name", false),
                ("full_name", false),
                ("comment", false),
                ("properties", false),
                ("owner", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("schema_id", false),
                ("storage_root", false),
                ("storage_location", false),
                ("browse_only", false),
            ],
        )
    }
}

impl ::core::convert::From<super::schemas::v1::Schema> for PySchema {
    fn from(value: super::schemas::v1::Schema) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PySchema> for super::schemas::v1::Schema {
    fn from(value: PySchema) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "DataObject",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyDataObject(pub super::shares::v1::DataObject);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDataObject {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            data_object_type = None,
            added_at = None,
            added_by = None,
            comment = None,
            shared_as = None,
            partitions = None,
            cdf_enabled = None,
            history_data_sharing_status = None,
            start_version = None,
            string_shared_as = None,
            status = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        data_object_type: ::core::option::Option<PyDataObjectType>,
        added_at: ::core::option::Option<i64>,
        added_by: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        shared_as: ::core::option::Option<::std::string::String>,
        partitions: ::core::option::Option<::std::vec::Vec<PyPartition>>,
        cdf_enabled: ::core::option::Option<bool>,
        history_data_sharing_status: ::core::option::Option<PyHistoryStatus>,
        start_version: ::core::option::Option<i64>,
        string_shared_as: ::core::option::Option<::std::string::String>,
        status: ::core::option::Option<PyDataObjectStatus>,
    ) -> Self {
        let mut inner = <super::shares::v1::DataObject as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = data_object_type;
            inner.data_object_type = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::shares::v1::DataObjectType as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        {
            let value = added_at;
            inner.added_at = value;
        }
        {
            let value = added_by;
            inner.added_by = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = shared_as;
            inner.shared_as = value;
        }
        if let ::core::option::Option::Some(value) = partitions {
            inner.partitions = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = cdf_enabled;
            inner.cdf_enabled = value;
        }
        {
            let value = history_data_sharing_status;
            inner.history_data_sharing_status = value.map(|e| {
                ::buffa::EnumValue::Known(
                    <super::shares::v1::HistoryStatus as ::core::convert::From<_>>::from(e),
                )
            });
        }
        {
            let value = start_version;
            inner.start_version = value;
        }
        {
            let value = string_shared_as;
            inner.string_shared_as = value;
        }
        {
            let value = status;
            inner.status = value.map(|e| {
                ::buffa::EnumValue::Known(
                    <super::shares::v1::DataObjectStatus as ::core::convert::From<_>>::from(e),
                )
            });
        }
        Self(inner)
    }

    /// A fully qualified name that uniquely identifies a data object.
    ///
    /// For example, a table's fully qualified name is in the format of <catalog>.<schema>.<table>,
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Type of the data object.
    #[getter]
    fn data_object_type(&self) -> ::core::option::Option<PyDataObjectType> {
        self.0
            .data_object_type
            .as_known()
            .filter(|e| <super::shares::v1::DataObjectType as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyDataObjectType::from)
    }

    /// The time when this data object is added to the share, in epoch milliseconds.
    #[getter]
    fn added_at(&self) -> ::core::option::Option<i64> {
        self.0.added_at
    }

    /// Username of the sharer.
    #[getter]
    fn added_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.added_by.clone()
    }

    /// A user-provided comment when adding the data object to the share.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// A user-provided new name for the data object within the share.
    ///
    /// If this new name is not provided, the object's original name will be used as the shared_as name.
    /// The shared_as name must be unique within a share.
    /// For tables, the new name must follow the format of <schema>.<table>: two parts, each
    /// consisting only of alphanumeric characters and underscores, without the catalog part.
    /// Do not set it for non-table data objects; use string_shared_as instead.
    #[getter]
    fn shared_as(&self) -> ::core::option::Option<::std::string::String> {
        self.0.shared_as.clone()
    }

    /// Array of partitions for the shared data.
    #[getter]
    fn partitions(&self) -> ::std::vec::Vec<PyPartition> {
        self.0
            .partitions
            .iter()
            .cloned()
            .map(PyPartition::from)
            .collect()
    }

    /// Whether to enable cdf or indicate if cdf is enabled on the shared object.
    #[getter]
    fn cdf_enabled(&self) -> ::core::option::Option<bool> {
        self.0.cdf_enabled
    }

    /// Whether to enable or disable sharing of data history. If not specified, the default is DISABLED.
    #[getter]
    fn history_data_sharing_status(&self) -> ::core::option::Option<PyHistoryStatus> {
        self.0
            .history_data_sharing_status
            .as_ref()
            .and_then(|e| e.as_known())
            .map(PyHistoryStatus::from)
    }

    /// The start version associated with the object.
    ///
    /// This allows data providers to control the lowest object version that is accessible by clients.
    /// If specified, clients can query snapshots or changes for versions >= start_version.
    /// If not specified, clients can only query starting from the version of the object at the time it was added to the share.
    ///
    /// NOTE: The start_version should be <= the current version of the object.
    #[getter]
    fn start_version(&self) -> ::core::option::Option<i64> {
        self.0.start_version
    }

    /// A user-provided alias for a non-table data object (e.g. a volume) within the share.
    ///
    /// For volumes, functions and models it is a two-part name <schema>.<name>, each part
    /// consisting only of alphanumeric characters and underscores. If not provided, the
    /// object's original name minus its catalog is used. It must be unique among objects of
    /// the same type within the share. Do not set it for table-like data objects; use
    /// shared_as instead.
    #[getter]
    fn string_shared_as(&self) -> ::core::option::Option<::std::string::String> {
        self.0.string_shared_as.clone()
    }

    /// Whether the data object is still accessible to the share.
    #[getter]
    fn status(&self) -> ::core::option::Option<PyDataObjectStatus> {
        self.0
            .status
            .as_ref()
            .and_then(|e| e.as_known())
            .filter(|e| {
                <super::shares::v1::DataObjectStatus as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PyDataObjectStatus::from)
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("data_object_type", false),
                ("added_at", false),
                ("added_by", false),
                ("comment", false),
                ("shared_as", false),
                ("partitions", false),
                ("cdf_enabled", false),
                ("history_data_sharing_status", false),
                ("start_version", false),
                ("string_shared_as", false),
                ("status", false),
            ],
        )
    }
}

impl ::core::convert::From<super::shares::v1::DataObject> for PyDataObject {
    fn from(value: super::shares::v1::DataObject) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyDataObject> for super::shares::v1::DataObject {
    fn from(value: PyDataObject) -> Self {
        value.0
    }
}

/// Data object update.
#[::pyo3::pyclass(
    name = "DataObjectUpdate",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyDataObjectUpdate(pub super::shares::v1::DataObjectUpdate);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDataObjectUpdate {
    #[new]
    #[pyo3(signature = (action = None, data_object = None))]
    fn new(
        action: ::core::option::Option<PyAction>,
        data_object: ::core::option::Option<PyDataObject>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::DataObjectUpdate as ::core::default::Default>::default();
        {
            let value = action;
            inner.action = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::shares::v1::Action as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        {
            let value = data_object;
            inner.data_object = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        Self(inner)
    }

    /// The action to perform on the data object.
    #[getter]
    fn action(&self) -> ::core::option::Option<PyAction> {
        self.0
            .action
            .as_known()
            .filter(|e| <super::shares::v1::Action as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyAction::from)
    }

    /// The data object that is being added, removed, or updated.
    #[getter]
    fn data_object(&self) -> ::core::option::Option<PyDataObject> {
        self.0
            .data_object
            .clone()
            .into_option()
            .map(PyDataObject::from)
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("action", false), ("data_object", false)])
    }
}

impl ::core::convert::From<super::shares::v1::DataObjectUpdate> for PyDataObjectUpdate {
    fn from(value: super::shares::v1::DataObjectUpdate) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyDataObjectUpdate> for super::shares::v1::DataObjectUpdate {
    fn from(value: PyDataObjectUpdate) -> Self {
        value.0
    }
}

/// Response to list shares.
#[::pyo3::pyclass(
    name = "GetPermissionsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyGetPermissionsResponse(pub super::shares::v1::GetPermissionsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetPermissionsResponse {
    /// The privileges assigned to each principal
    #[getter]
    fn privilege_assignments(&self) -> ::std::vec::Vec<PyPrivilegeAssignment> {
        self.0
            .privilege_assignments
            .iter()
            .cloned()
            .map(PyPrivilegeAssignment::from)
            .collect()
    }

    /// Opaque pagination token to go to next page based on previous query.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("privilege_assignments", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::shares::v1::GetPermissionsResponse> for PyGetPermissionsResponse {
    fn from(value: super::shares::v1::GetPermissionsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyGetPermissionsResponse> for super::shares::v1::GetPermissionsResponse {
    fn from(value: PyGetPermissionsResponse) -> Self {
        value.0
    }
}

/// Response to list shares.
#[::pyo3::pyclass(
    name = "ListSharesResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListSharesResponse(pub super::shares::v1::ListSharesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListSharesResponse {
    /// List of shares.
    #[getter]
    fn shares(&self) -> ::std::vec::Vec<PyShare> {
        self.0.shares.iter().cloned().map(PyShare::from).collect()
    }

    /// Opaque pagination token to go to next page based on previous query.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("shares", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::shares::v1::ListSharesResponse> for PyListSharesResponse {
    fn from(value: super::shares::v1::ListSharesResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListSharesResponse> for super::shares::v1::ListSharesResponse {
    fn from(value: PyListSharesResponse) -> Self {
        value.0
    }
}

/// A partition specification: the conjunction of its partition values.
#[::pyo3::pyclass(
    name = "Partition",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyPartition(pub super::shares::v1::Partition);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyPartition {
    #[new]
    #[pyo3(signature = (values = None))]
    fn new(values: ::core::option::Option<::std::vec::Vec<PyPartitionValue>>) -> Self {
        let mut inner = <super::shares::v1::Partition as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = values {
            inner.values = value.into_iter().map(::core::convert::Into::into).collect();
        }
        Self(inner)
    }

    /// An array of partition values.
    #[getter]
    fn values(&self) -> ::std::vec::Vec<PyPartitionValue> {
        self.0
            .values
            .iter()
            .cloned()
            .map(PyPartitionValue::from)
            .collect()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("values", false)])
    }
}

impl ::core::convert::From<super::shares::v1::Partition> for PyPartition {
    fn from(value: super::shares::v1::Partition) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyPartition> for super::shares::v1::Partition {
    fn from(value: PyPartition) -> Self {
        value.0
    }
}

/// A partition column value a shared table is restricted to.
#[::pyo3::pyclass(
    name = "PartitionValue",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyPartitionValue(pub super::shares::v1::PartitionValue);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyPartitionValue {
    #[new]
    #[pyo3(
        signature = (name = None, value = None, recipient_property_key = None, op = None)
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        value: ::core::option::Option<::std::string::String>,
        recipient_property_key: ::core::option::Option<::std::string::String>,
        op: ::core::option::Option<PyPartitionValueOp>,
    ) -> Self {
        let mut inner = <super::shares::v1::PartitionValue as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            inner.value = value;
        }
        {
            let value = recipient_property_key;
            inner.recipient_property_key = value;
        }
        {
            let value = op;
            inner.op = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::shares::v1::PartitionValueOp as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        Self(inner)
    }

    /// The name of the partition column.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// The value of the partition column; unset means a `null` value.
    ///
    /// Mutually exclusive with recipient_property_key.
    #[getter]
    fn value(&self) -> ::core::option::Option<::std::string::String> {
        self.0.value.clone()
    }

    /// The key of a Delta Sharing recipient property whose value the partition column is
    /// matched against, e.g. "databricks-account-id".
    ///
    /// Mutually exclusive with value.
    #[getter]
    fn recipient_property_key(&self) -> ::core::option::Option<::std::string::String> {
        self.0.recipient_property_key.clone()
    }

    /// The operator to apply for the value.
    #[getter]
    fn op(&self) -> ::core::option::Option<PyPartitionValueOp> {
        self.0
            .op
            .as_known()
            .filter(|e| {
                <super::shares::v1::PartitionValueOp as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PyPartitionValueOp::from)
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("value", false),
                ("recipient_property_key", false),
                ("op", false),
            ],
        )
    }
}

impl ::core::convert::From<super::shares::v1::PartitionValue> for PyPartitionValue {
    fn from(value: super::shares::v1::PartitionValue) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyPartitionValue> for super::shares::v1::PartitionValue {
    fn from(value: PyPartitionValue) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "PermissionsChange",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyPermissionsChange(pub super::shares::v1::PermissionsChange);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyPermissionsChange {
    #[new]
    #[pyo3(signature = (principal = None, add = None, remove = None))]
    fn new(
        principal: ::core::option::Option<::std::string::String>,
        add: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        remove: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::PermissionsChange as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = principal {
            inner.principal = value;
        }
        if let ::core::option::Option::Some(value) = add {
            inner.add = value;
        }
        if let ::core::option::Option::Some(value) = remove {
            inner.remove = value;
        }
        Self(inner)
    }

    /// The principal (user email address or group name).
    #[getter]
    fn principal(&self) -> ::std::string::String {
        self.0.principal.clone()
    }

    /// The set of privileges to add.
    #[getter]
    fn add(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.add.clone()
    }

    /// The set of privileges to remove.
    #[getter]
    fn remove(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.remove.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("principal", false), ("add", false), ("remove", false)],
        )
    }
}

impl ::core::convert::From<super::shares::v1::PermissionsChange> for PyPermissionsChange {
    fn from(value: super::shares::v1::PermissionsChange) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyPermissionsChange> for super::shares::v1::PermissionsChange {
    fn from(value: PyPermissionsChange) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "PrivilegeAssignment",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyPrivilegeAssignment(pub super::shares::v1::PrivilegeAssignment);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyPrivilegeAssignment {
    /// The principal (user email address or group name).
    ///
    /// For deleted principals, principal is empty while principal_id is populated.
    #[getter]
    fn principal(&self) -> ::std::string::String {
        self.0.principal.clone()
    }

    /// The privileges assigned to the principal.
    #[getter]
    fn privileges(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.privileges.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("principal", false), ("privileges", false)])
    }
}

impl ::core::convert::From<super::shares::v1::PrivilegeAssignment> for PyPrivilegeAssignment {
    fn from(value: super::shares::v1::PrivilegeAssignment) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyPrivilegeAssignment> for super::shares::v1::PrivilegeAssignment {
    fn from(value: PyPrivilegeAssignment) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "Share",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyShare(pub super::shares::v1::Share);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyShare {
    /// Unique ID of the share.
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }

    /// Name of the share.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// A list of shared data objects within the share.
    #[getter]
    fn objects(&self) -> ::std::vec::Vec<PyDataObject> {
        self.0
            .objects
            .iter()
            .cloned()
            .map(PyDataObject::from)
            .collect()
    }

    /// Username of current owner of share.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Storage Location URL (full path) for the share.
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }

    /// Storage root URL for the share.
    #[getter]
    fn storage_root(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_root.clone()
    }

    /// Time at which this share was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of the creator of the share.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this share was updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of share updater.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("id", false),
                ("name", false),
                ("objects", false),
                ("owner", false),
                ("comment", false),
                ("storage_location", false),
                ("storage_root", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
            ],
        )
    }
}

impl ::core::convert::From<super::shares::v1::Share> for PyShare {
    fn from(value: super::shares::v1::Share) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyShare> for super::shares::v1::Share {
    fn from(value: PyShare) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "UpdatePermissionsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.shares",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyUpdatePermissionsResponse(pub super::shares::v1::UpdatePermissionsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdatePermissionsResponse {
    /// The privileges assigned to each principal
    #[getter]
    fn privilege_assignments(&self) -> ::std::vec::Vec<PyPrivilegeAssignment> {
        self.0
            .privilege_assignments
            .iter()
            .cloned()
            .map(PyPrivilegeAssignment::from)
            .collect()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("privilege_assignments", false)])
    }
}

impl ::core::convert::From<super::shares::v1::UpdatePermissionsResponse>
    for PyUpdatePermissionsResponse
{
    fn from(value: super::shares::v1::UpdatePermissionsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyUpdatePermissionsResponse>
    for super::shares::v1::UpdatePermissionsResponse
{
    fn from(value: PyUpdatePermissionsResponse) -> Self {
        value.0
    }
}

/// A staging table is a transient reservation created while provisioning a
/// Unity Catalog managed table.
///
/// On creation the server allocates an immutable table id and storage location
/// under the parent schema/catalog's managed storage root. The client then
/// writes the initial Delta commit (`_delta_log/00...0.json`) at that location
/// and finalizes the table via the CreateTable API, which commits this staging
/// row. A staging table is not yet a usable table and is not returned by the
/// table APIs.
#[::pyo3::pyclass(
    name = "StagingTable",
    eq,
    frozen,
    module = "unitycatalog_client._client.staging_tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyStagingTable(pub super::staging_tables::v1::StagingTable);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyStagingTable {
    /// Unique identifier generated for the staging table. Becomes the managed
    /// table's id once finalized.
    #[getter]
    fn id(&self) -> ::std::string::String {
        self.0.id.clone()
    }

    /// Name of the staging table, relative to the parent schema.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Name of the parent schema relative to its parent catalog.
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }

    /// Name of the parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Storage location allocated for the staging table, where the client writes
    /// the initial Delta commit. Immutable once allocated.
    #[getter]
    fn staging_location(&self) -> ::std::string::String {
        self.0.staging_location.clone()
    }

    /// Username of the principal that created the staging table. Only this
    /// principal may finalize the table.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Whether the staging table has been finalized into a managed table.
    #[getter]
    fn stage_committed(&self) -> bool {
        self.0.stage_committed
    }

    /// Time at which this staging table was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("id", false),
                ("name", false),
                ("schema_name", false),
                ("catalog_name", false),
                ("staging_location", false),
                ("created_by", false),
                ("stage_committed", false),
                ("created_at", false),
            ],
        )
    }
}

impl ::core::convert::From<super::staging_tables::v1::StagingTable> for PyStagingTable {
    fn from(value: super::staging_tables::v1::StagingTable) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyStagingTable> for super::staging_tables::v1::StagingTable {
    fn from(value: PyStagingTable) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "Column",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyColumn(pub super::tables::v1::Column);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyColumn {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            type_text = None,
            type_json = None,
            position = None,
            type_name = None,
            type_precision = None,
            type_scale = None,
            type_interval_type = None,
            comment = None,
            nullable = None,
            partition_index = None,
            column_id = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        type_text: ::core::option::Option<::std::string::String>,
        type_json: ::core::option::Option<::std::string::String>,
        position: ::core::option::Option<i32>,
        type_name: ::core::option::Option<PyColumnTypeName>,
        type_precision: ::core::option::Option<i32>,
        type_scale: ::core::option::Option<i32>,
        type_interval_type: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        nullable: ::core::option::Option<bool>,
        partition_index: ::core::option::Option<i32>,
        column_id: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::tables::v1::Column as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = type_text {
            inner.type_text = value;
        }
        if let ::core::option::Option::Some(value) = type_json {
            inner.type_json = value;
        }
        {
            let value = position;
            inner.position = value;
        }
        {
            let value = type_name;
            inner.type_name = value
                .map(|e| {
                    ::buffa::EnumValue::Known(
                        <super::tables::v1::ColumnTypeName as ::core::convert::From<_>>::from(e),
                    )
                })
                .unwrap_or_default();
        }
        {
            let value = type_precision;
            inner.type_precision = value;
        }
        {
            let value = type_scale;
            inner.type_scale = value;
        }
        {
            let value = type_interval_type;
            inner.type_interval_type = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = nullable;
            inner.nullable = value;
        }
        {
            let value = partition_index;
            inner.partition_index = value;
        }
        {
            let value = column_id;
            inner.column_id = value;
        }
        Self(inner)
    }

    /// Name of the column
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Full data type specification as SQL/catalogString text.
    #[getter]
    fn type_text(&self) -> ::std::string::String {
        self.0.type_text.clone()
    }

    /// Full data type specification, JSON-serialized.
    #[getter]
    fn type_json(&self) -> ::std::string::String {
        self.0.type_json.clone()
    }

    /// Ordinal position of column (starting at position 0).
    #[getter]
    fn position(&self) -> ::core::option::Option<i32> {
        self.0.position
    }

    /// Data type name.
    #[getter]
    fn type_name(&self) -> ::core::option::Option<PyColumnTypeName> {
        self.0
            .type_name
            .as_known()
            .filter(|e| <super::tables::v1::ColumnTypeName as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyColumnTypeName::from)
    }

    /// Digits of precision; required for DecimalTypes.
    #[getter]
    fn type_precision(&self) -> ::core::option::Option<i32> {
        self.0.type_precision
    }

    /// Digits to right of decimal; Required for DecimalTypes.
    #[getter]
    fn type_scale(&self) -> ::core::option::Option<i32> {
        self.0.type_scale
    }

    /// Format of IntervalType.
    #[getter]
    fn type_interval_type(&self) -> ::core::option::Option<::std::string::String> {
        self.0.type_interval_type.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Whether field may be Null (default: true).
    #[getter]
    fn nullable(&self) -> ::core::option::Option<bool> {
        self.0.nullable
    }

    /// Partition index for column.
    #[getter]
    fn partition_index(&self) -> ::core::option::Option<i32> {
        self.0.partition_index
    }

    /// a unique id for the column
    #[getter]
    fn column_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.column_id.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("type_text", false),
                ("type_json", false),
                ("position", false),
                ("type_name", false),
                ("type_precision", false),
                ("type_scale", false),
                ("type_interval_type", false),
                ("comment", false),
                ("nullable", false),
                ("partition_index", false),
                ("column_id", false),
            ],
        )
    }
}

impl ::core::convert::From<super::tables::v1::Column> for PyColumn {
    fn from(value: super::tables::v1::Column) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyColumn> for super::tables::v1::Column {
    fn from(value: PyColumn) -> Self {
        value.0
    }
}

/// A dependency of a SQL object. Exactly one of the fields must be set.
#[::pyo3::pyclass(
    name = "Dependency",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyDependency(pub super::tables::v1::Dependency);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDependency {
    #[new]
    #[pyo3(signature = (table = None, function = None))]
    fn new(
        table: ::core::option::Option<PyTableDependency>,
        function: ::core::option::Option<PyFunctionDependency>,
    ) -> Self {
        let mut inner = <super::tables::v1::Dependency as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = table {
            inner.dependency =
                ::core::option::Option::Some(super::tables::v1::dependency::Dependency::Table(
                    ::std::boxed::Box::new(value.into()),
                ));
        }
        if let ::core::option::Option::Some(value) = function {
            inner.dependency =
                ::core::option::Option::Some(super::tables::v1::dependency::Dependency::Function(
                    ::std::boxed::Box::new(value.into()),
                ));
        }
        Self(inner)
    }

    #[getter]
    fn table(&self) -> ::core::option::Option<PyTableDependency> {
        match &self.0.dependency {
            ::core::option::Option::Some(super::tables::v1::dependency::Dependency::Table(
                value,
            )) => ::core::option::Option::Some(<PyTableDependency>::from((**value).clone())),
            _ => ::core::option::Option::None,
        }
    }

    #[getter]
    fn function(&self) -> ::core::option::Option<PyFunctionDependency> {
        match &self.0.dependency {
            ::core::option::Option::Some(super::tables::v1::dependency::Dependency::Function(
                value,
            )) => ::core::option::Option::Some(<PyFunctionDependency>::from((**value).clone())),
            _ => ::core::option::Option::None,
        }
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("table", false), ("function", false)])
    }
}

impl ::core::convert::From<super::tables::v1::Dependency> for PyDependency {
    fn from(value: super::tables::v1::Dependency) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyDependency> for super::tables::v1::Dependency {
    fn from(value: PyDependency) -> Self {
        value.0
    }
}

/// A list of dependencies referenced by a view-like table.
#[::pyo3::pyclass(
    name = "DependencyList",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyDependencyList(pub super::tables::v1::DependencyList);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDependencyList {
    #[new]
    #[pyo3(signature = (dependencies = None))]
    fn new(dependencies: ::core::option::Option<::std::vec::Vec<PyDependency>>) -> Self {
        let mut inner = <super::tables::v1::DependencyList as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = dependencies {
            inner.dependencies = value.into_iter().map(::core::convert::Into::into).collect();
        }
        Self(inner)
    }

    /// Array of dependencies.
    #[getter]
    fn dependencies(&self) -> ::std::vec::Vec<PyDependency> {
        self.0
            .dependencies
            .iter()
            .cloned()
            .map(PyDependency::from)
            .collect()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("dependencies", false)])
    }
}

impl ::core::convert::From<super::tables::v1::DependencyList> for PyDependencyList {
    fn from(value: super::tables::v1::DependencyList) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyDependencyList> for super::tables::v1::DependencyList {
    fn from(value: PyDependencyList) -> Self {
        value.0
    }
}

/// A function that a SQL object (such as a view or metric view) depends on.
#[::pyo3::pyclass(
    name = "FunctionDependency",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyFunctionDependency(pub super::tables::v1::FunctionDependency);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyFunctionDependency {
    #[new]
    #[pyo3(signature = (function_full_name = None))]
    fn new(function_full_name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::tables::v1::FunctionDependency as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = function_full_name {
            inner.function_full_name = value;
        }
        Self(inner)
    }

    /// Full name of the dependent function, in the form of
    /// catalog_name.schema_name.function_name.
    #[getter]
    fn function_full_name(&self) -> ::std::string::String {
        self.0.function_full_name.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("function_full_name", false)])
    }
}

impl ::core::convert::From<super::tables::v1::FunctionDependency> for PyFunctionDependency {
    fn from(value: super::tables::v1::FunctionDependency) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyFunctionDependency> for super::tables::v1::FunctionDependency {
    fn from(value: PyFunctionDependency) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "GetTableExistsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyGetTableExistsResponse(pub super::tables::v1::GetTableExistsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetTableExistsResponse {
    /// Boolean reflecting if table exists.
    #[getter]
    fn table_exists(&self) -> bool {
        self.0.table_exists
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("table_exists", false)])
    }
}

impl ::core::convert::From<super::tables::v1::GetTableExistsResponse> for PyGetTableExistsResponse {
    fn from(value: super::tables::v1::GetTableExistsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyGetTableExistsResponse> for super::tables::v1::GetTableExistsResponse {
    fn from(value: PyGetTableExistsResponse) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "ListTableSummariesResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListTableSummariesResponse(pub super::tables::v1::ListTableSummariesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListTableSummariesResponse {
    /// The table summaries returned.
    #[getter]
    fn tables(&self) -> ::std::vec::Vec<PyTableSummary> {
        self.0
            .tables
            .iter()
            .cloned()
            .map(PyTableSummary::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("tables", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::tables::v1::ListTableSummariesResponse>
    for PyListTableSummariesResponse
{
    fn from(value: super::tables::v1::ListTableSummariesResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListTableSummariesResponse>
    for super::tables::v1::ListTableSummariesResponse
{
    fn from(value: PyListTableSummariesResponse) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "ListTablesResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListTablesResponse(pub super::tables::v1::ListTablesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListTablesResponse {
    /// The tables returned.
    #[getter]
    fn tables(&self) -> ::std::vec::Vec<PyTable> {
        self.0.tables.iter().cloned().map(PyTable::from).collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("tables", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::tables::v1::ListTablesResponse> for PyListTablesResponse {
    fn from(value: super::tables::v1::ListTablesResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListTablesResponse> for super::tables::v1::ListTablesResponse {
    fn from(value: PyListTablesResponse) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "Table",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyTable(pub super::tables::v1::Table);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyTable {
    /// Name of table, relative to parent schema.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Name of parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Name of parent schema.
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }

    #[getter]
    fn table_type(&self) -> ::core::option::Option<PyTableType> {
        self.0
            .table_type
            .as_known()
            .filter(|e| <super::tables::v1::TableType as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyTableType::from)
    }

    /// Data source format of the table.
    #[getter]
    fn data_source_format(&self) -> ::core::option::Option<PyDataSourceFormat> {
        self.0
            .data_source_format
            .as_known()
            .filter(|e| {
                <super::tables::v1::DataSourceFormat as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PyDataSourceFormat::from)
    }

    /// The array of Column definitions of the table's columns.
    #[getter]
    fn columns(&self) -> ::std::vec::Vec<PyColumn> {
        self.0.columns.iter().cloned().map(PyColumn::from).collect()
    }

    /// Storage root URL for table (for MANAGED, EXTERNAL tables)
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }

    /// Definition text for view-like table types (VIEW, MATERIALIZED_VIEW,
    /// STREAMING_TABLE, METRIC_VIEW). The format depends on the table type:
    /// SQL for views, YAML for metric views.
    #[getter]
    fn view_definition(&self) -> ::core::option::Option<::std::string::String> {
        self.0.view_definition.clone()
    }

    /// Tables and functions the view-like table reads. For metric views this is
    /// derived from the view_definition by the server (the definition is the
    /// single source of truth).
    ///
    /// When unset, dependencies were not provided; an empty list means there are none.
    #[getter]
    fn view_dependencies(&self) -> ::core::option::Option<PyDependencyList> {
        self.0
            .view_dependencies
            .clone()
            .into_option()
            .map(PyDependencyList::from)
    }

    /// Username of current owner of table.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// A map of key-value properties attached to the securable.
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }

    /// Name of the storage credential, when a storage credential is configured for use with this table.
    #[getter]
    fn storage_credential_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_credential_name.clone()
    }

    /// Full name of table, in form of catalog_name.schema_name.table_name.
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }

    /// Time at which this table was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of table creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this table was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified table.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// Time at which this table was deleted, in epoch milliseconds. Field is omitted if table is not deleted.
    #[getter]
    fn deleted_at(&self) -> ::core::option::Option<i64> {
        self.0.deleted_at
    }

    /// Unique identifier for the table.
    #[getter]
    fn table_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.table_id.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("catalog_name", false),
                ("schema_name", false),
                ("table_type", false),
                ("data_source_format", false),
                ("columns", false),
                ("storage_location", false),
                ("view_definition", false),
                ("view_dependencies", false),
                ("owner", false),
                ("comment", false),
                ("properties", false),
                ("storage_credential_name", false),
                ("full_name", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("deleted_at", false),
                ("table_id", false),
            ],
        )
    }
}

impl ::core::convert::From<super::tables::v1::Table> for PyTable {
    fn from(value: super::tables::v1::Table) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyTable> for super::tables::v1::Table {
    fn from(value: PyTable) -> Self {
        value.0
    }
}

/// A table that a SQL object (such as a view or metric view) depends on.
#[::pyo3::pyclass(
    name = "TableDependency",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyTableDependency(pub super::tables::v1::TableDependency);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyTableDependency {
    #[new]
    #[pyo3(signature = (table_full_name = None))]
    fn new(table_full_name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner = <super::tables::v1::TableDependency as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = table_full_name {
            inner.table_full_name = value;
        }
        Self(inner)
    }

    /// Full name of the dependent table, in the form of
    /// catalog_name.schema_name.table_name.
    #[getter]
    fn table_full_name(&self) -> ::std::string::String {
        self.0.table_full_name.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("table_full_name", false)])
    }
}

impl ::core::convert::From<super::tables::v1::TableDependency> for PyTableDependency {
    fn from(value: super::tables::v1::TableDependency) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyTableDependency> for super::tables::v1::TableDependency {
    fn from(value: PyTableDependency) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "TableSummary",
    eq,
    frozen,
    module = "unitycatalog_client._client.tables",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyTableSummary(pub super::tables::v1::TableSummary);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyTableSummary {
    /// The full name of the table.
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }

    #[getter]
    fn table_type(&self) -> ::core::option::Option<PyTableType> {
        self.0
            .table_type
            .as_known()
            .filter(|e| <super::tables::v1::TableType as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyTableType::from)
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("full_name", false), ("table_type", false)])
    }
}

impl ::core::convert::From<super::tables::v1::TableSummary> for PyTableSummary {
    fn from(value: super::tables::v1::TableSummary) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyTableSummary> for super::tables::v1::TableSummary {
    fn from(value: PyTableSummary) -> Self {
        value.0
    }
}

/// The assignment of a tag to a Unity Catalog entity.
///
/// Unlike a TagPolicy (a governed-tag *definition*), an assignment is the application
/// of a tag to a specific securable. It has no identifier of its own — its identity is
/// the composite of (entity_type, entity_name, tag_key). It is intentionally NOT a
/// `google.api.resource`: assignments are stored as associations between the entity and
/// its tag, not as standalone objects.
#[::pyo3::pyclass(
    name = "EntityTagAssignment",
    eq,
    frozen,
    module = "unitycatalog_client._client.tags",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyEntityTagAssignment(pub super::tags::v1::EntityTagAssignment);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyEntityTagAssignment {
    /// The type of the entity to which the tag is assigned.
    ///
    /// Supported values: catalogs, schemas, tables, columns, volumes.
    #[getter]
    fn entity_type(&self) -> ::std::string::String {
        self.0.entity_type.clone()
    }

    /// The fully qualified name of the entity to which the tag is assigned.
    #[getter]
    fn entity_name(&self) -> ::std::string::String {
        self.0.entity_name.clone()
    }

    /// The key of the tag.
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }

    /// The value of the tag.
    #[getter]
    fn tag_value(&self) -> ::core::option::Option<::std::string::String> {
        self.0.tag_value.clone()
    }

    /// The user or principal who last updated the tag assignment.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// The source of the tag assignment, e.g. user-assigned or system-assigned.
    #[getter]
    fn source_type(&self) -> ::core::option::Option<PyTagAssignmentSourceType> {
        self.0
            .source_type
            .as_ref()
            .and_then(|e| e.as_known())
            .filter(|e| {
                <super::tags::v1::TagAssignmentSourceType as ::buffa::Enumeration>::to_i32(e) != 0
            })
            .map(PyTagAssignmentSourceType::from)
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("entity_type", false),
                ("entity_name", false),
                ("tag_key", false),
                ("tag_value", false),
                ("updated_by", false),
                ("source_type", false),
            ],
        )
    }
}

impl ::core::convert::From<super::tags::v1::EntityTagAssignment> for PyEntityTagAssignment {
    fn from(value: super::tags::v1::EntityTagAssignment) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyEntityTagAssignment> for super::tags::v1::EntityTagAssignment {
    fn from(value: PyEntityTagAssignment) -> Self {
        value.0
    }
}

/// List entity tag assignments response.
#[::pyo3::pyclass(
    name = "ListEntityTagAssignmentsResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.tags",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListEntityTagAssignmentsResponse(
    pub super::tags::v1::ListEntityTagAssignmentsResponse,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListEntityTagAssignmentsResponse {
    /// The tag assignments returned.
    #[getter]
    fn tag_assignments(&self) -> ::std::vec::Vec<PyEntityTagAssignment> {
        self.0
            .tag_assignments
            .iter()
            .cloned()
            .map(PyEntityTagAssignment::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("tag_assignments", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::tags::v1::ListEntityTagAssignmentsResponse>
    for PyListEntityTagAssignmentsResponse
{
    fn from(value: super::tags::v1::ListEntityTagAssignmentsResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListEntityTagAssignmentsResponse>
    for super::tags::v1::ListEntityTagAssignmentsResponse
{
    fn from(value: PyListEntityTagAssignmentsResponse) -> Self {
        value.0
    }
}

/// List tag policies response.
#[::pyo3::pyclass(
    name = "ListTagPoliciesResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.tags",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListTagPoliciesResponse(pub super::tags::v1::ListTagPoliciesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListTagPoliciesResponse {
    /// The tag policies returned.
    #[getter]
    fn tag_policies(&self) -> ::std::vec::Vec<PyTagPolicy> {
        self.0
            .tag_policies
            .iter()
            .cloned()
            .map(PyTagPolicy::from)
            .collect()
    }

    /// The next_page_token value to include in the next List request.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("tag_policies", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::tags::v1::ListTagPoliciesResponse> for PyListTagPoliciesResponse {
    fn from(value: super::tags::v1::ListTagPoliciesResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListTagPoliciesResponse> for super::tags::v1::ListTagPoliciesResponse {
    fn from(value: PyListTagPoliciesResponse) -> Self {
        value.0
    }
}

/// A governed tag definition (tag policy).
///
/// A tag policy defines a tag key together with the rules that govern how it can
/// be used, including the optional set of allowed values. Assigning a governed
/// tag to an entity is done through the Entity Tag Assignments API.
#[::pyo3::pyclass(
    name = "TagPolicy",
    eq,
    frozen,
    module = "unitycatalog_client._client.tags",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyTagPolicy(pub super::tags::v1::TagPolicy);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyTagPolicy {
    /// The key of the governed tag.
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }

    /// User-provided free-form text description of the tag policy.
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }

    /// The set of allowed values for the governed tag.
    ///
    /// When empty, the governed tag does not restrict the values that may be assigned.
    /// The list is unordered.
    #[getter]
    fn values(&self) -> ::std::vec::Vec<PyValue> {
        self.0.values.iter().cloned().map(PyValue::from).collect()
    }

    /// Unique identifier for the tag policy.
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }

    /// Time at which this tag policy was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Time at which this tag policy was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("tag_key", false),
                ("description", false),
                ("values", false),
                ("id", false),
                ("created_at", false),
                ("updated_at", false),
            ],
        )
    }
}

impl ::core::convert::From<super::tags::v1::TagPolicy> for PyTagPolicy {
    fn from(value: super::tags::v1::TagPolicy) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyTagPolicy> for super::tags::v1::TagPolicy {
    fn from(value: PyTagPolicy) -> Self {
        value.0
    }
}

/// An allowed value for a governed tag.
#[::pyo3::pyclass(
    name = "Value",
    eq,
    frozen,
    module = "unitycatalog_client._client.tags",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyValue(pub super::tags::v1::Value);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyValue {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner = <super::tags::v1::Value as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }

    /// The name of the allowed value.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("name", false)])
    }
}

impl ::core::convert::From<super::tags::v1::Value> for PyValue {
    fn from(value: super::tags::v1::Value) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyValue> for super::tags::v1::Value {
    fn from(value: PyValue) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "AwsTemporaryCredentials",
    eq,
    frozen,
    module = "unitycatalog_client._client.temporary_credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAwsTemporaryCredentials(pub super::temporary_credentials::v1::AwsTemporaryCredentials);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAwsTemporaryCredentials {
    /// The access key ID that identifies the temporary credentials.
    #[getter]
    fn access_key_id(&self) -> ::std::string::String {
        self.0.access_key_id.clone()
    }

    /// The Amazon Resource Name (ARN) of the S3 access point for temporary credentials related the external location.
    #[getter]
    fn access_point(&self) -> ::std::string::String {
        self.0.access_point.clone()
    }

    /// The secret access key that can be used to sign AWS API requests.
    #[getter]
    fn secret_access_key(&self) -> ::std::string::String {
        self.0.secret_access_key.clone()
    }

    /// The token that users must pass to AWS API to use the temporary credentials.
    #[getter]
    fn session_token(&self) -> ::std::string::String {
        self.0.session_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("access_key_id", false),
                ("access_point", false),
                ("secret_access_key", false),
                ("session_token", false),
            ],
        )
    }
}

impl ::core::convert::From<super::temporary_credentials::v1::AwsTemporaryCredentials>
    for PyAwsTemporaryCredentials
{
    fn from(value: super::temporary_credentials::v1::AwsTemporaryCredentials) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAwsTemporaryCredentials>
    for super::temporary_credentials::v1::AwsTemporaryCredentials
{
    fn from(value: PyAwsTemporaryCredentials) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "AzureAad",
    eq,
    frozen,
    module = "unitycatalog_client._client.temporary_credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAzureAad(pub super::temporary_credentials::v1::AzureAad);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAzureAad {
    /// Opaque token that contains claims that you can use in Azure Active Directory to access cloud services.
    #[getter]
    fn aad_token(&self) -> ::std::string::String {
        self.0.aad_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("aad_token", false)])
    }
}

impl ::core::convert::From<super::temporary_credentials::v1::AzureAad> for PyAzureAad {
    fn from(value: super::temporary_credentials::v1::AzureAad) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAzureAad> for super::temporary_credentials::v1::AzureAad {
    fn from(value: PyAzureAad) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "AzureUserDelegationSas",
    eq,
    frozen,
    module = "unitycatalog_client._client.temporary_credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyAzureUserDelegationSas(pub super::temporary_credentials::v1::AzureUserDelegationSas);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAzureUserDelegationSas {
    /// The signed URI (SAS Token) used to access blob services for a given path
    #[getter]
    fn sas_token(&self) -> ::std::string::String {
        self.0.sas_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("sas_token", false)])
    }
}

impl ::core::convert::From<super::temporary_credentials::v1::AzureUserDelegationSas>
    for PyAzureUserDelegationSas
{
    fn from(value: super::temporary_credentials::v1::AzureUserDelegationSas) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyAzureUserDelegationSas>
    for super::temporary_credentials::v1::AzureUserDelegationSas
{
    fn from(value: PyAzureUserDelegationSas) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "GcpOauthToken",
    eq,
    frozen,
    module = "unitycatalog_client._client.temporary_credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyGcpOauthToken(pub super::temporary_credentials::v1::GcpOauthToken);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGcpOauthToken {
    /// The OAuth token used to access Google Cloud services.
    #[getter]
    fn oauth_token(&self) -> ::std::string::String {
        self.0.oauth_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(slf.as_any(), &[("oauth_token", false)])
    }
}

impl ::core::convert::From<super::temporary_credentials::v1::GcpOauthToken> for PyGcpOauthToken {
    fn from(value: super::temporary_credentials::v1::GcpOauthToken) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyGcpOauthToken> for super::temporary_credentials::v1::GcpOauthToken {
    fn from(value: PyGcpOauthToken) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "R2TemporaryCredentials",
    eq,
    frozen,
    module = "unitycatalog_client._client.temporary_credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyR2TemporaryCredentials(pub super::temporary_credentials::v1::R2TemporaryCredentials);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyR2TemporaryCredentials {
    /// The access key ID that identifies the temporary credentials.
    #[getter]
    fn access_key_id(&self) -> ::std::string::String {
        self.0.access_key_id.clone()
    }

    /// The secret access key associated with the access key.
    #[getter]
    fn secret_access_key(&self) -> ::std::string::String {
        self.0.secret_access_key.clone()
    }

    /// The generated JWT that users must pass to use the temporary credentials.
    #[getter]
    fn session_token(&self) -> ::std::string::String {
        self.0.session_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("access_key_id", false),
                ("secret_access_key", false),
                ("session_token", false),
            ],
        )
    }
}

impl ::core::convert::From<super::temporary_credentials::v1::R2TemporaryCredentials>
    for PyR2TemporaryCredentials
{
    fn from(value: super::temporary_credentials::v1::R2TemporaryCredentials) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyR2TemporaryCredentials>
    for super::temporary_credentials::v1::R2TemporaryCredentials
{
    fn from(value: PyR2TemporaryCredentials) -> Self {
        value.0
    }
}

/// The response to the GenerateTemporaryTableCredentialsRequest.
#[::pyo3::pyclass(
    name = "TemporaryCredential",
    eq,
    frozen,
    module = "unitycatalog_client._client.temporary_credentials",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyTemporaryCredential(pub super::temporary_credentials::v1::TemporaryCredential);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyTemporaryCredential {
    /// Server time when the credential will expire, in epoch milliseconds.
    /// The API client is advised to cache the credential given this expiration time.
    #[getter]
    fn expiration_time(&self) -> i64 {
        self.0.expiration_time
    }

    /// The URL of the storage path accessible by the temporary credential.
    #[getter]
    fn url(&self) -> ::std::string::String {
        self.0.url.clone()
    }

    ///Credentials for Azure Blob Storage.
    #[getter]
    fn azure_user_delegation_sas(&self) -> ::core::option::Option<PyAzureUserDelegationSas> {
        match &self.0.credentials {
            ::core::option::Option::Some(
                super::temporary_credentials::v1::temporary_credential::Credentials::AzureUserDelegationSas(
                    value,
                ),
            ) => {
                ::core::option::Option::Some(
                    <PyAzureUserDelegationSas>::from((**value).clone()),
                )
            }
            _ => ::core::option::Option::None,
        }
    }

    ///Credentials for Azure Active Directory.
    #[getter]
    fn azure_aad(&self) -> ::core::option::Option<PyAzureAad> {
        match &self.0.credentials {
            ::core::option::Option::Some(
                super::temporary_credentials::v1::temporary_credential::Credentials::AzureAad(
                    value,
                ),
            ) => ::core::option::Option::Some(<PyAzureAad>::from((**value).clone())),
            _ => ::core::option::Option::None,
        }
    }

    ///Credentials for AWS S3.
    #[getter]
    fn aws_temp_credentials(&self) -> ::core::option::Option<PyAwsTemporaryCredentials> {
        match &self.0.credentials {
            ::core::option::Option::Some(
                super::temporary_credentials::v1::temporary_credential::Credentials::AwsTempCredentials(
                    value,
                ),
            ) => {
                ::core::option::Option::Some(
                    <PyAwsTemporaryCredentials>::from((**value).clone()),
                )
            }
            _ => ::core::option::Option::None,
        }
    }

    ///Credentials for Google Cloud Storage.
    #[getter]
    fn gcp_oauth_token(&self) -> ::core::option::Option<PyGcpOauthToken> {
        match &self.0.credentials {
            ::core::option::Option::Some(
                super::temporary_credentials::v1::temporary_credential::Credentials::GcpOauthToken(
                    value,
                ),
            ) => ::core::option::Option::Some(<PyGcpOauthToken>::from((**value).clone())),
            _ => ::core::option::Option::None,
        }
    }

    ///Credentials for R2.
    #[getter]
    fn r2_temp_credentials(&self) -> ::core::option::Option<PyR2TemporaryCredentials> {
        match &self.0.credentials {
            ::core::option::Option::Some(
                super::temporary_credentials::v1::temporary_credential::Credentials::R2TempCredentials(
                    value,
                ),
            ) => {
                ::core::option::Option::Some(
                    <PyR2TemporaryCredentials>::from((**value).clone()),
                )
            }
            _ => ::core::option::Option::None,
        }
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("expiration_time", false),
                ("url", false),
                ("azure_user_delegation_sas", false),
                ("azure_aad", false),
                ("aws_temp_credentials", false),
                ("gcp_oauth_token", false),
                ("r2_temp_credentials", false),
            ],
        )
    }
}

impl ::core::convert::From<super::temporary_credentials::v1::TemporaryCredential>
    for PyTemporaryCredential
{
    fn from(value: super::temporary_credentials::v1::TemporaryCredential) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyTemporaryCredential>
    for super::temporary_credentials::v1::TemporaryCredential
{
    fn from(value: PyTemporaryCredential) -> Self {
        value.0
    }
}

/// List volumes response.
#[::pyo3::pyclass(
    name = "ListVolumesResponse",
    eq,
    frozen,
    module = "unitycatalog_client._client.volumes",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyListVolumesResponse(pub super::volumes::v1::ListVolumesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListVolumesResponse {
    /// The volumes returned.
    #[getter]
    fn volumes(&self) -> ::std::vec::Vec<PyVolume> {
        self.0.volumes.iter().cloned().map(PyVolume::from).collect()
    }

    /// Opaque token to retrieve the next page of results.
    ///
    /// Absent if there are no more pages. page_token should be set to
    /// this value for the next request to retrieve the next page of results.
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[("volumes", false), ("next_page_token", false)],
        )
    }
}

impl ::core::convert::From<super::volumes::v1::ListVolumesResponse> for PyListVolumesResponse {
    fn from(value: super::volumes::v1::ListVolumesResponse) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyListVolumesResponse> for super::volumes::v1::ListVolumesResponse {
    fn from(value: PyListVolumesResponse) -> Self {
        value.0
    }
}

#[::pyo3::pyclass(
    name = "Volume",
    eq,
    frozen,
    module = "unitycatalog_client._client.volumes",
    from_py_object
)]
#[derive(Clone, Debug, PartialEq)]
pub struct PyVolume(pub super::volumes::v1::Volume);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyVolume {
    /// Name of volume, relative to parent schema.
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }

    /// Name of parent catalog.
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }

    /// Name of parent schema.
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }

    /// The three-level (fully qualified) name of the volume
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }

    /// The storage location on the cloud
    #[getter]
    fn storage_location(&self) -> ::std::string::String {
        self.0.storage_location.clone()
    }

    /// The unique identifier of the volume
    #[getter]
    fn volume_id(&self) -> ::std::string::String {
        self.0.volume_id.clone()
    }

    /// The type of the volume.
    ///
    /// An external volume is located in the specified external location.
    /// A managed volume is located in the default location which is specified
    /// by the parent schema, or the parent catalog, or the Metastore.
    #[getter]
    fn volume_type(&self) -> ::core::option::Option<PyVolumeType> {
        self.0
            .volume_type
            .as_known()
            .filter(|e| <super::volumes::v1::VolumeType as ::buffa::Enumeration>::to_i32(e) != 0)
            .map(PyVolumeType::from)
    }

    /// Username of current owner of volume.
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }

    /// User-provided free-form text description.
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }

    /// Time at which this volume was created, in epoch milliseconds.
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }

    /// Username of volume creator.
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }

    /// Time at which this volume was last updated, in epoch milliseconds.
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }

    /// Username of user who last modified the volume.
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }

    /// Indicates whether the principal is limited to retrieving metadata
    /// for the associated object through the BROWSE privilege when include_browse
    /// is enabled in the request.
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }

    /// The unique identifier of the metastore
    #[getter]
    fn metastore_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.metastore_id.clone()
    }

    fn __repr__(slf: &::pyo3::Bound<'_, Self>) -> ::pyo3::PyResult<::std::string::String> {
        __python_repr(
            slf.as_any(),
            &[
                ("name", false),
                ("catalog_name", false),
                ("schema_name", false),
                ("full_name", false),
                ("storage_location", false),
                ("volume_id", false),
                ("volume_type", false),
                ("owner", false),
                ("comment", false),
                ("created_at", false),
                ("created_by", false),
                ("updated_at", false),
                ("updated_by", false),
                ("browse_only", false),
                ("metastore_id", false),
            ],
        )
    }
}

impl ::core::convert::From<super::volumes::v1::Volume> for PyVolume {
    fn from(value: super::volumes::v1::Volume) -> Self {
        Self(value)
    }
}

impl ::core::convert::From<PyVolume> for super::volumes::v1::Volume {
    fn from(value: PyVolume) -> Self {
        value.0
    }
}

/// `Name(field=value, …)` without the `None` fields; a redacted field shows `'***'`.
fn __python_repr(
    obj: &::pyo3::Bound<'_, ::pyo3::PyAny>,
    fields: &[(&str, bool)],
) -> ::pyo3::PyResult<::std::string::String> {
    use ::pyo3::types::{PyAnyMethods, PyTypeMethods};
    let mut shown = ::std::vec::Vec::with_capacity(fields.len());
    for (field, redact) in fields {
        let value = obj.getattr(*field)?;
        if value.is_none() {
            continue;
        }
        if *redact {
            shown.push(::std::format!("{field}='***'"));
        } else {
            shown.push(::std::format!("{field}={}", value.repr()?));
        }
    }
    Ok(::std::format!(
        "{}({})",
        obj.get_type().name()?,
        shown.join(", ")
    ))
}
