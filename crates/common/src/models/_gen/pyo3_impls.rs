// @generated — do not edit by hand.
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "AgentSkillType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyAgentSkillType {
    AGENT_SKILL_TYPE_UNSPECIFIED = 0isize,
    EXTERNAL = 1isize,
    MANAGED = 2isize,
}
impl PyAgentSkillType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyAgentSkillType::AGENT_SKILL_TYPE_UNSPECIFIED => 0i32,
            PyAgentSkillType::EXTERNAL => 1i32,
            PyAgentSkillType::MANAGED => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyAgentSkillType::AGENT_SKILL_TYPE_UNSPECIFIED,
            1i32 => PyAgentSkillType::EXTERNAL,
            2i32 => PyAgentSkillType::MANAGED,
            _ => PyAgentSkillType::AGENT_SKILL_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::agent_skills::v0alpha1::AgentSkillType> for PyAgentSkillType {
    fn from(value: super::agent_skills::v0alpha1::AgentSkillType) -> Self {
        PyAgentSkillType::__from_proto_i32(
            <super::agent_skills::v0alpha1::AgentSkillType as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyAgentSkillType> for super::agent_skills::v0alpha1::AgentSkillType {
    fn from(value: PyAgentSkillType) -> Self {
        let n = value.__to_proto_i32();
        <super::agent_skills::v0alpha1::AgentSkillType as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "InvocationProtocol")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyInvocationProtocol {
    INVOCATION_PROTOCOL_UNSPECIFIED = 0isize,
    MCP = 1isize,
    A2A = 2isize,
    OPENAI = 3isize,
    ANTHROPIC = 4isize,
    REST = 5isize,
}
impl PyInvocationProtocol {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyInvocationProtocol::INVOCATION_PROTOCOL_UNSPECIFIED => 0i32,
            PyInvocationProtocol::MCP => 1i32,
            PyInvocationProtocol::A2A => 2i32,
            PyInvocationProtocol::OPENAI => 3i32,
            PyInvocationProtocol::ANTHROPIC => 4i32,
            PyInvocationProtocol::REST => 5i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyInvocationProtocol::INVOCATION_PROTOCOL_UNSPECIFIED,
            1i32 => PyInvocationProtocol::MCP,
            2i32 => PyInvocationProtocol::A2A,
            3i32 => PyInvocationProtocol::OPENAI,
            4i32 => PyInvocationProtocol::ANTHROPIC,
            5i32 => PyInvocationProtocol::REST,
            _ => PyInvocationProtocol::INVOCATION_PROTOCOL_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::agents::v0alpha1::InvocationProtocol> for PyInvocationProtocol {
    fn from(value: super::agents::v0alpha1::InvocationProtocol) -> Self {
        PyInvocationProtocol::__from_proto_i32(
            <super::agents::v0alpha1::InvocationProtocol as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyInvocationProtocol> for super::agents::v0alpha1::InvocationProtocol {
    fn from(value: PyInvocationProtocol) -> Self {
        let n = value.__to_proto_i32();
        <super::agents::v0alpha1::InvocationProtocol as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "CatalogType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyCatalogType {
    CATALOG_TYPE_UNSPECIFIED = 0isize,
    MANAGED_CATALOG = 1isize,
    DELTASHARING_CATALOG = 2isize,
    SYSTEM_CATALOG = 3isize,
}
impl PyCatalogType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyCatalogType::CATALOG_TYPE_UNSPECIFIED => 0i32,
            PyCatalogType::MANAGED_CATALOG => 1i32,
            PyCatalogType::DELTASHARING_CATALOG => 2i32,
            PyCatalogType::SYSTEM_CATALOG => 3i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyCatalogType::CATALOG_TYPE_UNSPECIFIED,
            1i32 => PyCatalogType::MANAGED_CATALOG,
            2i32 => PyCatalogType::DELTASHARING_CATALOG,
            3i32 => PyCatalogType::SYSTEM_CATALOG,
            _ => PyCatalogType::CATALOG_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::catalogs::v1::CatalogType> for PyCatalogType {
    fn from(value: super::catalogs::v1::CatalogType) -> Self {
        PyCatalogType::__from_proto_i32(
            <super::catalogs::v1::CatalogType as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyCatalogType> for super::catalogs::v1::CatalogType {
    fn from(value: PyCatalogType) -> Self {
        let n = value.__to_proto_i32();
        <super::catalogs::v1::CatalogType as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "Purpose")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyPurpose {
    PURPOSE_UNSPECIFIED = 0isize,
    STORAGE = 1isize,
    SERVICE = 2isize,
}
impl PyPurpose {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyPurpose::PURPOSE_UNSPECIFIED => 0i32,
            PyPurpose::STORAGE => 1i32,
            PyPurpose::SERVICE => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyPurpose::PURPOSE_UNSPECIFIED,
            1i32 => PyPurpose::STORAGE,
            2i32 => PyPurpose::SERVICE,
            _ => PyPurpose::PURPOSE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::credentials::v1::Purpose> for PyPurpose {
    fn from(value: super::credentials::v1::Purpose) -> Self {
        PyPurpose::__from_proto_i32(
            <super::credentials::v1::Purpose as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyPurpose> for super::credentials::v1::Purpose {
    fn from(value: PyPurpose) -> Self {
        let n = value.__to_proto_i32();
        <super::credentials::v1::Purpose as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "FunctionParameterType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyFunctionParameterType {
    FUNCTION_PARAMETER_TYPE_UNSPECIFIED = 0isize,
    COLUMN = 1isize,
    PARAM = 2isize,
}
impl PyFunctionParameterType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyFunctionParameterType::FUNCTION_PARAMETER_TYPE_UNSPECIFIED => 0i32,
            PyFunctionParameterType::COLUMN => 1i32,
            PyFunctionParameterType::PARAM => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyFunctionParameterType::FUNCTION_PARAMETER_TYPE_UNSPECIFIED,
            1i32 => PyFunctionParameterType::COLUMN,
            2i32 => PyFunctionParameterType::PARAM,
            _ => PyFunctionParameterType::FUNCTION_PARAMETER_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::functions::v1::FunctionParameterType>
    for PyFunctionParameterType
{
    fn from(value: super::functions::v1::FunctionParameterType) -> Self {
        PyFunctionParameterType::__from_proto_i32(
            <super::functions::v1::FunctionParameterType as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyFunctionParameterType>
    for super::functions::v1::FunctionParameterType
{
    fn from(value: PyFunctionParameterType) -> Self {
        let n = value.__to_proto_i32();
        <super::functions::v1::FunctionParameterType as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "ParameterMode")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyParameterMode {
    PARAMETER_MODE_UNSPECIFIED = 0isize,
    IN = 1isize,
}
impl PyParameterMode {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyParameterMode::PARAMETER_MODE_UNSPECIFIED => 0i32,
            PyParameterMode::IN => 1i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyParameterMode::PARAMETER_MODE_UNSPECIFIED,
            1i32 => PyParameterMode::IN,
            _ => PyParameterMode::PARAMETER_MODE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::functions::v1::ParameterMode> for PyParameterMode {
    fn from(value: super::functions::v1::ParameterMode) -> Self {
        PyParameterMode::__from_proto_i32(
            <super::functions::v1::ParameterMode as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyParameterMode> for super::functions::v1::ParameterMode {
    fn from(value: PyParameterMode) -> Self {
        let n = value.__to_proto_i32();
        <super::functions::v1::ParameterMode as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "ParameterStyle")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyParameterStyle {
    PARAMETER_STYLE_UNSPECIFIED = 0isize,
    S = 1isize,
}
impl PyParameterStyle {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyParameterStyle::PARAMETER_STYLE_UNSPECIFIED => 0i32,
            PyParameterStyle::S => 1i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyParameterStyle::PARAMETER_STYLE_UNSPECIFIED,
            1i32 => PyParameterStyle::S,
            _ => PyParameterStyle::PARAMETER_STYLE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::functions::v1::ParameterStyle> for PyParameterStyle {
    fn from(value: super::functions::v1::ParameterStyle) -> Self {
        PyParameterStyle::__from_proto_i32(
            <super::functions::v1::ParameterStyle as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyParameterStyle> for super::functions::v1::ParameterStyle {
    fn from(value: PyParameterStyle) -> Self {
        let n = value.__to_proto_i32();
        <super::functions::v1::ParameterStyle as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "RoutineBody")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyRoutineBody {
    ROUTINE_BODY_UNSPECIFIED = 0isize,
    SQL = 1isize,
    EXTERNAL = 2isize,
}
impl PyRoutineBody {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyRoutineBody::ROUTINE_BODY_UNSPECIFIED => 0i32,
            PyRoutineBody::SQL => 1i32,
            PyRoutineBody::EXTERNAL => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyRoutineBody::ROUTINE_BODY_UNSPECIFIED,
            1i32 => PyRoutineBody::SQL,
            2i32 => PyRoutineBody::EXTERNAL,
            _ => PyRoutineBody::ROUTINE_BODY_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::functions::v1::RoutineBody> for PyRoutineBody {
    fn from(value: super::functions::v1::RoutineBody) -> Self {
        PyRoutineBody::__from_proto_i32(
            <super::functions::v1::RoutineBody as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyRoutineBody> for super::functions::v1::RoutineBody {
    fn from(value: PyRoutineBody) -> Self {
        let n = value.__to_proto_i32();
        <super::functions::v1::RoutineBody as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "SecurityType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PySecurityType {
    SECURITY_TYPE_UNSPECIFIED = 0isize,
    DEFINER = 1isize,
}
impl PySecurityType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PySecurityType::SECURITY_TYPE_UNSPECIFIED => 0i32,
            PySecurityType::DEFINER => 1i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PySecurityType::SECURITY_TYPE_UNSPECIFIED,
            1i32 => PySecurityType::DEFINER,
            _ => PySecurityType::SECURITY_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::functions::v1::SecurityType> for PySecurityType {
    fn from(value: super::functions::v1::SecurityType) -> Self {
        PySecurityType::__from_proto_i32(
            <super::functions::v1::SecurityType as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PySecurityType> for super::functions::v1::SecurityType {
    fn from(value: PySecurityType) -> Self {
        let n = value.__to_proto_i32();
        <super::functions::v1::SecurityType as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "SqlDataAccess")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PySqlDataAccess {
    SQL_DATA_ACCESS_UNSPECIFIED = 0isize,
    CONTAINS_SQL = 1isize,
    READS_SQL_DATA = 2isize,
    NO_SQL = 3isize,
}
impl PySqlDataAccess {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PySqlDataAccess::SQL_DATA_ACCESS_UNSPECIFIED => 0i32,
            PySqlDataAccess::CONTAINS_SQL => 1i32,
            PySqlDataAccess::READS_SQL_DATA => 2i32,
            PySqlDataAccess::NO_SQL => 3i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PySqlDataAccess::SQL_DATA_ACCESS_UNSPECIFIED,
            1i32 => PySqlDataAccess::CONTAINS_SQL,
            2i32 => PySqlDataAccess::READS_SQL_DATA,
            3i32 => PySqlDataAccess::NO_SQL,
            _ => PySqlDataAccess::SQL_DATA_ACCESS_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::functions::v1::SqlDataAccess> for PySqlDataAccess {
    fn from(value: super::functions::v1::SqlDataAccess) -> Self {
        PySqlDataAccess::__from_proto_i32(
            <super::functions::v1::SqlDataAccess as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PySqlDataAccess> for super::functions::v1::SqlDataAccess {
    fn from(value: PySqlDataAccess) -> Self {
        let n = value.__to_proto_i32();
        <super::functions::v1::SqlDataAccess as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "PolicyType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyPolicyType {
    POLICY_TYPE_UNSPECIFIED = 0isize,
    POLICY_TYPE_ROW_FILTER = 1isize,
    POLICY_TYPE_COLUMN_MASK = 2isize,
}
impl PyPolicyType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyPolicyType::POLICY_TYPE_UNSPECIFIED => 0i32,
            PyPolicyType::POLICY_TYPE_ROW_FILTER => 1i32,
            PyPolicyType::POLICY_TYPE_COLUMN_MASK => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyPolicyType::POLICY_TYPE_UNSPECIFIED,
            1i32 => PyPolicyType::POLICY_TYPE_ROW_FILTER,
            2i32 => PyPolicyType::POLICY_TYPE_COLUMN_MASK,
            _ => PyPolicyType::POLICY_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::policies::v1::PolicyType> for PyPolicyType {
    fn from(value: super::policies::v1::PolicyType) -> Self {
        PyPolicyType::__from_proto_i32(
            <super::policies::v1::PolicyType as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyPolicyType> for super::policies::v1::PolicyType {
    fn from(value: PyPolicyType) -> Self {
        let n = value.__to_proto_i32();
        <super::policies::v1::PolicyType as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "ProviderAuthenticationType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyProviderAuthenticationType {
    PROVIDER_AUTHENTICATION_TYPE_UNSPECIFIED = 0isize,
    TOKEN = 1isize,
    OAUTH_CLIENT_CREDENTIALS = 2isize,
}
impl PyProviderAuthenticationType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyProviderAuthenticationType::PROVIDER_AUTHENTICATION_TYPE_UNSPECIFIED => 0i32,
            PyProviderAuthenticationType::TOKEN => 1i32,
            PyProviderAuthenticationType::OAUTH_CLIENT_CREDENTIALS => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyProviderAuthenticationType::PROVIDER_AUTHENTICATION_TYPE_UNSPECIFIED,
            1i32 => PyProviderAuthenticationType::TOKEN,
            2i32 => PyProviderAuthenticationType::OAUTH_CLIENT_CREDENTIALS,
            _ => PyProviderAuthenticationType::PROVIDER_AUTHENTICATION_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::providers::v1::ProviderAuthenticationType>
    for PyProviderAuthenticationType
{
    fn from(value: super::providers::v1::ProviderAuthenticationType) -> Self {
        PyProviderAuthenticationType::__from_proto_i32(
            <super::providers::v1::ProviderAuthenticationType as ::buffa::Enumeration>::to_i32(
                &value,
            ),
        )
    }
}
impl ::core::convert::From<PyProviderAuthenticationType>
    for super::providers::v1::ProviderAuthenticationType
{
    fn from(value: PyProviderAuthenticationType) -> Self {
        let n = value.__to_proto_i32();
        <super::providers::v1::ProviderAuthenticationType as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "AuthenticationType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyAuthenticationType {
    AUTHENTICATION_TYPE_UNSPECIFIED = 0isize,
    TOKEN = 1isize,
    OAUTH_CLIENT_CREDENTIALS = 2isize,
}
impl PyAuthenticationType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyAuthenticationType::AUTHENTICATION_TYPE_UNSPECIFIED => 0i32,
            PyAuthenticationType::TOKEN => 1i32,
            PyAuthenticationType::OAUTH_CLIENT_CREDENTIALS => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyAuthenticationType::AUTHENTICATION_TYPE_UNSPECIFIED,
            1i32 => PyAuthenticationType::TOKEN,
            2i32 => PyAuthenticationType::OAUTH_CLIENT_CREDENTIALS,
            _ => PyAuthenticationType::AUTHENTICATION_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::recipients::v1::AuthenticationType> for PyAuthenticationType {
    fn from(value: super::recipients::v1::AuthenticationType) -> Self {
        PyAuthenticationType::__from_proto_i32(
            <super::recipients::v1::AuthenticationType as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyAuthenticationType> for super::recipients::v1::AuthenticationType {
    fn from(value: PyAuthenticationType) -> Self {
        let n = value.__to_proto_i32();
        <super::recipients::v1::AuthenticationType as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "Action")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyAction {
    ACTION_UNSPECIFIED = 0isize,
    ADD = 1isize,
    REMOVE = 2isize,
    UPDATE = 3isize,
}
impl PyAction {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyAction::ACTION_UNSPECIFIED => 0i32,
            PyAction::ADD => 1i32,
            PyAction::REMOVE => 2i32,
            PyAction::UPDATE => 3i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyAction::ACTION_UNSPECIFIED,
            1i32 => PyAction::ADD,
            2i32 => PyAction::REMOVE,
            3i32 => PyAction::UPDATE,
            _ => PyAction::ACTION_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::shares::v1::Action> for PyAction {
    fn from(value: super::shares::v1::Action) -> Self {
        PyAction::__from_proto_i32(<super::shares::v1::Action as ::buffa::Enumeration>::to_i32(
            &value,
        ))
    }
}
impl ::core::convert::From<PyAction> for super::shares::v1::Action {
    fn from(value: PyAction) -> Self {
        let n = value.__to_proto_i32();
        <super::shares::v1::Action as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "DataObjectType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyDataObjectType {
    DATA_OBJECT_TYPE_UNSPECIFIED = 0isize,
    TABLE = 1isize,
    SCHEMA = 2isize,
    VOLUME = 10isize,
    AGENT_SKILL = 11isize,
}
impl PyDataObjectType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyDataObjectType::DATA_OBJECT_TYPE_UNSPECIFIED => 0i32,
            PyDataObjectType::TABLE => 1i32,
            PyDataObjectType::SCHEMA => 2i32,
            PyDataObjectType::VOLUME => 10i32,
            PyDataObjectType::AGENT_SKILL => 11i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyDataObjectType::DATA_OBJECT_TYPE_UNSPECIFIED,
            1i32 => PyDataObjectType::TABLE,
            2i32 => PyDataObjectType::SCHEMA,
            10i32 => PyDataObjectType::VOLUME,
            11i32 => PyDataObjectType::AGENT_SKILL,
            _ => PyDataObjectType::DATA_OBJECT_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::shares::v1::DataObjectType> for PyDataObjectType {
    fn from(value: super::shares::v1::DataObjectType) -> Self {
        PyDataObjectType::__from_proto_i32(
            <super::shares::v1::DataObjectType as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyDataObjectType> for super::shares::v1::DataObjectType {
    fn from(value: PyDataObjectType) -> Self {
        let n = value.__to_proto_i32();
        <super::shares::v1::DataObjectType as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "HistoryStatus")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyHistoryStatus {
    DISABLED = 0isize,
    ENABLED = 1isize,
}
impl PyHistoryStatus {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyHistoryStatus::DISABLED => 0i32,
            PyHistoryStatus::ENABLED => 1i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyHistoryStatus::DISABLED,
            1i32 => PyHistoryStatus::ENABLED,
            _ => PyHistoryStatus::DISABLED,
        }
    }
}
impl ::core::convert::From<super::shares::v1::HistoryStatus> for PyHistoryStatus {
    fn from(value: super::shares::v1::HistoryStatus) -> Self {
        PyHistoryStatus::__from_proto_i32(
            <super::shares::v1::HistoryStatus as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyHistoryStatus> for super::shares::v1::HistoryStatus {
    fn from(value: PyHistoryStatus) -> Self {
        let n = value.__to_proto_i32();
        <super::shares::v1::HistoryStatus as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "ColumnTypeName")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyColumnTypeName {
    COLUMN_TYPE_NAME_UNSPECIFIED = 0isize,
    BOOLEAN = 1isize,
    BYTE = 2isize,
    SHORT = 3isize,
    INT = 4isize,
    LONG = 5isize,
    FLOAT = 6isize,
    DOUBLE = 7isize,
    DATE = 8isize,
    TIMESTAMP = 9isize,
    STRING = 10isize,
    BINARY = 11isize,
    DECIMAL = 12isize,
    INTERVAL = 13isize,
    ARRAY = 14isize,
    STRUCT = 15isize,
    MAP = 16isize,
    CHAR = 17isize,
    NULL = 18isize,
    USER_DEFINED_TYPE = 19isize,
    TIMESTAMP_NTZ = 20isize,
    VARIANT = 21isize,
    TABLE_TYPE = 22isize,
}
impl PyColumnTypeName {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyColumnTypeName::COLUMN_TYPE_NAME_UNSPECIFIED => 0i32,
            PyColumnTypeName::BOOLEAN => 1i32,
            PyColumnTypeName::BYTE => 2i32,
            PyColumnTypeName::SHORT => 3i32,
            PyColumnTypeName::INT => 4i32,
            PyColumnTypeName::LONG => 5i32,
            PyColumnTypeName::FLOAT => 6i32,
            PyColumnTypeName::DOUBLE => 7i32,
            PyColumnTypeName::DATE => 8i32,
            PyColumnTypeName::TIMESTAMP => 9i32,
            PyColumnTypeName::STRING => 10i32,
            PyColumnTypeName::BINARY => 11i32,
            PyColumnTypeName::DECIMAL => 12i32,
            PyColumnTypeName::INTERVAL => 13i32,
            PyColumnTypeName::ARRAY => 14i32,
            PyColumnTypeName::STRUCT => 15i32,
            PyColumnTypeName::MAP => 16i32,
            PyColumnTypeName::CHAR => 17i32,
            PyColumnTypeName::NULL => 18i32,
            PyColumnTypeName::USER_DEFINED_TYPE => 19i32,
            PyColumnTypeName::TIMESTAMP_NTZ => 20i32,
            PyColumnTypeName::VARIANT => 21i32,
            PyColumnTypeName::TABLE_TYPE => 22i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyColumnTypeName::COLUMN_TYPE_NAME_UNSPECIFIED,
            1i32 => PyColumnTypeName::BOOLEAN,
            2i32 => PyColumnTypeName::BYTE,
            3i32 => PyColumnTypeName::SHORT,
            4i32 => PyColumnTypeName::INT,
            5i32 => PyColumnTypeName::LONG,
            6i32 => PyColumnTypeName::FLOAT,
            7i32 => PyColumnTypeName::DOUBLE,
            8i32 => PyColumnTypeName::DATE,
            9i32 => PyColumnTypeName::TIMESTAMP,
            10i32 => PyColumnTypeName::STRING,
            11i32 => PyColumnTypeName::BINARY,
            12i32 => PyColumnTypeName::DECIMAL,
            13i32 => PyColumnTypeName::INTERVAL,
            14i32 => PyColumnTypeName::ARRAY,
            15i32 => PyColumnTypeName::STRUCT,
            16i32 => PyColumnTypeName::MAP,
            17i32 => PyColumnTypeName::CHAR,
            18i32 => PyColumnTypeName::NULL,
            19i32 => PyColumnTypeName::USER_DEFINED_TYPE,
            20i32 => PyColumnTypeName::TIMESTAMP_NTZ,
            21i32 => PyColumnTypeName::VARIANT,
            22i32 => PyColumnTypeName::TABLE_TYPE,
            _ => PyColumnTypeName::COLUMN_TYPE_NAME_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::tables::v1::ColumnTypeName> for PyColumnTypeName {
    fn from(value: super::tables::v1::ColumnTypeName) -> Self {
        PyColumnTypeName::__from_proto_i32(
            <super::tables::v1::ColumnTypeName as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyColumnTypeName> for super::tables::v1::ColumnTypeName {
    fn from(value: PyColumnTypeName) -> Self {
        let n = value.__to_proto_i32();
        <super::tables::v1::ColumnTypeName as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "DataSourceFormat")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyDataSourceFormat {
    DATA_SOURCE_FORMAT_UNSPECIFIED = 0isize,
    DELTA = 1isize,
    ICEBERG = 2isize,
    HUDI = 3isize,
    PARQUET = 4isize,
    CSV = 5isize,
    JSON = 6isize,
    ORC = 7isize,
    AVRO = 8isize,
    TEXT = 9isize,
    UNITY_CATALOG = 10isize,
    DELTASHARING = 11isize,
}
impl PyDataSourceFormat {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyDataSourceFormat::DATA_SOURCE_FORMAT_UNSPECIFIED => 0i32,
            PyDataSourceFormat::DELTA => 1i32,
            PyDataSourceFormat::ICEBERG => 2i32,
            PyDataSourceFormat::HUDI => 3i32,
            PyDataSourceFormat::PARQUET => 4i32,
            PyDataSourceFormat::CSV => 5i32,
            PyDataSourceFormat::JSON => 6i32,
            PyDataSourceFormat::ORC => 7i32,
            PyDataSourceFormat::AVRO => 8i32,
            PyDataSourceFormat::TEXT => 9i32,
            PyDataSourceFormat::UNITY_CATALOG => 10i32,
            PyDataSourceFormat::DELTASHARING => 11i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyDataSourceFormat::DATA_SOURCE_FORMAT_UNSPECIFIED,
            1i32 => PyDataSourceFormat::DELTA,
            2i32 => PyDataSourceFormat::ICEBERG,
            3i32 => PyDataSourceFormat::HUDI,
            4i32 => PyDataSourceFormat::PARQUET,
            5i32 => PyDataSourceFormat::CSV,
            6i32 => PyDataSourceFormat::JSON,
            7i32 => PyDataSourceFormat::ORC,
            8i32 => PyDataSourceFormat::AVRO,
            9i32 => PyDataSourceFormat::TEXT,
            10i32 => PyDataSourceFormat::UNITY_CATALOG,
            11i32 => PyDataSourceFormat::DELTASHARING,
            _ => PyDataSourceFormat::DATA_SOURCE_FORMAT_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::tables::v1::DataSourceFormat> for PyDataSourceFormat {
    fn from(value: super::tables::v1::DataSourceFormat) -> Self {
        PyDataSourceFormat::__from_proto_i32(
            <super::tables::v1::DataSourceFormat as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyDataSourceFormat> for super::tables::v1::DataSourceFormat {
    fn from(value: PyDataSourceFormat) -> Self {
        let n = value.__to_proto_i32();
        <super::tables::v1::DataSourceFormat as ::buffa::Enumeration>::from_i32(n)
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "TableType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyTableType {
    TABLE_TYPE_UNSPECIFIED = 0isize,
    MANAGED = 1isize,
    EXTERNAL = 2isize,
    VIEW = 3isize,
    MATERIALIZED_VIEW = 4isize,
    STREAMING_TABLE = 5isize,
    METRIC_VIEW = 9isize,
}
impl PyTableType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyTableType::TABLE_TYPE_UNSPECIFIED => 0i32,
            PyTableType::MANAGED => 1i32,
            PyTableType::EXTERNAL => 2i32,
            PyTableType::VIEW => 3i32,
            PyTableType::MATERIALIZED_VIEW => 4i32,
            PyTableType::STREAMING_TABLE => 5i32,
            PyTableType::METRIC_VIEW => 9i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyTableType::TABLE_TYPE_UNSPECIFIED,
            1i32 => PyTableType::MANAGED,
            2i32 => PyTableType::EXTERNAL,
            3i32 => PyTableType::VIEW,
            4i32 => PyTableType::MATERIALIZED_VIEW,
            5i32 => PyTableType::STREAMING_TABLE,
            9i32 => PyTableType::METRIC_VIEW,
            _ => PyTableType::TABLE_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::tables::v1::TableType> for PyTableType {
    fn from(value: super::tables::v1::TableType) -> Self {
        PyTableType::__from_proto_i32(
            <super::tables::v1::TableType as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyTableType> for super::tables::v1::TableType {
    fn from(value: PyTableType) -> Self {
        let n = value.__to_proto_i32();
        <super::tables::v1::TableType as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "GenerateTemporaryPathCredentialsRequestOperation")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyGenerateTemporaryPathCredentialsRequestOperation {
    UNSPECIFIED = 0isize,
    PATH_READ = 1isize,
    PATH_READ_WRITE = 2isize,
    PATH_CREATE_TABLE = 3isize,
}
impl PyGenerateTemporaryPathCredentialsRequestOperation {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyGenerateTemporaryPathCredentialsRequestOperation::UNSPECIFIED => 0i32,
            PyGenerateTemporaryPathCredentialsRequestOperation::PATH_READ => 1i32,
            PyGenerateTemporaryPathCredentialsRequestOperation::PATH_READ_WRITE => 2i32,
            PyGenerateTemporaryPathCredentialsRequestOperation::PATH_CREATE_TABLE => 3i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyGenerateTemporaryPathCredentialsRequestOperation::UNSPECIFIED,
            1i32 => PyGenerateTemporaryPathCredentialsRequestOperation::PATH_READ,
            2i32 => PyGenerateTemporaryPathCredentialsRequestOperation::PATH_READ_WRITE,
            3i32 => PyGenerateTemporaryPathCredentialsRequestOperation::PATH_CREATE_TABLE,
            _ => PyGenerateTemporaryPathCredentialsRequestOperation::UNSPECIFIED,
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
        PyGenerateTemporaryPathCredentialsRequestOperation::__from_proto_i32(
            <super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation as ::buffa::Enumeration>::to_i32(
                &value,
            ),
        )
    }
}
impl ::core::convert::From<PyGenerateTemporaryPathCredentialsRequestOperation>
    for super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation
{
    fn from(value: PyGenerateTemporaryPathCredentialsRequestOperation) -> Self {
        let n = value.__to_proto_i32();
        <super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation as ::buffa::Enumeration>::from_i32(
                n,
            )
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "GenerateTemporaryTableCredentialsRequestOperation")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyGenerateTemporaryTableCredentialsRequestOperation {
    UNSPECIFIED = 0isize,
    READ = 1isize,
    READ_WRITE = 2isize,
}
impl PyGenerateTemporaryTableCredentialsRequestOperation {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyGenerateTemporaryTableCredentialsRequestOperation::UNSPECIFIED => 0i32,
            PyGenerateTemporaryTableCredentialsRequestOperation::READ => 1i32,
            PyGenerateTemporaryTableCredentialsRequestOperation::READ_WRITE => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyGenerateTemporaryTableCredentialsRequestOperation::UNSPECIFIED,
            1i32 => PyGenerateTemporaryTableCredentialsRequestOperation::READ,
            2i32 => PyGenerateTemporaryTableCredentialsRequestOperation::READ_WRITE,
            _ => PyGenerateTemporaryTableCredentialsRequestOperation::UNSPECIFIED,
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
        PyGenerateTemporaryTableCredentialsRequestOperation::__from_proto_i32(
            <super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation as ::buffa::Enumeration>::to_i32(
                &value,
            ),
        )
    }
}
impl ::core::convert::From<PyGenerateTemporaryTableCredentialsRequestOperation>
    for super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation
{
    fn from(value: PyGenerateTemporaryTableCredentialsRequestOperation) -> Self {
        let n = value.__to_proto_i32();
        <super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation as ::buffa::Enumeration>::from_i32(
                n,
            )
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(
    eq,
    eq_int,
    name = "GenerateTemporaryVolumeCredentialsRequestOperation"
)]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyGenerateTemporaryVolumeCredentialsRequestOperation {
    UNSPECIFIED = 0isize,
    READ_VOLUME = 1isize,
    WRITE_VOLUME = 2isize,
}
impl PyGenerateTemporaryVolumeCredentialsRequestOperation {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyGenerateTemporaryVolumeCredentialsRequestOperation::UNSPECIFIED => 0i32,
            PyGenerateTemporaryVolumeCredentialsRequestOperation::READ_VOLUME => 1i32,
            PyGenerateTemporaryVolumeCredentialsRequestOperation::WRITE_VOLUME => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyGenerateTemporaryVolumeCredentialsRequestOperation::UNSPECIFIED,
            1i32 => PyGenerateTemporaryVolumeCredentialsRequestOperation::READ_VOLUME,
            2i32 => PyGenerateTemporaryVolumeCredentialsRequestOperation::WRITE_VOLUME,
            _ => PyGenerateTemporaryVolumeCredentialsRequestOperation::UNSPECIFIED,
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
        PyGenerateTemporaryVolumeCredentialsRequestOperation::__from_proto_i32(
            <super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation as ::buffa::Enumeration>::to_i32(
                &value,
            ),
        )
    }
}
impl ::core::convert::From<PyGenerateTemporaryVolumeCredentialsRequestOperation>
    for super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation
{
    fn from(value: PyGenerateTemporaryVolumeCredentialsRequestOperation) -> Self {
        let n = value.__to_proto_i32();
        <super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation as ::buffa::Enumeration>::from_i32(
                n,
            )
            .unwrap_or_default()
    }
}
#[allow(non_camel_case_types)]
#[::pyo3::pyclass(eq, eq_int, name = "VolumeType")]
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum PyVolumeType {
    VOLUME_TYPE_UNSPECIFIED = 0isize,
    EXTERNAL = 1isize,
    MANAGED = 2isize,
}
impl PyVolumeType {
    fn __to_proto_i32(self) -> i32 {
        match self {
            PyVolumeType::VOLUME_TYPE_UNSPECIFIED => 0i32,
            PyVolumeType::EXTERNAL => 1i32,
            PyVolumeType::MANAGED => 2i32,
        }
    }
    fn __from_proto_i32(value: i32) -> Self {
        match value {
            0i32 => PyVolumeType::VOLUME_TYPE_UNSPECIFIED,
            1i32 => PyVolumeType::EXTERNAL,
            2i32 => PyVolumeType::MANAGED,
            _ => PyVolumeType::VOLUME_TYPE_UNSPECIFIED,
        }
    }
}
impl ::core::convert::From<super::volumes::v1::VolumeType> for PyVolumeType {
    fn from(value: super::volumes::v1::VolumeType) -> Self {
        PyVolumeType::__from_proto_i32(
            <super::volumes::v1::VolumeType as ::buffa::Enumeration>::to_i32(&value),
        )
    }
}
impl ::core::convert::From<PyVolumeType> for super::volumes::v1::VolumeType {
    fn from(value: PyVolumeType) -> Self {
        let n = value.__to_proto_i32();
        <super::volumes::v1::VolumeType as ::buffa::Enumeration>::from_i32(n).unwrap_or_default()
    }
}
#[::pyo3::pyclass(name = "AgentSkill")]
#[derive(Clone, Debug)]
pub struct PyAgentSkill(pub super::agent_skills::v0alpha1::AgentSkill);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAgentSkill {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            catalog_name = None,
            schema_name = None,
            full_name = None,
            storage_location = None,
            agent_skill_id = None,
            agent_skill_type = None,
            description = None,
            license = None,
            allowed_tools = None,
            metadata = None,
            owner = None,
            comment = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None,
            metastore_id = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        full_name: ::core::option::Option<::std::string::String>,
        storage_location: ::core::option::Option<::std::string::String>,
        agent_skill_id: ::core::option::Option<::std::string::String>,
        agent_skill_type: ::core::option::Option<PyAgentSkillType>,
        description: ::core::option::Option<::std::string::String>,
        license: ::core::option::Option<::std::string::String>,
        allowed_tools: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        metadata: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
        metastore_id: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::agent_skills::v0alpha1::AgentSkill as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        if let ::core::option::Option::Some(value) = storage_location {
            inner.storage_location = value;
        }
        if let ::core::option::Option::Some(value) = agent_skill_id {
            inner.agent_skill_id = value;
        }
        if let ::core::option::Option::Some(value) = agent_skill_type {
            inner.agent_skill_type = ::buffa::EnumValue::Known(
                <super::agent_skills::v0alpha1::AgentSkillType as ::core::convert::From<_>>::from(
                    value,
                ),
            );
        }
        {
            let value = description;
            inner.description = value;
        }
        {
            let value = license;
            inner.license = value;
        }
        if let ::core::option::Option::Some(value) = allowed_tools {
            inner.allowed_tools = value;
        }
        if let ::core::option::Option::Some(value) = metadata {
            inner.metadata = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        {
            let value = metastore_id;
            inner.metastore_id = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn storage_location(&self) -> ::std::string::String {
        self.0.storage_location.clone()
    }
    #[getter]
    fn agent_skill_id(&self) -> ::std::string::String {
        self.0.agent_skill_id.clone()
    }
    #[getter]
    fn agent_skill_type(&self) -> PyAgentSkillType {
        PyAgentSkillType::from(self.0.agent_skill_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }
    #[getter]
    fn license(&self) -> ::core::option::Option<::std::string::String> {
        self.0.license.clone()
    }
    #[getter]
    fn allowed_tools(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.allowed_tools.clone()
    }
    #[getter]
    fn metadata(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.metadata.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[getter]
    fn metastore_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.metastore_id.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(storage_location)]
    fn set_storage_location(&mut self, value: ::std::string::String) {
        self.0.storage_location = value;
    }
    #[setter(agent_skill_id)]
    fn set_agent_skill_id(&mut self, value: ::std::string::String) {
        self.0.agent_skill_id = value;
    }
    #[setter(agent_skill_type)]
    fn set_agent_skill_type(&mut self, value: PyAgentSkillType) {
        self.0.agent_skill_type = ::buffa::EnumValue::Known(
            <super::agent_skills::v0alpha1::AgentSkillType as ::core::convert::From<_>>::from(
                value,
            ),
        );
    }
    #[setter(description)]
    fn set_description(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.description = value;
    }
    #[setter(license)]
    fn set_license(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.license = value;
    }
    #[setter(allowed_tools)]
    fn set_allowed_tools(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.allowed_tools = value;
    }
    #[setter(metadata)]
    fn set_metadata(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.metadata = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    #[setter(metastore_id)]
    fn set_metastore_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.metastore_id = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "CreateAgentSkillRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateAgentSkillRequest(pub super::agent_skills::v0alpha1::CreateAgentSkillRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateAgentSkillRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name = None,
            name = None,
            agent_skill_type = None,
            storage_location = None,
            description = None,
            license = None,
            allowed_tools = None,
            metadata = None,
            comment = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
        agent_skill_type: ::core::option::Option<PyAgentSkillType>,
        storage_location: ::core::option::Option<::std::string::String>,
        description: ::core::option::Option<::std::string::String>,
        license: ::core::option::Option<::std::string::String>,
        allowed_tools: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        metadata: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        comment: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::agent_skills::v0alpha1::CreateAgentSkillRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = agent_skill_type {
            inner.agent_skill_type = ::buffa::EnumValue::Known(
                <super::agent_skills::v0alpha1::AgentSkillType as ::core::convert::From<_>>::from(
                    value,
                ),
            );
        }
        {
            let value = storage_location;
            inner.storage_location = value;
        }
        {
            let value = description;
            inner.description = value;
        }
        {
            let value = license;
            inner.license = value;
        }
        if let ::core::option::Option::Some(value) = allowed_tools {
            inner.allowed_tools = value;
        }
        if let ::core::option::Option::Some(value) = metadata {
            inner.metadata = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn agent_skill_type(&self) -> PyAgentSkillType {
        PyAgentSkillType::from(self.0.agent_skill_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }
    #[getter]
    fn license(&self) -> ::core::option::Option<::std::string::String> {
        self.0.license.clone()
    }
    #[getter]
    fn allowed_tools(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.allowed_tools.clone()
    }
    #[getter]
    fn metadata(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.metadata.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(agent_skill_type)]
    fn set_agent_skill_type(&mut self, value: PyAgentSkillType) {
        self.0.agent_skill_type = ::buffa::EnumValue::Known(
            <super::agent_skills::v0alpha1::AgentSkillType as ::core::convert::From<_>>::from(
                value,
            ),
        );
    }
    #[setter(storage_location)]
    fn set_storage_location(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_location = value;
    }
    #[setter(description)]
    fn set_description(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.description = value;
    }
    #[setter(license)]
    fn set_license(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.license = value;
    }
    #[setter(allowed_tools)]
    fn set_allowed_tools(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.allowed_tools = value;
    }
    #[setter(metadata)]
    fn set_metadata(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.metadata = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agent_skills::v0alpha1::CreateAgentSkillRequest>
    for PyCreateAgentSkillRequest
{
    fn from(value: super::agent_skills::v0alpha1::CreateAgentSkillRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateAgentSkillRequest>
    for super::agent_skills::v0alpha1::CreateAgentSkillRequest
{
    fn from(value: PyCreateAgentSkillRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteAgentSkillRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteAgentSkillRequest(pub super::agent_skills::v0alpha1::DeleteAgentSkillRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteAgentSkillRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner = <super::agent_skills::v0alpha1::DeleteAgentSkillRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agent_skills::v0alpha1::DeleteAgentSkillRequest>
    for PyDeleteAgentSkillRequest
{
    fn from(value: super::agent_skills::v0alpha1::DeleteAgentSkillRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteAgentSkillRequest>
    for super::agent_skills::v0alpha1::DeleteAgentSkillRequest
{
    fn from(value: PyDeleteAgentSkillRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetAgentSkillRequest")]
#[derive(Clone, Debug)]
pub struct PyGetAgentSkillRequest(pub super::agent_skills::v0alpha1::GetAgentSkillRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetAgentSkillRequest {
    #[new]
    #[pyo3(signature = (name = None, include_browse = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::agent_skills::v0alpha1::GetAgentSkillRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agent_skills::v0alpha1::GetAgentSkillRequest>
    for PyGetAgentSkillRequest
{
    fn from(value: super::agent_skills::v0alpha1::GetAgentSkillRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetAgentSkillRequest>
    for super::agent_skills::v0alpha1::GetAgentSkillRequest
{
    fn from(value: PyGetAgentSkillRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListAgentSkillsRequest")]
#[derive(Clone, Debug)]
pub struct PyListAgentSkillsRequest(pub super::agent_skills::v0alpha1::ListAgentSkillsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListAgentSkillsRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name = None,
            max_results = None,
            page_token = None,
            include_browse = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::agent_skills::v0alpha1::ListAgentSkillsRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agent_skills::v0alpha1::ListAgentSkillsRequest>
    for PyListAgentSkillsRequest
{
    fn from(value: super::agent_skills::v0alpha1::ListAgentSkillsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListAgentSkillsRequest>
    for super::agent_skills::v0alpha1::ListAgentSkillsRequest
{
    fn from(value: PyListAgentSkillsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListAgentSkillsResponse")]
#[derive(Clone, Debug)]
pub struct PyListAgentSkillsResponse(pub super::agent_skills::v0alpha1::ListAgentSkillsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListAgentSkillsResponse {
    #[new]
    #[pyo3(signature = (agent_skills = None, next_page_token = None))]
    fn new(
        agent_skills: ::core::option::Option<::std::vec::Vec<PyAgentSkill>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::agent_skills::v0alpha1::ListAgentSkillsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = agent_skills {
            inner.agent_skills = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn agent_skills(&self) -> ::std::vec::Vec<PyAgentSkill> {
        self.0
            .agent_skills
            .iter()
            .cloned()
            .map(PyAgentSkill::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(agent_skills)]
    fn set_agent_skills(&mut self, value: ::std::vec::Vec<PyAgentSkill>) {
        self.0.agent_skills = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateAgentSkillRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateAgentSkillRequest(pub super::agent_skills::v0alpha1::UpdateAgentSkillRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateAgentSkillRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            new_name = None,
            description = None,
            allowed_tools = None,
            comment = None,
            owner = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        new_name: ::core::option::Option<::std::string::String>,
        description: ::core::option::Option<::std::string::String>,
        allowed_tools: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        comment: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::agent_skills::v0alpha1::UpdateAgentSkillRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        {
            let value = description;
            inner.description = value;
        }
        if let ::core::option::Option::Some(value) = allowed_tools {
            inner.allowed_tools = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }
    #[getter]
    fn allowed_tools(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.allowed_tools.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    #[setter(description)]
    fn set_description(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.description = value;
    }
    #[setter(allowed_tools)]
    fn set_allowed_tools(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.allowed_tools = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agent_skills::v0alpha1::UpdateAgentSkillRequest>
    for PyUpdateAgentSkillRequest
{
    fn from(value: super::agent_skills::v0alpha1::UpdateAgentSkillRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateAgentSkillRequest>
    for super::agent_skills::v0alpha1::UpdateAgentSkillRequest
{
    fn from(value: PyUpdateAgentSkillRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "Agent")]
#[derive(Clone, Debug)]
pub struct PyAgent(pub super::agents::v0alpha1::Agent);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAgent {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            catalog_name = None,
            schema_name = None,
            full_name = None,
            agent_id = None,
            invocation_protocol = None,
            endpoint = None,
            description = None,
            capabilities = None,
            input_schema = None,
            owner = None,
            comment = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None,
            metastore_id = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        full_name: ::core::option::Option<::std::string::String>,
        agent_id: ::core::option::Option<::std::string::String>,
        invocation_protocol: ::core::option::Option<PyInvocationProtocol>,
        endpoint: ::core::option::Option<::std::string::String>,
        description: ::core::option::Option<::std::string::String>,
        capabilities: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        input_schema: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
        metastore_id: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::agents::v0alpha1::Agent as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        if let ::core::option::Option::Some(value) = agent_id {
            inner.agent_id = value;
        }
        if let ::core::option::Option::Some(value) = invocation_protocol {
            inner.invocation_protocol = ::buffa::EnumValue::Known(
                <super::agents::v0alpha1::InvocationProtocol as ::core::convert::From<_>>::from(
                    value,
                ),
            );
        }
        if let ::core::option::Option::Some(value) = endpoint {
            inner.endpoint = value;
        }
        {
            let value = description;
            inner.description = value;
        }
        if let ::core::option::Option::Some(value) = capabilities {
            inner.capabilities = value;
        }
        {
            let value = input_schema;
            inner.input_schema = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        {
            let value = metastore_id;
            inner.metastore_id = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn agent_id(&self) -> ::std::string::String {
        self.0.agent_id.clone()
    }
    #[getter]
    fn invocation_protocol(&self) -> PyInvocationProtocol {
        PyInvocationProtocol::from(self.0.invocation_protocol.as_known().unwrap_or_default())
    }
    #[getter]
    fn endpoint(&self) -> ::std::string::String {
        self.0.endpoint.clone()
    }
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }
    #[getter]
    fn capabilities(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.capabilities.clone()
    }
    #[getter]
    fn input_schema(&self) -> ::core::option::Option<::std::string::String> {
        self.0.input_schema.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[getter]
    fn metastore_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.metastore_id.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(agent_id)]
    fn set_agent_id(&mut self, value: ::std::string::String) {
        self.0.agent_id = value;
    }
    #[setter(invocation_protocol)]
    fn set_invocation_protocol(&mut self, value: PyInvocationProtocol) {
        self.0.invocation_protocol = ::buffa::EnumValue::Known(
            <super::agents::v0alpha1::InvocationProtocol as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(endpoint)]
    fn set_endpoint(&mut self, value: ::std::string::String) {
        self.0.endpoint = value;
    }
    #[setter(description)]
    fn set_description(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.description = value;
    }
    #[setter(capabilities)]
    fn set_capabilities(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.capabilities = value;
    }
    #[setter(input_schema)]
    fn set_input_schema(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.input_schema = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    #[setter(metastore_id)]
    fn set_metastore_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.metastore_id = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "CreateAgentRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateAgentRequest(pub super::agents::v0alpha1::CreateAgentRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateAgentRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name = None,
            name = None,
            invocation_protocol = None,
            endpoint = None,
            description = None,
            capabilities = None,
            input_schema = None,
            comment = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
        invocation_protocol: ::core::option::Option<PyInvocationProtocol>,
        endpoint: ::core::option::Option<::std::string::String>,
        description: ::core::option::Option<::std::string::String>,
        capabilities: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        input_schema: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::agents::v0alpha1::CreateAgentRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = invocation_protocol {
            inner.invocation_protocol = ::buffa::EnumValue::Known(
                <super::agents::v0alpha1::InvocationProtocol as ::core::convert::From<_>>::from(
                    value,
                ),
            );
        }
        if let ::core::option::Option::Some(value) = endpoint {
            inner.endpoint = value;
        }
        {
            let value = description;
            inner.description = value;
        }
        if let ::core::option::Option::Some(value) = capabilities {
            inner.capabilities = value;
        }
        {
            let value = input_schema;
            inner.input_schema = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn invocation_protocol(&self) -> PyInvocationProtocol {
        PyInvocationProtocol::from(self.0.invocation_protocol.as_known().unwrap_or_default())
    }
    #[getter]
    fn endpoint(&self) -> ::std::string::String {
        self.0.endpoint.clone()
    }
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }
    #[getter]
    fn capabilities(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.capabilities.clone()
    }
    #[getter]
    fn input_schema(&self) -> ::core::option::Option<::std::string::String> {
        self.0.input_schema.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(invocation_protocol)]
    fn set_invocation_protocol(&mut self, value: PyInvocationProtocol) {
        self.0.invocation_protocol = ::buffa::EnumValue::Known(
            <super::agents::v0alpha1::InvocationProtocol as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(endpoint)]
    fn set_endpoint(&mut self, value: ::std::string::String) {
        self.0.endpoint = value;
    }
    #[setter(description)]
    fn set_description(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.description = value;
    }
    #[setter(capabilities)]
    fn set_capabilities(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.capabilities = value;
    }
    #[setter(input_schema)]
    fn set_input_schema(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.input_schema = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agents::v0alpha1::CreateAgentRequest> for PyCreateAgentRequest {
    fn from(value: super::agents::v0alpha1::CreateAgentRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateAgentRequest> for super::agents::v0alpha1::CreateAgentRequest {
    fn from(value: PyCreateAgentRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteAgentRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteAgentRequest(pub super::agents::v0alpha1::DeleteAgentRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteAgentRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::agents::v0alpha1::DeleteAgentRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agents::v0alpha1::DeleteAgentRequest> for PyDeleteAgentRequest {
    fn from(value: super::agents::v0alpha1::DeleteAgentRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteAgentRequest> for super::agents::v0alpha1::DeleteAgentRequest {
    fn from(value: PyDeleteAgentRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetAgentRequest")]
#[derive(Clone, Debug)]
pub struct PyGetAgentRequest(pub super::agents::v0alpha1::GetAgentRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetAgentRequest {
    #[new]
    #[pyo3(signature = (name = None, include_browse = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::agents::v0alpha1::GetAgentRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agents::v0alpha1::GetAgentRequest> for PyGetAgentRequest {
    fn from(value: super::agents::v0alpha1::GetAgentRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetAgentRequest> for super::agents::v0alpha1::GetAgentRequest {
    fn from(value: PyGetAgentRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListAgentsRequest")]
#[derive(Clone, Debug)]
pub struct PyListAgentsRequest(pub super::agents::v0alpha1::ListAgentsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListAgentsRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name = None,
            max_results = None,
            page_token = None,
            include_browse = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::agents::v0alpha1::ListAgentsRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agents::v0alpha1::ListAgentsRequest> for PyListAgentsRequest {
    fn from(value: super::agents::v0alpha1::ListAgentsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListAgentsRequest> for super::agents::v0alpha1::ListAgentsRequest {
    fn from(value: PyListAgentsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListAgentsResponse")]
#[derive(Clone, Debug)]
pub struct PyListAgentsResponse(pub super::agents::v0alpha1::ListAgentsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListAgentsResponse {
    #[new]
    #[pyo3(signature = (agents = None, next_page_token = None))]
    fn new(
        agents: ::core::option::Option<::std::vec::Vec<PyAgent>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::agents::v0alpha1::ListAgentsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = agents {
            inner.agents = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn agents(&self) -> ::std::vec::Vec<PyAgent> {
        self.0.agents.iter().cloned().map(PyAgent::from).collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(agents)]
    fn set_agents(&mut self, value: ::std::vec::Vec<PyAgent>) {
        self.0.agents = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateAgentRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateAgentRequest(pub super::agents::v0alpha1::UpdateAgentRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateAgentRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            new_name = None,
            invocation_protocol = None,
            endpoint = None,
            description = None,
            capabilities = None,
            input_schema = None,
            comment = None,
            owner = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        new_name: ::core::option::Option<::std::string::String>,
        invocation_protocol: ::core::option::Option<PyInvocationProtocol>,
        endpoint: ::core::option::Option<::std::string::String>,
        description: ::core::option::Option<::std::string::String>,
        capabilities: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        input_schema: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::agents::v0alpha1::UpdateAgentRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        {
            let value = invocation_protocol;
            inner.invocation_protocol = value.map(|e| {
                ::buffa::EnumValue::Known(
                    <super::agents::v0alpha1::InvocationProtocol as ::core::convert::From<_>>::from(
                        e,
                    ),
                )
            });
        }
        {
            let value = endpoint;
            inner.endpoint = value;
        }
        {
            let value = description;
            inner.description = value;
        }
        if let ::core::option::Option::Some(value) = capabilities {
            inner.capabilities = value;
        }
        {
            let value = input_schema;
            inner.input_schema = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[getter]
    fn invocation_protocol(&self) -> ::core::option::Option<PyInvocationProtocol> {
        self.0
            .invocation_protocol
            .as_ref()
            .and_then(|e| e.as_known())
            .map(PyInvocationProtocol::from)
    }
    #[getter]
    fn endpoint(&self) -> ::core::option::Option<::std::string::String> {
        self.0.endpoint.clone()
    }
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }
    #[getter]
    fn capabilities(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.capabilities.clone()
    }
    #[getter]
    fn input_schema(&self) -> ::core::option::Option<::std::string::String> {
        self.0.input_schema.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    #[setter(invocation_protocol)]
    fn set_invocation_protocol(&mut self, value: ::core::option::Option<PyInvocationProtocol>) {
        self.0.invocation_protocol = value.map(|e| {
            ::buffa::EnumValue::Known(
                <super::agents::v0alpha1::InvocationProtocol as ::core::convert::From<_>>::from(e),
            )
        });
    }
    #[setter(endpoint)]
    fn set_endpoint(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.endpoint = value;
    }
    #[setter(description)]
    fn set_description(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.description = value;
    }
    #[setter(capabilities)]
    fn set_capabilities(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.capabilities = value;
    }
    #[setter(input_schema)]
    fn set_input_schema(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.input_schema = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::agents::v0alpha1::UpdateAgentRequest> for PyUpdateAgentRequest {
    fn from(value: super::agents::v0alpha1::UpdateAgentRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateAgentRequest> for super::agents::v0alpha1::UpdateAgentRequest {
    fn from(value: PyUpdateAgentRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "Catalog")]
#[derive(Clone, Debug)]
pub struct PyCatalog(pub super::catalogs::v1::Catalog);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCatalog {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            id = None,
            owner = None,
            comment = None,
            properties = None,
            storage_root = None,
            provider_name = None,
            share_name = None,
            catalog_type = None,
            storage_location = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None,
            browse_only = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        id: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        storage_root: ::core::option::Option<::std::string::String>,
        provider_name: ::core::option::Option<::std::string::String>,
        share_name: ::core::option::Option<::std::string::String>,
        catalog_type: ::core::option::Option<PyCatalogType>,
        storage_location: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
        browse_only: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::catalogs::v1::Catalog as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = id;
            inner.id = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = storage_root;
            inner.storage_root = value;
        }
        {
            let value = provider_name;
            inner.provider_name = value;
        }
        {
            let value = share_name;
            inner.share_name = value;
        }
        {
            let value = catalog_type;
            inner.catalog_type = value.map(|e| {
                ::buffa::EnumValue::Known(
                    <super::catalogs::v1::CatalogType as ::core::convert::From<_>>::from(e),
                )
            });
        }
        {
            let value = storage_location;
            inner.storage_location = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        {
            let value = browse_only;
            inner.browse_only = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn storage_root(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_root.clone()
    }
    #[getter]
    fn provider_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.provider_name.clone()
    }
    #[getter]
    fn share_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.share_name.clone()
    }
    #[getter]
    fn catalog_type(&self) -> ::core::option::Option<PyCatalogType> {
        self.0
            .catalog_type
            .as_ref()
            .and_then(|e| e.as_known())
            .map(PyCatalogType::from)
    }
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(id)]
    fn set_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.id = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(storage_root)]
    fn set_storage_root(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_root = value;
    }
    #[setter(provider_name)]
    fn set_provider_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.provider_name = value;
    }
    #[setter(share_name)]
    fn set_share_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.share_name = value;
    }
    #[setter(catalog_type)]
    fn set_catalog_type(&mut self, value: ::core::option::Option<PyCatalogType>) {
        self.0.catalog_type = value.map(|e| {
            ::buffa::EnumValue::Known(
                <super::catalogs::v1::CatalogType as ::core::convert::From<_>>::from(e),
            )
        });
    }
    #[setter(storage_location)]
    fn set_storage_location(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_location = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    #[setter(browse_only)]
    fn set_browse_only(&mut self, value: ::core::option::Option<bool>) {
        self.0.browse_only = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "CreateCatalogRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateCatalogRequest(pub super::catalogs::v1::CreateCatalogRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateCatalogRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            comment = None,
            properties = None,
            storage_root = None,
            provider_name = None,
            share_name = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        storage_root: ::core::option::Option<::std::string::String>,
        provider_name: ::core::option::Option<::std::string::String>,
        share_name: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::catalogs::v1::CreateCatalogRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = storage_root;
            inner.storage_root = value;
        }
        {
            let value = provider_name;
            inner.provider_name = value;
        }
        {
            let value = share_name;
            inner.share_name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn storage_root(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_root.clone()
    }
    #[getter]
    fn provider_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.provider_name.clone()
    }
    #[getter]
    fn share_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.share_name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(storage_root)]
    fn set_storage_root(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_root = value;
    }
    #[setter(provider_name)]
    fn set_provider_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.provider_name = value;
    }
    #[setter(share_name)]
    fn set_share_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.share_name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::catalogs::v1::CreateCatalogRequest> for PyCreateCatalogRequest {
    fn from(value: super::catalogs::v1::CreateCatalogRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateCatalogRequest> for super::catalogs::v1::CreateCatalogRequest {
    fn from(value: PyCreateCatalogRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteCatalogRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteCatalogRequest(pub super::catalogs::v1::DeleteCatalogRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteCatalogRequest {
    #[new]
    #[pyo3(signature = (name = None, force = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        force: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::catalogs::v1::DeleteCatalogRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = force;
            inner.force = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn force(&self) -> ::core::option::Option<bool> {
        self.0.force
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(force)]
    fn set_force(&mut self, value: ::core::option::Option<bool>) {
        self.0.force = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::catalogs::v1::DeleteCatalogRequest> for PyDeleteCatalogRequest {
    fn from(value: super::catalogs::v1::DeleteCatalogRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteCatalogRequest> for super::catalogs::v1::DeleteCatalogRequest {
    fn from(value: PyDeleteCatalogRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetCatalogRequest")]
#[derive(Clone, Debug)]
pub struct PyGetCatalogRequest(pub super::catalogs::v1::GetCatalogRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetCatalogRequest {
    #[new]
    #[pyo3(signature = (name = None, include_browse = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::catalogs::v1::GetCatalogRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::catalogs::v1::GetCatalogRequest> for PyGetCatalogRequest {
    fn from(value: super::catalogs::v1::GetCatalogRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetCatalogRequest> for super::catalogs::v1::GetCatalogRequest {
    fn from(value: PyGetCatalogRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListCatalogsRequest")]
#[derive(Clone, Debug)]
pub struct PyListCatalogsRequest(pub super::catalogs::v1::ListCatalogsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListCatalogsRequest {
    #[new]
    #[pyo3(signature = (max_results = None, page_token = None))]
    fn new(
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::catalogs::v1::ListCatalogsRequest as ::core::default::Default>::default();
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::catalogs::v1::ListCatalogsRequest> for PyListCatalogsRequest {
    fn from(value: super::catalogs::v1::ListCatalogsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListCatalogsRequest> for super::catalogs::v1::ListCatalogsRequest {
    fn from(value: PyListCatalogsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListCatalogsResponse")]
#[derive(Clone, Debug)]
pub struct PyListCatalogsResponse(pub super::catalogs::v1::ListCatalogsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListCatalogsResponse {
    #[new]
    #[pyo3(signature = (catalogs = None, next_page_token = None))]
    fn new(
        catalogs: ::core::option::Option<::std::vec::Vec<PyCatalog>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::catalogs::v1::ListCatalogsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalogs {
            inner.catalogs = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalogs(&self) -> ::std::vec::Vec<PyCatalog> {
        self.0
            .catalogs
            .iter()
            .cloned()
            .map(PyCatalog::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(catalogs)]
    fn set_catalogs(&mut self, value: ::std::vec::Vec<PyCatalog>) {
        self.0.catalogs = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateCatalogRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateCatalogRequest(pub super::catalogs::v1::UpdateCatalogRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateCatalogRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            owner = None,
            comment = None,
            properties = None,
            new_name = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        new_name: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::catalogs::v1::UpdateCatalogRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::catalogs::v1::UpdateCatalogRequest> for PyUpdateCatalogRequest {
    fn from(value: super::catalogs::v1::UpdateCatalogRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateCatalogRequest> for super::catalogs::v1::UpdateCatalogRequest {
    fn from(value: PyUpdateCatalogRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "AwsIamRole")]
#[derive(Clone, Debug)]
pub struct PyAwsIamRole(pub super::credentials::v1::AwsIamRole);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAwsIamRole {
    #[new]
    #[pyo3(
        signature = (external_id = None, role_arn = None, unity_catalog_iam_arn = None)
    )]
    fn new(
        external_id: ::core::option::Option<::std::string::String>,
        role_arn: ::core::option::Option<::std::string::String>,
        unity_catalog_iam_arn: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::credentials::v1::AwsIamRole as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = external_id {
            inner.external_id = value;
        }
        if let ::core::option::Option::Some(value) = role_arn {
            inner.role_arn = value;
        }
        if let ::core::option::Option::Some(value) = unity_catalog_iam_arn {
            inner.unity_catalog_iam_arn = value;
        }
        Self(inner)
    }
    #[getter]
    fn external_id(&self) -> ::std::string::String {
        self.0.external_id.clone()
    }
    #[getter]
    fn role_arn(&self) -> ::std::string::String {
        self.0.role_arn.clone()
    }
    #[getter]
    fn unity_catalog_iam_arn(&self) -> ::std::string::String {
        self.0.unity_catalog_iam_arn.clone()
    }
    #[setter(external_id)]
    fn set_external_id(&mut self, value: ::std::string::String) {
        self.0.external_id = value;
    }
    #[setter(role_arn)]
    fn set_role_arn(&mut self, value: ::std::string::String) {
        self.0.role_arn = value;
    }
    #[setter(unity_catalog_iam_arn)]
    fn set_unity_catalog_iam_arn(&mut self, value: ::std::string::String) {
        self.0.unity_catalog_iam_arn = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::credentials::v1::AwsIamRole> for PyAwsIamRole {
    fn from(value: super::credentials::v1::AwsIamRole) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyAwsIamRole> for super::credentials::v1::AwsIamRole {
    fn from(value: PyAwsIamRole) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "AwsIamRoleConfig")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn role_arn(&self) -> ::std::string::String {
        self.0.role_arn.clone()
    }
    #[getter]
    fn region(&self) -> ::core::option::Option<::std::string::String> {
        self.0.region.clone()
    }
    #[getter]
    fn access_key_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.access_key_id.clone()
    }
    #[getter]
    fn secret_access_key(&self) -> ::core::option::Option<::std::string::String> {
        self.0.secret_access_key.clone()
    }
    #[getter]
    fn session_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.session_token.clone()
    }
    #[setter(role_arn)]
    fn set_role_arn(&mut self, value: ::std::string::String) {
        self.0.role_arn = value;
    }
    #[setter(region)]
    fn set_region(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.region = value;
    }
    #[setter(access_key_id)]
    fn set_access_key_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.access_key_id = value;
    }
    #[setter(secret_access_key)]
    fn set_secret_access_key(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.secret_access_key = value;
    }
    #[setter(session_token)]
    fn set_session_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.session_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "AzureManagedIdentity")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn access_connector_id(&self) -> ::std::string::String {
        self.0.access_connector_id.clone()
    }
    #[getter]
    fn credential_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.credential_id.clone()
    }
    #[getter]
    fn managed_identity_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.managed_identity_id.clone()
    }
    #[setter(access_connector_id)]
    fn set_access_connector_id(&mut self, value: ::std::string::String) {
        self.0.access_connector_id = value;
    }
    #[setter(credential_id)]
    fn set_credential_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.credential_id = value;
    }
    #[setter(managed_identity_id)]
    fn set_managed_identity_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.managed_identity_id = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "AzureServicePrincipal")]
#[derive(Clone, Debug)]
pub struct PyAzureServicePrincipal(pub super::credentials::v1::AzureServicePrincipal);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAzureServicePrincipal {
    #[new]
    #[pyo3(signature = (directory_id = None, application_id = None))]
    fn new(
        directory_id: ::core::option::Option<::std::string::String>,
        application_id: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::credentials::v1::AzureServicePrincipal as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = directory_id {
            inner.directory_id = value;
        }
        if let ::core::option::Option::Some(value) = application_id {
            inner.application_id = value;
        }
        Self(inner)
    }
    #[getter]
    fn directory_id(&self) -> ::std::string::String {
        self.0.directory_id.clone()
    }
    #[getter]
    fn application_id(&self) -> ::std::string::String {
        self.0.application_id.clone()
    }
    #[setter(directory_id)]
    fn set_directory_id(&mut self, value: ::std::string::String) {
        self.0.directory_id = value;
    }
    #[setter(application_id)]
    fn set_application_id(&mut self, value: ::std::string::String) {
        self.0.application_id = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "AzureStorageKey")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn account_name(&self) -> ::std::string::String {
        self.0.account_name.clone()
    }
    #[getter]
    fn account_key(&self) -> ::std::string::String {
        self.0.account_key.clone()
    }
    #[setter(account_name)]
    fn set_account_name(&mut self, value: ::std::string::String) {
        self.0.account_name = value;
    }
    #[setter(account_key)]
    fn set_account_key(&mut self, value: ::std::string::String) {
        self.0.account_key = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "CreateCredentialRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateCredentialRequest(pub super::credentials::v1::CreateCredentialRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateCredentialRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            purpose = None,
            comment = None,
            read_only = None,
            skip_validation = None,
            azure_service_principal = None,
            azure_managed_identity = None,
            azure_storage_key = None,
            aws_iam_role = None,
            databricks_gcp_service_account = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        purpose: ::core::option::Option<PyPurpose>,
        comment: ::core::option::Option<::std::string::String>,
        read_only: ::core::option::Option<bool>,
        skip_validation: ::core::option::Option<bool>,
        azure_service_principal: ::core::option::Option<PyAzureServicePrincipal>,
        azure_managed_identity: ::core::option::Option<PyAzureManagedIdentity>,
        azure_storage_key: ::core::option::Option<PyAzureStorageKey>,
        aws_iam_role: ::core::option::Option<PyAwsIamRoleConfig>,
        databricks_gcp_service_account: ::core::option::Option<PyDatabricksGcpServiceAccount>,
    ) -> Self {
        let mut inner =
            <super::credentials::v1::CreateCredentialRequest as ::core::default::Default>::default(
            );
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = purpose {
            inner.purpose = ::buffa::EnumValue::Known(
                <super::credentials::v1::Purpose as ::core::convert::From<_>>::from(value),
            );
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = read_only;
            inner.read_only = value;
        }
        {
            let value = skip_validation;
            inner.skip_validation = value;
        }
        {
            let value = azure_service_principal;
            inner.azure_service_principal = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = azure_managed_identity;
            inner.azure_managed_identity = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = azure_storage_key;
            inner.azure_storage_key = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = aws_iam_role;
            inner.aws_iam_role = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = databricks_gcp_service_account;
            inner.databricks_gcp_service_account = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn purpose(&self) -> PyPurpose {
        PyPurpose::from(self.0.purpose.as_known().unwrap_or_default())
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn read_only(&self) -> ::core::option::Option<bool> {
        self.0.read_only
    }
    #[getter]
    fn skip_validation(&self) -> ::core::option::Option<bool> {
        self.0.skip_validation
    }
    #[getter]
    fn azure_service_principal(&self) -> ::core::option::Option<PyAzureServicePrincipal> {
        self.0
            .azure_service_principal
            .clone()
            .into_option()
            .map(PyAzureServicePrincipal::from)
    }
    #[getter]
    fn azure_managed_identity(&self) -> ::core::option::Option<PyAzureManagedIdentity> {
        self.0
            .azure_managed_identity
            .clone()
            .into_option()
            .map(PyAzureManagedIdentity::from)
    }
    #[getter]
    fn azure_storage_key(&self) -> ::core::option::Option<PyAzureStorageKey> {
        self.0
            .azure_storage_key
            .clone()
            .into_option()
            .map(PyAzureStorageKey::from)
    }
    #[getter]
    fn aws_iam_role(&self) -> ::core::option::Option<PyAwsIamRoleConfig> {
        self.0
            .aws_iam_role
            .clone()
            .into_option()
            .map(PyAwsIamRoleConfig::from)
    }
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
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(purpose)]
    fn set_purpose(&mut self, value: PyPurpose) {
        self.0.purpose =
            ::buffa::EnumValue::Known(<super::credentials::v1::Purpose as ::core::convert::From<
                _,
            >>::from(value));
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(read_only)]
    fn set_read_only(&mut self, value: ::core::option::Option<bool>) {
        self.0.read_only = value;
    }
    #[setter(skip_validation)]
    fn set_skip_validation(&mut self, value: ::core::option::Option<bool>) {
        self.0.skip_validation = value;
    }
    #[setter(azure_service_principal)]
    fn set_azure_service_principal(
        &mut self,
        value: ::core::option::Option<PyAzureServicePrincipal>,
    ) {
        self.0.azure_service_principal = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(azure_managed_identity)]
    fn set_azure_managed_identity(
        &mut self,
        value: ::core::option::Option<PyAzureManagedIdentity>,
    ) {
        self.0.azure_managed_identity = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(azure_storage_key)]
    fn set_azure_storage_key(&mut self, value: ::core::option::Option<PyAzureStorageKey>) {
        self.0.azure_storage_key = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(aws_iam_role)]
    fn set_aws_iam_role(&mut self, value: ::core::option::Option<PyAwsIamRoleConfig>) {
        self.0.aws_iam_role = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(databricks_gcp_service_account)]
    fn set_databricks_gcp_service_account(
        &mut self,
        value: ::core::option::Option<PyDatabricksGcpServiceAccount>,
    ) {
        self.0.databricks_gcp_service_account = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::credentials::v1::CreateCredentialRequest>
    for PyCreateCredentialRequest
{
    fn from(value: super::credentials::v1::CreateCredentialRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateCredentialRequest>
    for super::credentials::v1::CreateCredentialRequest
{
    fn from(value: PyCreateCredentialRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "Credential")]
#[derive(Clone, Debug)]
pub struct PyCredential(pub super::credentials::v1::Credential);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCredential {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            id = None,
            purpose = None,
            read_only = None,
            comment = None,
            owner = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None,
            used_for_managed_storage = None,
            full_name = None,
            azure_service_principal = None,
            azure_managed_identity = None,
            azure_storage_key = None,
            aws_iam_role = None,
            databricks_gcp_service_account = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        id: ::core::option::Option<::std::string::String>,
        purpose: ::core::option::Option<PyPurpose>,
        read_only: ::core::option::Option<bool>,
        comment: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
        used_for_managed_storage: ::core::option::Option<bool>,
        full_name: ::core::option::Option<::std::string::String>,
        azure_service_principal: ::core::option::Option<PyAzureServicePrincipal>,
        azure_managed_identity: ::core::option::Option<PyAzureManagedIdentity>,
        azure_storage_key: ::core::option::Option<PyAzureStorageKey>,
        aws_iam_role: ::core::option::Option<PyAwsIamRoleConfig>,
        databricks_gcp_service_account: ::core::option::Option<PyDatabricksGcpServiceAccount>,
    ) -> Self {
        let mut inner = <super::credentials::v1::Credential as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = id;
            inner.id = value;
        }
        if let ::core::option::Option::Some(value) = purpose {
            inner.purpose = ::buffa::EnumValue::Known(
                <super::credentials::v1::Purpose as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = read_only {
            inner.read_only = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        if let ::core::option::Option::Some(value) = used_for_managed_storage {
            inner.used_for_managed_storage = value;
        }
        {
            let value = full_name;
            inner.full_name = value;
        }
        {
            let value = azure_service_principal;
            inner.azure_service_principal = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = azure_managed_identity;
            inner.azure_managed_identity = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = azure_storage_key;
            inner.azure_storage_key = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = aws_iam_role;
            inner.aws_iam_role = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = databricks_gcp_service_account;
            inner.databricks_gcp_service_account = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }
    #[getter]
    fn purpose(&self) -> PyPurpose {
        PyPurpose::from(self.0.purpose.as_known().unwrap_or_default())
    }
    #[getter]
    fn read_only(&self) -> bool {
        self.0.read_only
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[getter]
    fn used_for_managed_storage(&self) -> bool {
        self.0.used_for_managed_storage
    }
    #[getter]
    fn full_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.full_name.clone()
    }
    #[getter]
    fn azure_service_principal(&self) -> ::core::option::Option<PyAzureServicePrincipal> {
        self.0
            .azure_service_principal
            .clone()
            .into_option()
            .map(PyAzureServicePrincipal::from)
    }
    #[getter]
    fn azure_managed_identity(&self) -> ::core::option::Option<PyAzureManagedIdentity> {
        self.0
            .azure_managed_identity
            .clone()
            .into_option()
            .map(PyAzureManagedIdentity::from)
    }
    #[getter]
    fn azure_storage_key(&self) -> ::core::option::Option<PyAzureStorageKey> {
        self.0
            .azure_storage_key
            .clone()
            .into_option()
            .map(PyAzureStorageKey::from)
    }
    #[getter]
    fn aws_iam_role(&self) -> ::core::option::Option<PyAwsIamRoleConfig> {
        self.0
            .aws_iam_role
            .clone()
            .into_option()
            .map(PyAwsIamRoleConfig::from)
    }
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
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(id)]
    fn set_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.id = value;
    }
    #[setter(purpose)]
    fn set_purpose(&mut self, value: PyPurpose) {
        self.0.purpose =
            ::buffa::EnumValue::Known(<super::credentials::v1::Purpose as ::core::convert::From<
                _,
            >>::from(value));
    }
    #[setter(read_only)]
    fn set_read_only(&mut self, value: bool) {
        self.0.read_only = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    #[setter(used_for_managed_storage)]
    fn set_used_for_managed_storage(&mut self, value: bool) {
        self.0.used_for_managed_storage = value;
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.full_name = value;
    }
    #[setter(azure_service_principal)]
    fn set_azure_service_principal(
        &mut self,
        value: ::core::option::Option<PyAzureServicePrincipal>,
    ) {
        self.0.azure_service_principal = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(azure_managed_identity)]
    fn set_azure_managed_identity(
        &mut self,
        value: ::core::option::Option<PyAzureManagedIdentity>,
    ) {
        self.0.azure_managed_identity = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(azure_storage_key)]
    fn set_azure_storage_key(&mut self, value: ::core::option::Option<PyAzureStorageKey>) {
        self.0.azure_storage_key = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(aws_iam_role)]
    fn set_aws_iam_role(&mut self, value: ::core::option::Option<PyAwsIamRoleConfig>) {
        self.0.aws_iam_role = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(databricks_gcp_service_account)]
    fn set_databricks_gcp_service_account(
        &mut self,
        value: ::core::option::Option<PyDatabricksGcpServiceAccount>,
    ) {
        self.0.databricks_gcp_service_account = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "DatabricksGcpServiceAccount")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn credential_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.credential_id.clone()
    }
    #[getter]
    fn email(&self) -> ::core::option::Option<::std::string::String> {
        self.0.email.clone()
    }
    #[getter]
    fn private_key_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.private_key_id.clone()
    }
    #[setter(credential_id)]
    fn set_credential_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.credential_id = value;
    }
    #[setter(email)]
    fn set_email(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.email = value;
    }
    #[setter(private_key_id)]
    fn set_private_key_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.private_key_id = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "DeleteCredentialRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteCredentialRequest(pub super::credentials::v1::DeleteCredentialRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteCredentialRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::credentials::v1::DeleteCredentialRequest as ::core::default::Default>::default(
            );
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::credentials::v1::DeleteCredentialRequest>
    for PyDeleteCredentialRequest
{
    fn from(value: super::credentials::v1::DeleteCredentialRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteCredentialRequest>
    for super::credentials::v1::DeleteCredentialRequest
{
    fn from(value: PyDeleteCredentialRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetCredentialRequest")]
#[derive(Clone, Debug)]
pub struct PyGetCredentialRequest(pub super::credentials::v1::GetCredentialRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetCredentialRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::credentials::v1::GetCredentialRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::credentials::v1::GetCredentialRequest>
    for PyGetCredentialRequest
{
    fn from(value: super::credentials::v1::GetCredentialRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetCredentialRequest>
    for super::credentials::v1::GetCredentialRequest
{
    fn from(value: PyGetCredentialRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListCredentialsRequest")]
#[derive(Clone, Debug)]
pub struct PyListCredentialsRequest(pub super::credentials::v1::ListCredentialsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListCredentialsRequest {
    #[new]
    #[pyo3(signature = (purpose = None, max_results = None, page_token = None))]
    fn new(
        purpose: ::core::option::Option<PyPurpose>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::credentials::v1::ListCredentialsRequest as ::core::default::Default>::default();
        {
            let value = purpose;
            inner.purpose = value.map(|e| {
                ::buffa::EnumValue::Known(
                    <super::credentials::v1::Purpose as ::core::convert::From<_>>::from(e),
                )
            });
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn purpose(&self) -> ::core::option::Option<PyPurpose> {
        self.0
            .purpose
            .as_ref()
            .and_then(|e| e.as_known())
            .map(PyPurpose::from)
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[setter(purpose)]
    fn set_purpose(&mut self, value: ::core::option::Option<PyPurpose>) {
        self.0.purpose = value.map(|e| {
            ::buffa::EnumValue::Known(<super::credentials::v1::Purpose as ::core::convert::From<
                _,
            >>::from(e))
        });
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::credentials::v1::ListCredentialsRequest>
    for PyListCredentialsRequest
{
    fn from(value: super::credentials::v1::ListCredentialsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListCredentialsRequest>
    for super::credentials::v1::ListCredentialsRequest
{
    fn from(value: PyListCredentialsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListCredentialsResponse")]
#[derive(Clone, Debug)]
pub struct PyListCredentialsResponse(pub super::credentials::v1::ListCredentialsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListCredentialsResponse {
    #[new]
    #[pyo3(signature = (credentials = None, next_page_token = None))]
    fn new(
        credentials: ::core::option::Option<::std::vec::Vec<PyCredential>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::credentials::v1::ListCredentialsResponse as ::core::default::Default>::default(
            );
        if let ::core::option::Option::Some(value) = credentials {
            inner.credentials = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn credentials(&self) -> ::std::vec::Vec<PyCredential> {
        self.0
            .credentials
            .iter()
            .cloned()
            .map(PyCredential::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(credentials)]
    fn set_credentials(&mut self, value: ::std::vec::Vec<PyCredential>) {
        self.0.credentials = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateCredentialRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateCredentialRequest(pub super::credentials::v1::UpdateCredentialRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateCredentialRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            new_name = None,
            comment = None,
            read_only = None,
            owner = None,
            skip_validation = None,
            force = None,
            azure_service_principal = None,
            azure_managed_identity = None,
            azure_storage_key = None,
            aws_iam_role = None,
            databricks_gcp_service_account = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        new_name: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        read_only: ::core::option::Option<bool>,
        owner: ::core::option::Option<::std::string::String>,
        skip_validation: ::core::option::Option<bool>,
        force: ::core::option::Option<bool>,
        azure_service_principal: ::core::option::Option<PyAzureServicePrincipal>,
        azure_managed_identity: ::core::option::Option<PyAzureManagedIdentity>,
        azure_storage_key: ::core::option::Option<PyAzureStorageKey>,
        aws_iam_role: ::core::option::Option<PyAwsIamRoleConfig>,
        databricks_gcp_service_account: ::core::option::Option<PyDatabricksGcpServiceAccount>,
    ) -> Self {
        let mut inner =
            <super::credentials::v1::UpdateCredentialRequest as ::core::default::Default>::default(
            );
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = read_only;
            inner.read_only = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = skip_validation;
            inner.skip_validation = value;
        }
        {
            let value = force;
            inner.force = value;
        }
        {
            let value = azure_service_principal;
            inner.azure_service_principal = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = azure_managed_identity;
            inner.azure_managed_identity = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = azure_storage_key;
            inner.azure_storage_key = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = aws_iam_role;
            inner.aws_iam_role = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = databricks_gcp_service_account;
            inner.databricks_gcp_service_account = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn read_only(&self) -> ::core::option::Option<bool> {
        self.0.read_only
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn skip_validation(&self) -> ::core::option::Option<bool> {
        self.0.skip_validation
    }
    #[getter]
    fn force(&self) -> ::core::option::Option<bool> {
        self.0.force
    }
    #[getter]
    fn azure_service_principal(&self) -> ::core::option::Option<PyAzureServicePrincipal> {
        self.0
            .azure_service_principal
            .clone()
            .into_option()
            .map(PyAzureServicePrincipal::from)
    }
    #[getter]
    fn azure_managed_identity(&self) -> ::core::option::Option<PyAzureManagedIdentity> {
        self.0
            .azure_managed_identity
            .clone()
            .into_option()
            .map(PyAzureManagedIdentity::from)
    }
    #[getter]
    fn azure_storage_key(&self) -> ::core::option::Option<PyAzureStorageKey> {
        self.0
            .azure_storage_key
            .clone()
            .into_option()
            .map(PyAzureStorageKey::from)
    }
    #[getter]
    fn aws_iam_role(&self) -> ::core::option::Option<PyAwsIamRoleConfig> {
        self.0
            .aws_iam_role
            .clone()
            .into_option()
            .map(PyAwsIamRoleConfig::from)
    }
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
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(read_only)]
    fn set_read_only(&mut self, value: ::core::option::Option<bool>) {
        self.0.read_only = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(skip_validation)]
    fn set_skip_validation(&mut self, value: ::core::option::Option<bool>) {
        self.0.skip_validation = value;
    }
    #[setter(force)]
    fn set_force(&mut self, value: ::core::option::Option<bool>) {
        self.0.force = value;
    }
    #[setter(azure_service_principal)]
    fn set_azure_service_principal(
        &mut self,
        value: ::core::option::Option<PyAzureServicePrincipal>,
    ) {
        self.0.azure_service_principal = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(azure_managed_identity)]
    fn set_azure_managed_identity(
        &mut self,
        value: ::core::option::Option<PyAzureManagedIdentity>,
    ) {
        self.0.azure_managed_identity = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(azure_storage_key)]
    fn set_azure_storage_key(&mut self, value: ::core::option::Option<PyAzureStorageKey>) {
        self.0.azure_storage_key = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(aws_iam_role)]
    fn set_aws_iam_role(&mut self, value: ::core::option::Option<PyAwsIamRoleConfig>) {
        self.0.aws_iam_role = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(databricks_gcp_service_account)]
    fn set_databricks_gcp_service_account(
        &mut self,
        value: ::core::option::Option<PyDatabricksGcpServiceAccount>,
    ) {
        self.0.databricks_gcp_service_account = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::credentials::v1::UpdateCredentialRequest>
    for PyUpdateCredentialRequest
{
    fn from(value: super::credentials::v1::UpdateCredentialRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateCredentialRequest>
    for super::credentials::v1::UpdateCredentialRequest
{
    fn from(value: PyUpdateCredentialRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "CreateExternalLocationRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateExternalLocationRequest(
    pub super::external_locations::v1::CreateExternalLocationRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateExternalLocationRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            url = None,
            credential_name = None,
            read_only = None,
            comment = None,
            skip_validation = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        url: ::core::option::Option<::std::string::String>,
        credential_name: ::core::option::Option<::std::string::String>,
        read_only: ::core::option::Option<bool>,
        comment: ::core::option::Option<::std::string::String>,
        skip_validation: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::external_locations::v1::CreateExternalLocationRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = url {
            inner.url = value;
        }
        if let ::core::option::Option::Some(value) = credential_name {
            inner.credential_name = value;
        }
        {
            let value = read_only;
            inner.read_only = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = skip_validation;
            inner.skip_validation = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn url(&self) -> ::std::string::String {
        self.0.url.clone()
    }
    #[getter]
    fn credential_name(&self) -> ::std::string::String {
        self.0.credential_name.clone()
    }
    #[getter]
    fn read_only(&self) -> ::core::option::Option<bool> {
        self.0.read_only
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn skip_validation(&self) -> ::core::option::Option<bool> {
        self.0.skip_validation
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(url)]
    fn set_url(&mut self, value: ::std::string::String) {
        self.0.url = value;
    }
    #[setter(credential_name)]
    fn set_credential_name(&mut self, value: ::std::string::String) {
        self.0.credential_name = value;
    }
    #[setter(read_only)]
    fn set_read_only(&mut self, value: ::core::option::Option<bool>) {
        self.0.read_only = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(skip_validation)]
    fn set_skip_validation(&mut self, value: ::core::option::Option<bool>) {
        self.0.skip_validation = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::external_locations::v1::CreateExternalLocationRequest>
    for PyCreateExternalLocationRequest
{
    fn from(value: super::external_locations::v1::CreateExternalLocationRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateExternalLocationRequest>
    for super::external_locations::v1::CreateExternalLocationRequest
{
    fn from(value: PyCreateExternalLocationRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteExternalLocationRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteExternalLocationRequest(
    pub super::external_locations::v1::DeleteExternalLocationRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteExternalLocationRequest {
    #[new]
    #[pyo3(signature = (name = None, force = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        force: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::external_locations::v1::DeleteExternalLocationRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = force;
            inner.force = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn force(&self) -> ::core::option::Option<bool> {
        self.0.force
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(force)]
    fn set_force(&mut self, value: ::core::option::Option<bool>) {
        self.0.force = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::external_locations::v1::DeleteExternalLocationRequest>
    for PyDeleteExternalLocationRequest
{
    fn from(value: super::external_locations::v1::DeleteExternalLocationRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteExternalLocationRequest>
    for super::external_locations::v1::DeleteExternalLocationRequest
{
    fn from(value: PyDeleteExternalLocationRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ExternalLocation")]
#[derive(Clone, Debug)]
pub struct PyExternalLocation(pub super::external_locations::v1::ExternalLocation);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyExternalLocation {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            url = None,
            credential_name = None,
            read_only = None,
            comment = None,
            owner = None,
            credential_id = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None,
            browse_only = None,
            external_location_id = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        url: ::core::option::Option<::std::string::String>,
        credential_name: ::core::option::Option<::std::string::String>,
        read_only: ::core::option::Option<bool>,
        comment: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
        credential_id: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
        browse_only: ::core::option::Option<bool>,
        external_location_id: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::external_locations::v1::ExternalLocation as ::core::default::Default>::default(
            );
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = url {
            inner.url = value;
        }
        if let ::core::option::Option::Some(value) = credential_name {
            inner.credential_name = value;
        }
        if let ::core::option::Option::Some(value) = read_only {
            inner.read_only = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        if let ::core::option::Option::Some(value) = credential_id {
            inner.credential_id = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        {
            let value = browse_only;
            inner.browse_only = value;
        }
        {
            let value = external_location_id;
            inner.external_location_id = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn url(&self) -> ::std::string::String {
        self.0.url.clone()
    }
    #[getter]
    fn credential_name(&self) -> ::std::string::String {
        self.0.credential_name.clone()
    }
    #[getter]
    fn read_only(&self) -> bool {
        self.0.read_only
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn credential_id(&self) -> ::std::string::String {
        self.0.credential_id.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }
    #[getter]
    fn external_location_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.external_location_id.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(url)]
    fn set_url(&mut self, value: ::std::string::String) {
        self.0.url = value;
    }
    #[setter(credential_name)]
    fn set_credential_name(&mut self, value: ::std::string::String) {
        self.0.credential_name = value;
    }
    #[setter(read_only)]
    fn set_read_only(&mut self, value: bool) {
        self.0.read_only = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(credential_id)]
    fn set_credential_id(&mut self, value: ::std::string::String) {
        self.0.credential_id = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    #[setter(browse_only)]
    fn set_browse_only(&mut self, value: ::core::option::Option<bool>) {
        self.0.browse_only = value;
    }
    #[setter(external_location_id)]
    fn set_external_location_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.external_location_id = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "GetExternalLocationRequest")]
#[derive(Clone, Debug)]
pub struct PyGetExternalLocationRequest(
    pub super::external_locations::v1::GetExternalLocationRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetExternalLocationRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner = <super::external_locations::v1::GetExternalLocationRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::external_locations::v1::GetExternalLocationRequest>
    for PyGetExternalLocationRequest
{
    fn from(value: super::external_locations::v1::GetExternalLocationRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetExternalLocationRequest>
    for super::external_locations::v1::GetExternalLocationRequest
{
    fn from(value: PyGetExternalLocationRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListExternalLocationsRequest")]
#[derive(Clone, Debug)]
pub struct PyListExternalLocationsRequest(
    pub super::external_locations::v1::ListExternalLocationsRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListExternalLocationsRequest {
    #[new]
    #[pyo3(signature = (max_results = None, page_token = None, include_browse = None))]
    fn new(
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::external_locations::v1::ListExternalLocationsRequest as ::core::default::Default>::default();
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::external_locations::v1::ListExternalLocationsRequest>
    for PyListExternalLocationsRequest
{
    fn from(value: super::external_locations::v1::ListExternalLocationsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListExternalLocationsRequest>
    for super::external_locations::v1::ListExternalLocationsRequest
{
    fn from(value: PyListExternalLocationsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListExternalLocationsResponse")]
#[derive(Clone, Debug)]
pub struct PyListExternalLocationsResponse(
    pub super::external_locations::v1::ListExternalLocationsResponse,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListExternalLocationsResponse {
    #[new]
    #[pyo3(signature = (external_locations = None, next_page_token = None))]
    fn new(
        external_locations: ::core::option::Option<::std::vec::Vec<PyExternalLocation>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::external_locations::v1::ListExternalLocationsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = external_locations {
            inner.external_locations = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn external_locations(&self) -> ::std::vec::Vec<PyExternalLocation> {
        self.0
            .external_locations
            .iter()
            .cloned()
            .map(PyExternalLocation::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(external_locations)]
    fn set_external_locations(&mut self, value: ::std::vec::Vec<PyExternalLocation>) {
        self.0.external_locations = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateExternalLocationRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateExternalLocationRequest(
    pub super::external_locations::v1::UpdateExternalLocationRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateExternalLocationRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            url = None,
            credential_name = None,
            read_only = None,
            owner = None,
            comment = None,
            new_name = None,
            force = None,
            skip_validation = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        url: ::core::option::Option<::std::string::String>,
        credential_name: ::core::option::Option<::std::string::String>,
        read_only: ::core::option::Option<bool>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        new_name: ::core::option::Option<::std::string::String>,
        force: ::core::option::Option<bool>,
        skip_validation: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::external_locations::v1::UpdateExternalLocationRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = url;
            inner.url = value;
        }
        {
            let value = credential_name;
            inner.credential_name = value;
        }
        {
            let value = read_only;
            inner.read_only = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        {
            let value = force;
            inner.force = value;
        }
        {
            let value = skip_validation;
            inner.skip_validation = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn url(&self) -> ::core::option::Option<::std::string::String> {
        self.0.url.clone()
    }
    #[getter]
    fn credential_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.credential_name.clone()
    }
    #[getter]
    fn read_only(&self) -> ::core::option::Option<bool> {
        self.0.read_only
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[getter]
    fn force(&self) -> ::core::option::Option<bool> {
        self.0.force
    }
    #[getter]
    fn skip_validation(&self) -> ::core::option::Option<bool> {
        self.0.skip_validation
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(url)]
    fn set_url(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.url = value;
    }
    #[setter(credential_name)]
    fn set_credential_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.credential_name = value;
    }
    #[setter(read_only)]
    fn set_read_only(&mut self, value: ::core::option::Option<bool>) {
        self.0.read_only = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    #[setter(force)]
    fn set_force(&mut self, value: ::core::option::Option<bool>) {
        self.0.force = value;
    }
    #[setter(skip_validation)]
    fn set_skip_validation(&mut self, value: ::core::option::Option<bool>) {
        self.0.skip_validation = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::external_locations::v1::UpdateExternalLocationRequest>
    for PyUpdateExternalLocationRequest
{
    fn from(value: super::external_locations::v1::UpdateExternalLocationRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateExternalLocationRequest>
    for super::external_locations::v1::UpdateExternalLocationRequest
{
    fn from(value: PyUpdateExternalLocationRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "CreateFunctionRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateFunctionRequest(pub super::functions::v1::CreateFunctionRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateFunctionRequest {
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
            routine_body_language = None,
            comment = None,
            properties = None
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
        routine_body_language: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
    ) -> Self {
        let mut inner =
            <super::functions::v1::CreateFunctionRequest as ::core::default::Default>::default();
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
        if let ::core::option::Option::Some(value) = parameter_style {
            inner.parameter_style = ::buffa::EnumValue::Known(
                <super::functions::v1::ParameterStyle as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = is_deterministic {
            inner.is_deterministic = value;
        }
        if let ::core::option::Option::Some(value) = sql_data_access {
            inner.sql_data_access = ::buffa::EnumValue::Known(
                <super::functions::v1::SqlDataAccess as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = is_null_call {
            inner.is_null_call = value;
        }
        if let ::core::option::Option::Some(value) = security_type {
            inner.security_type = ::buffa::EnumValue::Known(
                <super::functions::v1::SecurityType as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = routine_body {
            inner.routine_body = ::buffa::EnumValue::Known(
                <super::functions::v1::RoutineBody as ::core::convert::From<_>>::from(value),
            );
        }
        {
            let value = routine_definition;
            inner.routine_definition = value;
        }
        {
            let value = routine_body_language;
            inner.routine_body_language = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn data_type(&self) -> ::std::string::String {
        self.0.data_type.clone()
    }
    #[getter]
    fn full_data_type(&self) -> ::std::string::String {
        self.0.full_data_type.clone()
    }
    #[getter]
    fn input_params(&self) -> ::core::option::Option<PyFunctionParameterInfos> {
        self.0
            .input_params
            .clone()
            .into_option()
            .map(PyFunctionParameterInfos::from)
    }
    #[getter]
    fn parameter_style(&self) -> PyParameterStyle {
        PyParameterStyle::from(self.0.parameter_style.as_known().unwrap_or_default())
    }
    #[getter]
    fn is_deterministic(&self) -> bool {
        self.0.is_deterministic
    }
    #[getter]
    fn sql_data_access(&self) -> PySqlDataAccess {
        PySqlDataAccess::from(self.0.sql_data_access.as_known().unwrap_or_default())
    }
    #[getter]
    fn is_null_call(&self) -> bool {
        self.0.is_null_call
    }
    #[getter]
    fn security_type(&self) -> PySecurityType {
        PySecurityType::from(self.0.security_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn routine_body(&self) -> PyRoutineBody {
        PyRoutineBody::from(self.0.routine_body.as_known().unwrap_or_default())
    }
    #[getter]
    fn routine_definition(&self) -> ::core::option::Option<::std::string::String> {
        self.0.routine_definition.clone()
    }
    #[getter]
    fn routine_body_language(&self) -> ::core::option::Option<::std::string::String> {
        self.0.routine_body_language.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(data_type)]
    fn set_data_type(&mut self, value: ::std::string::String) {
        self.0.data_type = value;
    }
    #[setter(full_data_type)]
    fn set_full_data_type(&mut self, value: ::std::string::String) {
        self.0.full_data_type = value;
    }
    #[setter(input_params)]
    fn set_input_params(&mut self, value: ::core::option::Option<PyFunctionParameterInfos>) {
        self.0.input_params = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(parameter_style)]
    fn set_parameter_style(&mut self, value: PyParameterStyle) {
        self.0.parameter_style = ::buffa::EnumValue::Known(
            <super::functions::v1::ParameterStyle as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(is_deterministic)]
    fn set_is_deterministic(&mut self, value: bool) {
        self.0.is_deterministic = value;
    }
    #[setter(sql_data_access)]
    fn set_sql_data_access(&mut self, value: PySqlDataAccess) {
        self.0.sql_data_access = ::buffa::EnumValue::Known(
            <super::functions::v1::SqlDataAccess as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(is_null_call)]
    fn set_is_null_call(&mut self, value: bool) {
        self.0.is_null_call = value;
    }
    #[setter(security_type)]
    fn set_security_type(&mut self, value: PySecurityType) {
        self.0.security_type = ::buffa::EnumValue::Known(
            <super::functions::v1::SecurityType as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(routine_body)]
    fn set_routine_body(&mut self, value: PyRoutineBody) {
        self.0.routine_body = ::buffa::EnumValue::Known(
            <super::functions::v1::RoutineBody as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(routine_definition)]
    fn set_routine_definition(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.routine_definition = value;
    }
    #[setter(routine_body_language)]
    fn set_routine_body_language(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.routine_body_language = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::functions::v1::CreateFunctionRequest>
    for PyCreateFunctionRequest
{
    fn from(value: super::functions::v1::CreateFunctionRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateFunctionRequest>
    for super::functions::v1::CreateFunctionRequest
{
    fn from(value: PyCreateFunctionRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteFunctionRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteFunctionRequest(pub super::functions::v1::DeleteFunctionRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteFunctionRequest {
    #[new]
    #[pyo3(signature = (name = None, force = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        force: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::functions::v1::DeleteFunctionRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = force;
            inner.force = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn force(&self) -> ::core::option::Option<bool> {
        self.0.force
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(force)]
    fn set_force(&mut self, value: ::core::option::Option<bool>) {
        self.0.force = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::functions::v1::DeleteFunctionRequest>
    for PyDeleteFunctionRequest
{
    fn from(value: super::functions::v1::DeleteFunctionRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteFunctionRequest>
    for super::functions::v1::DeleteFunctionRequest
{
    fn from(value: PyDeleteFunctionRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "Function")]
#[derive(Clone, Debug)]
pub struct PyFunction(pub super::functions::v1::Function);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyFunction {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            catalog_name = None,
            schema_name = None,
            full_name = None,
            data_type = None,
            full_data_type = None,
            input_params = None,
            return_params = None,
            routine_body_language = None,
            routine_definition = None,
            routine_dependencies = None,
            parameter_style = None,
            is_deterministic = None,
            sql_data_access = None,
            is_null_call = None,
            security_type = None,
            specific_name = None,
            routine_body = None,
            comment = None,
            properties = None,
            owner = None,
            function_id = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        full_name: ::core::option::Option<::std::string::String>,
        data_type: ::core::option::Option<::std::string::String>,
        full_data_type: ::core::option::Option<::std::string::String>,
        input_params: ::core::option::Option<PyFunctionParameterInfos>,
        return_params: ::core::option::Option<::std::string::String>,
        routine_body_language: ::core::option::Option<::std::string::String>,
        routine_definition: ::core::option::Option<::std::string::String>,
        routine_dependencies: ::core::option::Option<::std::string::String>,
        parameter_style: ::core::option::Option<PyParameterStyle>,
        is_deterministic: ::core::option::Option<bool>,
        sql_data_access: ::core::option::Option<PySqlDataAccess>,
        is_null_call: ::core::option::Option<bool>,
        security_type: ::core::option::Option<PySecurityType>,
        specific_name: ::core::option::Option<::std::string::String>,
        routine_body: ::core::option::Option<PyRoutineBody>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        owner: ::core::option::Option<::std::string::String>,
        function_id: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::functions::v1::Function as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
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
            let value = return_params;
            inner.return_params = value;
        }
        {
            let value = routine_body_language;
            inner.routine_body_language = value;
        }
        {
            let value = routine_definition;
            inner.routine_definition = value;
        }
        {
            let value = routine_dependencies;
            inner.routine_dependencies = value;
        }
        if let ::core::option::Option::Some(value) = parameter_style {
            inner.parameter_style = ::buffa::EnumValue::Known(
                <super::functions::v1::ParameterStyle as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = is_deterministic {
            inner.is_deterministic = value;
        }
        if let ::core::option::Option::Some(value) = sql_data_access {
            inner.sql_data_access = ::buffa::EnumValue::Known(
                <super::functions::v1::SqlDataAccess as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = is_null_call {
            inner.is_null_call = value;
        }
        if let ::core::option::Option::Some(value) = security_type {
            inner.security_type = ::buffa::EnumValue::Known(
                <super::functions::v1::SecurityType as ::core::convert::From<_>>::from(value),
            );
        }
        {
            let value = specific_name;
            inner.specific_name = value;
        }
        if let ::core::option::Option::Some(value) = routine_body {
            inner.routine_body = ::buffa::EnumValue::Known(
                <super::functions::v1::RoutineBody as ::core::convert::From<_>>::from(value),
            );
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = function_id;
            inner.function_id = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn data_type(&self) -> ::std::string::String {
        self.0.data_type.clone()
    }
    #[getter]
    fn full_data_type(&self) -> ::std::string::String {
        self.0.full_data_type.clone()
    }
    #[getter]
    fn input_params(&self) -> ::core::option::Option<PyFunctionParameterInfos> {
        self.0
            .input_params
            .clone()
            .into_option()
            .map(PyFunctionParameterInfos::from)
    }
    #[getter]
    fn return_params(&self) -> ::core::option::Option<::std::string::String> {
        self.0.return_params.clone()
    }
    #[getter]
    fn routine_body_language(&self) -> ::core::option::Option<::std::string::String> {
        self.0.routine_body_language.clone()
    }
    #[getter]
    fn routine_definition(&self) -> ::core::option::Option<::std::string::String> {
        self.0.routine_definition.clone()
    }
    #[getter]
    fn routine_dependencies(&self) -> ::core::option::Option<::std::string::String> {
        self.0.routine_dependencies.clone()
    }
    #[getter]
    fn parameter_style(&self) -> PyParameterStyle {
        PyParameterStyle::from(self.0.parameter_style.as_known().unwrap_or_default())
    }
    #[getter]
    fn is_deterministic(&self) -> bool {
        self.0.is_deterministic
    }
    #[getter]
    fn sql_data_access(&self) -> PySqlDataAccess {
        PySqlDataAccess::from(self.0.sql_data_access.as_known().unwrap_or_default())
    }
    #[getter]
    fn is_null_call(&self) -> bool {
        self.0.is_null_call
    }
    #[getter]
    fn security_type(&self) -> PySecurityType {
        PySecurityType::from(self.0.security_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn specific_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.specific_name.clone()
    }
    #[getter]
    fn routine_body(&self) -> PyRoutineBody {
        PyRoutineBody::from(self.0.routine_body.as_known().unwrap_or_default())
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn function_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.function_id.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(data_type)]
    fn set_data_type(&mut self, value: ::std::string::String) {
        self.0.data_type = value;
    }
    #[setter(full_data_type)]
    fn set_full_data_type(&mut self, value: ::std::string::String) {
        self.0.full_data_type = value;
    }
    #[setter(input_params)]
    fn set_input_params(&mut self, value: ::core::option::Option<PyFunctionParameterInfos>) {
        self.0.input_params = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(return_params)]
    fn set_return_params(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.return_params = value;
    }
    #[setter(routine_body_language)]
    fn set_routine_body_language(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.routine_body_language = value;
    }
    #[setter(routine_definition)]
    fn set_routine_definition(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.routine_definition = value;
    }
    #[setter(routine_dependencies)]
    fn set_routine_dependencies(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.routine_dependencies = value;
    }
    #[setter(parameter_style)]
    fn set_parameter_style(&mut self, value: PyParameterStyle) {
        self.0.parameter_style = ::buffa::EnumValue::Known(
            <super::functions::v1::ParameterStyle as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(is_deterministic)]
    fn set_is_deterministic(&mut self, value: bool) {
        self.0.is_deterministic = value;
    }
    #[setter(sql_data_access)]
    fn set_sql_data_access(&mut self, value: PySqlDataAccess) {
        self.0.sql_data_access = ::buffa::EnumValue::Known(
            <super::functions::v1::SqlDataAccess as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(is_null_call)]
    fn set_is_null_call(&mut self, value: bool) {
        self.0.is_null_call = value;
    }
    #[setter(security_type)]
    fn set_security_type(&mut self, value: PySecurityType) {
        self.0.security_type = ::buffa::EnumValue::Known(
            <super::functions::v1::SecurityType as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(specific_name)]
    fn set_specific_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.specific_name = value;
    }
    #[setter(routine_body)]
    fn set_routine_body(&mut self, value: PyRoutineBody) {
        self.0.routine_body = ::buffa::EnumValue::Known(
            <super::functions::v1::RoutineBody as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(function_id)]
    fn set_function_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.function_id = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "FunctionParameterInfo")]
#[derive(Clone, Debug)]
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
        if let ::core::option::Option::Some(value) = type_name {
            inner.type_name = ::buffa::EnumValue::Known(
                <super::tables::v1::ColumnTypeName as ::core::convert::From<_>>::from(value),
            );
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
        if let ::core::option::Option::Some(value) = parameter_mode {
            inner.parameter_mode = ::buffa::EnumValue::Known(
                <super::functions::v1::ParameterMode as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = parameter_type {
            inner.parameter_type = ::buffa::EnumValue::Known(
                <super::functions::v1::FunctionParameterType as ::core::convert::From<_>>::from(
                    value,
                ),
            );
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
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn type_text(&self) -> ::std::string::String {
        self.0.type_text.clone()
    }
    #[getter]
    fn type_json(&self) -> ::core::option::Option<::std::string::String> {
        self.0.type_json.clone()
    }
    #[getter]
    fn type_name(&self) -> PyColumnTypeName {
        PyColumnTypeName::from(self.0.type_name.as_known().unwrap_or_default())
    }
    #[getter]
    fn type_precision(&self) -> ::core::option::Option<i32> {
        self.0.type_precision
    }
    #[getter]
    fn type_scale(&self) -> ::core::option::Option<i32> {
        self.0.type_scale
    }
    #[getter]
    fn type_interval_type(&self) -> ::core::option::Option<::std::string::String> {
        self.0.type_interval_type.clone()
    }
    #[getter]
    fn position(&self) -> ::core::option::Option<i32> {
        self.0.position
    }
    #[getter]
    fn parameter_mode(&self) -> PyParameterMode {
        PyParameterMode::from(self.0.parameter_mode.as_known().unwrap_or_default())
    }
    #[getter]
    fn parameter_type(&self) -> PyFunctionParameterType {
        PyFunctionParameterType::from(self.0.parameter_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn parameter_default(&self) -> ::core::option::Option<::std::string::String> {
        self.0.parameter_default.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(type_text)]
    fn set_type_text(&mut self, value: ::std::string::String) {
        self.0.type_text = value;
    }
    #[setter(type_json)]
    fn set_type_json(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.type_json = value;
    }
    #[setter(type_name)]
    fn set_type_name(&mut self, value: PyColumnTypeName) {
        self.0.type_name = ::buffa::EnumValue::Known(
            <super::tables::v1::ColumnTypeName as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(type_precision)]
    fn set_type_precision(&mut self, value: ::core::option::Option<i32>) {
        self.0.type_precision = value;
    }
    #[setter(type_scale)]
    fn set_type_scale(&mut self, value: ::core::option::Option<i32>) {
        self.0.type_scale = value;
    }
    #[setter(type_interval_type)]
    fn set_type_interval_type(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.type_interval_type = value;
    }
    #[setter(position)]
    fn set_position(&mut self, value: ::core::option::Option<i32>) {
        self.0.position = value;
    }
    #[setter(parameter_mode)]
    fn set_parameter_mode(&mut self, value: PyParameterMode) {
        self.0.parameter_mode = ::buffa::EnumValue::Known(
            <super::functions::v1::ParameterMode as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(parameter_type)]
    fn set_parameter_type(&mut self, value: PyFunctionParameterType) {
        self.0.parameter_type = ::buffa::EnumValue::Known(
            <super::functions::v1::FunctionParameterType as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(parameter_default)]
    fn set_parameter_default(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.parameter_default = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "FunctionParameterInfos")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn parameters(&self) -> ::std::vec::Vec<PyFunctionParameterInfo> {
        self.0
            .parameters
            .iter()
            .cloned()
            .map(PyFunctionParameterInfo::from)
            .collect()
    }
    #[setter(parameters)]
    fn set_parameters(&mut self, value: ::std::vec::Vec<PyFunctionParameterInfo>) {
        self.0.parameters = value.into_iter().map(::core::convert::Into::into).collect();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "GetFunctionRequest")]
#[derive(Clone, Debug)]
pub struct PyGetFunctionRequest(pub super::functions::v1::GetFunctionRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetFunctionRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::functions::v1::GetFunctionRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::functions::v1::GetFunctionRequest> for PyGetFunctionRequest {
    fn from(value: super::functions::v1::GetFunctionRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetFunctionRequest> for super::functions::v1::GetFunctionRequest {
    fn from(value: PyGetFunctionRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListFunctionsRequest")]
#[derive(Clone, Debug)]
pub struct PyListFunctionsRequest(pub super::functions::v1::ListFunctionsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListFunctionsRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name = None,
            max_results = None,
            page_token = None,
            include_browse = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::functions::v1::ListFunctionsRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::functions::v1::ListFunctionsRequest> for PyListFunctionsRequest {
    fn from(value: super::functions::v1::ListFunctionsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListFunctionsRequest> for super::functions::v1::ListFunctionsRequest {
    fn from(value: PyListFunctionsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListFunctionsResponse")]
#[derive(Clone, Debug)]
pub struct PyListFunctionsResponse(pub super::functions::v1::ListFunctionsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListFunctionsResponse {
    #[new]
    #[pyo3(signature = (functions = None, next_page_token = None))]
    fn new(
        functions: ::core::option::Option<::std::vec::Vec<PyFunction>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::functions::v1::ListFunctionsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = functions {
            inner.functions = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn functions(&self) -> ::std::vec::Vec<PyFunction> {
        self.0
            .functions
            .iter()
            .cloned()
            .map(PyFunction::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(functions)]
    fn set_functions(&mut self, value: ::std::vec::Vec<PyFunction>) {
        self.0.functions = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateFunctionRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateFunctionRequest(pub super::functions::v1::UpdateFunctionRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateFunctionRequest {
    #[new]
    #[pyo3(signature = (name = None, owner = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::functions::v1::UpdateFunctionRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::functions::v1::UpdateFunctionRequest>
    for PyUpdateFunctionRequest
{
    fn from(value: super::functions::v1::UpdateFunctionRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateFunctionRequest>
    for super::functions::v1::UpdateFunctionRequest
{
    fn from(value: PyUpdateFunctionRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "CreatePolicyRequest")]
#[derive(Clone, Debug)]
pub struct PyCreatePolicyRequest(pub super::policies::v1::CreatePolicyRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreatePolicyRequest {
    #[new]
    #[pyo3(
        signature = (
            on_securable_type = None,
            on_securable_fullname = None,
            policy_info = None
        )
    )]
    fn new(
        on_securable_type: ::core::option::Option<::std::string::String>,
        on_securable_fullname: ::core::option::Option<::std::string::String>,
        policy_info: ::core::option::Option<PyPolicyInfo>,
    ) -> Self {
        let mut inner =
            <super::policies::v1::CreatePolicyRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = on_securable_type {
            inner.on_securable_type = value;
        }
        if let ::core::option::Option::Some(value) = on_securable_fullname {
            inner.on_securable_fullname = value;
        }
        {
            let value = policy_info;
            inner.policy_info = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        Self(inner)
    }
    #[getter]
    fn on_securable_type(&self) -> ::std::string::String {
        self.0.on_securable_type.clone()
    }
    #[getter]
    fn on_securable_fullname(&self) -> ::std::string::String {
        self.0.on_securable_fullname.clone()
    }
    #[getter]
    fn policy_info(&self) -> ::core::option::Option<PyPolicyInfo> {
        self.0
            .policy_info
            .clone()
            .into_option()
            .map(PyPolicyInfo::from)
    }
    #[setter(on_securable_type)]
    fn set_on_securable_type(&mut self, value: ::std::string::String) {
        self.0.on_securable_type = value;
    }
    #[setter(on_securable_fullname)]
    fn set_on_securable_fullname(&mut self, value: ::std::string::String) {
        self.0.on_securable_fullname = value;
    }
    #[setter(policy_info)]
    fn set_policy_info(&mut self, value: ::core::option::Option<PyPolicyInfo>) {
        self.0.policy_info = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::policies::v1::CreatePolicyRequest> for PyCreatePolicyRequest {
    fn from(value: super::policies::v1::CreatePolicyRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreatePolicyRequest> for super::policies::v1::CreatePolicyRequest {
    fn from(value: PyCreatePolicyRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeletePolicyRequest")]
#[derive(Clone, Debug)]
pub struct PyDeletePolicyRequest(pub super::policies::v1::DeletePolicyRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeletePolicyRequest {
    #[new]
    #[pyo3(
        signature = (on_securable_type = None, on_securable_fullname = None, name = None)
    )]
    fn new(
        on_securable_type: ::core::option::Option<::std::string::String>,
        on_securable_fullname: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::policies::v1::DeletePolicyRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = on_securable_type {
            inner.on_securable_type = value;
        }
        if let ::core::option::Option::Some(value) = on_securable_fullname {
            inner.on_securable_fullname = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn on_securable_type(&self) -> ::std::string::String {
        self.0.on_securable_type.clone()
    }
    #[getter]
    fn on_securable_fullname(&self) -> ::std::string::String {
        self.0.on_securable_fullname.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(on_securable_type)]
    fn set_on_securable_type(&mut self, value: ::std::string::String) {
        self.0.on_securable_type = value;
    }
    #[setter(on_securable_fullname)]
    fn set_on_securable_fullname(&mut self, value: ::std::string::String) {
        self.0.on_securable_fullname = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::policies::v1::DeletePolicyRequest> for PyDeletePolicyRequest {
    fn from(value: super::policies::v1::DeletePolicyRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeletePolicyRequest> for super::policies::v1::DeletePolicyRequest {
    fn from(value: PyDeletePolicyRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "FunctionArg")]
#[derive(Clone, Debug)]
pub struct PyFunctionArg(pub super::policies::v1::FunctionArg);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyFunctionArg {
    #[new]
    fn new() -> Self {
        Self(::core::default::Default::default())
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "FunctionRef")]
#[derive(Clone, Debug)]
pub struct PyFunctionRef(pub super::policies::v1::FunctionRef);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyFunctionRef {
    #[new]
    #[pyo3(signature = (function_name = None, using = None))]
    fn new(
        function_name: ::core::option::Option<::std::string::String>,
        using: ::core::option::Option<::std::vec::Vec<PyFunctionArg>>,
    ) -> Self {
        let mut inner = <super::policies::v1::FunctionRef as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = function_name {
            inner.function_name = value;
        }
        if let ::core::option::Option::Some(value) = using {
            inner.using = value.into_iter().map(::core::convert::Into::into).collect();
        }
        Self(inner)
    }
    #[getter]
    fn function_name(&self) -> ::std::string::String {
        self.0.function_name.clone()
    }
    #[getter]
    fn using(&self) -> ::std::vec::Vec<PyFunctionArg> {
        self.0
            .using
            .iter()
            .cloned()
            .map(PyFunctionArg::from)
            .collect()
    }
    #[setter(function_name)]
    fn set_function_name(&mut self, value: ::std::string::String) {
        self.0.function_name = value;
    }
    #[setter(using)]
    fn set_using(&mut self, value: ::std::vec::Vec<PyFunctionArg>) {
        self.0.using = value.into_iter().map(::core::convert::Into::into).collect();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "GetPolicyRequest")]
#[derive(Clone, Debug)]
pub struct PyGetPolicyRequest(pub super::policies::v1::GetPolicyRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetPolicyRequest {
    #[new]
    #[pyo3(
        signature = (on_securable_type = None, on_securable_fullname = None, name = None)
    )]
    fn new(
        on_securable_type: ::core::option::Option<::std::string::String>,
        on_securable_fullname: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::policies::v1::GetPolicyRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = on_securable_type {
            inner.on_securable_type = value;
        }
        if let ::core::option::Option::Some(value) = on_securable_fullname {
            inner.on_securable_fullname = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn on_securable_type(&self) -> ::std::string::String {
        self.0.on_securable_type.clone()
    }
    #[getter]
    fn on_securable_fullname(&self) -> ::std::string::String {
        self.0.on_securable_fullname.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(on_securable_type)]
    fn set_on_securable_type(&mut self, value: ::std::string::String) {
        self.0.on_securable_type = value;
    }
    #[setter(on_securable_fullname)]
    fn set_on_securable_fullname(&mut self, value: ::std::string::String) {
        self.0.on_securable_fullname = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::policies::v1::GetPolicyRequest> for PyGetPolicyRequest {
    fn from(value: super::policies::v1::GetPolicyRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetPolicyRequest> for super::policies::v1::GetPolicyRequest {
    fn from(value: PyGetPolicyRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListPoliciesRequest")]
#[derive(Clone, Debug)]
pub struct PyListPoliciesRequest(pub super::policies::v1::ListPoliciesRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListPoliciesRequest {
    #[new]
    #[pyo3(
        signature = (
            on_securable_type = None,
            on_securable_fullname = None,
            include_inherited = None,
            max_results = None,
            page_token = None
        )
    )]
    fn new(
        on_securable_type: ::core::option::Option<::std::string::String>,
        on_securable_fullname: ::core::option::Option<::std::string::String>,
        include_inherited: ::core::option::Option<bool>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::policies::v1::ListPoliciesRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = on_securable_type {
            inner.on_securable_type = value;
        }
        if let ::core::option::Option::Some(value) = on_securable_fullname {
            inner.on_securable_fullname = value;
        }
        {
            let value = include_inherited;
            inner.include_inherited = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn on_securable_type(&self) -> ::std::string::String {
        self.0.on_securable_type.clone()
    }
    #[getter]
    fn on_securable_fullname(&self) -> ::std::string::String {
        self.0.on_securable_fullname.clone()
    }
    #[getter]
    fn include_inherited(&self) -> ::core::option::Option<bool> {
        self.0.include_inherited
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[setter(on_securable_type)]
    fn set_on_securable_type(&mut self, value: ::std::string::String) {
        self.0.on_securable_type = value;
    }
    #[setter(on_securable_fullname)]
    fn set_on_securable_fullname(&mut self, value: ::std::string::String) {
        self.0.on_securable_fullname = value;
    }
    #[setter(include_inherited)]
    fn set_include_inherited(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_inherited = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::policies::v1::ListPoliciesRequest> for PyListPoliciesRequest {
    fn from(value: super::policies::v1::ListPoliciesRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListPoliciesRequest> for super::policies::v1::ListPoliciesRequest {
    fn from(value: PyListPoliciesRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListPoliciesResponse")]
#[derive(Clone, Debug)]
pub struct PyListPoliciesResponse(pub super::policies::v1::ListPoliciesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListPoliciesResponse {
    #[new]
    #[pyo3(signature = (policies = None, next_page_token = None))]
    fn new(
        policies: ::core::option::Option<::std::vec::Vec<PyPolicyInfo>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::policies::v1::ListPoliciesResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = policies {
            inner.policies = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn policies(&self) -> ::std::vec::Vec<PyPolicyInfo> {
        self.0
            .policies
            .iter()
            .cloned()
            .map(PyPolicyInfo::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(policies)]
    fn set_policies(&mut self, value: ::std::vec::Vec<PyPolicyInfo>) {
        self.0.policies = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "MatchColumn")]
#[derive(Clone, Debug)]
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
        if let ::core::option::Option::Some(value) = alias {
            inner.alias = value;
        }
        if let ::core::option::Option::Some(value) = condition {
            inner.condition = value;
        }
        Self(inner)
    }
    #[getter]
    fn alias(&self) -> ::std::string::String {
        self.0.alias.clone()
    }
    #[getter]
    fn condition(&self) -> ::std::string::String {
        self.0.condition.clone()
    }
    #[setter(alias)]
    fn set_alias(&mut self, value: ::std::string::String) {
        self.0.alias = value;
    }
    #[setter(condition)]
    fn set_condition(&mut self, value: ::std::string::String) {
        self.0.condition = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "PolicyInfo")]
#[derive(Clone, Debug)]
pub struct PyPolicyInfo(pub super::policies::v1::PolicyInfo);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyPolicyInfo {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            id = None,
            on_securable_type = None,
            on_securable_fullname = None,
            policy_type = None,
            to_principals = None,
            except_principals = None,
            when_condition = None,
            match_columns = None,
            comment = None,
            created_at = None,
            updated_at = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        id: ::core::option::Option<::std::string::String>,
        on_securable_type: ::core::option::Option<::std::string::String>,
        on_securable_fullname: ::core::option::Option<::std::string::String>,
        policy_type: ::core::option::Option<PyPolicyType>,
        to_principals: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        except_principals: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        when_condition: ::core::option::Option<::std::string::String>,
        match_columns: ::core::option::Option<::std::vec::Vec<PyMatchColumn>>,
        comment: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        updated_at: ::core::option::Option<i64>,
    ) -> Self {
        let mut inner = <super::policies::v1::PolicyInfo as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = id;
            inner.id = value;
        }
        if let ::core::option::Option::Some(value) = on_securable_type {
            inner.on_securable_type = value;
        }
        if let ::core::option::Option::Some(value) = on_securable_fullname {
            inner.on_securable_fullname = value;
        }
        if let ::core::option::Option::Some(value) = policy_type {
            inner.policy_type = ::buffa::EnumValue::Known(
                <super::policies::v1::PolicyType as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = to_principals {
            inner.to_principals = value;
        }
        if let ::core::option::Option::Some(value) = except_principals {
            inner.except_principals = value;
        }
        {
            let value = when_condition;
            inner.when_condition = value;
        }
        if let ::core::option::Option::Some(value) = match_columns {
            inner.match_columns = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }
    #[getter]
    fn on_securable_type(&self) -> ::std::string::String {
        self.0.on_securable_type.clone()
    }
    #[getter]
    fn on_securable_fullname(&self) -> ::std::string::String {
        self.0.on_securable_fullname.clone()
    }
    #[getter]
    fn policy_type(&self) -> PyPolicyType {
        PyPolicyType::from(self.0.policy_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn to_principals(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.to_principals.clone()
    }
    #[getter]
    fn except_principals(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.except_principals.clone()
    }
    #[getter]
    fn when_condition(&self) -> ::core::option::Option<::std::string::String> {
        self.0.when_condition.clone()
    }
    #[getter]
    fn match_columns(&self) -> ::std::vec::Vec<PyMatchColumn> {
        self.0
            .match_columns
            .iter()
            .cloned()
            .map(PyMatchColumn::from)
            .collect()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(id)]
    fn set_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.id = value;
    }
    #[setter(on_securable_type)]
    fn set_on_securable_type(&mut self, value: ::std::string::String) {
        self.0.on_securable_type = value;
    }
    #[setter(on_securable_fullname)]
    fn set_on_securable_fullname(&mut self, value: ::std::string::String) {
        self.0.on_securable_fullname = value;
    }
    #[setter(policy_type)]
    fn set_policy_type(&mut self, value: PyPolicyType) {
        self.0.policy_type =
            ::buffa::EnumValue::Known(<super::policies::v1::PolicyType as ::core::convert::From<
                _,
            >>::from(value));
    }
    #[setter(to_principals)]
    fn set_to_principals(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.to_principals = value;
    }
    #[setter(except_principals)]
    fn set_except_principals(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.except_principals = value;
    }
    #[setter(when_condition)]
    fn set_when_condition(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.when_condition = value;
    }
    #[setter(match_columns)]
    fn set_match_columns(&mut self, value: ::std::vec::Vec<PyMatchColumn>) {
        self.0.match_columns = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdatePolicyRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdatePolicyRequest(pub super::policies::v1::UpdatePolicyRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdatePolicyRequest {
    #[new]
    #[pyo3(
        signature = (
            on_securable_type = None,
            on_securable_fullname = None,
            name = None,
            policy_info = None,
            update_mask = None
        )
    )]
    fn new(
        on_securable_type: ::core::option::Option<::std::string::String>,
        on_securable_fullname: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
        policy_info: ::core::option::Option<PyPolicyInfo>,
        update_mask: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::policies::v1::UpdatePolicyRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = on_securable_type {
            inner.on_securable_type = value;
        }
        if let ::core::option::Option::Some(value) = on_securable_fullname {
            inner.on_securable_fullname = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = policy_info;
            inner.policy_info = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = update_mask;
            inner.update_mask = value;
        }
        Self(inner)
    }
    #[getter]
    fn on_securable_type(&self) -> ::std::string::String {
        self.0.on_securable_type.clone()
    }
    #[getter]
    fn on_securable_fullname(&self) -> ::std::string::String {
        self.0.on_securable_fullname.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn policy_info(&self) -> ::core::option::Option<PyPolicyInfo> {
        self.0
            .policy_info
            .clone()
            .into_option()
            .map(PyPolicyInfo::from)
    }
    #[getter]
    fn update_mask(&self) -> ::core::option::Option<::std::string::String> {
        self.0.update_mask.clone()
    }
    #[setter(on_securable_type)]
    fn set_on_securable_type(&mut self, value: ::std::string::String) {
        self.0.on_securable_type = value;
    }
    #[setter(on_securable_fullname)]
    fn set_on_securable_fullname(&mut self, value: ::std::string::String) {
        self.0.on_securable_fullname = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(policy_info)]
    fn set_policy_info(&mut self, value: ::core::option::Option<PyPolicyInfo>) {
        self.0.policy_info = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(update_mask)]
    fn set_update_mask(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.update_mask = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::policies::v1::UpdatePolicyRequest> for PyUpdatePolicyRequest {
    fn from(value: super::policies::v1::UpdatePolicyRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdatePolicyRequest> for super::policies::v1::UpdatePolicyRequest {
    fn from(value: PyUpdatePolicyRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "CreateProviderRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateProviderRequest(pub super::providers::v1::CreateProviderRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateProviderRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            authentication_type = None,
            owner = None,
            comment = None,
            recipient_profile_str = None,
            properties = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        authentication_type: ::core::option::Option<PyProviderAuthenticationType>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        recipient_profile_str: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
    ) -> Self {
        let mut inner =
            <super::providers::v1::CreateProviderRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = authentication_type {
            inner.authentication_type =
                ::buffa::EnumValue::Known(
                    <super::providers::v1::ProviderAuthenticationType as ::core::convert::From<
                        _,
                    >>::from(value),
                );
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = recipient_profile_str;
            inner.recipient_profile_str = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn authentication_type(&self) -> PyProviderAuthenticationType {
        PyProviderAuthenticationType::from(
            self.0.authentication_type.as_known().unwrap_or_default(),
        )
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn recipient_profile_str(&self) -> ::core::option::Option<::std::string::String> {
        self.0.recipient_profile_str.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(authentication_type)]
    fn set_authentication_type(&mut self, value: PyProviderAuthenticationType) {
        self.0.authentication_type = ::buffa::EnumValue::Known(
            <super::providers::v1::ProviderAuthenticationType as ::core::convert::From<_>>::from(
                value,
            ),
        );
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(recipient_profile_str)]
    fn set_recipient_profile_str(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.recipient_profile_str = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::providers::v1::CreateProviderRequest>
    for PyCreateProviderRequest
{
    fn from(value: super::providers::v1::CreateProviderRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateProviderRequest>
    for super::providers::v1::CreateProviderRequest
{
    fn from(value: PyCreateProviderRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteProviderRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteProviderRequest(pub super::providers::v1::DeleteProviderRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteProviderRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::providers::v1::DeleteProviderRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::providers::v1::DeleteProviderRequest>
    for PyDeleteProviderRequest
{
    fn from(value: super::providers::v1::DeleteProviderRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteProviderRequest>
    for super::providers::v1::DeleteProviderRequest
{
    fn from(value: PyDeleteProviderRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetProviderRequest")]
#[derive(Clone, Debug)]
pub struct PyGetProviderRequest(pub super::providers::v1::GetProviderRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetProviderRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::providers::v1::GetProviderRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::providers::v1::GetProviderRequest> for PyGetProviderRequest {
    fn from(value: super::providers::v1::GetProviderRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetProviderRequest> for super::providers::v1::GetProviderRequest {
    fn from(value: PyGetProviderRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListProvidersRequest")]
#[derive(Clone, Debug)]
pub struct PyListProvidersRequest(pub super::providers::v1::ListProvidersRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListProvidersRequest {
    #[new]
    #[pyo3(signature = (max_results = None, page_token = None))]
    fn new(
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::providers::v1::ListProvidersRequest as ::core::default::Default>::default();
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::providers::v1::ListProvidersRequest> for PyListProvidersRequest {
    fn from(value: super::providers::v1::ListProvidersRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListProvidersRequest> for super::providers::v1::ListProvidersRequest {
    fn from(value: PyListProvidersRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListProvidersResponse")]
#[derive(Clone, Debug)]
pub struct PyListProvidersResponse(pub super::providers::v1::ListProvidersResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListProvidersResponse {
    #[new]
    #[pyo3(signature = (providers = None, next_page_token = None))]
    fn new(
        providers: ::core::option::Option<::std::vec::Vec<PyProvider>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::providers::v1::ListProvidersResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = providers {
            inner.providers = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn providers(&self) -> ::std::vec::Vec<PyProvider> {
        self.0
            .providers
            .iter()
            .cloned()
            .map(PyProvider::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(providers)]
    fn set_providers(&mut self, value: ::std::vec::Vec<PyProvider>) {
        self.0.providers = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "Provider")]
#[derive(Clone, Debug)]
pub struct PyProvider(pub super::providers::v1::Provider);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyProvider {
    #[new]
    #[pyo3(
        signature = (
            id = None,
            name = None,
            authentication_type = None,
            owner = None,
            comment = None,
            recipient_profile_str = None,
            properties = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None
        )
    )]
    fn new(
        id: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
        authentication_type: ::core::option::Option<PyProviderAuthenticationType>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        recipient_profile_str: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::providers::v1::Provider as ::core::default::Default>::default();
        {
            let value = id;
            inner.id = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = authentication_type {
            inner.authentication_type =
                ::buffa::EnumValue::Known(
                    <super::providers::v1::ProviderAuthenticationType as ::core::convert::From<
                        _,
                    >>::from(value),
                );
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = recipient_profile_str;
            inner.recipient_profile_str = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        Self(inner)
    }
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn authentication_type(&self) -> PyProviderAuthenticationType {
        PyProviderAuthenticationType::from(
            self.0.authentication_type.as_known().unwrap_or_default(),
        )
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn recipient_profile_str(&self) -> ::core::option::Option<::std::string::String> {
        self.0.recipient_profile_str.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[setter(id)]
    fn set_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.id = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(authentication_type)]
    fn set_authentication_type(&mut self, value: PyProviderAuthenticationType) {
        self.0.authentication_type = ::buffa::EnumValue::Known(
            <super::providers::v1::ProviderAuthenticationType as ::core::convert::From<_>>::from(
                value,
            ),
        );
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(recipient_profile_str)]
    fn set_recipient_profile_str(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.recipient_profile_str = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateProviderRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateProviderRequest(pub super::providers::v1::UpdateProviderRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateProviderRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            new_name = None,
            owner = None,
            comment = None,
            recipient_profile_str = None,
            properties = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        new_name: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        recipient_profile_str: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
    ) -> Self {
        let mut inner =
            <super::providers::v1::UpdateProviderRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = recipient_profile_str;
            inner.recipient_profile_str = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn recipient_profile_str(&self) -> ::core::option::Option<::std::string::String> {
        self.0.recipient_profile_str.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(recipient_profile_str)]
    fn set_recipient_profile_str(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.recipient_profile_str = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::providers::v1::UpdateProviderRequest>
    for PyUpdateProviderRequest
{
    fn from(value: super::providers::v1::UpdateProviderRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateProviderRequest>
    for super::providers::v1::UpdateProviderRequest
{
    fn from(value: PyUpdateProviderRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "CreateRecipientRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateRecipientRequest(pub super::recipients::v1::CreateRecipientRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateRecipientRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            authentication_type = None,
            owner = None,
            comment = None,
            properties = None,
            expiration_time = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        authentication_type: ::core::option::Option<PyAuthenticationType>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        expiration_time: ::core::option::Option<i64>,
    ) -> Self {
        let mut inner =
            <super::recipients::v1::CreateRecipientRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = authentication_type {
            inner.authentication_type = ::buffa::EnumValue::Known(
                <super::recipients::v1::AuthenticationType as ::core::convert::From<_>>::from(
                    value,
                ),
            );
        }
        if let ::core::option::Option::Some(value) = owner {
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = expiration_time;
            inner.expiration_time = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn authentication_type(&self) -> PyAuthenticationType {
        PyAuthenticationType::from(self.0.authentication_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn owner(&self) -> ::std::string::String {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn expiration_time(&self) -> ::core::option::Option<i64> {
        self.0.expiration_time
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(authentication_type)]
    fn set_authentication_type(&mut self, value: PyAuthenticationType) {
        self.0.authentication_type = ::buffa::EnumValue::Known(
            <super::recipients::v1::AuthenticationType as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::std::string::String) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(expiration_time)]
    fn set_expiration_time(&mut self, value: ::core::option::Option<i64>) {
        self.0.expiration_time = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::recipients::v1::CreateRecipientRequest>
    for PyCreateRecipientRequest
{
    fn from(value: super::recipients::v1::CreateRecipientRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateRecipientRequest>
    for super::recipients::v1::CreateRecipientRequest
{
    fn from(value: PyCreateRecipientRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteRecipientRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteRecipientRequest(pub super::recipients::v1::DeleteRecipientRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteRecipientRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::recipients::v1::DeleteRecipientRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::recipients::v1::DeleteRecipientRequest>
    for PyDeleteRecipientRequest
{
    fn from(value: super::recipients::v1::DeleteRecipientRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteRecipientRequest>
    for super::recipients::v1::DeleteRecipientRequest
{
    fn from(value: PyDeleteRecipientRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetRecipientRequest")]
#[derive(Clone, Debug)]
pub struct PyGetRecipientRequest(pub super::recipients::v1::GetRecipientRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetRecipientRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::recipients::v1::GetRecipientRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::recipients::v1::GetRecipientRequest> for PyGetRecipientRequest {
    fn from(value: super::recipients::v1::GetRecipientRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetRecipientRequest> for super::recipients::v1::GetRecipientRequest {
    fn from(value: PyGetRecipientRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListRecipientsRequest")]
#[derive(Clone, Debug)]
pub struct PyListRecipientsRequest(pub super::recipients::v1::ListRecipientsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListRecipientsRequest {
    #[new]
    #[pyo3(signature = (max_results = None, page_token = None))]
    fn new(
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::recipients::v1::ListRecipientsRequest as ::core::default::Default>::default();
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::recipients::v1::ListRecipientsRequest>
    for PyListRecipientsRequest
{
    fn from(value: super::recipients::v1::ListRecipientsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListRecipientsRequest>
    for super::recipients::v1::ListRecipientsRequest
{
    fn from(value: PyListRecipientsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListRecipientsResponse")]
#[derive(Clone, Debug)]
pub struct PyListRecipientsResponse(pub super::recipients::v1::ListRecipientsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListRecipientsResponse {
    #[new]
    #[pyo3(signature = (recipients = None, next_page_token = None))]
    fn new(
        recipients: ::core::option::Option<::std::vec::Vec<PyRecipient>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::recipients::v1::ListRecipientsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = recipients {
            inner.recipients = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn recipients(&self) -> ::std::vec::Vec<PyRecipient> {
        self.0
            .recipients
            .iter()
            .cloned()
            .map(PyRecipient::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(recipients)]
    fn set_recipients(&mut self, value: ::std::vec::Vec<PyRecipient>) {
        self.0.recipients = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "Recipient")]
#[derive(Clone, Debug)]
pub struct PyRecipient(pub super::recipients::v1::Recipient);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyRecipient {
    #[new]
    #[pyo3(
        signature = (
            id = None,
            name = None,
            authentication_type = None,
            owner = None,
            comment = None,
            properties = None,
            created_at = None,
            created_by = None,
            tokens = None,
            updated_at = None,
            updated_by = None
        )
    )]
    fn new(
        id: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
        authentication_type: ::core::option::Option<PyAuthenticationType>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        tokens: ::core::option::Option<::std::vec::Vec<PyRecipientToken>>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::recipients::v1::Recipient as ::core::default::Default>::default();
        {
            let value = id;
            inner.id = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = authentication_type {
            inner.authentication_type = ::buffa::EnumValue::Known(
                <super::recipients::v1::AuthenticationType as ::core::convert::From<_>>::from(
                    value,
                ),
            );
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        if let ::core::option::Option::Some(value) = tokens {
            inner.tokens = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        Self(inner)
    }
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn authentication_type(&self) -> PyAuthenticationType {
        PyAuthenticationType::from(self.0.authentication_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn tokens(&self) -> ::std::vec::Vec<PyRecipientToken> {
        self.0
            .tokens
            .iter()
            .cloned()
            .map(PyRecipientToken::from)
            .collect()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[setter(id)]
    fn set_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.id = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(authentication_type)]
    fn set_authentication_type(&mut self, value: PyAuthenticationType) {
        self.0.authentication_type = ::buffa::EnumValue::Known(
            <super::recipients::v1::AuthenticationType as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(tokens)]
    fn set_tokens(&mut self, value: ::std::vec::Vec<PyRecipientToken>) {
        self.0.tokens = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "RecipientToken")]
#[derive(Clone, Debug)]
pub struct PyRecipientToken(pub super::recipients::v1::RecipientToken);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyRecipientToken {
    #[new]
    #[pyo3(
        signature = (
            id = None,
            created_at = None,
            created_by = None,
            activation_url = None,
            expiration_time = None,
            updated_at = None,
            updated_by = None
        )
    )]
    fn new(
        id: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        activation_url: ::core::option::Option<::std::string::String>,
        expiration_time: ::core::option::Option<i64>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::recipients::v1::RecipientToken as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = id {
            inner.id = value;
        }
        if let ::core::option::Option::Some(value) = created_at {
            inner.created_at = value;
        }
        if let ::core::option::Option::Some(value) = created_by {
            inner.created_by = value;
        }
        if let ::core::option::Option::Some(value) = activation_url {
            inner.activation_url = value;
        }
        if let ::core::option::Option::Some(value) = expiration_time {
            inner.expiration_time = value;
        }
        if let ::core::option::Option::Some(value) = updated_at {
            inner.updated_at = value;
        }
        if let ::core::option::Option::Some(value) = updated_by {
            inner.updated_by = value;
        }
        Self(inner)
    }
    #[getter]
    fn id(&self) -> ::std::string::String {
        self.0.id.clone()
    }
    #[getter]
    fn created_at(&self) -> i64 {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::std::string::String {
        self.0.created_by.clone()
    }
    #[getter]
    fn activation_url(&self) -> ::std::string::String {
        self.0.activation_url.clone()
    }
    #[getter]
    fn expiration_time(&self) -> i64 {
        self.0.expiration_time
    }
    #[getter]
    fn updated_at(&self) -> i64 {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::std::string::String {
        self.0.updated_by.clone()
    }
    #[setter(id)]
    fn set_id(&mut self, value: ::std::string::String) {
        self.0.id = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: i64) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::std::string::String) {
        self.0.created_by = value;
    }
    #[setter(activation_url)]
    fn set_activation_url(&mut self, value: ::std::string::String) {
        self.0.activation_url = value;
    }
    #[setter(expiration_time)]
    fn set_expiration_time(&mut self, value: i64) {
        self.0.expiration_time = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: i64) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::std::string::String) {
        self.0.updated_by = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateRecipientRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateRecipientRequest(pub super::recipients::v1::UpdateRecipientRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateRecipientRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            new_name = None,
            owner = None,
            comment = None,
            properties = None,
            expiration_time = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        new_name: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        expiration_time: ::core::option::Option<i64>,
    ) -> Self {
        let mut inner =
            <super::recipients::v1::UpdateRecipientRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = expiration_time;
            inner.expiration_time = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn expiration_time(&self) -> ::core::option::Option<i64> {
        self.0.expiration_time
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(expiration_time)]
    fn set_expiration_time(&mut self, value: ::core::option::Option<i64>) {
        self.0.expiration_time = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::recipients::v1::UpdateRecipientRequest>
    for PyUpdateRecipientRequest
{
    fn from(value: super::recipients::v1::UpdateRecipientRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateRecipientRequest>
    for super::recipients::v1::UpdateRecipientRequest
{
    fn from(value: PyUpdateRecipientRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "CreateSchemaRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateSchemaRequest(pub super::schemas::v1::CreateSchemaRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateSchemaRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            catalog_name = None,
            comment = None,
            properties = None,
            storage_root = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        storage_root: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::schemas::v1::CreateSchemaRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = storage_root;
            inner.storage_root = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn storage_root(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_root.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(storage_root)]
    fn set_storage_root(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_root = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::schemas::v1::CreateSchemaRequest> for PyCreateSchemaRequest {
    fn from(value: super::schemas::v1::CreateSchemaRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateSchemaRequest> for super::schemas::v1::CreateSchemaRequest {
    fn from(value: PyCreateSchemaRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteSchemaRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteSchemaRequest(pub super::schemas::v1::DeleteSchemaRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteSchemaRequest {
    #[new]
    #[pyo3(signature = (full_name = None, force = None))]
    fn new(
        full_name: ::core::option::Option<::std::string::String>,
        force: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::schemas::v1::DeleteSchemaRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        {
            let value = force;
            inner.force = value;
        }
        Self(inner)
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn force(&self) -> ::core::option::Option<bool> {
        self.0.force
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(force)]
    fn set_force(&mut self, value: ::core::option::Option<bool>) {
        self.0.force = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::schemas::v1::DeleteSchemaRequest> for PyDeleteSchemaRequest {
    fn from(value: super::schemas::v1::DeleteSchemaRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteSchemaRequest> for super::schemas::v1::DeleteSchemaRequest {
    fn from(value: PyDeleteSchemaRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetSchemaRequest")]
#[derive(Clone, Debug)]
pub struct PyGetSchemaRequest(pub super::schemas::v1::GetSchemaRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetSchemaRequest {
    #[new]
    #[pyo3(signature = (full_name = None))]
    fn new(full_name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::schemas::v1::GetSchemaRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        Self(inner)
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::schemas::v1::GetSchemaRequest> for PyGetSchemaRequest {
    fn from(value: super::schemas::v1::GetSchemaRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetSchemaRequest> for super::schemas::v1::GetSchemaRequest {
    fn from(value: PyGetSchemaRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListSchemasRequest")]
#[derive(Clone, Debug)]
pub struct PyListSchemasRequest(pub super::schemas::v1::ListSchemasRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListSchemasRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            max_results = None,
            page_token = None,
            include_browse = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::schemas::v1::ListSchemasRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::schemas::v1::ListSchemasRequest> for PyListSchemasRequest {
    fn from(value: super::schemas::v1::ListSchemasRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListSchemasRequest> for super::schemas::v1::ListSchemasRequest {
    fn from(value: PyListSchemasRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListSchemasResponse")]
#[derive(Clone, Debug)]
pub struct PyListSchemasResponse(pub super::schemas::v1::ListSchemasResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListSchemasResponse {
    #[new]
    #[pyo3(signature = (schemas = None, next_page_token = None))]
    fn new(
        schemas: ::core::option::Option<::std::vec::Vec<PySchema>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::schemas::v1::ListSchemasResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = schemas {
            inner.schemas = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn schemas(&self) -> ::std::vec::Vec<PySchema> {
        self.0.schemas.iter().cloned().map(PySchema::from).collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(schemas)]
    fn set_schemas(&mut self, value: ::std::vec::Vec<PySchema>) {
        self.0.schemas = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "Schema")]
#[derive(Clone, Debug)]
pub struct PySchema(pub super::schemas::v1::Schema);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PySchema {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            catalog_name = None,
            full_name = None,
            comment = None,
            properties = None,
            owner = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None,
            schema_id = None,
            storage_root = None,
            storage_location = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        full_name: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        owner: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
        schema_id: ::core::option::Option<::std::string::String>,
        storage_root: ::core::option::Option<::std::string::String>,
        storage_location: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::schemas::v1::Schema as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        {
            let value = schema_id;
            inner.schema_id = value;
        }
        {
            let value = storage_root;
            inner.storage_root = value;
        }
        {
            let value = storage_location;
            inner.storage_location = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[getter]
    fn schema_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.schema_id.clone()
    }
    #[getter]
    fn storage_root(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_root.clone()
    }
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    #[setter(schema_id)]
    fn set_schema_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.schema_id = value;
    }
    #[setter(storage_root)]
    fn set_storage_root(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_root = value;
    }
    #[setter(storage_location)]
    fn set_storage_location(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_location = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateSchemaRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateSchemaRequest(pub super::schemas::v1::UpdateSchemaRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateSchemaRequest {
    #[new]
    #[pyo3(
        signature = (
            full_name = None,
            comment = None,
            properties = None,
            new_name = None
        )
    )]
    fn new(
        full_name: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        new_name: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::schemas::v1::UpdateSchemaRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        Self(inner)
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::schemas::v1::UpdateSchemaRequest> for PyUpdateSchemaRequest {
    fn from(value: super::schemas::v1::UpdateSchemaRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateSchemaRequest> for super::schemas::v1::UpdateSchemaRequest {
    fn from(value: PyUpdateSchemaRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "CreateShareRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateShareRequest(pub super::shares::v1::CreateShareRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateShareRequest {
    #[new]
    #[pyo3(signature = (name = None, comment = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::CreateShareRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::shares::v1::CreateShareRequest> for PyCreateShareRequest {
    fn from(value: super::shares::v1::CreateShareRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateShareRequest> for super::shares::v1::CreateShareRequest {
    fn from(value: PyCreateShareRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DataObject")]
#[derive(Clone, Debug)]
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
            enable_cdf = None,
            history_data_sharing_status = None,
            start_version = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        data_object_type: ::core::option::Option<PyDataObjectType>,
        added_at: ::core::option::Option<i64>,
        added_by: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        shared_as: ::core::option::Option<::std::string::String>,
        partitions: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
        enable_cdf: ::core::option::Option<bool>,
        history_data_sharing_status: ::core::option::Option<PyHistoryStatus>,
        start_version: ::core::option::Option<i64>,
    ) -> Self {
        let mut inner = <super::shares::v1::DataObject as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = data_object_type {
            inner.data_object_type = ::buffa::EnumValue::Known(
                <super::shares::v1::DataObjectType as ::core::convert::From<_>>::from(value),
            );
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
            inner.partitions = value;
        }
        {
            let value = enable_cdf;
            inner.enable_cdf = value;
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
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn data_object_type(&self) -> PyDataObjectType {
        PyDataObjectType::from(self.0.data_object_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn added_at(&self) -> ::core::option::Option<i64> {
        self.0.added_at
    }
    #[getter]
    fn added_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.added_by.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn shared_as(&self) -> ::core::option::Option<::std::string::String> {
        self.0.shared_as.clone()
    }
    #[getter]
    fn partitions(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.partitions.clone()
    }
    #[getter]
    fn enable_cdf(&self) -> ::core::option::Option<bool> {
        self.0.enable_cdf
    }
    #[getter]
    fn history_data_sharing_status(&self) -> ::core::option::Option<PyHistoryStatus> {
        self.0
            .history_data_sharing_status
            .as_ref()
            .and_then(|e| e.as_known())
            .map(PyHistoryStatus::from)
    }
    #[getter]
    fn start_version(&self) -> ::core::option::Option<i64> {
        self.0.start_version
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(data_object_type)]
    fn set_data_object_type(&mut self, value: PyDataObjectType) {
        self.0.data_object_type = ::buffa::EnumValue::Known(
            <super::shares::v1::DataObjectType as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(added_at)]
    fn set_added_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.added_at = value;
    }
    #[setter(added_by)]
    fn set_added_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.added_by = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(shared_as)]
    fn set_shared_as(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.shared_as = value;
    }
    #[setter(partitions)]
    fn set_partitions(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.partitions = value;
    }
    #[setter(enable_cdf)]
    fn set_enable_cdf(&mut self, value: ::core::option::Option<bool>) {
        self.0.enable_cdf = value;
    }
    #[setter(history_data_sharing_status)]
    fn set_history_data_sharing_status(&mut self, value: ::core::option::Option<PyHistoryStatus>) {
        self.0.history_data_sharing_status = value.map(|e| {
            ::buffa::EnumValue::Known(
                <super::shares::v1::HistoryStatus as ::core::convert::From<_>>::from(e),
            )
        });
    }
    #[setter(start_version)]
    fn set_start_version(&mut self, value: ::core::option::Option<i64>) {
        self.0.start_version = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "DataObjectUpdate")]
#[derive(Clone, Debug)]
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
        if let ::core::option::Option::Some(value) = action {
            inner.action =
                ::buffa::EnumValue::Known(<super::shares::v1::Action as ::core::convert::From<
                    _,
                >>::from(value));
        }
        {
            let value = data_object;
            inner.data_object = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        Self(inner)
    }
    #[getter]
    fn action(&self) -> PyAction {
        PyAction::from(self.0.action.as_known().unwrap_or_default())
    }
    #[getter]
    fn data_object(&self) -> ::core::option::Option<PyDataObject> {
        self.0
            .data_object
            .clone()
            .into_option()
            .map(PyDataObject::from)
    }
    #[setter(action)]
    fn set_action(&mut self, value: PyAction) {
        self.0.action = ::buffa::EnumValue::Known(
            <super::shares::v1::Action as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(data_object)]
    fn set_data_object(&mut self, value: ::core::option::Option<PyDataObject>) {
        self.0.data_object = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "DeleteShareRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteShareRequest(pub super::shares::v1::DeleteShareRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteShareRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::shares::v1::DeleteShareRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::shares::v1::DeleteShareRequest> for PyDeleteShareRequest {
    fn from(value: super::shares::v1::DeleteShareRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteShareRequest> for super::shares::v1::DeleteShareRequest {
    fn from(value: PyDeleteShareRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetPermissionsRequest")]
#[derive(Clone, Debug)]
pub struct PyGetPermissionsRequest(pub super::shares::v1::GetPermissionsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetPermissionsRequest {
    #[new]
    #[pyo3(signature = (name = None, max_results = None, page_token = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::GetPermissionsRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::shares::v1::GetPermissionsRequest> for PyGetPermissionsRequest {
    fn from(value: super::shares::v1::GetPermissionsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetPermissionsRequest> for super::shares::v1::GetPermissionsRequest {
    fn from(value: PyGetPermissionsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetPermissionsResponse")]
#[derive(Clone, Debug)]
pub struct PyGetPermissionsResponse(pub super::shares::v1::GetPermissionsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetPermissionsResponse {
    #[new]
    #[pyo3(signature = (privilege_assignments = None, next_page_token = None))]
    fn new(
        privilege_assignments: ::core::option::Option<::std::vec::Vec<PyPrivilegeAssignment>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::GetPermissionsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = privilege_assignments {
            inner.privilege_assignments =
                value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn privilege_assignments(&self) -> ::std::vec::Vec<PyPrivilegeAssignment> {
        self.0
            .privilege_assignments
            .iter()
            .cloned()
            .map(PyPrivilegeAssignment::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(privilege_assignments)]
    fn set_privilege_assignments(&mut self, value: ::std::vec::Vec<PyPrivilegeAssignment>) {
        self.0.privilege_assignments = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "GetShareRequest")]
#[derive(Clone, Debug)]
pub struct PyGetShareRequest(pub super::shares::v1::GetShareRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetShareRequest {
    #[new]
    #[pyo3(signature = (name = None, include_shared_data = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        include_shared_data: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::shares::v1::GetShareRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = include_shared_data;
            inner.include_shared_data = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn include_shared_data(&self) -> ::core::option::Option<bool> {
        self.0.include_shared_data
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(include_shared_data)]
    fn set_include_shared_data(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_shared_data = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::shares::v1::GetShareRequest> for PyGetShareRequest {
    fn from(value: super::shares::v1::GetShareRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetShareRequest> for super::shares::v1::GetShareRequest {
    fn from(value: PyGetShareRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListSharesRequest")]
#[derive(Clone, Debug)]
pub struct PyListSharesRequest(pub super::shares::v1::ListSharesRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListSharesRequest {
    #[new]
    #[pyo3(signature = (max_results = None, page_token = None))]
    fn new(
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::ListSharesRequest as ::core::default::Default>::default();
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::shares::v1::ListSharesRequest> for PyListSharesRequest {
    fn from(value: super::shares::v1::ListSharesRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListSharesRequest> for super::shares::v1::ListSharesRequest {
    fn from(value: PyListSharesRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListSharesResponse")]
#[derive(Clone, Debug)]
pub struct PyListSharesResponse(pub super::shares::v1::ListSharesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListSharesResponse {
    #[new]
    #[pyo3(signature = (shares = None, next_page_token = None))]
    fn new(
        shares: ::core::option::Option<::std::vec::Vec<PyShare>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::ListSharesResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = shares {
            inner.shares = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn shares(&self) -> ::std::vec::Vec<PyShare> {
        self.0.shares.iter().cloned().map(PyShare::from).collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(shares)]
    fn set_shares(&mut self, value: ::std::vec::Vec<PyShare>) {
        self.0.shares = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "PermissionsChange")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn principal(&self) -> ::std::string::String {
        self.0.principal.clone()
    }
    #[getter]
    fn add(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.add.clone()
    }
    #[getter]
    fn remove(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.remove.clone()
    }
    #[setter(principal)]
    fn set_principal(&mut self, value: ::std::string::String) {
        self.0.principal = value;
    }
    #[setter(add)]
    fn set_add(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.add = value;
    }
    #[setter(remove)]
    fn set_remove(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.remove = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "PrivilegeAssignment")]
#[derive(Clone, Debug)]
pub struct PyPrivilegeAssignment(pub super::shares::v1::PrivilegeAssignment);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyPrivilegeAssignment {
    #[new]
    #[pyo3(signature = (principal = None, privileges = None))]
    fn new(
        principal: ::core::option::Option<::std::string::String>,
        privileges: ::core::option::Option<::std::vec::Vec<::std::string::String>>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::PrivilegeAssignment as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = principal {
            inner.principal = value;
        }
        if let ::core::option::Option::Some(value) = privileges {
            inner.privileges = value;
        }
        Self(inner)
    }
    #[getter]
    fn principal(&self) -> ::std::string::String {
        self.0.principal.clone()
    }
    #[getter]
    fn privileges(&self) -> ::std::vec::Vec<::std::string::String> {
        self.0.privileges.clone()
    }
    #[setter(principal)]
    fn set_principal(&mut self, value: ::std::string::String) {
        self.0.principal = value;
    }
    #[setter(privileges)]
    fn set_privileges(&mut self, value: ::std::vec::Vec<::std::string::String>) {
        self.0.privileges = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "Share")]
#[derive(Clone, Debug)]
pub struct PyShare(pub super::shares::v1::Share);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyShare {
    #[new]
    #[pyo3(
        signature = (
            id = None,
            name = None,
            objects = None,
            owner = None,
            comment = None,
            storage_location = None,
            storage_root = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None
        )
    )]
    fn new(
        id: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
        objects: ::core::option::Option<::std::vec::Vec<PyDataObject>>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        storage_location: ::core::option::Option<::std::string::String>,
        storage_root: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::shares::v1::Share as ::core::default::Default>::default();
        {
            let value = id;
            inner.id = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = objects {
            inner.objects = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = storage_location;
            inner.storage_location = value;
        }
        {
            let value = storage_root;
            inner.storage_root = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        Self(inner)
    }
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn objects(&self) -> ::std::vec::Vec<PyDataObject> {
        self.0
            .objects
            .iter()
            .cloned()
            .map(PyDataObject::from)
            .collect()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }
    #[getter]
    fn storage_root(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_root.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[setter(id)]
    fn set_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.id = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(objects)]
    fn set_objects(&mut self, value: ::std::vec::Vec<PyDataObject>) {
        self.0.objects = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(storage_location)]
    fn set_storage_location(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_location = value;
    }
    #[setter(storage_root)]
    fn set_storage_root(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_root = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdatePermissionsRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdatePermissionsRequest(pub super::shares::v1::UpdatePermissionsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdatePermissionsRequest {
    #[new]
    #[pyo3(signature = (name = None, changes = None, omit_permissions_list = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        changes: ::core::option::Option<::std::vec::Vec<PyPermissionsChange>>,
        omit_permissions_list: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::UpdatePermissionsRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = changes {
            inner.changes = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = omit_permissions_list;
            inner.omit_permissions_list = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn changes(&self) -> ::std::vec::Vec<PyPermissionsChange> {
        self.0
            .changes
            .iter()
            .cloned()
            .map(PyPermissionsChange::from)
            .collect()
    }
    #[getter]
    fn omit_permissions_list(&self) -> ::core::option::Option<bool> {
        self.0.omit_permissions_list
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(changes)]
    fn set_changes(&mut self, value: ::std::vec::Vec<PyPermissionsChange>) {
        self.0.changes = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(omit_permissions_list)]
    fn set_omit_permissions_list(&mut self, value: ::core::option::Option<bool>) {
        self.0.omit_permissions_list = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::shares::v1::UpdatePermissionsRequest>
    for PyUpdatePermissionsRequest
{
    fn from(value: super::shares::v1::UpdatePermissionsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdatePermissionsRequest>
    for super::shares::v1::UpdatePermissionsRequest
{
    fn from(value: PyUpdatePermissionsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "UpdatePermissionsResponse")]
#[derive(Clone, Debug)]
pub struct PyUpdatePermissionsResponse(pub super::shares::v1::UpdatePermissionsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdatePermissionsResponse {
    #[new]
    #[pyo3(signature = (privilege_assignments = None))]
    fn new(
        privilege_assignments: ::core::option::Option<::std::vec::Vec<PyPrivilegeAssignment>>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::UpdatePermissionsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = privilege_assignments {
            inner.privilege_assignments =
                value.into_iter().map(::core::convert::Into::into).collect();
        }
        Self(inner)
    }
    #[getter]
    fn privilege_assignments(&self) -> ::std::vec::Vec<PyPrivilegeAssignment> {
        self.0
            .privilege_assignments
            .iter()
            .cloned()
            .map(PyPrivilegeAssignment::from)
            .collect()
    }
    #[setter(privilege_assignments)]
    fn set_privilege_assignments(&mut self, value: ::std::vec::Vec<PyPrivilegeAssignment>) {
        self.0.privilege_assignments = value.into_iter().map(::core::convert::Into::into).collect();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateShareRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateShareRequest(pub super::shares::v1::UpdateShareRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateShareRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            updates = None,
            new_name = None,
            owner = None,
            comment = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        updates: ::core::option::Option<::std::vec::Vec<PyDataObjectUpdate>>,
        new_name: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::shares::v1::UpdateShareRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = updates {
            inner.updates = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn updates(&self) -> ::std::vec::Vec<PyDataObjectUpdate> {
        self.0
            .updates
            .iter()
            .cloned()
            .map(PyDataObjectUpdate::from)
            .collect()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(updates)]
    fn set_updates(&mut self, value: ::std::vec::Vec<PyDataObjectUpdate>) {
        self.0.updates = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::shares::v1::UpdateShareRequest> for PyUpdateShareRequest {
    fn from(value: super::shares::v1::UpdateShareRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateShareRequest> for super::shares::v1::UpdateShareRequest {
    fn from(value: PyUpdateShareRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "CreateStagingTableRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateStagingTableRequest(pub super::staging_tables::v1::CreateStagingTableRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateStagingTableRequest {
    #[new]
    #[pyo3(signature = (name = None, catalog_name = None, schema_name = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::staging_tables::v1::CreateStagingTableRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::staging_tables::v1::CreateStagingTableRequest>
    for PyCreateStagingTableRequest
{
    fn from(value: super::staging_tables::v1::CreateStagingTableRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateStagingTableRequest>
    for super::staging_tables::v1::CreateStagingTableRequest
{
    fn from(value: PyCreateStagingTableRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "StagingTable")]
#[derive(Clone, Debug)]
pub struct PyStagingTable(pub super::staging_tables::v1::StagingTable);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyStagingTable {
    #[new]
    #[pyo3(
        signature = (
            id = None,
            name = None,
            schema_name = None,
            catalog_name = None,
            staging_location = None,
            created_by = None,
            stage_committed = None,
            created_at = None
        )
    )]
    fn new(
        id: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        staging_location: ::core::option::Option<::std::string::String>,
        created_by: ::core::option::Option<::std::string::String>,
        stage_committed: ::core::option::Option<bool>,
        created_at: ::core::option::Option<i64>,
    ) -> Self {
        let mut inner =
            <super::staging_tables::v1::StagingTable as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = id {
            inner.id = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = staging_location {
            inner.staging_location = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        if let ::core::option::Option::Some(value) = stage_committed {
            inner.stage_committed = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        Self(inner)
    }
    #[getter]
    fn id(&self) -> ::std::string::String {
        self.0.id.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn staging_location(&self) -> ::std::string::String {
        self.0.staging_location.clone()
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn stage_committed(&self) -> bool {
        self.0.stage_committed
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[setter(id)]
    fn set_id(&mut self, value: ::std::string::String) {
        self.0.id = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(staging_location)]
    fn set_staging_location(&mut self, value: ::std::string::String) {
        self.0.staging_location = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(stage_committed)]
    fn set_stage_committed(&mut self, value: bool) {
        self.0.stage_committed = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "Column")]
#[derive(Clone, Debug)]
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
        if let ::core::option::Option::Some(value) = type_name {
            inner.type_name = ::buffa::EnumValue::Known(
                <super::tables::v1::ColumnTypeName as ::core::convert::From<_>>::from(value),
            );
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
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn type_text(&self) -> ::std::string::String {
        self.0.type_text.clone()
    }
    #[getter]
    fn type_json(&self) -> ::std::string::String {
        self.0.type_json.clone()
    }
    #[getter]
    fn position(&self) -> ::core::option::Option<i32> {
        self.0.position
    }
    #[getter]
    fn type_name(&self) -> PyColumnTypeName {
        PyColumnTypeName::from(self.0.type_name.as_known().unwrap_or_default())
    }
    #[getter]
    fn type_precision(&self) -> ::core::option::Option<i32> {
        self.0.type_precision
    }
    #[getter]
    fn type_scale(&self) -> ::core::option::Option<i32> {
        self.0.type_scale
    }
    #[getter]
    fn type_interval_type(&self) -> ::core::option::Option<::std::string::String> {
        self.0.type_interval_type.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn nullable(&self) -> ::core::option::Option<bool> {
        self.0.nullable
    }
    #[getter]
    fn partition_index(&self) -> ::core::option::Option<i32> {
        self.0.partition_index
    }
    #[getter]
    fn column_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.column_id.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(type_text)]
    fn set_type_text(&mut self, value: ::std::string::String) {
        self.0.type_text = value;
    }
    #[setter(type_json)]
    fn set_type_json(&mut self, value: ::std::string::String) {
        self.0.type_json = value;
    }
    #[setter(position)]
    fn set_position(&mut self, value: ::core::option::Option<i32>) {
        self.0.position = value;
    }
    #[setter(type_name)]
    fn set_type_name(&mut self, value: PyColumnTypeName) {
        self.0.type_name = ::buffa::EnumValue::Known(
            <super::tables::v1::ColumnTypeName as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(type_precision)]
    fn set_type_precision(&mut self, value: ::core::option::Option<i32>) {
        self.0.type_precision = value;
    }
    #[setter(type_scale)]
    fn set_type_scale(&mut self, value: ::core::option::Option<i32>) {
        self.0.type_scale = value;
    }
    #[setter(type_interval_type)]
    fn set_type_interval_type(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.type_interval_type = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(nullable)]
    fn set_nullable(&mut self, value: ::core::option::Option<bool>) {
        self.0.nullable = value;
    }
    #[setter(partition_index)]
    fn set_partition_index(&mut self, value: ::core::option::Option<i32>) {
        self.0.partition_index = value;
    }
    #[setter(column_id)]
    fn set_column_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.column_id = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "CreateTableRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateTableRequest(pub super::tables::v1::CreateTableRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateTableRequest {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            schema_name = None,
            catalog_name = None,
            table_type = None,
            data_source_format = None,
            columns = None,
            storage_location = None,
            comment = None,
            properties = None,
            view_definition = None,
            view_dependencies = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        table_type: ::core::option::Option<PyTableType>,
        data_source_format: ::core::option::Option<PyDataSourceFormat>,
        columns: ::core::option::Option<::std::vec::Vec<PyColumn>>,
        storage_location: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        view_definition: ::core::option::Option<::std::string::String>,
        view_dependencies: ::core::option::Option<PyDependencyList>,
    ) -> Self {
        let mut inner =
            <super::tables::v1::CreateTableRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = table_type {
            inner.table_type = ::buffa::EnumValue::Known(
                <super::tables::v1::TableType as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = data_source_format {
            inner.data_source_format = ::buffa::EnumValue::Known(
                <super::tables::v1::DataSourceFormat as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = columns {
            inner.columns = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = storage_location;
            inner.storage_location = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = view_definition;
            inner.view_definition = value;
        }
        {
            let value = view_dependencies;
            inner.view_dependencies = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn table_type(&self) -> PyTableType {
        PyTableType::from(self.0.table_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn data_source_format(&self) -> PyDataSourceFormat {
        PyDataSourceFormat::from(self.0.data_source_format.as_known().unwrap_or_default())
    }
    #[getter]
    fn columns(&self) -> ::std::vec::Vec<PyColumn> {
        self.0.columns.iter().cloned().map(PyColumn::from).collect()
    }
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn view_definition(&self) -> ::core::option::Option<::std::string::String> {
        self.0.view_definition.clone()
    }
    #[getter]
    fn view_dependencies(&self) -> ::core::option::Option<PyDependencyList> {
        self.0
            .view_dependencies
            .clone()
            .into_option()
            .map(PyDependencyList::from)
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(table_type)]
    fn set_table_type(&mut self, value: PyTableType) {
        self.0.table_type = ::buffa::EnumValue::Known(
            <super::tables::v1::TableType as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(data_source_format)]
    fn set_data_source_format(&mut self, value: PyDataSourceFormat) {
        self.0.data_source_format = ::buffa::EnumValue::Known(
            <super::tables::v1::DataSourceFormat as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(columns)]
    fn set_columns(&mut self, value: ::std::vec::Vec<PyColumn>) {
        self.0.columns = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(storage_location)]
    fn set_storage_location(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_location = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(view_definition)]
    fn set_view_definition(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.view_definition = value;
    }
    #[setter(view_dependencies)]
    fn set_view_dependencies(&mut self, value: ::core::option::Option<PyDependencyList>) {
        self.0.view_dependencies = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tables::v1::CreateTableRequest> for PyCreateTableRequest {
    fn from(value: super::tables::v1::CreateTableRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateTableRequest> for super::tables::v1::CreateTableRequest {
    fn from(value: PyCreateTableRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteTableRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteTableRequest(pub super::tables::v1::DeleteTableRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteTableRequest {
    #[new]
    #[pyo3(signature = (full_name = None))]
    fn new(full_name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::tables::v1::DeleteTableRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        Self(inner)
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tables::v1::DeleteTableRequest> for PyDeleteTableRequest {
    fn from(value: super::tables::v1::DeleteTableRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteTableRequest> for super::tables::v1::DeleteTableRequest {
    fn from(value: PyDeleteTableRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "Dependency")]
#[derive(Clone, Debug)]
pub struct PyDependency(pub super::tables::v1::Dependency);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDependency {
    #[new]
    fn new() -> Self {
        Self(::core::default::Default::default())
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "DependencyList")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn dependencies(&self) -> ::std::vec::Vec<PyDependency> {
        self.0
            .dependencies
            .iter()
            .cloned()
            .map(PyDependency::from)
            .collect()
    }
    #[setter(dependencies)]
    fn set_dependencies(&mut self, value: ::std::vec::Vec<PyDependency>) {
        self.0.dependencies = value.into_iter().map(::core::convert::Into::into).collect();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "FunctionDependency")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn function_full_name(&self) -> ::std::string::String {
        self.0.function_full_name.clone()
    }
    #[setter(function_full_name)]
    fn set_function_full_name(&mut self, value: ::std::string::String) {
        self.0.function_full_name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "GetTableExistsRequest")]
#[derive(Clone, Debug)]
pub struct PyGetTableExistsRequest(pub super::tables::v1::GetTableExistsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetTableExistsRequest {
    #[new]
    #[pyo3(signature = (full_name = None))]
    fn new(full_name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::tables::v1::GetTableExistsRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        Self(inner)
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tables::v1::GetTableExistsRequest> for PyGetTableExistsRequest {
    fn from(value: super::tables::v1::GetTableExistsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetTableExistsRequest> for super::tables::v1::GetTableExistsRequest {
    fn from(value: PyGetTableExistsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetTableExistsResponse")]
#[derive(Clone, Debug)]
pub struct PyGetTableExistsResponse(pub super::tables::v1::GetTableExistsResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetTableExistsResponse {
    #[new]
    #[pyo3(signature = (table_exists = None))]
    fn new(table_exists: ::core::option::Option<bool>) -> Self {
        let mut inner =
            <super::tables::v1::GetTableExistsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = table_exists {
            inner.table_exists = value;
        }
        Self(inner)
    }
    #[getter]
    fn table_exists(&self) -> bool {
        self.0.table_exists
    }
    #[setter(table_exists)]
    fn set_table_exists(&mut self, value: bool) {
        self.0.table_exists = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "GetTableRequest")]
#[derive(Clone, Debug)]
pub struct PyGetTableRequest(pub super::tables::v1::GetTableRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetTableRequest {
    #[new]
    #[pyo3(
        signature = (
            full_name = None,
            include_delta_metadata = None,
            include_browse = None,
            include_manifest_capabilities = None
        )
    )]
    fn new(
        full_name: ::core::option::Option<::std::string::String>,
        include_delta_metadata: ::core::option::Option<bool>,
        include_browse: ::core::option::Option<bool>,
        include_manifest_capabilities: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::tables::v1::GetTableRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        {
            let value = include_delta_metadata;
            inner.include_delta_metadata = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        {
            let value = include_manifest_capabilities;
            inner.include_manifest_capabilities = value;
        }
        Self(inner)
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn include_delta_metadata(&self) -> ::core::option::Option<bool> {
        self.0.include_delta_metadata
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[getter]
    fn include_manifest_capabilities(&self) -> ::core::option::Option<bool> {
        self.0.include_manifest_capabilities
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(include_delta_metadata)]
    fn set_include_delta_metadata(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_delta_metadata = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    #[setter(include_manifest_capabilities)]
    fn set_include_manifest_capabilities(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_manifest_capabilities = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tables::v1::GetTableRequest> for PyGetTableRequest {
    fn from(value: super::tables::v1::GetTableRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetTableRequest> for super::tables::v1::GetTableRequest {
    fn from(value: PyGetTableRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListTableSummariesRequest")]
#[derive(Clone, Debug)]
pub struct PyListTableSummariesRequest(pub super::tables::v1::ListTableSummariesRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListTableSummariesRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name_pattern = None,
            table_name_pattern = None,
            max_results = None,
            page_token = None,
            include_manifest_capabilities = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name_pattern: ::core::option::Option<::std::string::String>,
        table_name_pattern: ::core::option::Option<::std::string::String>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
        include_manifest_capabilities: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::tables::v1::ListTableSummariesRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        {
            let value = schema_name_pattern;
            inner.schema_name_pattern = value;
        }
        {
            let value = table_name_pattern;
            inner.table_name_pattern = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        {
            let value = include_manifest_capabilities;
            inner.include_manifest_capabilities = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name_pattern(&self) -> ::core::option::Option<::std::string::String> {
        self.0.schema_name_pattern.clone()
    }
    #[getter]
    fn table_name_pattern(&self) -> ::core::option::Option<::std::string::String> {
        self.0.table_name_pattern.clone()
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[getter]
    fn include_manifest_capabilities(&self) -> ::core::option::Option<bool> {
        self.0.include_manifest_capabilities
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name_pattern)]
    fn set_schema_name_pattern(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.schema_name_pattern = value;
    }
    #[setter(table_name_pattern)]
    fn set_table_name_pattern(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.table_name_pattern = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    #[setter(include_manifest_capabilities)]
    fn set_include_manifest_capabilities(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_manifest_capabilities = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tables::v1::ListTableSummariesRequest>
    for PyListTableSummariesRequest
{
    fn from(value: super::tables::v1::ListTableSummariesRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListTableSummariesRequest>
    for super::tables::v1::ListTableSummariesRequest
{
    fn from(value: PyListTableSummariesRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListTableSummariesResponse")]
#[derive(Clone, Debug)]
pub struct PyListTableSummariesResponse(pub super::tables::v1::ListTableSummariesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListTableSummariesResponse {
    #[new]
    #[pyo3(signature = (tables = None, next_page_token = None))]
    fn new(
        tables: ::core::option::Option<::std::vec::Vec<PyTableSummary>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::tables::v1::ListTableSummariesResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = tables {
            inner.tables = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn tables(&self) -> ::std::vec::Vec<PyTableSummary> {
        self.0
            .tables
            .iter()
            .cloned()
            .map(PyTableSummary::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(tables)]
    fn set_tables(&mut self, value: ::std::vec::Vec<PyTableSummary>) {
        self.0.tables = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "ListTablesRequest")]
#[derive(Clone, Debug)]
pub struct PyListTablesRequest(pub super::tables::v1::ListTablesRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListTablesRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name = None,
            max_results = None,
            page_token = None,
            include_delta_metadata = None,
            omit_columns = None,
            omit_properties = None,
            omit_username = None,
            include_browse = None,
            include_manifest_capabilities = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
        include_delta_metadata: ::core::option::Option<bool>,
        omit_columns: ::core::option::Option<bool>,
        omit_properties: ::core::option::Option<bool>,
        omit_username: ::core::option::Option<bool>,
        include_browse: ::core::option::Option<bool>,
        include_manifest_capabilities: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::tables::v1::ListTablesRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        {
            let value = include_delta_metadata;
            inner.include_delta_metadata = value;
        }
        {
            let value = omit_columns;
            inner.omit_columns = value;
        }
        {
            let value = omit_properties;
            inner.omit_properties = value;
        }
        {
            let value = omit_username;
            inner.omit_username = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        {
            let value = include_manifest_capabilities;
            inner.include_manifest_capabilities = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[getter]
    fn include_delta_metadata(&self) -> ::core::option::Option<bool> {
        self.0.include_delta_metadata
    }
    #[getter]
    fn omit_columns(&self) -> ::core::option::Option<bool> {
        self.0.omit_columns
    }
    #[getter]
    fn omit_properties(&self) -> ::core::option::Option<bool> {
        self.0.omit_properties
    }
    #[getter]
    fn omit_username(&self) -> ::core::option::Option<bool> {
        self.0.omit_username
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[getter]
    fn include_manifest_capabilities(&self) -> ::core::option::Option<bool> {
        self.0.include_manifest_capabilities
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    #[setter(include_delta_metadata)]
    fn set_include_delta_metadata(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_delta_metadata = value;
    }
    #[setter(omit_columns)]
    fn set_omit_columns(&mut self, value: ::core::option::Option<bool>) {
        self.0.omit_columns = value;
    }
    #[setter(omit_properties)]
    fn set_omit_properties(&mut self, value: ::core::option::Option<bool>) {
        self.0.omit_properties = value;
    }
    #[setter(omit_username)]
    fn set_omit_username(&mut self, value: ::core::option::Option<bool>) {
        self.0.omit_username = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    #[setter(include_manifest_capabilities)]
    fn set_include_manifest_capabilities(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_manifest_capabilities = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tables::v1::ListTablesRequest> for PyListTablesRequest {
    fn from(value: super::tables::v1::ListTablesRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListTablesRequest> for super::tables::v1::ListTablesRequest {
    fn from(value: PyListTablesRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListTablesResponse")]
#[derive(Clone, Debug)]
pub struct PyListTablesResponse(pub super::tables::v1::ListTablesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListTablesResponse {
    #[new]
    #[pyo3(signature = (tables = None, next_page_token = None))]
    fn new(
        tables: ::core::option::Option<::std::vec::Vec<PyTable>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::tables::v1::ListTablesResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = tables {
            inner.tables = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn tables(&self) -> ::std::vec::Vec<PyTable> {
        self.0.tables.iter().cloned().map(PyTable::from).collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(tables)]
    fn set_tables(&mut self, value: ::std::vec::Vec<PyTable>) {
        self.0.tables = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "Table")]
#[derive(Clone, Debug)]
pub struct PyTable(pub super::tables::v1::Table);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyTable {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            catalog_name = None,
            schema_name = None,
            table_type = None,
            data_source_format = None,
            columns = None,
            storage_location = None,
            view_definition = None,
            view_dependencies = None,
            owner = None,
            comment = None,
            properties = None,
            storage_credential_name = None,
            full_name = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None,
            deleted_at = None,
            table_id = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        table_type: ::core::option::Option<PyTableType>,
        data_source_format: ::core::option::Option<PyDataSourceFormat>,
        columns: ::core::option::Option<::std::vec::Vec<PyColumn>>,
        storage_location: ::core::option::Option<::std::string::String>,
        view_definition: ::core::option::Option<::std::string::String>,
        view_dependencies: ::core::option::Option<PyDependencyList>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        properties: ::core::option::Option<
            ::std::collections::HashMap<::std::string::String, ::std::string::String>,
        >,
        storage_credential_name: ::core::option::Option<::std::string::String>,
        full_name: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
        deleted_at: ::core::option::Option<i64>,
        table_id: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::tables::v1::Table as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = table_type {
            inner.table_type = ::buffa::EnumValue::Known(
                <super::tables::v1::TableType as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = data_source_format {
            inner.data_source_format = ::buffa::EnumValue::Known(
                <super::tables::v1::DataSourceFormat as ::core::convert::From<_>>::from(value),
            );
        }
        if let ::core::option::Option::Some(value) = columns {
            inner.columns = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = storage_location;
            inner.storage_location = value;
        }
        {
            let value = view_definition;
            inner.view_definition = value;
        }
        {
            let value = view_dependencies;
            inner.view_dependencies = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        if let ::core::option::Option::Some(value) = properties {
            inner.properties = value;
        }
        {
            let value = storage_credential_name;
            inner.storage_credential_name = value;
        }
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        {
            let value = deleted_at;
            inner.deleted_at = value;
        }
        {
            let value = table_id;
            inner.table_id = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn table_type(&self) -> PyTableType {
        PyTableType::from(self.0.table_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn data_source_format(&self) -> PyDataSourceFormat {
        PyDataSourceFormat::from(self.0.data_source_format.as_known().unwrap_or_default())
    }
    #[getter]
    fn columns(&self) -> ::std::vec::Vec<PyColumn> {
        self.0.columns.iter().cloned().map(PyColumn::from).collect()
    }
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }
    #[getter]
    fn view_definition(&self) -> ::core::option::Option<::std::string::String> {
        self.0.view_definition.clone()
    }
    #[getter]
    fn view_dependencies(&self) -> ::core::option::Option<PyDependencyList> {
        self.0
            .view_dependencies
            .clone()
            .into_option()
            .map(PyDependencyList::from)
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn properties(
        &self,
    ) -> ::std::collections::HashMap<::std::string::String, ::std::string::String> {
        self.0.properties.clone()
    }
    #[getter]
    fn storage_credential_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_credential_name.clone()
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[getter]
    fn deleted_at(&self) -> ::core::option::Option<i64> {
        self.0.deleted_at
    }
    #[getter]
    fn table_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.table_id.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(table_type)]
    fn set_table_type(&mut self, value: PyTableType) {
        self.0.table_type = ::buffa::EnumValue::Known(
            <super::tables::v1::TableType as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(data_source_format)]
    fn set_data_source_format(&mut self, value: PyDataSourceFormat) {
        self.0.data_source_format = ::buffa::EnumValue::Known(
            <super::tables::v1::DataSourceFormat as ::core::convert::From<_>>::from(value),
        );
    }
    #[setter(columns)]
    fn set_columns(&mut self, value: ::std::vec::Vec<PyColumn>) {
        self.0.columns = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(storage_location)]
    fn set_storage_location(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_location = value;
    }
    #[setter(view_definition)]
    fn set_view_definition(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.view_definition = value;
    }
    #[setter(view_dependencies)]
    fn set_view_dependencies(&mut self, value: ::core::option::Option<PyDependencyList>) {
        self.0.view_dependencies = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(properties)]
    fn set_properties(
        &mut self,
        value: ::std::collections::HashMap<::std::string::String, ::std::string::String>,
    ) {
        self.0.properties = value;
    }
    #[setter(storage_credential_name)]
    fn set_storage_credential_name(
        &mut self,
        value: ::core::option::Option<::std::string::String>,
    ) {
        self.0.storage_credential_name = value;
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    #[setter(deleted_at)]
    fn set_deleted_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.deleted_at = value;
    }
    #[setter(table_id)]
    fn set_table_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.table_id = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "TableDependency")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn table_full_name(&self) -> ::std::string::String {
        self.0.table_full_name.clone()
    }
    #[setter(table_full_name)]
    fn set_table_full_name(&mut self, value: ::std::string::String) {
        self.0.table_full_name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "TableSummary")]
#[derive(Clone, Debug)]
pub struct PyTableSummary(pub super::tables::v1::TableSummary);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyTableSummary {
    #[new]
    #[pyo3(signature = (full_name = None, table_type = None))]
    fn new(
        full_name: ::core::option::Option<::std::string::String>,
        table_type: ::core::option::Option<PyTableType>,
    ) -> Self {
        let mut inner = <super::tables::v1::TableSummary as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        if let ::core::option::Option::Some(value) = table_type {
            inner.table_type = ::buffa::EnumValue::Known(
                <super::tables::v1::TableType as ::core::convert::From<_>>::from(value),
            );
        }
        Self(inner)
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn table_type(&self) -> PyTableType {
        PyTableType::from(self.0.table_type.as_known().unwrap_or_default())
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(table_type)]
    fn set_table_type(&mut self, value: PyTableType) {
        self.0.table_type = ::buffa::EnumValue::Known(
            <super::tables::v1::TableType as ::core::convert::From<_>>::from(value),
        );
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "CreateEntityTagAssignmentRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateEntityTagAssignmentRequest(
    pub super::tags::v1::CreateEntityTagAssignmentRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateEntityTagAssignmentRequest {
    #[new]
    #[pyo3(signature = (tag_assignment = None))]
    fn new(tag_assignment: ::core::option::Option<PyEntityTagAssignment>) -> Self {
        let mut inner = <super::tags::v1::CreateEntityTagAssignmentRequest as ::core::default::Default>::default();
        {
            let value = tag_assignment;
            inner.tag_assignment = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        Self(inner)
    }
    #[getter]
    fn tag_assignment(&self) -> ::core::option::Option<PyEntityTagAssignment> {
        self.0
            .tag_assignment
            .clone()
            .into_option()
            .map(PyEntityTagAssignment::from)
    }
    #[setter(tag_assignment)]
    fn set_tag_assignment(&mut self, value: ::core::option::Option<PyEntityTagAssignment>) {
        self.0.tag_assignment = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::CreateEntityTagAssignmentRequest>
    for PyCreateEntityTagAssignmentRequest
{
    fn from(value: super::tags::v1::CreateEntityTagAssignmentRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateEntityTagAssignmentRequest>
    for super::tags::v1::CreateEntityTagAssignmentRequest
{
    fn from(value: PyCreateEntityTagAssignmentRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "CreateTagPolicyRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateTagPolicyRequest(pub super::tags::v1::CreateTagPolicyRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateTagPolicyRequest {
    #[new]
    #[pyo3(signature = (tag_policy = None))]
    fn new(tag_policy: ::core::option::Option<PyTagPolicy>) -> Self {
        let mut inner =
            <super::tags::v1::CreateTagPolicyRequest as ::core::default::Default>::default();
        {
            let value = tag_policy;
            inner.tag_policy = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        Self(inner)
    }
    #[getter]
    fn tag_policy(&self) -> ::core::option::Option<PyTagPolicy> {
        self.0
            .tag_policy
            .clone()
            .into_option()
            .map(PyTagPolicy::from)
    }
    #[setter(tag_policy)]
    fn set_tag_policy(&mut self, value: ::core::option::Option<PyTagPolicy>) {
        self.0.tag_policy = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::CreateTagPolicyRequest> for PyCreateTagPolicyRequest {
    fn from(value: super::tags::v1::CreateTagPolicyRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateTagPolicyRequest> for super::tags::v1::CreateTagPolicyRequest {
    fn from(value: PyCreateTagPolicyRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteEntityTagAssignmentRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteEntityTagAssignmentRequest(
    pub super::tags::v1::DeleteEntityTagAssignmentRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteEntityTagAssignmentRequest {
    #[new]
    #[pyo3(signature = (entity_type = None, entity_name = None, tag_key = None))]
    fn new(
        entity_type: ::core::option::Option<::std::string::String>,
        entity_name: ::core::option::Option<::std::string::String>,
        tag_key: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::tags::v1::DeleteEntityTagAssignmentRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = entity_type {
            inner.entity_type = value;
        }
        if let ::core::option::Option::Some(value) = entity_name {
            inner.entity_name = value;
        }
        if let ::core::option::Option::Some(value) = tag_key {
            inner.tag_key = value;
        }
        Self(inner)
    }
    #[getter]
    fn entity_type(&self) -> ::std::string::String {
        self.0.entity_type.clone()
    }
    #[getter]
    fn entity_name(&self) -> ::std::string::String {
        self.0.entity_name.clone()
    }
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }
    #[setter(entity_type)]
    fn set_entity_type(&mut self, value: ::std::string::String) {
        self.0.entity_type = value;
    }
    #[setter(entity_name)]
    fn set_entity_name(&mut self, value: ::std::string::String) {
        self.0.entity_name = value;
    }
    #[setter(tag_key)]
    fn set_tag_key(&mut self, value: ::std::string::String) {
        self.0.tag_key = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::DeleteEntityTagAssignmentRequest>
    for PyDeleteEntityTagAssignmentRequest
{
    fn from(value: super::tags::v1::DeleteEntityTagAssignmentRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteEntityTagAssignmentRequest>
    for super::tags::v1::DeleteEntityTagAssignmentRequest
{
    fn from(value: PyDeleteEntityTagAssignmentRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteTagPolicyRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteTagPolicyRequest(pub super::tags::v1::DeleteTagPolicyRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteTagPolicyRequest {
    #[new]
    #[pyo3(signature = (tag_key = None))]
    fn new(tag_key: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::tags::v1::DeleteTagPolicyRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = tag_key {
            inner.tag_key = value;
        }
        Self(inner)
    }
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }
    #[setter(tag_key)]
    fn set_tag_key(&mut self, value: ::std::string::String) {
        self.0.tag_key = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::DeleteTagPolicyRequest> for PyDeleteTagPolicyRequest {
    fn from(value: super::tags::v1::DeleteTagPolicyRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteTagPolicyRequest> for super::tags::v1::DeleteTagPolicyRequest {
    fn from(value: PyDeleteTagPolicyRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "EntityTagAssignment")]
#[derive(Clone, Debug)]
pub struct PyEntityTagAssignment(pub super::tags::v1::EntityTagAssignment);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyEntityTagAssignment {
    #[new]
    #[pyo3(
        signature = (
            entity_type = None,
            entity_name = None,
            tag_key = None,
            tag_value = None
        )
    )]
    fn new(
        entity_type: ::core::option::Option<::std::string::String>,
        entity_name: ::core::option::Option<::std::string::String>,
        tag_key: ::core::option::Option<::std::string::String>,
        tag_value: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::tags::v1::EntityTagAssignment as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = entity_type {
            inner.entity_type = value;
        }
        if let ::core::option::Option::Some(value) = entity_name {
            inner.entity_name = value;
        }
        if let ::core::option::Option::Some(value) = tag_key {
            inner.tag_key = value;
        }
        {
            let value = tag_value;
            inner.tag_value = value;
        }
        Self(inner)
    }
    #[getter]
    fn entity_type(&self) -> ::std::string::String {
        self.0.entity_type.clone()
    }
    #[getter]
    fn entity_name(&self) -> ::std::string::String {
        self.0.entity_name.clone()
    }
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }
    #[getter]
    fn tag_value(&self) -> ::core::option::Option<::std::string::String> {
        self.0.tag_value.clone()
    }
    #[setter(entity_type)]
    fn set_entity_type(&mut self, value: ::std::string::String) {
        self.0.entity_type = value;
    }
    #[setter(entity_name)]
    fn set_entity_name(&mut self, value: ::std::string::String) {
        self.0.entity_name = value;
    }
    #[setter(tag_key)]
    fn set_tag_key(&mut self, value: ::std::string::String) {
        self.0.tag_key = value;
    }
    #[setter(tag_value)]
    fn set_tag_value(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.tag_value = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "GetEntityTagAssignmentRequest")]
#[derive(Clone, Debug)]
pub struct PyGetEntityTagAssignmentRequest(pub super::tags::v1::GetEntityTagAssignmentRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetEntityTagAssignmentRequest {
    #[new]
    #[pyo3(signature = (entity_type = None, entity_name = None, tag_key = None))]
    fn new(
        entity_type: ::core::option::Option<::std::string::String>,
        entity_name: ::core::option::Option<::std::string::String>,
        tag_key: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::tags::v1::GetEntityTagAssignmentRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = entity_type {
            inner.entity_type = value;
        }
        if let ::core::option::Option::Some(value) = entity_name {
            inner.entity_name = value;
        }
        if let ::core::option::Option::Some(value) = tag_key {
            inner.tag_key = value;
        }
        Self(inner)
    }
    #[getter]
    fn entity_type(&self) -> ::std::string::String {
        self.0.entity_type.clone()
    }
    #[getter]
    fn entity_name(&self) -> ::std::string::String {
        self.0.entity_name.clone()
    }
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }
    #[setter(entity_type)]
    fn set_entity_type(&mut self, value: ::std::string::String) {
        self.0.entity_type = value;
    }
    #[setter(entity_name)]
    fn set_entity_name(&mut self, value: ::std::string::String) {
        self.0.entity_name = value;
    }
    #[setter(tag_key)]
    fn set_tag_key(&mut self, value: ::std::string::String) {
        self.0.tag_key = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::GetEntityTagAssignmentRequest>
    for PyGetEntityTagAssignmentRequest
{
    fn from(value: super::tags::v1::GetEntityTagAssignmentRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetEntityTagAssignmentRequest>
    for super::tags::v1::GetEntityTagAssignmentRequest
{
    fn from(value: PyGetEntityTagAssignmentRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetTagPolicyRequest")]
#[derive(Clone, Debug)]
pub struct PyGetTagPolicyRequest(pub super::tags::v1::GetTagPolicyRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetTagPolicyRequest {
    #[new]
    #[pyo3(signature = (tag_key = None))]
    fn new(tag_key: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::tags::v1::GetTagPolicyRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = tag_key {
            inner.tag_key = value;
        }
        Self(inner)
    }
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }
    #[setter(tag_key)]
    fn set_tag_key(&mut self, value: ::std::string::String) {
        self.0.tag_key = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::GetTagPolicyRequest> for PyGetTagPolicyRequest {
    fn from(value: super::tags::v1::GetTagPolicyRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetTagPolicyRequest> for super::tags::v1::GetTagPolicyRequest {
    fn from(value: PyGetTagPolicyRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListEntityTagAssignmentsRequest")]
#[derive(Clone, Debug)]
pub struct PyListEntityTagAssignmentsRequest(pub super::tags::v1::ListEntityTagAssignmentsRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListEntityTagAssignmentsRequest {
    #[new]
    #[pyo3(
        signature = (
            entity_type = None,
            entity_name = None,
            max_results = None,
            page_token = None
        )
    )]
    fn new(
        entity_type: ::core::option::Option<::std::string::String>,
        entity_name: ::core::option::Option<::std::string::String>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::tags::v1::ListEntityTagAssignmentsRequest as ::core::default::Default>::default(
            );
        if let ::core::option::Option::Some(value) = entity_type {
            inner.entity_type = value;
        }
        if let ::core::option::Option::Some(value) = entity_name {
            inner.entity_name = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn entity_type(&self) -> ::std::string::String {
        self.0.entity_type.clone()
    }
    #[getter]
    fn entity_name(&self) -> ::std::string::String {
        self.0.entity_name.clone()
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[setter(entity_type)]
    fn set_entity_type(&mut self, value: ::std::string::String) {
        self.0.entity_type = value;
    }
    #[setter(entity_name)]
    fn set_entity_name(&mut self, value: ::std::string::String) {
        self.0.entity_name = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::ListEntityTagAssignmentsRequest>
    for PyListEntityTagAssignmentsRequest
{
    fn from(value: super::tags::v1::ListEntityTagAssignmentsRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListEntityTagAssignmentsRequest>
    for super::tags::v1::ListEntityTagAssignmentsRequest
{
    fn from(value: PyListEntityTagAssignmentsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListEntityTagAssignmentsResponse")]
#[derive(Clone, Debug)]
pub struct PyListEntityTagAssignmentsResponse(
    pub super::tags::v1::ListEntityTagAssignmentsResponse,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListEntityTagAssignmentsResponse {
    #[new]
    #[pyo3(signature = (tag_assignments = None, next_page_token = None))]
    fn new(
        tag_assignments: ::core::option::Option<::std::vec::Vec<PyEntityTagAssignment>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::tags::v1::ListEntityTagAssignmentsResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = tag_assignments {
            inner.tag_assignments = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn tag_assignments(&self) -> ::std::vec::Vec<PyEntityTagAssignment> {
        self.0
            .tag_assignments
            .iter()
            .cloned()
            .map(PyEntityTagAssignment::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(tag_assignments)]
    fn set_tag_assignments(&mut self, value: ::std::vec::Vec<PyEntityTagAssignment>) {
        self.0.tag_assignments = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "ListTagPoliciesRequest")]
#[derive(Clone, Debug)]
pub struct PyListTagPoliciesRequest(pub super::tags::v1::ListTagPoliciesRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListTagPoliciesRequest {
    #[new]
    #[pyo3(signature = (max_results = None, page_token = None))]
    fn new(
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::tags::v1::ListTagPoliciesRequest as ::core::default::Default>::default();
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::ListTagPoliciesRequest> for PyListTagPoliciesRequest {
    fn from(value: super::tags::v1::ListTagPoliciesRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListTagPoliciesRequest> for super::tags::v1::ListTagPoliciesRequest {
    fn from(value: PyListTagPoliciesRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListTagPoliciesResponse")]
#[derive(Clone, Debug)]
pub struct PyListTagPoliciesResponse(pub super::tags::v1::ListTagPoliciesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListTagPoliciesResponse {
    #[new]
    #[pyo3(signature = (tag_policies = None, next_page_token = None))]
    fn new(
        tag_policies: ::core::option::Option<::std::vec::Vec<PyTagPolicy>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::tags::v1::ListTagPoliciesResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = tag_policies {
            inner.tag_policies = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn tag_policies(&self) -> ::std::vec::Vec<PyTagPolicy> {
        self.0
            .tag_policies
            .iter()
            .cloned()
            .map(PyTagPolicy::from)
            .collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(tag_policies)]
    fn set_tag_policies(&mut self, value: ::std::vec::Vec<PyTagPolicy>) {
        self.0.tag_policies = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "TagPolicy")]
#[derive(Clone, Debug)]
pub struct PyTagPolicy(pub super::tags::v1::TagPolicy);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyTagPolicy {
    #[new]
    #[pyo3(
        signature = (
            tag_key = None,
            description = None,
            values = None,
            id = None,
            created_at = None,
            updated_at = None
        )
    )]
    fn new(
        tag_key: ::core::option::Option<::std::string::String>,
        description: ::core::option::Option<::std::string::String>,
        values: ::core::option::Option<::std::vec::Vec<PyValue>>,
        id: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        updated_at: ::core::option::Option<i64>,
    ) -> Self {
        let mut inner = <super::tags::v1::TagPolicy as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = tag_key {
            inner.tag_key = value;
        }
        {
            let value = description;
            inner.description = value;
        }
        if let ::core::option::Option::Some(value) = values {
            inner.values = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = id;
            inner.id = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        Self(inner)
    }
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }
    #[getter]
    fn description(&self) -> ::core::option::Option<::std::string::String> {
        self.0.description.clone()
    }
    #[getter]
    fn values(&self) -> ::std::vec::Vec<PyValue> {
        self.0.values.iter().cloned().map(PyValue::from).collect()
    }
    #[getter]
    fn id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.id.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[setter(tag_key)]
    fn set_tag_key(&mut self, value: ::std::string::String) {
        self.0.tag_key = value;
    }
    #[setter(description)]
    fn set_description(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.description = value;
    }
    #[setter(values)]
    fn set_values(&mut self, value: ::std::vec::Vec<PyValue>) {
        self.0.values = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(id)]
    fn set_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.id = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateEntityTagAssignmentRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateEntityTagAssignmentRequest(
    pub super::tags::v1::UpdateEntityTagAssignmentRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateEntityTagAssignmentRequest {
    #[new]
    #[pyo3(
        signature = (
            entity_type = None,
            entity_name = None,
            tag_key = None,
            tag_assignment = None,
            update_mask = None
        )
    )]
    fn new(
        entity_type: ::core::option::Option<::std::string::String>,
        entity_name: ::core::option::Option<::std::string::String>,
        tag_key: ::core::option::Option<::std::string::String>,
        tag_assignment: ::core::option::Option<PyEntityTagAssignment>,
        update_mask: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::tags::v1::UpdateEntityTagAssignmentRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = entity_type {
            inner.entity_type = value;
        }
        if let ::core::option::Option::Some(value) = entity_name {
            inner.entity_name = value;
        }
        if let ::core::option::Option::Some(value) = tag_key {
            inner.tag_key = value;
        }
        {
            let value = tag_assignment;
            inner.tag_assignment = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = update_mask;
            inner.update_mask = value;
        }
        Self(inner)
    }
    #[getter]
    fn entity_type(&self) -> ::std::string::String {
        self.0.entity_type.clone()
    }
    #[getter]
    fn entity_name(&self) -> ::std::string::String {
        self.0.entity_name.clone()
    }
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }
    #[getter]
    fn tag_assignment(&self) -> ::core::option::Option<PyEntityTagAssignment> {
        self.0
            .tag_assignment
            .clone()
            .into_option()
            .map(PyEntityTagAssignment::from)
    }
    #[getter]
    fn update_mask(&self) -> ::core::option::Option<::std::string::String> {
        self.0.update_mask.clone()
    }
    #[setter(entity_type)]
    fn set_entity_type(&mut self, value: ::std::string::String) {
        self.0.entity_type = value;
    }
    #[setter(entity_name)]
    fn set_entity_name(&mut self, value: ::std::string::String) {
        self.0.entity_name = value;
    }
    #[setter(tag_key)]
    fn set_tag_key(&mut self, value: ::std::string::String) {
        self.0.tag_key = value;
    }
    #[setter(tag_assignment)]
    fn set_tag_assignment(&mut self, value: ::core::option::Option<PyEntityTagAssignment>) {
        self.0.tag_assignment = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(update_mask)]
    fn set_update_mask(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.update_mask = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::UpdateEntityTagAssignmentRequest>
    for PyUpdateEntityTagAssignmentRequest
{
    fn from(value: super::tags::v1::UpdateEntityTagAssignmentRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateEntityTagAssignmentRequest>
    for super::tags::v1::UpdateEntityTagAssignmentRequest
{
    fn from(value: PyUpdateEntityTagAssignmentRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "UpdateTagPolicyRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateTagPolicyRequest(pub super::tags::v1::UpdateTagPolicyRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateTagPolicyRequest {
    #[new]
    #[pyo3(signature = (tag_key = None, tag_policy = None, update_mask = None))]
    fn new(
        tag_key: ::core::option::Option<::std::string::String>,
        tag_policy: ::core::option::Option<PyTagPolicy>,
        update_mask: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::tags::v1::UpdateTagPolicyRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = tag_key {
            inner.tag_key = value;
        }
        {
            let value = tag_policy;
            inner.tag_policy = value
                .map(|w| ::buffa::MessageField::some(w.into()))
                .unwrap_or_default();
        }
        {
            let value = update_mask;
            inner.update_mask = value;
        }
        Self(inner)
    }
    #[getter]
    fn tag_key(&self) -> ::std::string::String {
        self.0.tag_key.clone()
    }
    #[getter]
    fn tag_policy(&self) -> ::core::option::Option<PyTagPolicy> {
        self.0
            .tag_policy
            .clone()
            .into_option()
            .map(PyTagPolicy::from)
    }
    #[getter]
    fn update_mask(&self) -> ::core::option::Option<::std::string::String> {
        self.0.update_mask.clone()
    }
    #[setter(tag_key)]
    fn set_tag_key(&mut self, value: ::std::string::String) {
        self.0.tag_key = value;
    }
    #[setter(tag_policy)]
    fn set_tag_policy(&mut self, value: ::core::option::Option<PyTagPolicy>) {
        self.0.tag_policy = value
            .map(|w| ::buffa::MessageField::some(w.into()))
            .unwrap_or_default();
    }
    #[setter(update_mask)]
    fn set_update_mask(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.update_mask = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::tags::v1::UpdateTagPolicyRequest> for PyUpdateTagPolicyRequest {
    fn from(value: super::tags::v1::UpdateTagPolicyRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateTagPolicyRequest> for super::tags::v1::UpdateTagPolicyRequest {
    fn from(value: PyUpdateTagPolicyRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "Value")]
#[derive(Clone, Debug)]
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
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "AwsTemporaryCredentials")]
#[derive(Clone, Debug)]
pub struct PyAwsTemporaryCredentials(pub super::temporary_credentials::v1::AwsTemporaryCredentials);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAwsTemporaryCredentials {
    #[new]
    #[pyo3(
        signature = (
            access_key_id = None,
            access_point = None,
            secret_access_key = None,
            session_token = None
        )
    )]
    fn new(
        access_key_id: ::core::option::Option<::std::string::String>,
        access_point: ::core::option::Option<::std::string::String>,
        secret_access_key: ::core::option::Option<::std::string::String>,
        session_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::temporary_credentials::v1::AwsTemporaryCredentials as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = access_key_id {
            inner.access_key_id = value;
        }
        if let ::core::option::Option::Some(value) = access_point {
            inner.access_point = value;
        }
        if let ::core::option::Option::Some(value) = secret_access_key {
            inner.secret_access_key = value;
        }
        if let ::core::option::Option::Some(value) = session_token {
            inner.session_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn access_key_id(&self) -> ::std::string::String {
        self.0.access_key_id.clone()
    }
    #[getter]
    fn access_point(&self) -> ::std::string::String {
        self.0.access_point.clone()
    }
    #[getter]
    fn secret_access_key(&self) -> ::std::string::String {
        self.0.secret_access_key.clone()
    }
    #[getter]
    fn session_token(&self) -> ::std::string::String {
        self.0.session_token.clone()
    }
    #[setter(access_key_id)]
    fn set_access_key_id(&mut self, value: ::std::string::String) {
        self.0.access_key_id = value;
    }
    #[setter(access_point)]
    fn set_access_point(&mut self, value: ::std::string::String) {
        self.0.access_point = value;
    }
    #[setter(secret_access_key)]
    fn set_secret_access_key(&mut self, value: ::std::string::String) {
        self.0.secret_access_key = value;
    }
    #[setter(session_token)]
    fn set_session_token(&mut self, value: ::std::string::String) {
        self.0.session_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "AzureAad")]
#[derive(Clone, Debug)]
pub struct PyAzureAad(pub super::temporary_credentials::v1::AzureAad);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAzureAad {
    #[new]
    #[pyo3(signature = (aad_token = None))]
    fn new(aad_token: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::temporary_credentials::v1::AzureAad as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = aad_token {
            inner.aad_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn aad_token(&self) -> ::std::string::String {
        self.0.aad_token.clone()
    }
    #[setter(aad_token)]
    fn set_aad_token(&mut self, value: ::std::string::String) {
        self.0.aad_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "AzureUserDelegationSas")]
#[derive(Clone, Debug)]
pub struct PyAzureUserDelegationSas(pub super::temporary_credentials::v1::AzureUserDelegationSas);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyAzureUserDelegationSas {
    #[new]
    #[pyo3(signature = (sas_token = None))]
    fn new(sas_token: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner = <super::temporary_credentials::v1::AzureUserDelegationSas as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = sas_token {
            inner.sas_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn sas_token(&self) -> ::std::string::String {
        self.0.sas_token.clone()
    }
    #[setter(sas_token)]
    fn set_sas_token(&mut self, value: ::std::string::String) {
        self.0.sas_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "GcpOauthToken")]
#[derive(Clone, Debug)]
pub struct PyGcpOauthToken(pub super::temporary_credentials::v1::GcpOauthToken);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGcpOauthToken {
    #[new]
    #[pyo3(signature = (oauth_token = None))]
    fn new(oauth_token: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::temporary_credentials::v1::GcpOauthToken as ::core::default::Default>::default(
            );
        if let ::core::option::Option::Some(value) = oauth_token {
            inner.oauth_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn oauth_token(&self) -> ::std::string::String {
        self.0.oauth_token.clone()
    }
    #[setter(oauth_token)]
    fn set_oauth_token(&mut self, value: ::std::string::String) {
        self.0.oauth_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "GenerateTemporaryPathCredentialsRequest")]
#[derive(Clone, Debug)]
pub struct PyGenerateTemporaryPathCredentialsRequest(
    pub super::temporary_credentials::v1::GenerateTemporaryPathCredentialsRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGenerateTemporaryPathCredentialsRequest {
    #[new]
    #[pyo3(signature = (url = None, operation = None, dry_run = None))]
    fn new(
        url: ::core::option::Option<::std::string::String>,
        operation: ::core::option::Option<PyGenerateTemporaryPathCredentialsRequestOperation>,
        dry_run: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner = <super::temporary_credentials::v1::GenerateTemporaryPathCredentialsRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = url {
            inner.url = value;
        }
        if let ::core::option::Option::Some(value) = operation {
            inner.operation = ::buffa::EnumValue::Known(
                <super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation as ::core::convert::From<
                    _,
                >>::from(value),
            );
        }
        {
            let value = dry_run;
            inner.dry_run = value;
        }
        Self(inner)
    }
    #[getter]
    fn url(&self) -> ::std::string::String {
        self.0.url.clone()
    }
    #[getter]
    fn operation(&self) -> PyGenerateTemporaryPathCredentialsRequestOperation {
        PyGenerateTemporaryPathCredentialsRequestOperation::from(
            self.0.operation.as_known().unwrap_or_default(),
        )
    }
    #[getter]
    fn dry_run(&self) -> ::core::option::Option<bool> {
        self.0.dry_run
    }
    #[setter(url)]
    fn set_url(&mut self, value: ::std::string::String) {
        self.0.url = value;
    }
    #[setter(operation)]
    fn set_operation(&mut self, value: PyGenerateTemporaryPathCredentialsRequestOperation) {
        self.0.operation = ::buffa::EnumValue::Known(
            <super::temporary_credentials::v1::generate_temporary_path_credentials_request::Operation as ::core::convert::From<
                _,
            >>::from(value),
        );
    }
    #[setter(dry_run)]
    fn set_dry_run(&mut self, value: ::core::option::Option<bool>) {
        self.0.dry_run = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl
    ::core::convert::From<super::temporary_credentials::v1::GenerateTemporaryPathCredentialsRequest>
    for PyGenerateTemporaryPathCredentialsRequest
{
    fn from(
        value: super::temporary_credentials::v1::GenerateTemporaryPathCredentialsRequest,
    ) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGenerateTemporaryPathCredentialsRequest>
    for super::temporary_credentials::v1::GenerateTemporaryPathCredentialsRequest
{
    fn from(value: PyGenerateTemporaryPathCredentialsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GenerateTemporaryTableCredentialsRequest")]
#[derive(Clone, Debug)]
pub struct PyGenerateTemporaryTableCredentialsRequest(
    pub super::temporary_credentials::v1::GenerateTemporaryTableCredentialsRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGenerateTemporaryTableCredentialsRequest {
    #[new]
    #[pyo3(signature = (table_id = None, operation = None))]
    fn new(
        table_id: ::core::option::Option<::std::string::String>,
        operation: ::core::option::Option<PyGenerateTemporaryTableCredentialsRequestOperation>,
    ) -> Self {
        let mut inner = <super::temporary_credentials::v1::GenerateTemporaryTableCredentialsRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = table_id {
            inner.table_id = value;
        }
        if let ::core::option::Option::Some(value) = operation {
            inner.operation = ::buffa::EnumValue::Known(
                <super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation as ::core::convert::From<
                    _,
                >>::from(value),
            );
        }
        Self(inner)
    }
    #[getter]
    fn table_id(&self) -> ::std::string::String {
        self.0.table_id.clone()
    }
    #[getter]
    fn operation(&self) -> PyGenerateTemporaryTableCredentialsRequestOperation {
        PyGenerateTemporaryTableCredentialsRequestOperation::from(
            self.0.operation.as_known().unwrap_or_default(),
        )
    }
    #[setter(table_id)]
    fn set_table_id(&mut self, value: ::std::string::String) {
        self.0.table_id = value;
    }
    #[setter(operation)]
    fn set_operation(&mut self, value: PyGenerateTemporaryTableCredentialsRequestOperation) {
        self.0.operation = ::buffa::EnumValue::Known(
            <super::temporary_credentials::v1::generate_temporary_table_credentials_request::Operation as ::core::convert::From<
                _,
            >>::from(value),
        );
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl
    ::core::convert::From<
        super::temporary_credentials::v1::GenerateTemporaryTableCredentialsRequest,
    > for PyGenerateTemporaryTableCredentialsRequest
{
    fn from(
        value: super::temporary_credentials::v1::GenerateTemporaryTableCredentialsRequest,
    ) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGenerateTemporaryTableCredentialsRequest>
    for super::temporary_credentials::v1::GenerateTemporaryTableCredentialsRequest
{
    fn from(value: PyGenerateTemporaryTableCredentialsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GenerateTemporaryVolumeCredentialsRequest")]
#[derive(Clone, Debug)]
pub struct PyGenerateTemporaryVolumeCredentialsRequest(
    pub super::temporary_credentials::v1::GenerateTemporaryVolumeCredentialsRequest,
);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGenerateTemporaryVolumeCredentialsRequest {
    #[new]
    #[pyo3(signature = (volume_id = None, operation = None))]
    fn new(
        volume_id: ::core::option::Option<::std::string::String>,
        operation: ::core::option::Option<PyGenerateTemporaryVolumeCredentialsRequestOperation>,
    ) -> Self {
        let mut inner = <super::temporary_credentials::v1::GenerateTemporaryVolumeCredentialsRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = volume_id {
            inner.volume_id = value;
        }
        if let ::core::option::Option::Some(value) = operation {
            inner.operation = ::buffa::EnumValue::Known(
                <super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation as ::core::convert::From<
                    _,
                >>::from(value),
            );
        }
        Self(inner)
    }
    #[getter]
    fn volume_id(&self) -> ::std::string::String {
        self.0.volume_id.clone()
    }
    #[getter]
    fn operation(&self) -> PyGenerateTemporaryVolumeCredentialsRequestOperation {
        PyGenerateTemporaryVolumeCredentialsRequestOperation::from(
            self.0.operation.as_known().unwrap_or_default(),
        )
    }
    #[setter(volume_id)]
    fn set_volume_id(&mut self, value: ::std::string::String) {
        self.0.volume_id = value;
    }
    #[setter(operation)]
    fn set_operation(&mut self, value: PyGenerateTemporaryVolumeCredentialsRequestOperation) {
        self.0.operation = ::buffa::EnumValue::Known(
            <super::temporary_credentials::v1::generate_temporary_volume_credentials_request::Operation as ::core::convert::From<
                _,
            >>::from(value),
        );
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl
    ::core::convert::From<
        super::temporary_credentials::v1::GenerateTemporaryVolumeCredentialsRequest,
    > for PyGenerateTemporaryVolumeCredentialsRequest
{
    fn from(
        value: super::temporary_credentials::v1::GenerateTemporaryVolumeCredentialsRequest,
    ) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGenerateTemporaryVolumeCredentialsRequest>
    for super::temporary_credentials::v1::GenerateTemporaryVolumeCredentialsRequest
{
    fn from(value: PyGenerateTemporaryVolumeCredentialsRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "R2TemporaryCredentials")]
#[derive(Clone, Debug)]
pub struct PyR2TemporaryCredentials(pub super::temporary_credentials::v1::R2TemporaryCredentials);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyR2TemporaryCredentials {
    #[new]
    #[pyo3(
        signature = (
            access_key_id = None,
            secret_access_key = None,
            session_token = None
        )
    )]
    fn new(
        access_key_id: ::core::option::Option<::std::string::String>,
        secret_access_key: ::core::option::Option<::std::string::String>,
        session_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::temporary_credentials::v1::R2TemporaryCredentials as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = access_key_id {
            inner.access_key_id = value;
        }
        if let ::core::option::Option::Some(value) = secret_access_key {
            inner.secret_access_key = value;
        }
        if let ::core::option::Option::Some(value) = session_token {
            inner.session_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn access_key_id(&self) -> ::std::string::String {
        self.0.access_key_id.clone()
    }
    #[getter]
    fn secret_access_key(&self) -> ::std::string::String {
        self.0.secret_access_key.clone()
    }
    #[getter]
    fn session_token(&self) -> ::std::string::String {
        self.0.session_token.clone()
    }
    #[setter(access_key_id)]
    fn set_access_key_id(&mut self, value: ::std::string::String) {
        self.0.access_key_id = value;
    }
    #[setter(secret_access_key)]
    fn set_secret_access_key(&mut self, value: ::std::string::String) {
        self.0.secret_access_key = value;
    }
    #[setter(session_token)]
    fn set_session_token(&mut self, value: ::std::string::String) {
        self.0.session_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "TemporaryCredential")]
#[derive(Clone, Debug)]
pub struct PyTemporaryCredential(pub super::temporary_credentials::v1::TemporaryCredential);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyTemporaryCredential {
    #[new]
    #[pyo3(signature = (expiration_time = None, url = None))]
    fn new(
        expiration_time: ::core::option::Option<i64>,
        url: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::temporary_credentials::v1::TemporaryCredential as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = expiration_time {
            inner.expiration_time = value;
        }
        if let ::core::option::Option::Some(value) = url {
            inner.url = value;
        }
        Self(inner)
    }
    #[getter]
    fn expiration_time(&self) -> i64 {
        self.0.expiration_time
    }
    #[getter]
    fn url(&self) -> ::std::string::String {
        self.0.url.clone()
    }
    #[setter(expiration_time)]
    fn set_expiration_time(&mut self, value: i64) {
        self.0.expiration_time = value;
    }
    #[setter(url)]
    fn set_url(&mut self, value: ::std::string::String) {
        self.0.url = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "CreateVolumeRequest")]
#[derive(Clone, Debug)]
pub struct PyCreateVolumeRequest(pub super::volumes::v1::CreateVolumeRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyCreateVolumeRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name = None,
            name = None,
            volume_type = None,
            storage_location = None,
            comment = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        name: ::core::option::Option<::std::string::String>,
        volume_type: ::core::option::Option<PyVolumeType>,
        storage_location: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::volumes::v1::CreateVolumeRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = volume_type {
            inner.volume_type = ::buffa::EnumValue::Known(
                <super::volumes::v1::VolumeType as ::core::convert::From<_>>::from(value),
            );
        }
        {
            let value = storage_location;
            inner.storage_location = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn volume_type(&self) -> PyVolumeType {
        PyVolumeType::from(self.0.volume_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn storage_location(&self) -> ::core::option::Option<::std::string::String> {
        self.0.storage_location.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(volume_type)]
    fn set_volume_type(&mut self, value: PyVolumeType) {
        self.0.volume_type =
            ::buffa::EnumValue::Known(<super::volumes::v1::VolumeType as ::core::convert::From<
                _,
            >>::from(value));
    }
    #[setter(storage_location)]
    fn set_storage_location(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.storage_location = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::volumes::v1::CreateVolumeRequest> for PyCreateVolumeRequest {
    fn from(value: super::volumes::v1::CreateVolumeRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyCreateVolumeRequest> for super::volumes::v1::CreateVolumeRequest {
    fn from(value: PyCreateVolumeRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "DeleteVolumeRequest")]
#[derive(Clone, Debug)]
pub struct PyDeleteVolumeRequest(pub super::volumes::v1::DeleteVolumeRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyDeleteVolumeRequest {
    #[new]
    #[pyo3(signature = (name = None))]
    fn new(name: ::core::option::Option<::std::string::String>) -> Self {
        let mut inner =
            <super::volumes::v1::DeleteVolumeRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::volumes::v1::DeleteVolumeRequest> for PyDeleteVolumeRequest {
    fn from(value: super::volumes::v1::DeleteVolumeRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyDeleteVolumeRequest> for super::volumes::v1::DeleteVolumeRequest {
    fn from(value: PyDeleteVolumeRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "GetVolumeRequest")]
#[derive(Clone, Debug)]
pub struct PyGetVolumeRequest(pub super::volumes::v1::GetVolumeRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyGetVolumeRequest {
    #[new]
    #[pyo3(signature = (name = None, include_browse = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::volumes::v1::GetVolumeRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::volumes::v1::GetVolumeRequest> for PyGetVolumeRequest {
    fn from(value: super::volumes::v1::GetVolumeRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyGetVolumeRequest> for super::volumes::v1::GetVolumeRequest {
    fn from(value: PyGetVolumeRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListVolumesRequest")]
#[derive(Clone, Debug)]
pub struct PyListVolumesRequest(pub super::volumes::v1::ListVolumesRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListVolumesRequest {
    #[new]
    #[pyo3(
        signature = (
            catalog_name = None,
            schema_name = None,
            max_results = None,
            page_token = None,
            include_browse = None
        )
    )]
    fn new(
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        max_results: ::core::option::Option<i32>,
        page_token: ::core::option::Option<::std::string::String>,
        include_browse: ::core::option::Option<bool>,
    ) -> Self {
        let mut inner =
            <super::volumes::v1::ListVolumesRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        {
            let value = max_results;
            inner.max_results = value;
        }
        {
            let value = page_token;
            inner.page_token = value;
        }
        {
            let value = include_browse;
            inner.include_browse = value;
        }
        Self(inner)
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn max_results(&self) -> ::core::option::Option<i32> {
        self.0.max_results
    }
    #[getter]
    fn page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.page_token.clone()
    }
    #[getter]
    fn include_browse(&self) -> ::core::option::Option<bool> {
        self.0.include_browse
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(max_results)]
    fn set_max_results(&mut self, value: ::core::option::Option<i32>) {
        self.0.max_results = value;
    }
    #[setter(page_token)]
    fn set_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.page_token = value;
    }
    #[setter(include_browse)]
    fn set_include_browse(&mut self, value: ::core::option::Option<bool>) {
        self.0.include_browse = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::volumes::v1::ListVolumesRequest> for PyListVolumesRequest {
    fn from(value: super::volumes::v1::ListVolumesRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyListVolumesRequest> for super::volumes::v1::ListVolumesRequest {
    fn from(value: PyListVolumesRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "ListVolumesResponse")]
#[derive(Clone, Debug)]
pub struct PyListVolumesResponse(pub super::volumes::v1::ListVolumesResponse);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyListVolumesResponse {
    #[new]
    #[pyo3(signature = (volumes = None, next_page_token = None))]
    fn new(
        volumes: ::core::option::Option<::std::vec::Vec<PyVolume>>,
        next_page_token: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::volumes::v1::ListVolumesResponse as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = volumes {
            inner.volumes = value.into_iter().map(::core::convert::Into::into).collect();
        }
        {
            let value = next_page_token;
            inner.next_page_token = value;
        }
        Self(inner)
    }
    #[getter]
    fn volumes(&self) -> ::std::vec::Vec<PyVolume> {
        self.0.volumes.iter().cloned().map(PyVolume::from).collect()
    }
    #[getter]
    fn next_page_token(&self) -> ::core::option::Option<::std::string::String> {
        self.0.next_page_token.clone()
    }
    #[setter(volumes)]
    fn set_volumes(&mut self, value: ::std::vec::Vec<PyVolume>) {
        self.0.volumes = value.into_iter().map(::core::convert::Into::into).collect();
    }
    #[setter(next_page_token)]
    fn set_next_page_token(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.next_page_token = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
#[::pyo3::pyclass(name = "UpdateVolumeRequest")]
#[derive(Clone, Debug)]
pub struct PyUpdateVolumeRequest(pub super::volumes::v1::UpdateVolumeRequest);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyUpdateVolumeRequest {
    #[new]
    #[pyo3(signature = (name = None, new_name = None, comment = None, owner = None))]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        new_name: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        owner: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner =
            <super::volumes::v1::UpdateVolumeRequest as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        {
            let value = new_name;
            inner.new_name = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = owner;
            inner.owner = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn new_name(&self) -> ::core::option::Option<::std::string::String> {
        self.0.new_name.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(new_name)]
    fn set_new_name(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.new_name = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}
impl ::core::convert::From<super::volumes::v1::UpdateVolumeRequest> for PyUpdateVolumeRequest {
    fn from(value: super::volumes::v1::UpdateVolumeRequest) -> Self {
        Self(value)
    }
}
impl ::core::convert::From<PyUpdateVolumeRequest> for super::volumes::v1::UpdateVolumeRequest {
    fn from(value: PyUpdateVolumeRequest) -> Self {
        value.0
    }
}
#[::pyo3::pyclass(name = "Volume")]
#[derive(Clone, Debug)]
pub struct PyVolume(pub super::volumes::v1::Volume);
#[allow(clippy::too_many_arguments, clippy::useless_conversion)]
#[::pyo3::pymethods]
impl PyVolume {
    #[new]
    #[pyo3(
        signature = (
            name = None,
            catalog_name = None,
            schema_name = None,
            full_name = None,
            storage_location = None,
            volume_id = None,
            volume_type = None,
            owner = None,
            comment = None,
            created_at = None,
            created_by = None,
            updated_at = None,
            updated_by = None,
            browse_only = None,
            metastore_id = None
        )
    )]
    fn new(
        name: ::core::option::Option<::std::string::String>,
        catalog_name: ::core::option::Option<::std::string::String>,
        schema_name: ::core::option::Option<::std::string::String>,
        full_name: ::core::option::Option<::std::string::String>,
        storage_location: ::core::option::Option<::std::string::String>,
        volume_id: ::core::option::Option<::std::string::String>,
        volume_type: ::core::option::Option<PyVolumeType>,
        owner: ::core::option::Option<::std::string::String>,
        comment: ::core::option::Option<::std::string::String>,
        created_at: ::core::option::Option<i64>,
        created_by: ::core::option::Option<::std::string::String>,
        updated_at: ::core::option::Option<i64>,
        updated_by: ::core::option::Option<::std::string::String>,
        browse_only: ::core::option::Option<bool>,
        metastore_id: ::core::option::Option<::std::string::String>,
    ) -> Self {
        let mut inner = <super::volumes::v1::Volume as ::core::default::Default>::default();
        if let ::core::option::Option::Some(value) = name {
            inner.name = value;
        }
        if let ::core::option::Option::Some(value) = catalog_name {
            inner.catalog_name = value;
        }
        if let ::core::option::Option::Some(value) = schema_name {
            inner.schema_name = value;
        }
        if let ::core::option::Option::Some(value) = full_name {
            inner.full_name = value;
        }
        if let ::core::option::Option::Some(value) = storage_location {
            inner.storage_location = value;
        }
        if let ::core::option::Option::Some(value) = volume_id {
            inner.volume_id = value;
        }
        if let ::core::option::Option::Some(value) = volume_type {
            inner.volume_type = ::buffa::EnumValue::Known(
                <super::volumes::v1::VolumeType as ::core::convert::From<_>>::from(value),
            );
        }
        {
            let value = owner;
            inner.owner = value;
        }
        {
            let value = comment;
            inner.comment = value;
        }
        {
            let value = created_at;
            inner.created_at = value;
        }
        {
            let value = created_by;
            inner.created_by = value;
        }
        {
            let value = updated_at;
            inner.updated_at = value;
        }
        {
            let value = updated_by;
            inner.updated_by = value;
        }
        {
            let value = browse_only;
            inner.browse_only = value;
        }
        {
            let value = metastore_id;
            inner.metastore_id = value;
        }
        Self(inner)
    }
    #[getter]
    fn name(&self) -> ::std::string::String {
        self.0.name.clone()
    }
    #[getter]
    fn catalog_name(&self) -> ::std::string::String {
        self.0.catalog_name.clone()
    }
    #[getter]
    fn schema_name(&self) -> ::std::string::String {
        self.0.schema_name.clone()
    }
    #[getter]
    fn full_name(&self) -> ::std::string::String {
        self.0.full_name.clone()
    }
    #[getter]
    fn storage_location(&self) -> ::std::string::String {
        self.0.storage_location.clone()
    }
    #[getter]
    fn volume_id(&self) -> ::std::string::String {
        self.0.volume_id.clone()
    }
    #[getter]
    fn volume_type(&self) -> PyVolumeType {
        PyVolumeType::from(self.0.volume_type.as_known().unwrap_or_default())
    }
    #[getter]
    fn owner(&self) -> ::core::option::Option<::std::string::String> {
        self.0.owner.clone()
    }
    #[getter]
    fn comment(&self) -> ::core::option::Option<::std::string::String> {
        self.0.comment.clone()
    }
    #[getter]
    fn created_at(&self) -> ::core::option::Option<i64> {
        self.0.created_at
    }
    #[getter]
    fn created_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.created_by.clone()
    }
    #[getter]
    fn updated_at(&self) -> ::core::option::Option<i64> {
        self.0.updated_at
    }
    #[getter]
    fn updated_by(&self) -> ::core::option::Option<::std::string::String> {
        self.0.updated_by.clone()
    }
    #[getter]
    fn browse_only(&self) -> ::core::option::Option<bool> {
        self.0.browse_only
    }
    #[getter]
    fn metastore_id(&self) -> ::core::option::Option<::std::string::String> {
        self.0.metastore_id.clone()
    }
    #[setter(name)]
    fn set_name(&mut self, value: ::std::string::String) {
        self.0.name = value;
    }
    #[setter(catalog_name)]
    fn set_catalog_name(&mut self, value: ::std::string::String) {
        self.0.catalog_name = value;
    }
    #[setter(schema_name)]
    fn set_schema_name(&mut self, value: ::std::string::String) {
        self.0.schema_name = value;
    }
    #[setter(full_name)]
    fn set_full_name(&mut self, value: ::std::string::String) {
        self.0.full_name = value;
    }
    #[setter(storage_location)]
    fn set_storage_location(&mut self, value: ::std::string::String) {
        self.0.storage_location = value;
    }
    #[setter(volume_id)]
    fn set_volume_id(&mut self, value: ::std::string::String) {
        self.0.volume_id = value;
    }
    #[setter(volume_type)]
    fn set_volume_type(&mut self, value: PyVolumeType) {
        self.0.volume_type =
            ::buffa::EnumValue::Known(<super::volumes::v1::VolumeType as ::core::convert::From<
                _,
            >>::from(value));
    }
    #[setter(owner)]
    fn set_owner(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.owner = value;
    }
    #[setter(comment)]
    fn set_comment(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.comment = value;
    }
    #[setter(created_at)]
    fn set_created_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.created_at = value;
    }
    #[setter(created_by)]
    fn set_created_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.created_by = value;
    }
    #[setter(updated_at)]
    fn set_updated_at(&mut self, value: ::core::option::Option<i64>) {
        self.0.updated_at = value;
    }
    #[setter(updated_by)]
    fn set_updated_by(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.updated_by = value;
    }
    #[setter(browse_only)]
    fn set_browse_only(&mut self, value: ::core::option::Option<bool>) {
        self.0.browse_only = value;
    }
    #[setter(metastore_id)]
    fn set_metastore_id(&mut self, value: ::core::option::Option<::std::string::String>) {
        self.0.metastore_id = value;
    }
    fn __repr__(&self) -> ::std::string::String {
        ::std::format!("{:?}", self.0)
    }
    fn __eq__(&self, other: &Self) -> bool {
        self.0 == other.0
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
