# @generated — do not edit by hand.
from __future__ import annotations
from typing import Optional, List, Dict
import enum

class Agent:
    """An agent registered in Unity Catalog."""

    agent_id: str
    """The unique identifier of the agent."""
    capabilities: List[str]
    """
    Capability identifiers advertised by the agent (e.g. `sql_query`, `document_search`,
    `code_execution`).
    """
    catalog_name: str
    """Name of parent catalog."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this agent was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of agent creator."""
    description: Optional[str]
    """An LLM-readable description of what the agent does and the inputs it expects."""
    endpoint: str
    """The agent's invocation endpoint URL."""
    full_name: str
    """The three-level (fully qualified) name of the agent."""
    input_schema: Optional[str]
    """
    A JSON Schema (encoded as a JSON string) describing the agent's expected input payload.
    Kept as an opaque string so the schema can evolve without changing this message.
    """
    invocation_protocol: InvocationProtocol
    """The protocol a recipient uses to invoke the agent."""
    metastore_id: Optional[str]
    """The unique identifier of the metastore."""
    name: str
    """Name of the agent, relative to parent schema."""
    owner: Optional[str]
    """Username of current owner of the agent."""
    schema_name: str
    """Name of parent schema."""
    updated_at: Optional[int]
    """Time at which this agent was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified the agent."""

    def __init__(
        self,
        agent_id: Optional[str] = None,
        capabilities: Optional[List[str]] = None,
        catalog_name: Optional[str] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        description: Optional[str] = None,
        endpoint: Optional[str] = None,
        full_name: Optional[str] = None,
        input_schema: Optional[str] = None,
        invocation_protocol: Optional[InvocationProtocol] = None,
        metastore_id: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        schema_name: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class AgentSkill:
    """An agent skill registered in Unity Catalog."""

    agent_skill_id: str
    """The unique identifier of the agent skill."""
    agent_skill_type: AgentSkillType
    """How the storage location is provisioned (external or managed)."""
    allowed_tools: List[str]
    """The tools the skill is permitted to use, as declared in its SKILL.md."""
    catalog_name: str
    """Name of parent catalog."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this agent skill was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of agent skill creator."""
    description: Optional[str]
    """
    A human-readable description of what the skill does and when to use it. Mirrors the Agent
    Skills spec `description` frontmatter field.
    """
    full_name: str
    """The three-level (fully qualified) name of the agent skill."""
    license: Optional[str]
    """SPDX license identifier or free-form license text for the skill."""
    metadata: Dict[str, str]
    """Arbitrary additional metadata declared by the skill."""
    metastore_id: Optional[str]
    """The unique identifier of the metastore."""
    name: str
    """
    Name of the agent skill, relative to parent schema. Per the Agent Skills spec a skill
    name is lowercase letters, numbers, and hyphens, but Unity Catalog object names disallow
    hyphens, so UC stores the catalog object name with underscores; the spec- shaped name
    lives in the skill's own SKILL.md.
    """
    owner: Optional[str]
    """Username of current owner of the agent skill."""
    schema_name: str
    """Name of parent schema."""
    storage_location: str
    """The storage location of the skill's directory (the root that holds `SKILL.md`)."""
    updated_at: Optional[int]
    """Time at which this agent skill was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified the agent skill."""

    def __init__(
        self,
        agent_skill_id: Optional[str] = None,
        agent_skill_type: Optional[AgentSkillType] = None,
        allowed_tools: Optional[List[str]] = None,
        catalog_name: Optional[str] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        description: Optional[str] = None,
        full_name: Optional[str] = None,
        license: Optional[str] = None,
        metadata: Optional[Dict[str, str]] = None,
        metastore_id: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        schema_name: Optional[str] = None,
        storage_location: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class AwsIamRoleConfig:
    """The AWS IAM role configuration used server-side to call STS AssumeRole.

    This is an internal configuration type, not exposed in the public API response. It stores the static
    credentials (or ambient credential fallback) used to authorize the STS:AssumeRole call when vending
    temporary credentials."""

    access_key_id: Optional[str]
    """
    AWS access key ID used to authorize the STS AssumeRole call. When set together with
    secret_access_key, these static credentials are used as the base identity to call
    STS:AssumeRole. When omitted, the server falls back to ambient credentials (instance
    profile, ECS task role, WebIdentity, etc.).
    """
    region: Optional[str]
    """The AWS region where STS calls are made. Defaults to "us-east-1" if absent."""
    role_arn: str
    """The Amazon Resource Name (ARN) of the AWS IAM role used to vend temporary credentials."""
    secret_access_key: Optional[str]
    """AWS secret access key paired with access_key_id."""
    session_token: Optional[str]
    """
    Optional AWS session token for temporary base credentials (e.g. when the caller itself
    holds STS- vended credentials as the base identity).
    """

    def __init__(
        self,
        access_key_id: Optional[str] = None,
        region: Optional[str] = None,
        role_arn: Optional[str] = None,
        secret_access_key: Optional[str] = None,
        session_token: Optional[str] = None,
    ) -> None: ...

class AwsTemporaryCredentials:
    access_key_id: str
    """The access key ID that identifies the temporary credentials."""
    access_point: str
    """
    The Amazon Resource Name (ARN) of the S3 access point for temporary credentials related
    the external location.
    """
    secret_access_key: str
    """The secret access key that can be used to sign AWS API requests."""
    session_token: str
    """The token that users must pass to AWS API to use the temporary credentials."""

    def __init__(
        self,
        access_key_id: Optional[str] = None,
        access_point: Optional[str] = None,
        secret_access_key: Optional[str] = None,
        session_token: Optional[str] = None,
    ) -> None: ...

class AzureAad:
    aad_token: str
    """
    Opaque token that contains claims that you can use in Azure Active Directory to access
    cloud services.
    """

    def __init__(self, aad_token: Optional[str] = None) -> None: ...

class AzureManagedIdentity:
    """The Azure managed identity configuration."""

    access_connector_id: str
    """
    The Azure resource ID of the Azure Databricks Access Connector. Use the format /
    subscriptions/ {guid}/resourceGroups/{rg-name}/providers/ Microsoft.Databricks/
    accessConnectors/{connector-name}.
    """
    credential_id: Optional[str]
    """The Databricks internal ID that represents this managed identity."""
    managed_identity_id: Optional[str]
    """
    The Azure resource ID of the managed identity. Use the format /subscriptions/{guid}/
    resourceGroups/ {rg-name}/providers/ Microsoft.ManagedIdentity/userAssignedIdentities/
    {identity-name}. This is only available for user-assigned identities. For system-assigned
    identities, the access_connector_id is used to identify the identity. If this field is not
    provided, then the system-assigned identity of the Access Connector is used.
    """

    def __init__(
        self,
        access_connector_id: Optional[str] = None,
        credential_id: Optional[str] = None,
        managed_identity_id: Optional[str] = None,
    ) -> None: ...

class AzureServicePrincipal:
    """The Azure service principal configuration."""

    application_id: str
    """The application ID of the application registration within the referenced AAD tenant."""
    directory_id: str
    """
    The directory ID corresponding to the Azure Active Directory (AAD) tenant of the
    application.
    """
    client_secret: Optional[str]
    """The client secret generated for the above app ID in AAD."""
    federated_token_file: Optional[str]
    """Location of the file containing a federated token.

Specifically useful for workload identity federation."""

    def __init__(
        self,
        application_id: Optional[str] = None,
        directory_id: Optional[str] = None,
        client_secret: Optional[str] = None,
        federated_token_file: Optional[str] = None,
    ) -> None: ...

class AzureStorageKey:
    """The Azure storage key configuration."""

    account_key: str
    """The account key of the storage account."""
    account_name: str
    """The name of the storage account."""

    def __init__(
        self, account_key: Optional[str] = None, account_name: Optional[str] = None
    ) -> None: ...

class AzureUserDelegationSas:
    sas_token: str
    """The signed URI (SAS Token) used to access blob services for a given path"""

    def __init__(self, sas_token: Optional[str] = None) -> None: ...

class Catalog:
    """A catalog is a root-level namespace that contains schemas."""

    browse_only: Optional[bool]
    """
    Indicates whether the principal is limited to retrieving metadata for the associated
    object through the BROWSE privilege when include_browse is enabled in the request.
    """
    catalog_type: Optional[CatalogType]
    """The type of the catalog."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this catalog was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of catalog creator."""
    id: Optional[str]
    """Unique identifier for the catalog."""
    name: str
    """Name of catalog."""
    owner: Optional[str]
    """Username of current owner of catalog."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    provider_name: Optional[str]
    """
    The name of delta sharing provider. A Delta Sharing catalog is a catalog that is based on
    a Delta share on a remote sharing server.
    """
    share_name: Optional[str]
    """The name of the share under the share provider."""
    storage_location: Optional[str]
    """
    Storage location URL (full path) for managed storage of the catalog. A unique path under
    `storage_root`, used as the managed-storage parent for managed tables/volumes whose
    schema has no storage location of its own. Absent when the catalog has no `storage_root`.
    Example: `s3://bucket/ucroot/__unitystorage/catalogs/{catalog_id}`.
    """
    storage_root: Optional[str]
    """Storage root URL for managed tables within catalog."""
    updated_at: Optional[int]
    """Time at which this catalog was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified catalog."""

    def __init__(
        self,
        browse_only: Optional[bool] = None,
        catalog_type: Optional[CatalogType] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        id: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        provider_name: Optional[str] = None,
        share_name: Optional[str] = None,
        storage_location: Optional[str] = None,
        storage_root: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class Column:
    column_id: Optional[str]
    """a unique id for the column"""
    comment: Optional[str]
    """User-provided free-form text description."""
    name: str
    """Name of the column"""
    nullable: Optional[bool]
    """Whether field may be Null."""
    partition_index: Optional[int]
    """Partition index for column."""
    position: Optional[int]
    """Ordinal position of column (starting at position 0)."""
    type_interval_type: Optional[str]
    """Format of IntervalType."""
    type_json: str
    """Full data type specification, JSON-serialized."""
    type_name: ColumnTypeName
    """Data type name."""
    type_precision: Optional[int]
    """Digits of precision; required for DecimalTypes."""
    type_scale: Optional[int]
    """Digits to right of decimal; Required for DecimalTypes."""
    type_text: str
    """Full data type specification as SQL/catalogString text."""

    def __init__(
        self,
        column_id: Optional[str] = None,
        comment: Optional[str] = None,
        name: Optional[str] = None,
        nullable: Optional[bool] = None,
        partition_index: Optional[int] = None,
        position: Optional[int] = None,
        type_interval_type: Optional[str] = None,
        type_json: Optional[str] = None,
        type_name: Optional[ColumnTypeName] = None,
        type_precision: Optional[int] = None,
        type_scale: Optional[int] = None,
        type_text: Optional[str] = None,
    ) -> None: ...

class CommitInfo:
    """Information about a single Delta commit ratified by the catalog.

    Mirrors the `CommitInfo` shape used by the Unity Catalog OSS commit coordinator (`uc_delta_commits`
    rows). All fields are required and must be positive / non-empty (see the `commit` RPC)."""

    file_modification_timestamp: int
    """
    The filesystem modification timestamp of the staged commit file (milliseconds since
    epoch).
    """
    file_name: str
    """
    The name of the staged commit file, e.g. `00000000000000000005.<uuid>.json` under
    `_delta_log/ _staged_commits/`.
    """
    file_size: int
    """The size of the staged commit file in bytes."""
    timestamp: int
    """
    The in-commit timestamp (milliseconds since epoch). Monotonicity is the client's
    responsibility; the server only requires it to be positive.
    """
    version: int
    """
    The table version this commit produces. Posted commits are >= 1; version 0 is established
    out-of- band by create-table.
    """

    def __init__(
        self,
        file_modification_timestamp: Optional[int] = None,
        file_name: Optional[str] = None,
        file_size: Optional[int] = None,
        timestamp: Optional[int] = None,
        version: Optional[int] = None,
    ) -> None: ...

class Credential:
    """A credential used to access external data sources or services."""

    aws_iam_role: Optional[AwsIamRoleConfig]
    """The AWS IAM role configuration."""
    azure_managed_identity: Optional[AzureManagedIdentity]
    """The Azure managed identity configuration."""
    azure_service_principal: Optional[AzureServicePrincipal]
    """The Azure service principal configuration."""
    azure_storage_key: Optional[AzureStorageKey]
    """The Azure storage key configuration."""
    comment: Optional[str]
    """Comment associated with the credential."""
    created_at: Optional[int]
    """Time at which this credential was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of credential creator."""
    databricks_gcp_service_account: Optional[DatabricksGcpServiceAccount]
    """The Databricks managed GCP service account configuration."""
    full_name: Optional[str]
    """The full name of the credential."""
    id: Optional[str]
    """The unique identifier of the credential."""
    name: str
    """
    The credential name. The name must be unique among storage and service credentials within
    the metastore.
    """
    owner: Optional[str]
    """Username of current owner of credential."""
    purpose: Purpose
    """Indicates the purpose of the credential."""
    read_only: bool
    """
    Whether the credential is usable only for read operations. Only applicable when purpose
    is STORAGE.
    """
    updated_at: Optional[int]
    """Time at which this credential was last modified, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified the credential."""
    used_for_managed_storage: bool
    """
    Whether this credential is the current metastore's root storage credential. Only
    applicable when purpose is STORAGE.
    """

    def __init__(
        self,
        aws_iam_role: Optional[AwsIamRoleConfig] = None,
        azure_managed_identity: Optional[AzureManagedIdentity] = None,
        azure_service_principal: Optional[AzureServicePrincipal] = None,
        azure_storage_key: Optional[AzureStorageKey] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        databricks_gcp_service_account: Optional[DatabricksGcpServiceAccount] = None,
        full_name: Optional[str] = None,
        id: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        purpose: Optional[Purpose] = None,
        read_only: Optional[bool] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
        used_for_managed_storage: Optional[bool] = None,
    ) -> None: ...

class DataObject:
    added_at: Optional[int]
    """The time when this data object is added to the share, in epoch milliseconds."""
    added_by: Optional[str]
    """Username of the sharer."""
    comment: Optional[str]
    """A user-provided comment when adding the data object to the share."""
    data_object_type: DataObjectType
    """Type of the data object."""
    enable_cdf: Optional[bool]
    """Whether to enable cdf or indicate if cdf is enabled on the shared object."""
    history_data_sharing_status: Optional[HistoryStatus]
    """
    Whether to enable or disable sharing of data history. If not specified, the default is
    DISABLED.
    """
    name: str
    """
    A fully qualified name that uniquely identifies a data object. For example, a table's
    fully qualified name is in the format of <catalog>.<schema>.<table>,
    """
    partitions: List[str]
    """Array of partitions for the shared data."""
    shared_as: Optional[str]
    """
    A user-provided new name for the data object within the share. If this new name is not
    provided, the object's original name will be used as the shared_as name. The shared_as
    name must be unique within a share. For tables, the new name must follow the format of
    <schema>.<table>.
    """
    start_version: Optional[int]
    """
    The start version associated with the object. This allows data providers to control the
    lowest object version that is accessible by clients. If specified, clients can query
    snapshots or changes for versions >= start_version. If not specified, clients can only
    query starting from the version of the object at the time it was added to the share. NOTE:
    The start_version should be <= the current version of the object.
    """

    def __init__(
        self,
        added_at: Optional[int] = None,
        added_by: Optional[str] = None,
        comment: Optional[str] = None,
        data_object_type: Optional[DataObjectType] = None,
        enable_cdf: Optional[bool] = None,
        history_data_sharing_status: Optional[HistoryStatus] = None,
        name: Optional[str] = None,
        partitions: Optional[List[str]] = None,
        shared_as: Optional[str] = None,
        start_version: Optional[int] = None,
    ) -> None: ...

class DataObjectUpdate:
    """Data object update."""

    action: Action
    """Name of the share."""
    data_object: Optional[DataObject]
    """User-provided free-form text description."""

    def __init__(
        self, action: Optional[Action] = None, data_object: Optional[DataObject] = None
    ) -> None: ...

class DatabricksGcpServiceAccount:
    """The Databricks managed GCP service account configuration."""

    credential_id: Optional[str]
    """The Databricks internal ID that represents this managed identity."""
    email: Optional[str]
    """The email of the service account."""
    private_key_id: Optional[str]
    """The ID that represents the private key for this Service Account."""

    def __init__(
        self,
        credential_id: Optional[str] = None,
        email: Optional[str] = None,
        private_key_id: Optional[str] = None,
    ) -> None: ...

class Dependency:
    """A dependency of a SQL object. Exactly one of the fields must be set."""

    function: Optional[FunctionDependency]
    table: Optional[TableDependency]

    def __init__(
        self, function: Optional[FunctionDependency] = None, table: Optional[TableDependency] = None
    ) -> None: ...

class DependencyList:
    """A list of dependencies referenced by a view-like table."""

    dependencies: List[Dependency]
    """Array of dependencies."""

    def __init__(self, dependencies: Optional[List[Dependency]] = None) -> None: ...

class EntityTagAssignment:
    """The assignment of a tag to a Unity Catalog entity.

    Unlike a TagPolicy (a governed-tag *definition*), an assignment is the application of a tag to a
    specific securable. It has no identifier of its own — its identity is the composite of (entity_type,
    entity_name, tag_key). It is intentionally NOT a `google.api.resource`: assignments are stored as
    associations between the entity and its tag, not as standalone objects."""

    entity_name: str
    """The fully qualified name of the entity to which the tag is assigned."""
    entity_type: str
    """
    The type of the entity to which the tag is assigned. Supported values: catalogs, schemas,
    tables, columns, volumes.
    """
    tag_key: str
    """The key of the tag."""
    tag_value: Optional[str]
    """The value of the tag."""

    def __init__(
        self,
        entity_name: Optional[str] = None,
        entity_type: Optional[str] = None,
        tag_key: Optional[str] = None,
        tag_value: Optional[str] = None,
    ) -> None: ...

class ExternalLocation:
    browse_only: Optional[bool]
    """
    Indicates whether the principal is limited to retrieving metadata for the associated
    object through the BROWSE privilege when include_browse is enabled in the request.
    """
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this catalog was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of catalog creator."""
    credential_id: str
    """Unique ID of the location's storage credential."""
    credential_name: str
    """Name of the storage credential used with this location."""
    external_location_id: Optional[str]
    name: str
    """Name of the external location."""
    owner: Optional[str]
    """The owner of the external location."""
    read_only: bool
    """Indicates whether the external location is read-only."""
    updated_at: Optional[int]
    """Time at which this catalog was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified catalog."""
    url: str
    """Path URL of the external location."""

    def __init__(
        self,
        browse_only: Optional[bool] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        credential_id: Optional[str] = None,
        credential_name: Optional[str] = None,
        external_location_id: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        read_only: Optional[bool] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
        url: Optional[str] = None,
    ) -> None: ...

class Function:
    """A User-Defined Function (UDF) registered under a catalog + schema hierarchy."""

    catalog_name: str
    """Name of parent catalog."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this function was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of function creator."""
    data_type: str
    """Full data type specification of the return type of the function."""
    full_data_type: str
    """Full data type specification as SQL/catalogString text."""
    full_name: str
    """
    The three-level (fully qualified) name of the function. Format:
    catalog_name.schema_name.function_name
    """
    function_id: Optional[str]
    """Unique identifier for the function."""
    input_params: Optional[FunctionParameterInfos]
    """The array of function parameter infos."""
    is_deterministic: bool
    """Indicates whether the function is deterministic."""
    is_null_call: bool
    """Indicates whether the function is null-calling."""
    name: str
    """Name of function, relative to parent schema."""
    owner: Optional[str]
    """Username of current owner of the function."""
    parameter_style: ParameterStyle
    """The parameter-passing style."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    return_params: Optional[str]
    """The return type of the function in JSON format."""
    routine_body: RoutineBody
    """The routine body."""
    routine_body_language: Optional[str]
    """The language of the function routine body."""
    routine_definition: Optional[str]
    """Function body."""
    routine_dependencies: Optional[str]
    """Function dependencies (in JSON form)."""
    schema_name: str
    """Name of parent schema."""
    security_type: SecurityType
    """The security type of the function."""
    specific_name: Optional[str]
    """The type of the function (SCALAR or TABLE)."""
    sql_data_access: SqlDataAccess
    """SQL data access information."""
    updated_at: Optional[int]
    """Time at which this function was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified the function."""

    def __init__(
        self,
        catalog_name: Optional[str] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        data_type: Optional[str] = None,
        full_data_type: Optional[str] = None,
        full_name: Optional[str] = None,
        function_id: Optional[str] = None,
        input_params: Optional[FunctionParameterInfos] = None,
        is_deterministic: Optional[bool] = None,
        is_null_call: Optional[bool] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        parameter_style: Optional[ParameterStyle] = None,
        properties: Optional[Dict[str, str]] = None,
        return_params: Optional[str] = None,
        routine_body: Optional[RoutineBody] = None,
        routine_body_language: Optional[str] = None,
        routine_definition: Optional[str] = None,
        routine_dependencies: Optional[str] = None,
        schema_name: Optional[str] = None,
        security_type: Optional[SecurityType] = None,
        specific_name: Optional[str] = None,
        sql_data_access: Optional[SqlDataAccess] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class FunctionArg:
    """An argument passed to a row-filter or column-mask function."""

    alias: Optional[str]
    """References a column via its `MatchColumn.alias`."""
    constant: Optional[str]
    """A literal constant value, passed as-is to the function."""

    def __init__(self, alias: Optional[str] = None, constant: Optional[str] = None) -> None: ...

class FunctionDependency:
    """A function that a SQL object (such as a view or metric view) depends on."""

    function_full_name: str
    """
    Full name of the dependent function, in the form of catalog_name.schema_name.function_name.
    """

    def __init__(self, function_full_name: Optional[str] = None) -> None: ...

class FunctionParameterInfo:
    """Information about a single function parameter."""

    comment: Optional[str]
    """User-provided free-form text description."""
    name: str
    """Name of parameter."""
    parameter_default: Optional[str]
    """Default value of the parameter."""
    parameter_mode: ParameterMode
    """The mode of the function parameter."""
    parameter_type: FunctionParameterType
    """The type of function parameter."""
    position: Optional[int]
    """Ordinal position of column (starting at position 0)."""
    type_interval_type: Optional[str]
    """Format of IntervalType."""
    type_json: Optional[str]
    """Full data type specification, JSON-serialized."""
    type_name: ColumnTypeName
    """Data type name."""
    type_precision: Optional[int]
    """Digits of precision; required for DecimalTypes."""
    type_scale: Optional[int]
    """Digits to right of decimal; required for DecimalTypes."""
    type_text: str
    """Full data type specification as SQL/catalogString text."""

    def __init__(
        self,
        comment: Optional[str] = None,
        name: Optional[str] = None,
        parameter_default: Optional[str] = None,
        parameter_mode: Optional[ParameterMode] = None,
        parameter_type: Optional[FunctionParameterType] = None,
        position: Optional[int] = None,
        type_interval_type: Optional[str] = None,
        type_json: Optional[str] = None,
        type_name: Optional[ColumnTypeName] = None,
        type_precision: Optional[int] = None,
        type_scale: Optional[int] = None,
        type_text: Optional[str] = None,
    ) -> None: ...

class FunctionParameterInfos:
    """A collection of function parameters."""

    parameters: List[FunctionParameterInfo]
    """The parameters of the function."""

    def __init__(self, parameters: Optional[List[FunctionParameterInfo]] = None) -> None: ...

class FunctionRef:
    """A reference to the SQL function that implements a row-filter or column-mask policy, together with
    the arguments it is invoked with."""

    function_name: str
    """Fully qualified name of the function (catalog.schema.function) to invoke."""
    using: List[FunctionArg]
    """The arguments passed to the function, in declaration order."""

    def __init__(
        self, function_name: Optional[str] = None, using: Optional[List[FunctionArg]] = None
    ) -> None: ...

class GcpOauthToken:
    oauth_token: str
    """The OAuth token used to access Google Cloud services."""

    def __init__(self, oauth_token: Optional[str] = None) -> None: ...

class GetCommitsResponse:
    """Response listing ratified-but-unpublished commits for a table."""

    commits: List[CommitInfo]
    """
    Ratified commits in `[start_version, end_version]`, contiguous and ordered ascending by
    version. Excludes the internal fully-backfilled marker row.
    """
    latest_table_version: int
    """
    The latest table version the catalog tracks. `0` indicates a managed table with no commits
    yet.
    """

    def __init__(
        self, commits: Optional[List[CommitInfo]] = None, latest_table_version: Optional[int] = None
    ) -> None: ...

class GetPermissionsResponse:
    """Response to list shares."""

    next_page_token: Optional[str]
    """Opaque pagination token to go to next page based on previous query."""
    privilege_assignments: List[PrivilegeAssignment]
    """The privileges assigned to each principal"""

    def __init__(
        self,
        next_page_token: Optional[str] = None,
        privilege_assignments: Optional[List[PrivilegeAssignment]] = None,
    ) -> None: ...

class GetTableExistsResponse:
    table_exists: bool
    """Boolean reflecting if table exists."""

    def __init__(self, table_exists: Optional[bool] = None) -> None: ...

class ListEntityTagAssignmentsResponse:
    """List entity tag assignments response."""

    next_page_token: Optional[str]
    """The next_page_token value to include in the next List request."""
    tag_assignments: List[EntityTagAssignment]
    """The tag assignments returned."""

    def __init__(
        self,
        next_page_token: Optional[str] = None,
        tag_assignments: Optional[List[EntityTagAssignment]] = None,
    ) -> None: ...

class ListPoliciesResponse:
    """List policies response."""

    next_page_token: Optional[str]
    """The next_page_token value to include in the next List request."""
    policies: List[PolicyInfo]
    """The policies returned."""

    def __init__(
        self, next_page_token: Optional[str] = None, policies: Optional[List[PolicyInfo]] = None
    ) -> None: ...

class ListTableSummariesResponse:
    next_page_token: Optional[str]
    """The next_page_token value to include in the next List request."""
    tables: List[TableSummary]
    """The table summaries returned."""

    def __init__(
        self, next_page_token: Optional[str] = None, tables: Optional[List[TableSummary]] = None
    ) -> None: ...

class MatchColumn:
    """A column referenced by a policy, together with the alias it is bound to in the row-filter or column-
    mask function invocation."""

    alias: str
    """
    The name of the alias the column is bound to when invoking the row-filter or column-mask
    function.
    """
    condition: str
    """The condition on the column, e.g. its name, that must match for this binding to apply."""

    def __init__(self, alias: Optional[str] = None, condition: Optional[str] = None) -> None: ...

class Metadata:
    """A Delta metadata change accompanying a commit. Modeled minimally; the coordinator stores it opaquely
    and does not interpret it."""

    configuration: Dict[str, str]
    """Configuration key/value pairs from the Delta metadata action."""
    id: Optional[str]
    """The Delta metadata `id`."""
    schema_string: Optional[str]
    """The serialized schema string from the Delta metadata action."""

    def __init__(
        self,
        configuration: Optional[Dict[str, str]] = None,
        id: Optional[str] = None,
        schema_string: Optional[str] = None,
    ) -> None: ...

class PermissionsChange:
    add: List[str]
    """The set of privileges to add."""
    principal: str
    """The principal (user email address or group name)."""
    remove: List[str]
    """The set of privileges to remove."""

    def __init__(
        self,
        add: Optional[List[str]] = None,
        principal: Optional[str] = None,
        remove: Optional[List[str]] = None,
    ) -> None: ...

class PolicyInfo:
    """A row-filter or column-mask policy bound to principals and scoped to a securable.

    Policies are read as whole documents; enforcement is performed by the query engine, not by this
    service. `when_condition` is stored and returned opaque — this service does not parse or evaluate
    it."""

    comment: Optional[str]
    """User-provided free-form text description of the policy."""
    created_at: Optional[int]
    """Time at which this policy was created, in epoch milliseconds."""
    except_principals: List[str]
    """The principals explicitly excluded from the policy, even if matched by to_principals."""
    id: Optional[str]
    """Server-assigned unique identifier for the policy."""
    match_columns: List[MatchColumn]
    """
    The columns referenced by the row-filter or column-mask function, and the aliases they are
    bound to.
    """
    name: str
    """The name of the policy. Unique within the securable it is defined on."""
    on_securable_fullname: str
    """The fully qualified name of the securable the policy is defined on."""
    on_securable_type: str
    """
    The type of securable the policy is defined on. Supported values: catalogs, schemas,
    tables.
    """
    policy_type: PolicyType
    """The kind of enforcement this policy applies."""
    to_principals: List[str]
    """The principals the policy applies to. Empty means "all principals"."""
    updated_at: Optional[int]
    """Time at which this policy was last updated, in epoch milliseconds."""
    when_condition: Optional[str]
    """
    An opaque condition expression, e.g. "hasTagValue('env','prod')", evaluated by the query
    engine at enforcement time. Stored and returned as-is; not parsed server-side.
    """
    column_mask: Optional[FunctionRef]
    """The column-mask function to invoke. Set only when policy_type is POLICY_TYPE_COLUMN_MASK."""
    row_filter: Optional[FunctionRef]
    """The row-filter function to invoke. Set only when policy_type is POLICY_TYPE_ROW_FILTER."""

    def __init__(
        self,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        except_principals: Optional[List[str]] = None,
        id: Optional[str] = None,
        match_columns: Optional[List[MatchColumn]] = None,
        name: Optional[str] = None,
        on_securable_fullname: Optional[str] = None,
        on_securable_type: Optional[str] = None,
        policy_type: Optional[PolicyType] = None,
        to_principals: Optional[List[str]] = None,
        updated_at: Optional[int] = None,
        when_condition: Optional[str] = None,
        column_mask: Optional[FunctionRef] = None,
        row_filter: Optional[FunctionRef] = None,
    ) -> None: ...

class PrivilegeAssignment:
    principal: str
    """
    The principal (user email address or group name). For deleted principals, principal is
    empty while principal_id is populated.
    """
    privileges: List[str]
    """The privileges assigned to the principal."""

    def __init__(
        self, principal: Optional[str] = None, privileges: Optional[List[str]] = None
    ) -> None: ...

class Provider:
    """A provider represents an organization sharing data with this metastore.

    A provider is the inbound counterpart of a recipient: it is registered from a share activation/
    credential file and is used to access shares offered by an upstream Delta Sharing server."""

    authentication_type: ProviderAuthenticationType
    """The delta sharing authentication type."""
    comment: Optional[str]
    """Description about the provider."""
    created_at: Optional[int]
    """Time at which this provider was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of the creator of the provider."""
    id: Optional[str]
    """Unique ID of the provider."""
    name: str
    """The name of the provider."""
    owner: Optional[str]
    """Username of the provider owner."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the provider."""
    recipient_profile_str: Optional[str]
    """
    The recipient profile (credential file contents) used to connect to the sharing server,
    present only for TOKEN authentication.
    """
    updated_at: Optional[int]
    """Time at which this provider was updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of the provider updater."""

    def __init__(
        self,
        authentication_type: Optional[ProviderAuthenticationType] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        id: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        recipient_profile_str: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class R2TemporaryCredentials:
    access_key_id: str
    """The access key ID that identifies the temporary credentials."""
    secret_access_key: str
    """The secret access key associated with the access key."""
    session_token: str
    """The generated JWT that users must pass to use the temporary credentials."""

    def __init__(
        self,
        access_key_id: Optional[str] = None,
        secret_access_key: Optional[str] = None,
        session_token: Optional[str] = None,
    ) -> None: ...

class Recipient:
    authentication_type: AuthenticationType
    """The delta sharing authentication type."""
    comment: Optional[str]
    """Description about the recipient."""
    created_at: Optional[int]
    """Time at which this share was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of the creator of the share."""
    id: Optional[str]
    """Unique ID of the recipient."""
    name: str
    """The name of the recipient."""
    owner: Optional[str]
    """Username of the recipient owner."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    tokens: List[RecipientToken]
    """This field is only present when the authentication_type is TOKEN."""
    updated_at: Optional[int]
    """Time at which this share was updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of share updater."""

    def __init__(
        self,
        authentication_type: Optional[AuthenticationType] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        id: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        tokens: Optional[List[RecipientToken]] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class RecipientToken:
    activation_url: str
    """
    Full activation URL to retrieve the access token. It will be empty if the token is already
    retrieved.
    """
    created_at: int
    """Time at which this recipient token was created, in epoch milliseconds."""
    created_by: str
    """Username of recipient token creator."""
    expiration_time: int
    """Expiration timestamp of the token in epoch milliseconds."""
    id: str
    """Unique ID of the recipient token."""
    updated_at: int
    """Time at which this recipient token was updated, in epoch milliseconds."""
    updated_by: str
    """Username of recipient token updater."""

    def __init__(
        self,
        activation_url: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        expiration_time: Optional[int] = None,
        id: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class Schema:
    """A schema is a namespace within a catalog that contains tables."""

    catalog_name: str
    """Name of parent catalog."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this schema was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of schema creator."""
    full_name: str
    """Full name of schema, in form of catalog_name.schema_name."""
    name: str
    """Name of schema, relative to parent catalog."""
    owner: Optional[str]
    """Username of current owner of schema."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    schema_id: Optional[str]
    """Unique identifier for the schema."""
    storage_location: Optional[str]
    """
    Storage location URL (full path) for managed storage of the schema. A unique path under
    `storage_root`. Absent when the schema has no `storage_root`, in which case managed
    securables fall back to the parent catalog's storage location. Example: `s3://bucket/
    ucroot/ __unitystorage/schemas/{schema_id}`.
    """
    storage_root: Optional[str]
    """
    Storage root URL for managed storage location of the schema. Can be set when creating a
    schema. When set, managed tables/volumes created in this schema are rooted here; otherwise
    the parent catalog's storage location is used. Example: `s3://bucket/ucroot`.
    """
    updated_at: Optional[int]
    """Time at which this schema was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified schema."""

    def __init__(
        self,
        catalog_name: Optional[str] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        full_name: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        schema_id: Optional[str] = None,
        storage_location: Optional[str] = None,
        storage_root: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class Share:
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this share was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of the creator of the share."""
    id: Optional[str]
    """Unique ID of the recipient."""
    name: str
    """Name of the share."""
    objects: List[DataObject]
    """A list of shared data objects within the share."""
    owner: Optional[str]
    """Username of current owner of share."""
    storage_location: Optional[str]
    """Storage Location URL (full path) for the share."""
    storage_root: Optional[str]
    """Storage root URL for the share."""
    updated_at: Optional[int]
    """Time at which this share was updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of share updater."""

    def __init__(
        self,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        id: Optional[str] = None,
        name: Optional[str] = None,
        objects: Optional[List[DataObject]] = None,
        owner: Optional[str] = None,
        storage_location: Optional[str] = None,
        storage_root: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
    ) -> None: ...

class StagingTable:
    """A staging table is a transient reservation created while provisioning a Unity Catalog managed table.

    On creation the server allocates an immutable table id and storage location under the parent
    schema/catalog's managed storage root. The client then writes the initial Delta commit
    (`_delta_log/00...0.json`) at that location and finalizes the table via the CreateTable API, which
    commits this staging row. A staging table is not yet a usable table and is not returned by the table
    APIs."""

    catalog_name: str
    """Name of the parent catalog."""
    created_at: Optional[int]
    """Time at which this staging table was created, in epoch milliseconds."""
    created_by: Optional[str]
    """
    Username of the principal that created the staging table. Only this principal may finalize
    the table.
    """
    id: str
    """
    Unique identifier generated for the staging table. Becomes the managed table's id once
    finalized.
    """
    name: str
    """Name of the staging table, relative to the parent schema."""
    schema_name: str
    """Name of the parent schema relative to its parent catalog."""
    stage_committed: bool
    """Whether the staging table has been finalized into a managed table."""
    staging_location: str
    """
    Storage location allocated for the staging table, where the client writes the initial
    Delta commit. Immutable once allocated.
    """

    def __init__(
        self,
        catalog_name: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        id: Optional[str] = None,
        name: Optional[str] = None,
        schema_name: Optional[str] = None,
        stage_committed: Optional[bool] = None,
        staging_location: Optional[str] = None,
    ) -> None: ...

class Table:
    catalog_name: str
    """Name of parent catalog."""
    columns: List[Column]
    """The array of Column definitions of the table's columns."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this table was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of table creator."""
    data_source_format: DataSourceFormat
    """Data source format of the table."""
    deleted_at: Optional[int]
    """
    Time at which this table was deleted, in epoch milliseconds. Field is omitted if table is
    not deleted.
    """
    full_name: str
    """Full name of table, in form of catalog_name.schema_name.table_name."""
    name: str
    """Name of table, relative to parent schema."""
    owner: Optional[str]
    """Username of current owner of table."""
    properties: Dict[str, str]
    """A map of key-value properties attached to the securable."""
    schema_name: str
    """Name of parent schema."""
    storage_credential_name: Optional[str]
    """
    Name of the storage credential, when a storage credential is configured for use with this
    table.
    """
    storage_location: Optional[str]
    """Storage root URL for table (for MANAGED, EXTERNAL tables)"""
    table_id: Optional[str]
    """Unique identifier for the table."""
    table_type: TableType
    updated_at: Optional[int]
    """Time at which this table was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified table."""
    view_definition: Optional[str]
    """
    Definition text for view-like table types (VIEW, MATERIALIZED_VIEW, STREAMING_TABLE,
    METRIC_VIEW). The format depends on the table type: SQL for views, YAML for metric views.
    """
    view_dependencies: Optional[DependencyList]
    """
    Tables and functions the view-like table reads. For metric views this is derived from the
    view_definition by the server (the definition is the single source of truth).
    """

    def __init__(
        self,
        catalog_name: Optional[str] = None,
        columns: Optional[List[Column]] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        data_source_format: Optional[DataSourceFormat] = None,
        deleted_at: Optional[int] = None,
        full_name: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        schema_name: Optional[str] = None,
        storage_credential_name: Optional[str] = None,
        storage_location: Optional[str] = None,
        table_id: Optional[str] = None,
        table_type: Optional[TableType] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
        view_definition: Optional[str] = None,
        view_dependencies: Optional[DependencyList] = None,
    ) -> None: ...

class TableDependency:
    """A table that a SQL object (such as a view or metric view) depends on."""

    table_full_name: str
    """Full name of the dependent table, in the form of catalog_name.schema_name.table_name."""

    def __init__(self, table_full_name: Optional[str] = None) -> None: ...

class TableSummary:
    full_name: str
    """The full name of the table."""
    table_type: TableType

    def __init__(
        self, full_name: Optional[str] = None, table_type: Optional[TableType] = None
    ) -> None: ...

class TagPolicy:
    """A governed tag definition (tag policy).

    A tag policy defines a tag key together with the rules that govern how it can be used, including the
    optional set of allowed values. Assigning a governed tag to an entity is done through the Entity Tag
    Assignments API."""

    created_at: Optional[int]
    """Time at which this tag policy was created, in epoch milliseconds."""
    description: Optional[str]
    """User-provided free-form text description of the tag policy."""
    id: Optional[str]
    """Unique identifier for the tag policy."""
    tag_key: str
    """The key of the governed tag."""
    updated_at: Optional[int]
    """Time at which this tag policy was last updated, in epoch milliseconds."""
    values: List[Value]
    """
    The set of allowed values for the governed tag. When empty, the governed tag does not
    restrict the values that may be assigned.
    """

    def __init__(
        self,
        created_at: Optional[int] = None,
        description: Optional[str] = None,
        id: Optional[str] = None,
        tag_key: Optional[str] = None,
        updated_at: Optional[int] = None,
        values: Optional[List[Value]] = None,
    ) -> None: ...

class TemporaryCredential:
    """The response to the GenerateTemporaryTableCredentialsRequest."""

    expiration_time: int
    """
    Server time when the credential will expire, in epoch milliseconds. The API client is
    advised to cache the credential given this expiration time.
    """
    url: str
    """The URL of the storage path accessible by the temporary credential."""
    aws_temp_credentials: Optional[AwsTemporaryCredentials]
    """Credentials for AWS S3."""
    azure_aad: Optional[AzureAad]
    """Credentials for Azure Active Directory."""
    azure_user_delegation_sas: Optional[AzureUserDelegationSas]
    """Credentials for Azure Blob Storage."""
    gcp_oauth_token: Optional[GcpOauthToken]
    """Credentials for Google Cloud Storage."""
    r2_temp_credentials: Optional[R2TemporaryCredentials]
    """Credentials for R2."""

    def __init__(
        self,
        expiration_time: Optional[int] = None,
        url: Optional[str] = None,
        aws_temp_credentials: Optional[AwsTemporaryCredentials] = None,
        azure_aad: Optional[AzureAad] = None,
        azure_user_delegation_sas: Optional[AzureUserDelegationSas] = None,
        gcp_oauth_token: Optional[GcpOauthToken] = None,
        r2_temp_credentials: Optional[R2TemporaryCredentials] = None,
    ) -> None: ...

class UpdatePermissionsResponse:
    privilege_assignments: List[PrivilegeAssignment]
    """The privileges assigned to each principal"""

    def __init__(
        self, privilege_assignments: Optional[List[PrivilegeAssignment]] = None
    ) -> None: ...

class Value:
    """An allowed value for a governed tag."""

    name: str
    """The name of the allowed value."""

    def __init__(self, name: Optional[str] = None) -> None: ...

class Volume:
    browse_only: Optional[bool]
    """
    Indicates whether the principal is limited to retrieving metadata for the associated
    object through the BROWSE privilege when include_browse is enabled in the request.
    """
    catalog_name: str
    """Name of parent catalog."""
    comment: Optional[str]
    """User-provided free-form text description."""
    created_at: Optional[int]
    """Time at which this catalog was created, in epoch milliseconds."""
    created_by: Optional[str]
    """Username of catalog creator."""
    full_name: str
    """The three-level (fully qualified) name of the volume"""
    metastore_id: Optional[str]
    """The unique identifier of the metastore"""
    name: str
    """Name of volume, relative to parent schema."""
    owner: Optional[str]
    """Username of current owner of table."""
    schema_name: str
    """Name of parent schema."""
    storage_location: str
    """The storage location on the cloud"""
    updated_at: Optional[int]
    """Time at which this catalog was last updated, in epoch milliseconds."""
    updated_by: Optional[str]
    """Username of user who last modified catalog."""
    volume_id: str
    """The unique identifier of the volume"""
    volume_type: VolumeType
    """
    The type of the volume. An external volume is located in the specified external location.
    A managed volume is located in the default location which is specified by the parent
    schema, or the parent catalog, or the Metastore.
    """

    def __init__(
        self,
        browse_only: Optional[bool] = None,
        catalog_name: Optional[str] = None,
        comment: Optional[str] = None,
        created_at: Optional[int] = None,
        created_by: Optional[str] = None,
        full_name: Optional[str] = None,
        metastore_id: Optional[str] = None,
        name: Optional[str] = None,
        owner: Optional[str] = None,
        schema_name: Optional[str] = None,
        storage_location: Optional[str] = None,
        updated_at: Optional[int] = None,
        updated_by: Optional[str] = None,
        volume_id: Optional[str] = None,
        volume_type: Optional[VolumeType] = None,
    ) -> None: ...

class Action(enum.Enum):
    ACTION_UNSPECIFIED = "ACTION_UNSPECIFIED"
    """Unspecified action."""
    ADD = "ADD"
    REMOVE = "REMOVE"
    UPDATE = "UPDATE"

class AgentSkillType(enum.Enum):
    """How an agent skill's storage location is provisioned."""

    AGENT_SKILL_TYPE_UNSPECIFIED = "AGENT_SKILL_TYPE_UNSPECIFIED"
    EXTERNAL = "EXTERNAL"
    """The skill directory lives at a caller-supplied location within a registered external location."""
    MANAGED = "MANAGED"
    """The skill directory lives at a server-derived location under the parent schema/catalog managed
    storage root."""

class AuthenticationType(enum.Enum):
    AUTHENTICATION_TYPE_UNSPECIFIED = "AUTHENTICATION_TYPE_UNSPECIFIED"
    """No authentication is required."""
    OAUTH_CLIENT_CREDENTIALS = "OAUTH_CLIENT_CREDENTIALS"
    """OAuth2 authentication is required."""
    TOKEN = "TOKEN"
    """Basic authentication is required."""

class CatalogType(enum.Enum):
    """The type of the catalog."""

    CATALOG_TYPE_UNSPECIFIED = "CATALOG_TYPE_UNSPECIFIED"
    """Unknown catalog type."""
    DELTASHARING_CATALOG = "DELTASHARING_CATALOG"
    MANAGED_CATALOG = "MANAGED_CATALOG"
    SYSTEM_CATALOG = "SYSTEM_CATALOG"

class ColumnTypeName(enum.Enum):
    ARRAY = "ARRAY"
    BINARY = "BINARY"
    BOOLEAN = "BOOLEAN"
    BYTE = "BYTE"
    CHAR = "CHAR"
    COLUMN_TYPE_NAME_UNSPECIFIED = "COLUMN_TYPE_NAME_UNSPECIFIED"
    DATE = "DATE"
    DECIMAL = "DECIMAL"
    DOUBLE = "DOUBLE"
    FLOAT = "FLOAT"
    INT = "INT"
    INTERVAL = "INTERVAL"
    LONG = "LONG"
    MAP = "MAP"
    NULL = "NULL"
    SHORT = "SHORT"
    STRING = "STRING"
    STRUCT = "STRUCT"
    TABLE_TYPE = "TABLE_TYPE"
    TIMESTAMP = "TIMESTAMP"
    TIMESTAMP_NTZ = "TIMESTAMP_NTZ"
    USER_DEFINED_TYPE = "USER_DEFINED_TYPE"
    VARIANT = "VARIANT"

class DataObjectType(enum.Enum):
    AGENT_SKILL = "AGENT_SKILL"
    """An agent skill (a directory of SKILL.md + scripts/assets) shared as a storage-backed asset on top of
    a volume (Open Sharing)."""
    DATA_OBJECT_TYPE_UNSPECIFIED = "DATA_OBJECT_TYPE_UNSPECIFIED"
    """Unknown data object type."""
    SCHEMA = "SCHEMA"
    TABLE = "TABLE"
    VOLUME = "VOLUME"
    """A Unity Catalog volume shared as a storage-backed asset (Open Sharing)."""

class DataSourceFormat(enum.Enum):
    AVRO = "AVRO"
    CSV = "CSV"
    DATA_SOURCE_FORMAT_UNSPECIFIED = "DATA_SOURCE_FORMAT_UNSPECIFIED"
    DELTA = "DELTA"
    DELTASHARING = "DELTASHARING"
    HUDI = "HUDI"
    ICEBERG = "ICEBERG"
    JSON = "JSON"
    ORC = "ORC"
    PARQUET = "PARQUET"
    TEXT = "TEXT"
    UNITY_CATALOG = "UNITY_CATALOG"

class FunctionParameterType(enum.Enum):
    """The type of the function parameter."""

    COLUMN = "COLUMN"
    """A named column parameter."""
    FUNCTION_PARAMETER_TYPE_UNSPECIFIED = "FUNCTION_PARAMETER_TYPE_UNSPECIFIED"
    PARAM = "PARAM"
    """A named parameter (default)."""

class GenerateTemporaryPathCredentialsRequestOperation(enum.Enum):
    PATH_CREATE_TABLE = "PATH_CREATE_TABLE"
    """The operation creates a table at the path."""
    PATH_READ = "PATH_READ"
    """The operation is read only."""
    PATH_READ_WRITE = "PATH_READ_WRITE"
    """The operation is read and write."""
    UNSPECIFIED = "UNSPECIFIED"
    """The operation is not specified."""

class GenerateTemporaryTableCredentialsRequestOperation(enum.Enum):
    READ = "READ"
    """The operation is read only."""
    READ_WRITE = "READ_WRITE"
    """The operation is read and write."""
    UNSPECIFIED = "UNSPECIFIED"
    """The operation is not specified."""

class GenerateTemporaryVolumeCredentialsRequestOperation(enum.Enum):
    READ_VOLUME = "READ_VOLUME"
    """The operation is read only."""
    UNSPECIFIED = "UNSPECIFIED"
    """The operation is not specified."""
    WRITE_VOLUME = "WRITE_VOLUME"
    """The operation is read and write."""

class HistoryStatus(enum.Enum):
    DISABLED = "DISABLED"
    """Data history sharing is disabled."""
    ENABLED = "ENABLED"
    """Data history sharing is enabled."""

class InvocationProtocol(enum.Enum):
    """The protocol a recipient uses to invoke an agent."""

    A2A = "A2A"
    """Agent-to-Agent protocol (Google A2A spec)."""
    ANTHROPIC = "ANTHROPIC"
    """Anthropic-compatible messages endpoint."""
    INVOCATION_PROTOCOL_UNSPECIFIED = "INVOCATION_PROTOCOL_UNSPECIFIED"
    MCP = "MCP"
    """Model Context Protocol server."""
    OPENAI = "OPENAI"
    """OpenAI-compatible chat completions endpoint."""
    REST = "REST"
    """Minimal REST baseline (`POST {endpoint}/invoke`)."""

class ParameterMode(enum.Enum):
    """The mode of the function parameter."""

    IN = "IN"
    """Input parameter."""
    PARAMETER_MODE_UNSPECIFIED = "PARAMETER_MODE_UNSPECIFIED"

class ParameterStyle(enum.Enum):
    """The parameter-passing style."""

    PARAMETER_STYLE_UNSPECIFIED = "PARAMETER_STYLE_UNSPECIFIED"
    S = "S"
    """The parameters are passed positionally (S = SQL)."""

class PolicyType(enum.Enum):
    """The kind of enforcement a policy applies."""

    POLICY_TYPE_COLUMN_MASK = "POLICY_TYPE_COLUMN_MASK"
    """Masks a column's values for callers matched by the when_condition."""
    POLICY_TYPE_ROW_FILTER = "POLICY_TYPE_ROW_FILTER"
    """Filters rows returned from the securable based on the when_condition and match_columns."""
    POLICY_TYPE_UNSPECIFIED = "POLICY_TYPE_UNSPECIFIED"

class ProviderAuthenticationType(enum.Enum):
    """The delta sharing authentication type used by a provider."""

    OAUTH_CLIENT_CREDENTIALS = "OAUTH_CLIENT_CREDENTIALS"
    """OAuth2 client-credentials authentication."""
    PROVIDER_AUTHENTICATION_TYPE_UNSPECIFIED = "PROVIDER_AUTHENTICATION_TYPE_UNSPECIFIED"
    """No authentication type specified."""
    TOKEN = "TOKEN"
    """Token-based authentication (open sharing via a credential file)."""

class Purpose(enum.Enum):
    """The purpose of a credential."""

    PURPOSE_UNSPECIFIED = "PURPOSE_UNSPECIFIED"
    SERVICE = "SERVICE"
    """The credential is used to access external services."""
    STORAGE = "STORAGE"
    """The credential is used to access external storage locations."""

class RoutineBody(enum.Enum):
    """Determines whether the function body is interpreted as SQL or as an external function."""

    EXTERNAL = "EXTERNAL"
    """The function is defined externally."""
    ROUTINE_BODY_UNSPECIFIED = "ROUTINE_BODY_UNSPECIFIED"
    SQL = "SQL"
    """The function is defined in SQL."""

class SecurityType(enum.Enum):
    """The security type of the function."""

    DEFINER = "DEFINER"
    """The function runs as the invoking user (DEFINER = standard SQL semantics)."""
    SECURITY_TYPE_UNSPECIFIED = "SECURITY_TYPE_UNSPECIFIED"

class SqlDataAccess(enum.Enum):
    """Information about the SQL data access capability of the function."""

    CONTAINS_SQL = "CONTAINS_SQL"
    """Function contains no SQL."""
    NO_SQL = "NO_SQL"
    """Function does not use SQL and does not access data."""
    READS_SQL_DATA = "READS_SQL_DATA"
    """Function reads from SQL tables or views."""
    SQL_DATA_ACCESS_UNSPECIFIED = "SQL_DATA_ACCESS_UNSPECIFIED"

class TableType(enum.Enum):
    """The type of the table."""

    EXTERNAL = "EXTERNAL"
    MANAGED = "MANAGED"
    MATERIALIZED_VIEW = "MATERIALIZED_VIEW"
    METRIC_VIEW = "METRIC_VIEW"
    STREAMING_TABLE = "STREAMING_TABLE"
    TABLE_TYPE_UNSPECIFIED = "TABLE_TYPE_UNSPECIFIED"
    VIEW = "VIEW"

class VolumeType(enum.Enum):
    EXTERNAL = "EXTERNAL"
    MANAGED = "MANAGED"
    VOLUME_TYPE_UNSPECIFIED = "VOLUME_TYPE_UNSPECIFIED"

class AgentSkillClient:
    def delete(self) -> None:
        """
        Returns:
            None
        """
        ...
    def get(self, include_browse: Optional[bool] = None) -> AgentSkill:
        """
        Args:
            include_browse: Whether to include agent skills in the response for which the principal can only
                            access selective metadata for.


        Returns:
            An agent skill registered in Unity Catalog.
        """
        ...
    def update(
        self,
        new_name: Optional[str] = None,
        description: Optional[str] = None,
        allowed_tools: Optional[List[str]] = None,
        comment: Optional[str] = None,
        owner: Optional[str] = None,
    ) -> AgentSkill:
        """
        Args:
            new_name: New name for the agent skill.
            description: Updated description of what the skill does and when to use it.
            allowed_tools: Updated tools the skill is permitted to use.
            comment: The comment attached to the agent skill.
            owner: The identifier of the user who owns the agent skill.


        Returns:
            An agent skill registered in Unity Catalog.
        """
        ...

class AgentClient:
    def delete(self) -> None:
        """
        Returns:
            None
        """
        ...
    def get(self, include_browse: Optional[bool] = None) -> Agent:
        """
        Args:
            include_browse: Whether to include agents in the response for which the principal can only
                            access selective metadata for.


        Returns:
            An agent registered in Unity Catalog.
        """
        ...
    def update(
        self,
        new_name: Optional[str] = None,
        invocation_protocol: Optional[InvocationProtocol] = None,
        endpoint: Optional[str] = None,
        description: Optional[str] = None,
        capabilities: Optional[List[str]] = None,
        input_schema: Optional[str] = None,
        comment: Optional[str] = None,
        owner: Optional[str] = None,
    ) -> Agent:
        """
        Args:
            new_name: New name for the agent.
            invocation_protocol: The protocol a recipient uses to invoke the agent.
            endpoint: The agent's invocation endpoint URL.
            description: Updated LLM-readable description.
            capabilities: Updated capability identifiers advertised by the agent.
            input_schema: Updated JSON Schema (encoded as a JSON string) describing the expected input.
            comment: The comment attached to the agent.
            owner: The identifier of the user who owns the agent.


        Returns:
            An agent registered in Unity Catalog.
        """
        ...

class CatalogClient:
    def delete(self, force: Optional[bool] = None) -> None:
        """
        Delete a catalog

        Deletes the catalog that matches the supplied name. The caller must be a metastore admin or the
        owner of the catalog.


        Args:
            force: Force deletion even if the catalog is not empty.


        Returns:
            None
        """
        ...
    def get(self, include_browse: Optional[bool] = None) -> Catalog:
        """
        Get a catalog

        Gets the specified catalog in a metastore. The caller must be a metastore admin, the owner of the
        catalog, or a user that has the USE_CATALOG privilege set for their account.


        Args:
            include_browse: Whether to include catalogs in the response for which the principal can only
                            access selective metadata for


        Returns:
            A catalog is a root-level namespace that contains schemas.
        """
        ...
    def update(
        self,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        new_name: Optional[str] = None,
    ) -> Catalog:
        """
        Update a catalog

        Updates the catalog that matches the supplied name. The caller must be either the owner of the
        catalog, or a metastore admin (when changing the owner field of the catalog).


        Args:
            owner: Username of new owner of catalog.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            new_name: Name of catalog.


        Returns:
            A catalog is a root-level namespace that contains schemas.
        """
        ...
    def agent_skill(
        self, catalog_name: str, schema_name: str, agent_skill_name: str
    ) -> AgentSkillClient: ...
    def agent(self, catalog_name: str, schema_name: str, agent_name: str) -> AgentClient: ...
    def function(
        self, catalog_name: str, schema_name: str, function_name: str
    ) -> FunctionClient: ...
    def schema(self, catalog_name: str, schema_name: str) -> SchemaClient: ...
    def table(self, catalog_name: str, schema_name: str, table_name: str) -> TableClient: ...
    def volume(self, catalog_name: str, schema_name: str, volume_name: str) -> VolumeClient: ...

class CredentialClient:
    def delete(self) -> None:
        """
        Returns:
            None
        """
        ...
    def get(self) -> Credential:
        """
        Returns:
            A credential used to access external data sources or services.
        """
        ...
    def update(
        self,
        new_name: Optional[str] = None,
        comment: Optional[str] = None,
        read_only: Optional[bool] = None,
        owner: Optional[str] = None,
        skip_validation: Optional[bool] = None,
        force: Optional[bool] = None,
        azure_service_principal: Optional[AzureServicePrincipal] = None,
        azure_managed_identity: Optional[AzureManagedIdentity] = None,
        azure_storage_key: Optional[AzureStorageKey] = None,
        aws_iam_role: Optional[AwsIamRoleConfig] = None,
        databricks_gcp_service_account: Optional[DatabricksGcpServiceAccount] = None,
    ) -> Credential:
        """
        Args:
            new_name: New name of the credential.
            comment: Comment associated with the credential.
            read_only: Whether the credential is usable only for read operations. Only applicable when
                       purpose is STORAGE.
            owner: Username of current owner of credential.
            skip_validation: Supply true to this argument to skip validation of the updated credential.
            force: Force an update even if there are dependent services (when purpose is SERVICE) or
                   dependent external locations and external tables (when purpose is STORAGE).
            azure_service_principal: The Azure service principal configuration.
            azure_managed_identity: The Azure managed identity configuration.
            azure_storage_key: The Azure storage key configuration.
            aws_iam_role: The AWS IAM role configuration.
            databricks_gcp_service_account: The Databricks managed GCP service account configuration.


        Returns:
            A credential used to access external data sources or services.
        """
        ...

class ExternalLocationClient:
    def delete(self, force: Optional[bool] = None) -> None:
        """
        Delete an external location


        Args:
            force: Force deletion even if the external location is not empty.


        Returns:
            None
        """
        ...
    def get(self) -> ExternalLocation:
        """
        Get an external location


        Returns:
            The requested resource
        """
        ...
    def update(
        self,
        url: Optional[str] = None,
        credential_name: Optional[str] = None,
        read_only: Optional[bool] = None,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
        new_name: Optional[str] = None,
        force: Optional[bool] = None,
        skip_validation: Optional[bool] = None,
    ) -> ExternalLocation:
        """
        Update an external location


        Args:
            url: Path URL of the external location.
            credential_name: Name of the storage credential used with this location.
            read_only: Indicates whether the external location is read-only.
            owner: owner of the external location.
            comment: User-provided free-form text description.
            new_name: new name of the external location.
            force: force update of the external location.
            skip_validation: Skips validation of the storage credential associated with the external
                             location.


        Returns:
            The requested resource
        """
        ...

class FunctionClient:
    def delete(self, force: Optional[bool] = None) -> None:
        """
        Delete a function

        Deletes the function that matches the supplied name. For the deletion to succeed, the caller must be
        the owner of the function.


        Args:
            force: Force deletion even if the function is not empty.


        Returns:
            None
        """
        ...
    def get(self) -> Function:
        """
        Get a function

        Gets a function from within a parent catalog and schema. For the fetch to succeed, the caller must
        be a metastore admin, the owner of the function, or have SELECT on the function.


        Returns:
            A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
        """
        ...
    def update(self, owner: Optional[str] = None) -> Function:
        """
        Update a function

        Updates the function that matches the supplied name. Only the owner of the function can be updated.


        Args:
            owner: Username of new owner of the function.


        Returns:
            A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
        """
        ...

class PolicyClient:
    def create_policy(self, policy_info: PolicyInfo) -> PolicyInfo:
        """
        Create a new policy

        Creates a new row-filter or column-mask policy on the specified securable.


        Args:
            policy_info: The policy to create.


        Returns:
            A row-filter or column-mask policy bound to principals and scoped to a securable. Policies
            are
            read as whole documents; enforcement is performed by the query engine, not by this service.
            `when_condition` is stored and returned opaque — this service does not parse or evaluate it.
        """
        ...
    def delete(self) -> None:
        """
        Delete a policy

        Deletes the policy that matches the supplied name, defined on the specified securable.


        Returns:
            None
        """
        ...
    def get(self) -> PolicyInfo:
        """
        Get a policy

        Gets the policy that matches the supplied name, defined on the specified securable.


        Returns:
            A row-filter or column-mask policy bound to principals and scoped to a securable. Policies
            are
            read as whole documents; enforcement is performed by the query engine, not by this service.
            `when_condition` is stored and returned opaque — this service does not parse or evaluate it.
        """
        ...
    def update(self, policy_info: PolicyInfo, update_mask: Optional[str] = None) -> PolicyInfo:
        """
        Update a policy

        Updates the policy that matches the supplied name, defined on the specified securable.


        Args:
            policy_info: The policy with the updated fields.
            update_mask: The list of fields to update, as a comma-separated string.


        Returns:
            A row-filter or column-mask policy bound to principals and scoped to a securable. Policies
            are
            read as whole documents; enforcement is performed by the query engine, not by this service.
            `when_condition` is stored and returned opaque — this service does not parse or evaluate it.
        """
        ...

class ProviderClient:
    def delete(self) -> None:
        """
        Delete a provider.


        Returns:
            None
        """
        ...
    def get(self) -> Provider:
        """
        Get a provider by name.


        Returns:
            A provider represents an organization sharing data with this metastore. A provider is the
            inbound counterpart of a recipient: it is registered from a share activation/ credential
            file
            and is used to access shares offered by an upstream Delta Sharing server.
        """
        ...
    def update(
        self,
        new_name: Optional[str] = None,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
        recipient_profile_str: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
    ) -> Provider:
        """
        Update a provider.


        Args:
            new_name: New name for the provider.
            owner: Username of the provider owner.
            comment: Description about the provider.
            recipient_profile_str: The recipient profile (credential file contents) used to connect to the
                                   sharing server.
            properties: Provider properties as map of string key-value pairs. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.


        Returns:
            A provider represents an organization sharing data with this metastore. A provider is the
            inbound counterpart of a recipient: it is registered from a share activation/ credential
            file
            and is used to access shares offered by an upstream Delta Sharing server.
        """
        ...

class RecipientClient:
    def delete(self) -> None:
        """
        Delete a recipient.


        Returns:
            None
        """
        ...
    def get(self) -> Recipient:
        """
        Get a recipient by name.


        Returns:
            The requested resource
        """
        ...
    def update(
        self,
        new_name: Optional[str] = None,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        expiration_time: Optional[int] = None,
    ) -> Recipient:
        """
        Update a recipient.


        Args:
            new_name: New name for the recipient
            owner: Username of the recipient owner.
            comment: Description about the recipient.
            properties: Recipient properties as map of string key-value pairs. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            expiration_time: Expiration timestamp of the token, in epoch milliseconds.


        Returns:
            The requested resource
        """
        ...

class SchemaClient:
    def delete(self, force: Optional[bool] = None) -> None:
        """
        Deletes the specified schema from the parent catalog. The caller must be the owner of the schema or
        an owner of the parent catalog.


        Args:
            force: Force deletion even if the schema is not empty.


        Returns:
            None
        """
        ...
    def get(self) -> Schema:
        """
        Gets the specified schema within the metastore. The caller must be a metastore admin, the owner of
        the schema, or a user that has the USE_SCHEMA privilege on the schema.


        Returns:
            A schema is a namespace within a catalog that contains tables.
        """
        ...
    def update(
        self,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        new_name: Optional[str] = None,
    ) -> Schema:
        """
        Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
        If the caller is a metastore admin, only the owner field can be changed in the update. If the name
        field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA privilege on
        the parent catalog.


        Args:
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            new_name: Name of schema.


        Returns:
            A schema is a namespace within a catalog that contains tables.
        """
        ...
    def agent_skill(
        self, catalog_name: str, schema_name: str, agent_skill_name: str
    ) -> AgentSkillClient: ...
    def agent(self, catalog_name: str, schema_name: str, agent_name: str) -> AgentClient: ...
    def function(
        self, catalog_name: str, schema_name: str, function_name: str
    ) -> FunctionClient: ...
    def table(self, catalog_name: str, schema_name: str, table_name: str) -> TableClient: ...
    def volume(self, catalog_name: str, schema_name: str, volume_name: str) -> VolumeClient: ...

class ShareClient:
    def delete(self) -> None:
        """
        Deletes a share.


        Returns:
            None
        """
        ...
    def get(self, include_shared_data: Optional[bool] = None) -> Share:
        """
        Get a share by name.


        Args:
            include_shared_data: Query for data to include in the share.


        Returns:
            The requested resource
        """
        ...
    def update_permissions(
        self,
        changes: Optional[List[PermissionsChange]] = None,
        omit_permissions_list: Optional[bool] = None,
    ) -> UpdatePermissionsResponse:
        """
        Updates the permissions for a data share in the metastore.


        Args:
            changes: Array of permissions change objects.
            omit_permissions_list: Whether to return the latest permissions list of the share in the
                                   response.


        Returns:
            The requested resource
        """
        ...
    def update(
        self,
        updates: Optional[List[DataObjectUpdate]] = None,
        new_name: Optional[str] = None,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
    ) -> Share:
        """
        Update a share.


        Args:
            updates: Array of shared data object updates.
            new_name: A new name for the share.
            owner: Owner of the share.
            comment: User-provided free-form text description.


        Returns:
            The requested resource
        """
        ...

class StagingTableClient: ...

class TableClient:
    def delete(self) -> None:
        """
        Delete a table


        Returns:
            None
        """
        ...
    def get(
        self,
        include_delta_metadata: Optional[bool] = None,
        include_browse: Optional[bool] = None,
        include_manifest_capabilities: Optional[bool] = None,
    ) -> Table:
        """
        Get a table


        Args:
            include_delta_metadata: Whether delta metadata should be included in the response.
            include_browse: Whether to include tables in the response for which the principal can only
                            access selective metadata for
            include_manifest_capabilities: Whether to include a manifest containing capabilities the table
                                           has.


        Returns:
            The requested resource
        """
        ...

class TagPolicyClient:
    def delete(self) -> None:
        """
        Delete a tag policy

        Deletes the governed tag definition that matches the supplied tag key.


        Returns:
            None
        """
        ...
    def get(self) -> TagPolicy:
        """
        Get a tag policy

        Gets the governed tag definition for the specified tag key.


        Returns:
            A governed tag definition (tag policy). A tag policy defines a tag key together with the
            rules
            that govern how it can be used, including the optional set of allowed values. Assigning a
            governed tag to an entity is done through the Entity Tag Assignments API.
        """
        ...
    def update(self, tag_policy: TagPolicy, update_mask: Optional[str] = None) -> TagPolicy:
        """
        Update a tag policy

        Updates the governed tag definition that matches the supplied tag key.


        Args:
            tag_policy: The tag policy with the updated fields.
            update_mask: The list of fields to update, as a comma-separated string.


        Returns:
            A governed tag definition (tag policy). A tag policy defines a tag key together with the
            rules
            that govern how it can be used, including the optional set of allowed values. Assigning a
            governed tag to an entity is done through the Entity Tag Assignments API.
        """
        ...

class VolumeClient:
    def delete(self) -> None:
        """
        Returns:
            None
        """
        ...
    def get(self, include_browse: Optional[bool] = None) -> Volume:
        """
        Args:
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for


        Returns:
            The requested resource
        """
        ...
    def update(
        self,
        new_name: Optional[str] = None,
        comment: Optional[str] = None,
        owner: Optional[str] = None,
    ) -> Volume:
        """
        Args:
            new_name: New name for the volume.
            comment: The comment attached to the volume
            owner: The identifier of the user who owns the volume


        Returns:
            The requested resource
        """
        ...

class UnityCatalogClient:
    def __init__(self, base_url: str, token: Optional[str] = None) -> None: ...
    def commit(
        self,
        table_id: str,
        table_uri: str,
        commit_info: Optional[CommitInfo] = None,
        latest_backfilled_version: Optional[int] = None,
        metadata: Optional[Metadata] = None,
    ) -> None:
        """
        Ratify a staged commit at the requested version (first-writer-wins), and/or notify the catalog that
        commits have been backfilled to the Delta log.


        Args:
            table_id: UUID of the catalog-managed table being committed to.
            table_uri: The storage URI of the table. Must match the table's registered storage location
                       (normalized) on the commit path.
            commit_info: The commit to ratify. Absent for a backfill-only notification.
            latest_backfilled_version: Notify the catalog that commits up to and including this version
                                       have been published (backfilled) to the Delta log. The catalog prunes
                                       ratified commits accordingly.
            metadata: An optional Delta metadata change accompanying the commit.


        Returns:
            The requested resource
        """
        ...
    def create_agent(
        self,
        catalog_name: str,
        schema_name: str,
        name: str,
        invocation_protocol: InvocationProtocol,
        endpoint: str,
        description: Optional[str] = None,
        capabilities: Optional[List[str]] = None,
        input_schema: Optional[str] = None,
        comment: Optional[str] = None,
    ) -> Agent:
        """
        Args:
            catalog_name: The identifier of the catalog.
            schema_name: The identifier of the schema.
            name: The identifier of the agent.
            invocation_protocol: The protocol a recipient uses to invoke the agent.
            endpoint: The agent's invocation endpoint URL.
            description: An LLM-readable description of what the agent does and the inputs it expects.
            capabilities: Capability identifiers advertised by the agent.
            input_schema: A JSON Schema (encoded as a JSON string) describing the expected input.
            comment: User-provided free-form text description.


        Returns:
            An agent registered in Unity Catalog.
        """
        ...
    def create_agent_skill(
        self,
        catalog_name: str,
        schema_name: str,
        name: str,
        agent_skill_type: AgentSkillType,
        storage_location: Optional[str] = None,
        description: Optional[str] = None,
        license: Optional[str] = None,
        allowed_tools: Optional[List[str]] = None,
        metadata: Optional[Dict[str, str]] = None,
        comment: Optional[str] = None,
    ) -> AgentSkill:
        """
        Args:
            catalog_name: The identifier of the catalog.
            schema_name: The identifier of the schema.
            name: The identifier of the agent skill.
            agent_skill_type: How the storage location is provisioned (external or managed).
            storage_location: The storage location of the skill directory on the cloud. Required for
                              EXTERNAL skills; ignored (server-derived) for MANAGED skills.
            description: A human-readable description of what the skill does and when to use it.
            license: SPDX license identifier or free-form license text for the skill.
            allowed_tools: The tools the skill is permitted to use.
            metadata: Arbitrary additional metadata declared by the skill.
            comment: User-provided free-form text description.


        Returns:
            An agent skill registered in Unity Catalog.
        """
        ...
    def create_catalog(
        self,
        name: str,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        storage_root: Optional[str] = None,
        provider_name: Optional[str] = None,
        share_name: Optional[str] = None,
    ) -> Catalog:
        """
        Create a new catalog

        Creates a new catalog instance in the parent metastore if the caller is a metastore admin or has the
        CREATE_CATALOG privilege.


        Args:
            name: Name of catalog.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable.
            storage_root: Storage root URL for managed tables within catalog.
            provider_name: The name of delta sharing provider. A Delta Sharing catalog is a catalog that is
                           based on a Delta share on a remote sharing server.
            share_name: The name of the share under the share provider.


        Returns:
            A catalog is a root-level namespace that contains schemas.
        """
        ...
    def create_credential(
        self,
        name: str,
        purpose: Purpose,
        comment: Optional[str] = None,
        read_only: Optional[bool] = None,
        skip_validation: Optional[bool] = None,
        azure_service_principal: Optional[AzureServicePrincipal] = None,
        azure_managed_identity: Optional[AzureManagedIdentity] = None,
        azure_storage_key: Optional[AzureStorageKey] = None,
        aws_iam_role: Optional[AwsIamRoleConfig] = None,
        databricks_gcp_service_account: Optional[DatabricksGcpServiceAccount] = None,
    ) -> Credential:
        """
        Args:
            name: The credential name. The name must be unique among storage and service credentials within
                  the metastore.
            purpose: Indicates the purpose of the credential.
            comment: Comment associated with the credential.
            read_only: Whether the credential is usable only for read operations. Only applicable when
                       purpose is STORAGE.
            skip_validation: Optional. Supplying true to this argument skips validation of the created set
                             of credentials.
            azure_service_principal: The Azure service principal configuration.
            azure_managed_identity: The Azure managed identity configuration.
            azure_storage_key: The Azure storage key configuration.
            aws_iam_role: The AWS IAM role configuration.
            databricks_gcp_service_account: The Databricks managed GCP service account configuration.


        Returns:
            A credential used to access external data sources or services.
        """
        ...
    def create_entity_tag_assignment(
        self, tag_assignment: EntityTagAssignment
    ) -> EntityTagAssignment:
        """
        Create an entity tag assignment

        Assigns a tag to a Unity Catalog entity.


        Args:
            tag_assignment: The tag assignment to create.


        Returns:
            The assignment of a tag to a Unity Catalog entity. Unlike a TagPolicy (a governed-tag
            *definition*), an assignment is the application of a tag to a specific securable. It has no
            identifier of its own — its identity is the composite of (entity_type, entity_name,
            tag_key). It
            is intentionally NOT a `google.api.resource`: assignments are stored as associations between
            the
            entity and its tag, not as standalone objects.
        """
        ...
    def create_external_location(
        self,
        name: str,
        url: str,
        credential_name: str,
        read_only: Optional[bool] = None,
        comment: Optional[str] = None,
        skip_validation: Optional[bool] = None,
    ) -> ExternalLocation:
        """
        Create a new external location


        Args:
            name: Name of external location.
            url: Path URL of the external location.
            credential_name: Name of the storage credential used with this location.
            read_only: Indicates whether the external location is read-only.
            comment: User-provided free-form text description.
            skip_validation: Skips validation of the storage credential associated with the external
                             location.


        Returns:
            The requested resource
        """
        ...
    def create_function(
        self,
        name: str,
        catalog_name: str,
        schema_name: str,
        data_type: str,
        full_data_type: str,
        parameter_style: ParameterStyle,
        is_deterministic: bool,
        sql_data_access: SqlDataAccess,
        is_null_call: bool,
        security_type: SecurityType,
        routine_body: RoutineBody,
        input_params: Optional[FunctionParameterInfos] = None,
        routine_definition: Optional[str] = None,
        routine_body_language: Optional[str] = None,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
    ) -> Function:
        """
        Create a function

        Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION privilege
        on the parent catalog and schema.


        Args:
            name: Name of function, relative to parent schema.
            catalog_name: Name of parent catalog.
            schema_name: Name of parent schema.
            data_type: Full data type specification of the return type of the function.
            full_data_type: Full data type specification as SQL/catalogString text.
            parameter_style: The parameter-passing style.
            is_deterministic: Indicates whether the function is deterministic.
            sql_data_access: SQL data access information.
            is_null_call: Indicates whether the function is null-calling.
            security_type: The security type of the function.
            routine_body: The routine body.
            input_params: The array of function parameter infos.
            routine_definition: Function body.
            routine_body_language: The language of the function routine body.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable.


        Returns:
            A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
        """
        ...
    def create_provider(
        self,
        name: str,
        authentication_type: ProviderAuthenticationType,
        owner: Optional[str] = None,
        comment: Optional[str] = None,
        recipient_profile_str: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
    ) -> Provider:
        """
        Create a new provider.


        Args:
            name: Name of the provider.
            authentication_type: The delta sharing authentication type.
            owner: Username of the provider owner.
            comment: Description about the provider.
            recipient_profile_str: The recipient profile (credential file contents) used to connect to the
                                   sharing server, required for TOKEN authentication.
            properties: Provider properties as map of string key-value pairs.


        Returns:
            A provider represents an organization sharing data with this metastore. A provider is the
            inbound counterpart of a recipient: it is registered from a share activation/ credential
            file
            and is used to access shares offered by an upstream Delta Sharing server.
        """
        ...
    def create_recipient(
        self,
        name: str,
        authentication_type: AuthenticationType,
        owner: str,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        expiration_time: Optional[int] = None,
    ) -> Recipient:
        """
        Create a new recipient.


        Args:
            name: Name of the recipient.
            authentication_type: The delta sharing authentication type.
            owner: Username of the recipient owner.
            comment: Description about the recipient.
            properties: Recipient properties as map of string key-value pairs. When provided in update
                        request, the specified properties will override the existing properties. To add and
                        remove properties, one would need to perform a read-modify-write.
            expiration_time: Expiration timestamp of the token, in epoch milliseconds.


        Returns:
            The requested resource
        """
        ...
    def create_schema(
        self,
        name: str,
        catalog_name: str,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        storage_root: Optional[str] = None,
    ) -> Schema:
        """
        Creates a new schema for catalog in the Metatastore. The caller must be a metastore admin, or have
        the CREATE_SCHEMA privilege in the parent catalog.


        Args:
            name: Name of schema, relative to parent catalog.
            catalog_name: Name of parent catalog.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable.
            storage_root: Storage root URL for managed storage location of the schema. If not set, managed
                          securables under this schema fall back to the parent catalog's storage location.
                          Example: `s3://bucket/ucroot`.


        Returns:
            A schema is a namespace within a catalog that contains tables.
        """
        ...
    def create_share(self, name: str, comment: Optional[str] = None) -> Share:
        """
        Create a new share.


        Args:
            name: Name of the share.
            comment: User-provided free-form text description.


        Returns:
            The requested resource
        """
        ...
    def create_staging_table(self, name: str, catalog_name: str, schema_name: str) -> StagingTable:
        """
        Creates a new staging table, allocating an immutable table id and a storage location under the
        parent schema/catalog managed storage root. The caller must have the CREATE privilege on the parent
        schema.


        Args:
            name: Name of the staging table, relative to the parent schema.
            catalog_name: Name of the parent catalog.
            schema_name: Name of the parent schema relative to its parent catalog.


        Returns:
            A staging table is a transient reservation created while provisioning a Unity Catalog
            managed
            table. On creation the server allocates an immutable table id and storage location under the
            parent schema/catalog's managed storage root. The client then writes the initial Delta
            commit
            (`_delta_log/00...0.json`) at that location and finalizes the table via the CreateTable API,
            which commits this staging row. A staging table is not yet a usable table and is not
            returned by
            the table APIs.
        """
        ...
    def create_table(
        self,
        name: str,
        schema_name: str,
        catalog_name: str,
        table_type: TableType,
        data_source_format: DataSourceFormat,
        columns: Optional[List[Column]] = None,
        storage_location: Optional[str] = None,
        comment: Optional[str] = None,
        properties: Optional[Dict[str, str]] = None,
        view_definition: Optional[str] = None,
        view_dependencies: Optional[DependencyList] = None,
    ) -> Table:
        """
        Create a table


        Args:
            name: Name of table, relative to parent schema.
            schema_name: Name of parent schema relative to its parent catalog.
            catalog_name: Name of parent catalog.
            columns: The array of Column definitions of the table's columns.
            storage_location: Storage root URL for external table.
            comment: User-provided free-form text description.
            properties: A map of key-value properties attached to the securable.
            view_definition: Definition text for view-like table types (VIEW, MATERIALIZED_VIEW,
                             STREAMING_TABLE, METRIC_VIEW). The format depends on the table type: SQL for
                             views, YAML for metric views. Required for METRIC_VIEW.
            view_dependencies: Tables and functions the view-like table reads. For metric views the server
                               derives this from view_definition and rejects a supplied list that diverges
                               from the derived set (the definition is the single source of truth).


        Returns:
            The requested resource
        """
        ...
    def create_tag_policy(self, tag_policy: TagPolicy) -> TagPolicy:
        """
        Create a new tag policy

        Creates a new governed tag definition.


        Args:
            tag_policy: The tag policy to create.


        Returns:
            A governed tag definition (tag policy). A tag policy defines a tag key together with the
            rules
            that govern how it can be used, including the optional set of allowed values. Assigning a
            governed tag to an entity is done through the Entity Tag Assignments API.
        """
        ...
    def create_volume(
        self,
        catalog_name: str,
        schema_name: str,
        name: str,
        volume_type: VolumeType,
        storage_location: Optional[str] = None,
        comment: Optional[str] = None,
    ) -> Volume:
        """
        Args:
            catalog_name: The identifier of the catalog
            schema_name: The identifier of the schema
            name: The identifier of the volume
            volume_type: The type of the volume. An external volume is located in the specified external
                         location. A managed volume is located in the default location which is specified by
                         the parent schema, or the parent catalog, or the Metastore.
            storage_location: The storage location on the cloud
            comment: The storage location on the cloud


        Returns:
            The requested resource
        """
        ...
    def delete_entity_tag_assignment(
        self, entity_type: str, entity_name: str, tag_key: str
    ) -> None:
        """
        Delete an entity tag assignment

        Deletes the tag assignment for the specified entity and tag key.


        Returns:
            The requested resource
        """
        ...
    def generate_temporary_path_credentials(
        self,
        url: str,
        operation: GenerateTemporaryPathCredentialsRequestOperation,
        dry_run: Optional[bool] = None,
    ) -> TemporaryCredential:
        """
        Generate a new set of credentials for a path.


        Args:
            url: URL for path-based access.
            operation: The operation being performed on the path.
            dry_run: When set to true, the service will not validate that the generated credentials can
                     perform write operations, therefore no new paths will be created and the response will
                     not contain valid credentials. Defaults to false.


        Returns:
            The response to the GenerateTemporaryTableCredentialsRequest.
        """
        ...
    def generate_temporary_table_credentials(
        self, table_id: str, operation: GenerateTemporaryTableCredentialsRequestOperation
    ) -> TemporaryCredential:
        """
        Generate a new set of credentials for a table.


        Args:
            table_id: UUID of the table to read or write.
            operation: The operation performed against the table data, either READ or READ_WRITE. If
                       READ_WRITE is specified, the credentials returned will have write permissions,
                       otherwise, it will be read only.


        Returns:
            The response to the GenerateTemporaryTableCredentialsRequest.
        """
        ...
    def generate_temporary_volume_credentials(
        self, volume_id: str, operation: GenerateTemporaryVolumeCredentialsRequestOperation
    ) -> TemporaryCredential:
        """
        Generate a new set of credentials for a volume.

        The metastore must have the `external_access_enabled` flag set to true (default false). The caller
        must have the `EXTERNAL_USE_SCHEMA` privilege on the parent schema (granted by a catalog owner).


        Args:
            volume_id: UUID of the volume to read or write.
            operation: The operation performed against the volume data, either READ_VOLUME or WRITE_VOLUME.
                       If WRITE_VOLUME is specified, the credentials returned will have write permissions,
                       otherwise, it will be read only.


        Returns:
            The response to the GenerateTemporaryTableCredentialsRequest.
        """
        ...
    def get_commits(
        self, table_id: str, table_uri: str, start_version: int, end_version: Optional[int] = None
    ) -> GetCommitsResponse:
        """
        Return ratified-but-unpublished commits for a table, plus the latest version the catalog tracks.


        Args:
            table_id: UUID of the catalog-managed table.
            table_uri: The storage URI of the table.
            start_version: The lowest version to return (inclusive). Defaults to 0.
            end_version: The highest version to return (inclusive). When set, must be `>= start_version`.
                         Defaults to the latest version.


        Returns:
            Response listing ratified-but-unpublished commits for a table.
        """
        ...
    def get_entity_tag_assignment(
        self, entity_type: str, entity_name: str, tag_key: str
    ) -> EntityTagAssignment:
        """
        Get an entity tag assignment

        Gets the tag assignment for the specified entity and tag key.


        Returns:
            The assignment of a tag to a Unity Catalog entity. Unlike a TagPolicy (a governed-tag
            *definition*), an assignment is the application of a tag to a specific securable. It has no
            identifier of its own — its identity is the composite of (entity_type, entity_name,
            tag_key). It
            is intentionally NOT a `google.api.resource`: assignments are stored as associations between
            the
            entity and its tag, not as standalone objects.
        """
        ...
    def list_agent_skills(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: Optional[int] = None,
        include_browse: Optional[bool] = None,
    ) -> List[AgentSkill]:
        """
        Lists agent skills.


        Args:
            catalog_name: The identifier of the catalog.
            schema_name: The identifier of the schema.
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include agent skills in the response for which the principal can only
                            access selective metadata for.


        Returns:
            List of The agent skills returned.
        """
        ...
    def list_agents(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: Optional[int] = None,
        include_browse: Optional[bool] = None,
    ) -> List[Agent]:
        """
        Lists agents.


        Args:
            catalog_name: The identifier of the catalog.
            schema_name: The identifier of the schema.
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include agents in the response for which the principal can only
                            access selective metadata for.


        Returns:
            List of The agents returned.
        """
        ...
    def list_catalogs(self, max_results: Optional[int] = None) -> List[Catalog]:
        """
        List catalogs

        Gets an array of catalogs in the metastore. If the caller is the metastore admin, all catalogs
        will be retrieved. Otherwise, only catalogs owned by the caller (or for which the caller has the
        USE_CATALOG privilege) will be retrieved. There is no guarantee of a specific ordering of the
        elements in the array.


        Args:
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of The catalogs returned.
        """
        ...
    def list_credentials(
        self, purpose: Optional[Purpose] = None, max_results: Optional[int] = None
    ) -> List[Credential]:
        """
        Args:
            purpose: Return only credentials for the specified purpose.
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of The credentials returned.
        """
        ...
    def list_entity_tag_assignments(
        self,
        entity_type: str,
        entity_name: str,
        max_results: Optional[int] = None,
        page_token: Optional[str] = None,
    ) -> ListEntityTagAssignmentsResponse:
        """
        List entity tag assignments

        Gets the tag assignments for the specified entity.


        Args:
            max_results: The maximum number of results per page that should be returned.
            page_token: Opaque pagination token to go to next page based on previous query.


        Returns:
            List entity tag assignments response.
        """
        ...
    def list_external_locations(
        self, max_results: Optional[int] = None, include_browse: Optional[bool] = None
    ) -> List[ExternalLocation]:
        """
        List external locations


        Args:
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for


        Returns:
            List of The external locations returned.
        """
        ...
    def list_functions(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: Optional[int] = None,
        include_browse: Optional[bool] = None,
    ) -> List[Function]:
        """
        List functions

        List functions within the specified parent catalog and schema. If the caller is the metastore admin,
        all functions are returned in the response. Otherwise, the caller must have USE_CATALOG on the
        parent catalog and USE_SCHEMA on the parent schema, and the function must either be owned by the
        caller or have SELECT on the function.


        Args:
            catalog_name: Name of parent catalog for functions of interest.
            schema_name: Parent schema of functions.
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include functions in the response for which the principal can only
                            access selective metadata for.


        Returns:
            List of The functions returned.
        """
        ...
    def list_providers(self, max_results: Optional[int] = None) -> List[Provider]:
        """
        List providers.


        Args:
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of List of providers.
        """
        ...
    def list_recipients(self, max_results: Optional[int] = None) -> List[Recipient]:
        """
        List recipients.


        Args:
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of List of recipients.
        """
        ...
    def list_schemas(
        self,
        catalog_name: str,
        max_results: Optional[int] = None,
        include_browse: Optional[bool] = None,
    ) -> List[Schema]:
        """
        Gets an array of schemas for a catalog in the metastore. If the caller is the metastore admin or the
        owner of the parent catalog, all schemas for the catalog will be retrieved. Otherwise, only schemas
        owned by the caller (or for which the caller has the USE_SCHEMA privilege) will be retrieved. There
        is no guarantee of a specific ordering of the elements in the array.


        Args:
            catalog_name: Name of parent catalog.
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for


        Returns:
            List of The schemas returned.
        """
        ...
    def list_shares(self, max_results: Optional[int] = None) -> List[Share]:
        """
        List shares.


        Args:
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of List of shares.
        """
        ...
    def list_tables(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: Optional[int] = None,
        include_delta_metadata: Optional[bool] = None,
        omit_columns: Optional[bool] = None,
        omit_properties: Optional[bool] = None,
        omit_username: Optional[bool] = None,
        include_browse: Optional[bool] = None,
        include_manifest_capabilities: Optional[bool] = None,
    ) -> List[Table]:
        """
        Gets an array of all tables for the current metastore under the parent catalog and schema.

        The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table. For
        the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the parent
        catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a specific
        ordering of the elements in the array.


        Args:
            catalog_name: Name of parent catalog for tables of interest.
            schema_name: Name of parent schema for tables of interest.
            max_results: The maximum number of results per page that should be returned.
            include_delta_metadata: Whether delta metadata should be included in the response.
            omit_columns: Whether to omit the columns of the table from the response or not.
            omit_properties: Whether to omit the properties of the table from the response or not.
            omit_username: Whether to omit the username of the table (e.g. owner, updated_by, created_by)
                           from the response or not.
            include_browse: Whether to include tables in the response for which the principal can only
                            access selective metadata for
            include_manifest_capabilities: Whether to include a manifest containing capabilities the table
                                           has.


        Returns:
            List of The tables returned.
        """
        ...
    def list_tag_policies(self, max_results: Optional[int] = None) -> List[TagPolicy]:
        """
        List tag policies

        Gets an array of tag policies. There is no guarantee of a specific ordering of the elements in the
        array.


        Args:
            max_results: The maximum number of results per page that should be returned.


        Returns:
            List of The tag policies returned.
        """
        ...
    def list_volumes(
        self,
        catalog_name: str,
        schema_name: str,
        max_results: Optional[int] = None,
        include_browse: Optional[bool] = None,
    ) -> List[Volume]:
        """
        Lists volumes.


        Args:
            catalog_name: The identifier of the catalog
            schema_name: The identifier of the schema
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for


        Returns:
            List of The volumes returned.
        """
        ...
    def update_entity_tag_assignment(
        self,
        entity_type: str,
        entity_name: str,
        tag_key: str,
        tag_assignment: EntityTagAssignment,
        update_mask: Optional[str] = None,
    ) -> EntityTagAssignment:
        """
        Update an entity tag assignment

        Updates the tag assignment for the specified entity and tag key.


        Args:
            tag_assignment: The tag assignment with the updated fields.
            update_mask: The list of fields to update, as a comma-separated string.


        Returns:
            The assignment of a tag to a Unity Catalog entity. Unlike a TagPolicy (a governed-tag
            *definition*), an assignment is the application of a tag to a specific securable. It has no
            identifier of its own — its identity is the composite of (entity_type, entity_name,
            tag_key). It
            is intentionally NOT a `google.api.resource`: assignments are stored as associations between
            the
            entity and its tag, not as standalone objects.
        """
        ...
    def agent(self, catalog_name: str, schema_name: str, agent_name: str) -> AgentClient: ...
    def agent_skill(
        self, catalog_name: str, schema_name: str, agent_skill_name: str
    ) -> AgentSkillClient: ...
    def catalog(self, catalog_name: str) -> CatalogClient: ...
    def credential(self, credential_name: str) -> CredentialClient: ...
    def external_location(self, external_location_name: str) -> ExternalLocationClient: ...
    def function(
        self, catalog_name: str, schema_name: str, function_name: str
    ) -> FunctionClient: ...
    def policy(self, policy_name: str) -> PolicyClient: ...
    def provider(self, provider_name: str) -> ProviderClient: ...
    def recipient(self, recipient_name: str) -> RecipientClient: ...
    def schema(self, catalog_name: str, schema_name: str) -> SchemaClient: ...
    def share(self, share_name: str) -> ShareClient: ...
    def staging_table(self, staging_table_name: str) -> StagingTableClient: ...
    def table(self, catalog_name: str, schema_name: str, table_name: str) -> TableClient: ...
    def tag_policy(self, tag_policy_name: str) -> TagPolicyClient: ...
    def volume(self, catalog_name: str, schema_name: str, volume_name: str) -> VolumeClient: ...
# Hand-written supplement appended to `_client.pyi` after codegen.
#
# Declares the PyO3 symbols hand-registered in
# `python/client/src/{error,reference}.rs` that the protobuf-driven
# trestle codegen does not see. Edit this file (not `_client.pyi`) when
# adding new hand-written `#[pyfunction]`s, `#[pyclass]`es, or
# `create_exception!` calls to the `_client` PyO3 module.
#
# This file is appended verbatim to `_client.pyi` by `just generate-code`.
# It is also a valid stub on its own (the import below is harmless when
# appended to `_client.pyi`, which already imports the same names).

from typing import Literal

# Exceptions (declared in python/client/src/error.rs).

class UnityCatalogError(Exception):
    """Base class for all Python-facing Unity Catalog exceptions."""

    ...

class GenericError(UnityCatalogError):
    """Wraps a generic server-side error."""

    ...

class NotFoundError(UnityCatalogError):
    """Raised when a requested resource does not exist (`RESOURCE_NOT_FOUND`)."""

    ...

class AlreadyExistsError(UnityCatalogError):
    """Raised when a resource already exists (`RESOURCE_ALREADY_EXISTS`)."""

    ...

class PermissionDeniedError(UnityCatalogError):
    """Raised when the caller lacks permission (`PERMISSION_DENIED`)."""

    ...

class UnauthenticatedError(UnityCatalogError):
    """Raised when the request is not authenticated (`UNAUTHENTICATED`)."""

    ...

class InvalidParameterError(UnityCatalogError):
    """Raised when a request parameter is invalid (`INVALID_PARAMETER_VALUE`)."""

    ...

class RequestLimitError(UnityCatalogError):
    """Raised when the request rate limit is exceeded (`REQUEST_LIMIT_EXCEEDED`)."""

    ...

class InternalServerError(UnityCatalogError):
    """Raised when the server encounters an internal error (`INTERNAL_ERROR`)."""

    ...

class ServiceUnavailableError(UnityCatalogError):
    """Raised when the service is temporarily unavailable (`TEMPORARILY_UNAVAILABLE`)."""

    ...

# Free functions (declared in python/client/src/reference.rs).

def parse_uc_url(
    url: str,
    /,
) -> tuple[Literal["volume", "table", "path"], dict[str, str]]:
    """Parse a Unity Catalog URL into ``(kind, payload)``.

    Delegates to ``unitycatalog_common::UCReference::parse``. See that
    type's documentation for the supported grammar (``uc://`` URLs,
    raw cloud URLs, ``vol+dbfs:`` aliases, case-insensitive kind
    segments, percent-decoded path components).

    ``payload`` carries the variant-specific fields:

    * ``"volume"`` → ``{catalog, schema, volume, path}``
    * ``"table"``  → ``{catalog, schema, table}``
    * ``"path"``   → ``{url}``
    """
    ...

# Hand-written `#[pymethods]` for `TemporaryCredentialClient`
# (declared in python/client/src/client.rs). These wrap the underlying
# `Generate*Credentials` RPCs with name → UUID resolution. The trestle
# codegen does not see them, so they are declared here as a complete
# replacement for the empty `class TemporaryCredentialClient: ...`
# placeholder emitted by the codegen — the `just generate-code` recipe
# strips that placeholder line before appending this file.

# `TemporaryCredential` is declared earlier in the concatenated
# `_client.pyi` (codegen-emitted from the proto schema). Use a string
# forward reference so this file also type-checks in isolation.

class TemporaryCredentialClient:
    def __init__(self, base_url: str, token: str | None = None) -> None: ...
    def temporary_table_credential(
        self,
        table: str,
        operation: Literal["read", "read_write"],
    ) -> tuple[TemporaryCredential, str]:
        """Vend a temporary credential for a Unity Catalog table.

        ``table`` is the three-level ``catalog.schema.table`` name.
        The returned tuple holds the credential and the resolved table
        UUID (as a string).
        """
        ...

    def temporary_volume_credential(
        self,
        volume: str,
        operation: Literal["read", "read_write", "write"],
    ) -> tuple[TemporaryCredential, str]:
        """Vend a temporary credential for a Unity Catalog volume.

        ``volume`` is the three-level ``catalog.schema.volume`` name.
        Server support requires the metastore's
        ``external_access_enabled`` flag and the caller's
        ``EXTERNAL_USE_SCHEMA`` privilege.
        """
        ...

    def temporary_path_credential(
        self,
        path: str,
        operation: Literal["read", "read_write", "create_table"],
        dry_run: bool | None = None,
    ) -> tuple[TemporaryCredential, str]:
        """Vend a temporary credential for an arbitrary cloud URL."""
        ...
