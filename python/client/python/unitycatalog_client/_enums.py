# @generated — do not edit by hand.
"""The enums of the API.

Each member is a `str` equal to its name, so a member and its name compare equal, and a
parameter takes either.
"""

import enum


class AgentSkillType(enum.StrEnum):
    """How an agent skill's storage location is provisioned."""

    EXTERNAL = "EXTERNAL"
    """The skill directory lives at a caller-supplied location within a
    registered external location.
    """
    MANAGED = "MANAGED"
    """The skill directory lives at a server-derived location under the parent
    schema/catalog managed storage root.
    """


class InvocationProtocol(enum.StrEnum):
    """The protocol a recipient uses to invoke an agent."""

    MCP = "MCP"
    """Model Context Protocol server."""
    A2A = "A2A"
    """Agent-to-Agent protocol (Google A2A spec)."""
    OPENAI = "OPENAI"
    """OpenAI-compatible chat completions endpoint."""
    ANTHROPIC = "ANTHROPIC"
    """Anthropic-compatible messages endpoint."""
    REST = "REST"
    """Minimal REST baseline (`POST {endpoint}/invoke`)."""


class CatalogType(enum.StrEnum):
    """The type of the catalog."""

    MANAGED_CATALOG = "MANAGED_CATALOG"
    DELTASHARING_CATALOG = "DELTASHARING_CATALOG"
    SYSTEM_CATALOG = "SYSTEM_CATALOG"


class Purpose(enum.StrEnum):
    """The purpose of a credential."""

    STORAGE = "STORAGE"
    """The credential is used to access external storage locations."""
    SERVICE = "SERVICE"
    """The credential is used to access external services."""


class FunctionParameterType(enum.StrEnum):
    """The type of the function parameter."""

    COLUMN = "COLUMN"
    """A named column parameter."""
    PARAM = "PARAM"
    """A named parameter (default)."""


class ParameterMode(enum.StrEnum):
    """The mode of the function parameter."""

    IN = "IN"
    """Input parameter."""


class ParameterStyle(enum.StrEnum):
    """The parameter-passing style."""

    S = "S"
    """The parameters are passed positionally (S = SQL)."""


class RoutineBody(enum.StrEnum):
    """Determines whether the function body is interpreted as SQL or as an external function."""

    SQL = "SQL"
    """The function is defined in SQL."""
    EXTERNAL = "EXTERNAL"
    """The function is defined externally."""


class SecurityType(enum.StrEnum):
    """The security type of the function."""

    DEFINER = "DEFINER"
    """The function runs as the invoking user (DEFINER = standard SQL semantics)."""


class SqlDataAccess(enum.StrEnum):
    """Information about the SQL data access capability of the function."""

    CONTAINS_SQL = "CONTAINS_SQL"
    """Function contains no SQL."""
    READS_SQL_DATA = "READS_SQL_DATA"
    """Function reads from SQL tables or views."""
    NO_SQL = "NO_SQL"
    """Function does not use SQL and does not access data."""


class ModelVersionStatus(enum.StrEnum):
    """The status of a model version, reflecting the lifecycle of its artifacts in
    external storage.
    """

    PENDING_REGISTRATION = "PENDING_REGISTRATION"
    """The client has not yet completely written all model artifacts to external
    storage. This is the initial status assigned on creation.
    """
    FAILED_REGISTRATION = "FAILED_REGISTRATION"
    """The client failed to write all model artifacts to external storage."""
    READY = "READY"
    """The client successfully wrote all model artifacts to external storage and
    the version has been finalized.
    """


class PolicyType(enum.StrEnum):
    """The kind of enforcement a policy applies."""

    POLICY_TYPE_ROW_FILTER = "POLICY_TYPE_ROW_FILTER"
    """Filters rows returned from the securable based on the when_condition and match_columns."""
    POLICY_TYPE_COLUMN_MASK = "POLICY_TYPE_COLUMN_MASK"
    """Masks a column's values for callers matched by the when_condition."""
    POLICY_TYPE_DENY = "POLICY_TYPE_DENY"
    """Denies the listed privileges to callers matched by the when_condition."""
    POLICY_TYPE_GRANT = "POLICY_TYPE_GRANT"
    """Grants the listed privileges to callers matched by the when_condition."""


class ProviderAuthenticationType(enum.StrEnum):
    """The delta sharing authentication type used by a provider."""

    TOKEN = "TOKEN"
    """Token-based authentication (open sharing via a credential file)."""
    OAUTH_CLIENT_CREDENTIALS = "OAUTH_CLIENT_CREDENTIALS"
    """OAuth2 client-credentials authentication."""


class AuthenticationType(enum.StrEnum):
    TOKEN = "TOKEN"
    """Basic authentication is required."""
    OAUTH_CLIENT_CREDENTIALS = "OAUTH_CLIENT_CREDENTIALS"
    """OAuth2 authentication is required."""


class Action(enum.StrEnum):
    ADD = "ADD"
    REMOVE = "REMOVE"
    UPDATE = "UPDATE"


class DataObjectStatus(enum.StrEnum):
    """Whether a shared data object is still accessible to the share."""

    ACTIVE = "ACTIVE"
    """The data object is accessible."""
    PERMISSION_DENIED = "PERMISSION_DENIED"
    """The share owner lost access to the data object."""


class DataObjectType(enum.StrEnum):
    TABLE = "TABLE"
    SCHEMA = "SCHEMA"
    VIEW = "VIEW"
    MATERIALIZED_VIEW = "MATERIALIZED_VIEW"
    STREAMING_TABLE = "STREAMING_TABLE"
    MODEL = "MODEL"
    FUNCTION = "FUNCTION"
    VOLUME = "VOLUME"
    """A Unity Catalog volume shared as a storage-backed asset (OpenSharing)."""
    AGENT_SKILL = "AGENT_SKILL"
    """An agent skill (a directory of SKILL.md + scripts/assets) shared as a
    storage-backed asset on top of a volume (OpenSharing).
    """


class HistoryStatus(enum.StrEnum):
    DISABLED = "DISABLED"
    """Data history sharing is disabled."""
    ENABLED = "ENABLED"
    """Data history sharing is enabled."""


class PartitionValueOp(enum.StrEnum):
    """The operator a partition value is matched with."""

    EQUAL = "EQUAL"
    """The partition column equals the value."""
    LIKE = "LIKE"
    """The partition column matches the value as a SQL LIKE pattern."""


class ColumnTypeName(enum.StrEnum):
    BOOLEAN = "BOOLEAN"
    BYTE = "BYTE"
    SHORT = "SHORT"
    INT = "INT"
    LONG = "LONG"
    FLOAT = "FLOAT"
    DOUBLE = "DOUBLE"
    DATE = "DATE"
    TIMESTAMP = "TIMESTAMP"
    STRING = "STRING"
    BINARY = "BINARY"
    DECIMAL = "DECIMAL"
    INTERVAL = "INTERVAL"
    ARRAY = "ARRAY"
    STRUCT = "STRUCT"
    MAP = "MAP"
    CHAR = "CHAR"
    NULL = "NULL"
    USER_DEFINED_TYPE = "USER_DEFINED_TYPE"
    TIMESTAMP_NTZ = "TIMESTAMP_NTZ"
    VARIANT = "VARIANT"
    TABLE_TYPE = "TABLE_TYPE"
    GEOMETRY = "GEOMETRY"
    GEOGRAPHY = "GEOGRAPHY"


class DataSourceFormat(enum.StrEnum):
    DELTA = "DELTA"
    ICEBERG = "ICEBERG"
    HUDI = "HUDI"
    PARQUET = "PARQUET"
    CSV = "CSV"
    JSON = "JSON"
    ORC = "ORC"
    AVRO = "AVRO"
    TEXT = "TEXT"
    UNITY_CATALOG = "UNITY_CATALOG"
    DELTASHARING = "DELTASHARING"
    DELTA_UNIFORM_ICEBERG = "DELTA_UNIFORM_ICEBERG"
    DELTA_UNIFORM_HUDI = "DELTA_UNIFORM_HUDI"
    HIVE = "HIVE"


class TableType(enum.StrEnum):
    """The type of the table."""

    MANAGED = "MANAGED"
    EXTERNAL = "EXTERNAL"
    VIEW = "VIEW"
    MATERIALIZED_VIEW = "MATERIALIZED_VIEW"
    STREAMING_TABLE = "STREAMING_TABLE"
    MANAGED_SHALLOW_CLONE = "MANAGED_SHALLOW_CLONE"
    EXTERNAL_SHALLOW_CLONE = "EXTERNAL_SHALLOW_CLONE"
    METRIC_VIEW = "METRIC_VIEW"


class TagAssignmentSourceType(enum.StrEnum):
    """The source of an entity tag assignment."""

    TAG_ASSIGNMENT_SOURCE_TYPE_SYSTEM_DATA_CLASSIFICATION = "TAG_ASSIGNMENT_SOURCE_TYPE_SYSTEM_DATA_CLASSIFICATION"
    """Assigned by system data classification."""


class GenerateTemporaryModelVersionCredentialsRequestOperation(enum.StrEnum):
    UNSPECIFIED = "UNSPECIFIED"
    """The operation is not specified."""
    READ_MODEL_VERSION = "READ_MODEL_VERSION"
    """The operation is read only."""
    READ_WRITE_MODEL_VERSION = "READ_WRITE_MODEL_VERSION"
    """The operation is read and write."""


class GenerateTemporaryPathCredentialsRequestOperation(enum.StrEnum):
    UNSPECIFIED = "UNSPECIFIED"
    """The operation is not specified."""
    PATH_READ = "PATH_READ"
    """The operation is read only."""
    PATH_READ_WRITE = "PATH_READ_WRITE"
    """The operation is read and write."""
    PATH_CREATE_TABLE = "PATH_CREATE_TABLE"
    """The operation creates a table at the path."""


class GenerateTemporaryTableCredentialsRequestOperation(enum.StrEnum):
    UNSPECIFIED = "UNSPECIFIED"
    """The operation is not specified."""
    READ = "READ"
    """The operation is read only."""
    READ_WRITE = "READ_WRITE"
    """The operation is read and write."""


class GenerateTemporaryVolumeCredentialsRequestOperation(enum.StrEnum):
    UNSPECIFIED = "UNSPECIFIED"
    """The operation is not specified."""
    READ_VOLUME = "READ_VOLUME"
    """The operation is read only."""
    WRITE_VOLUME = "WRITE_VOLUME"
    """The operation is read and write."""


class VolumeType(enum.StrEnum):
    EXTERNAL = "EXTERNAL"
    MANAGED = "MANAGED"
