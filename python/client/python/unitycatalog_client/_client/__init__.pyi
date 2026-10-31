"""
The extension module `unitycatalog_client._client`.
"""

from . import agent_skills as agent_skills, agents as agents, catalogs as catalogs, credentials as credentials, external_locations as external_locations, functions as functions, model_versions as model_versions, policies as policies, providers as providers, recipients as recipients, registered_models as registered_models, schemas as schemas, shares as shares, staging_tables as staging_tables, tables as tables, tags as tags, temporary_credentials as temporary_credentials, volumes as volumes
from .agent_skills import AgentSkill, AgentSkillClient
from .agents import Agent, AgentClient
from .catalogs import Catalog, CatalogClient
from .credentials import AwsIamRoleConfig, AzureManagedIdentity, AzureServicePrincipal, AzureStorageKey, Credential, CredentialClient, DatabricksGcpServiceAccount
from .external_locations import ExternalLocation, ExternalLocationClient
from .functions import CreateFunction, Function, FunctionClient
from .model_versions import ModelVersion
from .policies import FunctionRef, MatchColumn, PolicyClient, PolicyInfo, PrivilegeOptions
from .providers import Provider, ProviderClient
from .recipients import Recipient, RecipientClient, SecurablePropertiesKvPairs
from .registered_models import RegisteredModel, RegisteredModelClient
from .schemas import Schema, SchemaClient
from .shares import Share, ShareClient
from .staging_tables import StagingTable, StagingTableClient
from .tables import Column, DependencyList, Table, TableClient
from .tags import EntityTagAssignment, ListEntityTagAssignmentsResponse, TagPolicy, TagPolicyClient, Value
from .temporary_credentials import TemporaryCredential
from .volumes import Volume, VolumeClient
from _typeshed import SupportsGetItem
from typing import Final, SupportsIndex, final
from typing_extensions import disjoint_base
from unitycatalog_client._enums import AgentSkillType, AuthenticationType, DataSourceFormat, GenerateTemporaryModelVersionCredentialsRequestOperation, GenerateTemporaryPathCredentialsRequestOperation, GenerateTemporaryTableCredentialsRequestOperation, GenerateTemporaryVolumeCredentialsRequestOperation, InvocationProtocol, PolicyType, ProviderAuthenticationType, Purpose, TableType, VolumeType

__all__ = ["__version__", "BadRequestError", "ConflictError", "NotFoundError", "PermissionDeniedError", "RateLimitError", "ServerError", "TemporaryCredentialClient", "UnauthenticatedError", "UnityCatalogClient", "UnityCatalogError", "parse_uc_url", "agent_skills", "agents", "catalogs", "credentials", "external_locations", "functions", "model_versions", "policies", "providers", "recipients", "registered_models", "schemas", "shares", "staging_tables", "tables", "tags", "temporary_credentials", "volumes"]

__version__: Final[str]
"""
The version of the bindings crate.
"""

@final
class BadRequestError(UnityCatalogError):
    """
    The server answered `400 Bad Request`: the request is not valid.
    """
    def __new__(cls, /, message: str, code: str | None = None, status: SupportsIndex | None = None, details: list | None = None) -> BadRequestError: ...

@final
class ConflictError(UnityCatalogError):
    """
    The server answered `409 Conflict`: the resource exists or changed.
    """
    def __new__(cls, /, message: str, code: str | None = None, status: SupportsIndex | None = None, details: list | None = None) -> ConflictError: ...

@final
class NotFoundError(UnityCatalogError):
    """
    The server answered `404 Not Found`: the resource does not exist.
    """
    def __new__(cls, /, message: str, code: str | None = None, status: SupportsIndex | None = None, details: list | None = None) -> NotFoundError: ...

@final
class PermissionDeniedError(UnityCatalogError):
    """
    The server answered `403 Forbidden`: the caller may not do this.
    """
    def __new__(cls, /, message: str, code: str | None = None, status: SupportsIndex | None = None, details: list | None = None) -> PermissionDeniedError: ...

@final
class RateLimitError(UnityCatalogError):
    """
    The server answered `429 Too Many Requests`.
    """
    def __new__(cls, /, message: str, code: str | None = None, status: SupportsIndex | None = None, details: list | None = None) -> RateLimitError: ...

@final
class ServerError(UnityCatalogError):
    """
    The server answered with a `5xx` status: it failed to do the request.
    """
    def __new__(cls, /, message: str, code: str | None = None, status: SupportsIndex | None = None, details: list | None = None) -> ServerError: ...

@final
class TemporaryCredentialClient:
    """
    Client for vending short-lived credentials for tables, volumes, and
    raw external paths governed by Unity Catalog.

    Construct directly with `TemporaryCredentialClient(base_url=...,
    token=...)`, or use the convenience helpers in
    [`unitycatalog_client.obstore`](crate::obstore) to plug the credentials
    into the `obstore` Python library.
    """
    def __new__(cls, /, base_url: str, token: str | None = None) -> TemporaryCredentialClient: ...
    def temporary_model_version_credential(self, /, full_name: str, version: SupportsIndex, operation: str) -> TemporaryCredential:
        """
        Vend a temporary credential for a Unity Catalog model version.

        `full_name` is the three-level `catalog.schema.model` name of the parent
        registered model; `version` is its integer version number. Server support
        requires the metastore's `external_access_enabled` flag and the caller's
        `EXTERNAL_USE_SCHEMA` privilege.
        """
    def temporary_path_credential(self, /, path: str, operation: str, dry_run: bool | None = None) -> tuple[TemporaryCredential, str]: ...
    def temporary_table_credential(self, /, table: str, operation: str) -> tuple[TemporaryCredential, str]:
        """
        Vend a temporary credential for a Unity Catalog table.
        """
    def temporary_volume_credential(self, /, volume: str, operation: str) -> tuple[TemporaryCredential, str]:
        """
        Vend a temporary credential for a Unity Catalog volume.

        `volume` is the three-level `catalog.schema.volume` name. Server
        support requires the metastore's `external_access_enabled` flag and
        the caller's `EXTERNAL_USE_SCHEMA` privilege.
        """

@final
class UnauthenticatedError(UnityCatalogError):
    """
    The server answered `401 Unauthorized`: the request has no valid credentials.
    """
    def __new__(cls, /, message: str, code: str | None = None, status: SupportsIndex | None = None, details: list | None = None) -> UnauthenticatedError: ...

@final
class UnityCatalogClient:
    """
    A client of the API at one base URL.
    """
    def __new__(cls, /, base_url: str, token: str | None = None) -> UnityCatalogClient:
        """
        A client of the API at `base_url`. `token`, when set, goes in every
        request as the bearer token.
        """
    def agent(self, /, catalog_name: str, schema_name: str, agent_name: str) -> AgentClient:
        """
        The requests on one `Agent`. It sends no request itself.
        """
    def agent_skill(self, /, catalog_name: str, schema_name: str, agent_skill_name: str) -> AgentSkillClient:
        """
        The requests on one `AgentSkill`. It sends no request itself.
        """
    def catalog(self, /, catalog_name: str) -> CatalogClient:
        """
        The requests on one `Catalog`. It sends no request itself.
        """
    def create_agent(self, /, catalog_name: str, schema_name: str, name: str, invocation_protocol: InvocationProtocol | str, endpoint: str, description: str | None = None, capabilities: SupportsGetItem[int, str] | None = None, input_schema: str | None = None, comment: str | None = None) -> Agent:
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
    def create_agent_skill(self, /, catalog_name: str, schema_name: str, name: str, agent_skill_type: AgentSkillType | str, storage_location: str | None = None, description: str | None = None, license: str | None = None, allowed_tools: SupportsGetItem[int, str] | None = None, metadata: dict[str, str] | None = None, comment: str | None = None) -> AgentSkill:
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
    def create_catalog(self, /, name: str, comment: str | None = None, properties: dict[str, str] | None = None, storage_root: str | None = None, provider_name: str | None = None, share_name: str | None = None) -> Catalog:
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
    def create_credential(self, /, name: str, purpose: Purpose | str, comment: str | None = None, read_only: bool | None = None, skip_validation: bool | None = None, azure_service_principal: AzureServicePrincipal | None = None, azure_managed_identity: AzureManagedIdentity | None = None, azure_storage_key: AzureStorageKey | None = None, aws_iam_role: AwsIamRoleConfig | None = None, databricks_gcp_service_account: DatabricksGcpServiceAccount | None = None) -> Credential:
        """
        Args:
            name: The credential name. The name must be unique among storage and service credentials within
                  the metastore.
            purpose: Indicates the purpose of the credential.
            comment: Comment associated with the credential.
            read_only: Whether the credential is usable only for read operations. Only applicable when
                       purpose is STORAGE.
            skip_validation: Supplying true to this argument skips validation of the created set of
                             credentials. Defaults to false.
            azure_service_principal: The Azure service principal configuration.
            azure_managed_identity: The Azure managed identity configuration.
            azure_storage_key: The Azure storage key configuration.
            aws_iam_role: The AWS IAM role configuration.
            databricks_gcp_service_account: The Databricks managed GCP service account configuration.

        Returns:
            A credential used to access external data sources or services.
        """
    def create_entity_tag_assignment(self, /, entity_type: str, entity_name: str, tag_key: str, tag_value: str | None = None) -> EntityTagAssignment:
        """
        Create an entity tag assignment

        Assigns a tag to a Unity Catalog entity.

        Args:
            entity_type: The type of the entity to which the tag is assigned. Supported values: catalogs,
                         schemas, tables, columns, volumes.
            entity_name: The fully qualified name of the entity to which the tag is assigned.
            tag_key: The key of the tag.
            tag_value: The value of the tag.

        Returns:
            The assignment of a tag to a Unity Catalog entity. Unlike a TagPolicy (a governed-tag
            *definition*), an assignment is the application of a tag to a specific securable. It has no
            identifier of its own — its identity is the composite of (entity_type, entity_name,
            tag_key). It
            is intentionally NOT a `google.api.resource`: assignments are stored as associations between
            the
            entity and its tag, not as standalone objects.
        """
    def create_external_location(self, /, name: str, url: str, credential_name: str, read_only: bool | None = None, comment: str | None = None, skip_validation: bool | None = None) -> ExternalLocation:
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
    def create_function(self, /, function_info: CreateFunction) -> Function:
        """
        Create a function

        Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION privilege
        on the parent catalog and schema.

        Args:
            function_info: The function to create.

        Returns:
            A User-Defined Function (UDF) registered under a catalog + schema hierarchy.
        """
    def create_model_version(self, /, model_name: str, catalog_name: str, schema_name: str, source: str, run_id: str | None = None, comment: str | None = None) -> ModelVersion:
        """
        Create a model version

        Creates a new model version in PENDING_REGISTRATION status. The server assigns the version number
        and a storage location for the artifacts. The caller must be a metastore admin or the owner of the
        parent registered model.

        Args:
            model_name: Name of the parent registered model, relative to parent schema.
            catalog_name: Name of parent catalog.
            schema_name: Name of parent schema.
            source: URI indicating the location of the source artifacts (e.g. an MLflow run artifact path)
                    used to create the model version.
            run_id: The run id used by the ML package that generated this model.
            comment: User-provided free-form text description.

        Returns:
            A model version under a registered model. A version is identified by its parent
            model's three-
            level name plus an integer `version` number assigned by the server. It carries its own
            artifact
            storage location and moves through the PENDING_REGISTRATION -> READY lifecycle via the
            finalize
            operation.
        """
    def create_policy(self, /, name: str, on_securable_type: str, on_securable_fullname: str, policy_type: PolicyType | str, comment: str | None = None, to_principals: SupportsGetItem[int, str] | None = None, except_principals: SupportsGetItem[int, str] | None = None, for_securable_type: str | None = None, when_condition: str | None = None, match_columns: SupportsGetItem[int, MatchColumn] | None = None, row_filter: FunctionRef | None = None, column_mask: FunctionRef | None = None, deny: PrivilegeOptions | None = None, grant: PrivilegeOptions | None = None) -> PolicyInfo:
        """
        Create a new policy

        Creates a new row-filter or column-mask policy on the specified securable.

        Args:
            name: The name of the policy. Unique within the securable it is defined on.
            on_securable_type: The type of securable the policy is defined on. Supported values: CATALOG,
                               SCHEMA, TABLE.
            on_securable_fullname: The fully qualified name of the securable the policy is defined on.
            policy_type: The kind of enforcement this policy applies.
            comment: User-provided free-form text description of the policy.
            to_principals: The principals the policy applies to. Empty means "all principals".
            except_principals: The principals explicitly excluded from the policy, even if matched by
                               to_principals.
            for_securable_type: The type of securables the policy takes effect on.
            when_condition: An opaque condition expression evaluated by the query engine at enforcement
                            time.
            match_columns: The columns referenced by the policy function, and the aliases they are bound to.
            row_filter: The row-filter function to invoke. Set only when policy_type is
                        POLICY_TYPE_ROW_FILTER.
            column_mask: The column-mask function to invoke. Set only when policy_type is
                         POLICY_TYPE_COLUMN_MASK.
            deny: The privileges to deny. Set only when policy_type is POLICY_TYPE_DENY.
            grant: The privileges to grant. Set only when policy_type is POLICY_TYPE_GRANT.

        Returns:
            A row-filter or column-mask policy bound to principals and scoped to a securable. Policies
            are
            read as whole documents; enforcement is performed by the query engine, not by this service.
            `when_condition` is stored and returned opaque — this service does not parse or evaluate it.
        """
    def create_provider(self, /, name: str, authentication_type: ProviderAuthenticationType | str, owner: str | None = None, comment: str | None = None, recipient_profile_str: str | None = None, properties: dict[str, str] | None = None) -> Provider:
        """
        Create a new provider.

        Args:
            name: Name of the provider.
            authentication_type: The delta sharing authentication type.
            owner: Username of the provider owner.
            comment: Description about the provider.
            recipient_profile_str: The recipient profile (credential file contents) used to connect to
                                   the sharing server. Required when authentication_type is TOKEN or
                                   OAUTH_CLIENT_CREDENTIALS.
            properties: Provider properties as map of string key-value pairs.

        Returns:
            A provider represents an organization sharing data with this metastore. A provider is the
            inbound counterpart of a recipient: it is registered from a share activation/ credential
            file
            and is used to access shares offered by an upstream Delta Sharing server.
        """
    def create_recipient(self, /, name: str, authentication_type: AuthenticationType | str, owner: str, comment: str | None = None, properties_kvpairs: SecurablePropertiesKvPairs | None = None, expiration_time: SupportsIndex | None = None) -> Recipient:
        """
        Create a new recipient.

        Args:
            name: Name of the recipient.
            authentication_type: The delta sharing authentication type.
            owner: Username of the recipient owner.
            comment: Description about the recipient.
            properties_kvpairs: Recipient properties as map of string key-value pairs. When provided
                                in update request, the specified properties will override the existing
                                properties. To add and remove properties, one would need to perform a read-
                                modify-write.
            expiration_time: Expiration timestamp of the token, in epoch milliseconds.

        Returns:
            The requested resource
        """
    def create_registered_model(self, /, name: str, catalog_name: str, schema_name: str, comment: str | None = None) -> RegisteredModel:
        """
        Create a registered model

        Creates a new registered model. The caller must be a metastore admin or have the CREATE_MODEL
        privilege on the parent catalog and schema.

        Args:
            name: Name of registered model, relative to parent schema.
            catalog_name: Name of parent catalog.
            schema_name: Name of parent schema.
            comment: User-provided free-form text description.

        Returns:
            A registered model within the three-level namespace (catalog.schema.model). Registered
            models
            group a collection of model versions. Individual versions carry their own artifact storage
            location and are created, finalized, and governed independently.
        """
    def create_schema(self, /, name: str, catalog_name: str, comment: str | None = None, properties: dict[str, str] | None = None, storage_root: str | None = None) -> Schema:
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
    def create_share(self, /, name: str, comment: str | None = None, storage_root: str | None = None) -> Share:
        """
        Create a new share.

        Args:
            name: Name of the share.
            comment: User-provided free-form text description.
            storage_root: Storage root URL for the share.

        Returns:
            The requested resource
        """
    def create_staging_table(self, /, name: str, catalog_name: str, schema_name: str) -> StagingTable:
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
    def create_table(self, /, name: str, schema_name: str, catalog_name: str, table_type: TableType | str, data_source_format: DataSourceFormat | str, columns: SupportsGetItem[int, Column] | None = None, storage_location: str | None = None, comment: str | None = None, properties: dict[str, str] | None = None, view_definition: str | None = None, view_dependencies: DependencyList | None = None) -> Table:
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
    def create_tag_policy(self, /, tag_key: str, description: str | None = None, values: SupportsGetItem[int, Value] | None = None) -> TagPolicy:
        """
        Create a new tag policy

        Creates a new governed tag definition.

        Args:
            tag_key: The key of the governed tag.
            description: User-provided free-form text description of the tag policy.
            values: The set of allowed values for the governed tag. The list is unordered.

        Returns:
            A governed tag definition (tag policy). A tag policy defines a tag key together with the
            rules
            that govern how it can be used, including the optional set of allowed values. Assigning a
            governed tag to an entity is done through the Entity Tag Assignments API.
        """
    def create_volume(self, /, catalog_name: str, schema_name: str, name: str, volume_type: VolumeType | str, storage_location: str | None = None, comment: str | None = None) -> Volume:
        """
        Args:
            catalog_name: The identifier of the catalog
            schema_name: The identifier of the schema
            name: The identifier of the volume
            volume_type: The type of the volume. An external volume is located in the specified external
                         location. A managed volume is located in the default location which is specified by
                         the parent schema, or the parent catalog, or the Metastore.
            storage_location: The storage location on the cloud
            comment: The comment attached to the volume.

        Returns:
            The requested resource
        """
    def credential(self, /, credential_name: str) -> CredentialClient:
        """
        The requests on one `Credential`. It sends no request itself.
        """
    def delete_entity_tag_assignment(self, /, entity_type: str, entity_name: str, tag_key: str) -> None:
        """
        Delete an entity tag assignment

        Deletes the tag assignment for the specified entity and tag key.
        """
    def delete_model_version(self, /, full_name: str, version: SupportsIndex) -> None:
        """
        Delete a model version

        Deletes the model version that matches the supplied name and version. For the deletion to succeed,
        the caller must be the owner of the parent registered model.
        """
    def external_location(self, /, external_location_name: str) -> ExternalLocationClient:
        """
        The requests on one `ExternalLocation`. It sends no request itself.
        """
    def finalize_model_version(self, /, full_name: str, version: SupportsIndex) -> ModelVersion:
        """
        Finalize a model version

        Transitions a model version to READY once all artifacts have been written to its storage location.

        Returns:
            A model version under a registered model. A version is identified by its parent
            model's three-
            level name plus an integer `version` number assigned by the server. It carries its own
            artifact
            storage location and moves through the PENDING_REGISTRATION -> READY lifecycle via the
            finalize
            operation.
        """
    def function(self, /, catalog_name: str, schema_name: str, function_name: str) -> FunctionClient:
        """
        The requests on one `Function`. It sends no request itself.
        """
    def generate_temporary_model_version_credentials(self, /, catalog_name: str, schema_name: str, model_name: str, version: SupportsIndex, operation: GenerateTemporaryModelVersionCredentialsRequestOperation | str) -> TemporaryCredential:
        """
        Generate a new set of credentials for a model version.

        The metastore must have the `external_access_enabled` flag set to true (default false). The caller
        must have the `EXTERNAL_USE_SCHEMA` privilege on the parent schema (granted by a catalog owner).

        Args:
            catalog_name: Name of parent catalog of the model version.
            schema_name: Name of parent schema of the model version.
            model_name: Name of the parent registered model.
            version: The integer version number of the model version.
            operation: The operation performed against the model version data, either READ_MODEL_VERSION or
                       READ_WRITE_MODEL_VERSION. If READ_WRITE_MODEL_VERSION is specified, the credentials
                       returned will have write permissions, otherwise, it will be read only.

        Returns:
            The response to the GenerateTemporaryTableCredentialsRequest.
        """
    def generate_temporary_path_credentials(self, /, url: str, operation: GenerateTemporaryPathCredentialsRequestOperation | str, dry_run: bool | None = None) -> TemporaryCredential:
        """
        Generate a new set of credentials for a path.

        Only external storage paths (external locations and external tables) are supported; managed tables
        are not. The metastore must have the `external_access_enabled` flag set to true (default false).
        The caller must have the `EXTERNAL_USE_LOCATION` privilege on the external location (granted by its
        owner) and, for existing external tables, the `EXTERNAL_USE_SCHEMA` privilege on the parent schema.

        Args:
            url: URL for path-based access.
            operation: The operation being performed on the path.
            dry_run: When set to true, the service will not validate that the generated credentials can
                     perform write operations, therefore no new paths will be created and the response will
                     not contain valid credentials. Defaults to false.

        Returns:
            The response to the GenerateTemporaryTableCredentialsRequest.
        """
    def generate_temporary_table_credentials(self, /, table_id: str, operation: GenerateTemporaryTableCredentialsRequestOperation | str) -> TemporaryCredential:
        """
        Generate a new set of credentials for a table.

        The metastore must have the `external_access_enabled` flag set to true (default false). The caller
        must have the `EXTERNAL_USE_SCHEMA` privilege on the parent schema (granted by a catalog owner).

        Args:
            table_id: UUID of the table to read or write.
            operation: The operation performed against the table data, either READ or READ_WRITE. If
                       READ_WRITE is specified, the credentials returned will have write permissions,
                       otherwise, it will be read only.

        Returns:
            The response to the GenerateTemporaryTableCredentialsRequest.
        """
    def generate_temporary_volume_credentials(self, /, volume_id: str, operation: GenerateTemporaryVolumeCredentialsRequestOperation | str) -> TemporaryCredential:
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
    def get_entity_tag_assignment(self, /, entity_type: str, entity_name: str, tag_key: str) -> EntityTagAssignment:
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
    def get_model_version(self, /, full_name: str, version: SupportsIndex, include_browse: bool | None = None) -> ModelVersion:
        """
        Get a model version

        Gets a model version by its parent model name and version number.

        Args:
            include_browse: Whether to include model versions in the response for which the principal can
                            only access selective metadata for.

        Returns:
            A model version under a registered model. A version is identified by its parent
            model's three-
            level name plus an integer `version` number assigned by the server. It carries its own
            artifact
            storage location and moves through the PENDING_REGISTRATION -> READY lifecycle via the
            finalize
            operation.
        """
    def list_agent_skills(self, /, catalog_name: str, schema_name: str, max_results: SupportsIndex | None = None, include_browse: bool | None = None) -> list[AgentSkill]:
        """
        Lists agent skills.

        Args:
            catalog_name: The identifier of the catalog.
            schema_name: The identifier of the schema.
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include agent skills in the response for which the principal can only
                            access selective metadata for.

        Returns:
            The agent skills returned.
        """
    def list_agents(self, /, catalog_name: str, schema_name: str, max_results: SupportsIndex | None = None, include_browse: bool | None = None) -> list[Agent]:
        """
        Lists agents.

        Args:
            catalog_name: The identifier of the catalog.
            schema_name: The identifier of the schema.
            max_results: The maximum number of results per page that should be returned.
            include_browse: Whether to include agents in the response for which the principal can only
                            access selective metadata for.

        Returns:
            The agents returned.
        """
    def list_catalogs(self, /, max_results: SupportsIndex | None = None) -> list[Catalog]:
        """
        List catalogs

        Gets an array of catalogs in the metastore. If the caller is the metastore admin, all catalogs
        will be retrieved. Otherwise, only catalogs owned by the caller (or for which the caller has the
        USE_CATALOG privilege) will be retrieved. There is no guarantee of a specific ordering of the
        elements in the array.

        Args:
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.

        Returns:
            The catalogs returned.
        """
    def list_credentials(self, /, purpose: Purpose | str | None = None, max_results: SupportsIndex | None = None) -> list[Credential]:
        """
        Args:
            purpose: Return only credentials for the specified purpose.
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.

        Returns:
            The credentials returned.
        """
    def list_entity_tag_assignments(self, /, entity_type: str, entity_name: str, max_results: SupportsIndex | None = None, page_token: str | None = None) -> ListEntityTagAssignmentsResponse:
        """
        List entity tag assignments

        Gets the tag assignments for the specified entity.

        Args:
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.
            page_token: Opaque pagination token to go to next page based on previous query.

        Returns:
            List entity tag assignments response.
        """
    def list_external_locations(self, /, max_results: SupportsIndex | None = None, include_browse: bool | None = None) -> list[ExternalLocation]:
        """
        List external locations

        Args:
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for

        Returns:
            The external locations returned.
        """
    def list_functions(self, /, catalog_name: str, schema_name: str, max_results: SupportsIndex | None = None, include_browse: bool | None = None) -> list[Function]:
        """
        List functions

        List functions within the specified parent catalog and schema. If the caller is the metastore admin,
        all functions are returned in the response. Otherwise, the caller must have USE_CATALOG on the
        parent catalog and USE_SCHEMA on the parent schema, and the function must either be owned by the
        caller or have SELECT on the function.

        Args:
            catalog_name: Name of parent catalog for functions of interest.
            schema_name: Parent schema of functions.
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.
            include_browse: Whether to include functions in the response for which the principal can only
                            access selective metadata for.

        Returns:
            The functions returned.
        """
    def list_model_versions(self, /, full_name: str, max_results: SupportsIndex | None = None, include_browse: bool | None = None) -> list[ModelVersion]:
        """
        List model versions

        List the model versions of the specified registered model. If the caller is the metastore admin,
        all model versions are returned. Otherwise, the caller must have the appropriate privileges on the
        parent model.

        Args:
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.
            include_browse: Whether to include model versions in the response for which the principal can
                            only access selective metadata for.

        Returns:
            The model versions returned.
        """
    def list_providers(self, /, max_results: SupportsIndex | None = None) -> list[Provider]:
        """
        List providers.

        Args:
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.

        Returns:
            List of providers.
        """
    def list_recipients(self, /, max_results: SupportsIndex | None = None) -> list[Recipient]:
        """
        List recipients.

        Args:
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.

        Returns:
            List of recipients.
        """
    def list_registered_models(self, /, catalog_name: str | None = None, schema_name: str | None = None, max_results: SupportsIndex | None = None, include_browse: bool | None = None) -> list[RegisteredModel]:
        """
        List registered models

        List registered models within the specified parent catalog and schema. If the caller is the
        metastore admin, all registered models are returned in the response. Otherwise, the caller must have
        USE_CATALOG on the parent catalog and USE_SCHEMA on the parent schema, and the model must either be
        owned by the caller or the caller must have a privilege on the model.

        Args:
            catalog_name: Name of parent catalog for models of interest.
            schema_name: Name of parent schema for models of interest.
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.
            include_browse: Whether to include registered models in the response for which the principal can
                            only access selective metadata for.

        Returns:
            The registered models returned.
        """
    def list_schemas(self, /, catalog_name: str, max_results: SupportsIndex | None = None, include_browse: bool | None = None) -> list[Schema]:
        """
        Gets an array of schemas for a catalog in the metastore. If the caller is the metastore admin or the
        owner of the parent catalog, all schemas for the catalog will be retrieved. Otherwise, only schemas
        owned by the caller (or for which the caller has the USE_SCHEMA privilege) will be retrieved. There
        is no guarantee of a specific ordering of the elements in the array.

        Args:
            catalog_name: Name of parent catalog.
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.
            include_browse: Whether to include schemas in the response for which the principal can only
                            access selective metadata for

        Returns:
            The schemas returned.
        """
    def list_shares(self, /, max_results: SupportsIndex | None = None) -> list[Share]:
        """
        List shares.

        Args:
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.

        Returns:
            List of shares.
        """
    def list_tables(self, /, catalog_name: str, schema_name: str, max_results: SupportsIndex | None = None, include_delta_metadata: bool | None = None, omit_columns: bool | None = None, omit_properties: bool | None = None, omit_username: bool | None = None, include_browse: bool | None = None, include_manifest_capabilities: bool | None = None) -> list[Table]:
        """
        Gets an array of all tables for the current metastore under the parent catalog and schema.

        The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table. For
        the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the parent
        catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a specific
        ordering of the elements in the array.

        Args:
            catalog_name: Name of parent catalog for tables of interest.
            schema_name: Name of parent schema for tables of interest.
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.
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
            The tables returned.
        """
    def list_tag_policies(self, /, max_results: SupportsIndex | None = None) -> list[TagPolicy]:
        """
        List tag policies

        Gets an array of tag policies. There is no guarantee of a specific ordering of the elements in the
        array.

        Args:
            max_results: The maximum number of results to return in this request. Fewer results may be
                         returned than requested. If unset or 0, this defaults to 1000; larger values are
                         coerced down to 1000. Negative values are rejected.

        Returns:
            The tag policies returned.
        """
    def list_volumes(self, /, catalog_name: str, schema_name: str, max_results: SupportsIndex | None = None, include_browse: bool | None = None) -> list[Volume]:
        """
        Lists volumes.

        Args:
            catalog_name: The identifier of the catalog
            schema_name: The identifier of the schema
            max_results: The maximum number of results per page that should be returned. If unset or 0, the
                         server's default page size is used. Negative values are rejected.
            include_browse: Whether to include volumes in the response for which the principal can only
                            access selective metadata for

        Returns:
            The volumes returned.
        """
    def policy(self, /, policy_name: str) -> PolicyClient:
        """
        The requests on one `PolicyInfo`. It sends no request itself.
        """
    def provider(self, /, provider_name: str) -> ProviderClient:
        """
        The requests on one `Provider`. It sends no request itself.
        """
    def recipient(self, /, recipient_name: str) -> RecipientClient:
        """
        The requests on one `Recipient`. It sends no request itself.
        """
    def registered_model(self, /, catalog_name: str, schema_name: str, registered_model_name: str) -> RegisteredModelClient:
        """
        The requests on one `RegisteredModel`. It sends no request itself.
        """
    def schema(self, /, catalog_name: str, schema_name: str) -> SchemaClient:
        """
        The requests on one `Schema`. It sends no request itself.
        """
    def share(self, /, share_name: str) -> ShareClient:
        """
        The requests on one `Share`. It sends no request itself.
        """
    def staging_table(self, /, staging_table_name: str) -> StagingTableClient:
        """
        The requests on one `StagingTable`. It sends no request itself.
        """
    def table(self, /, catalog_name: str, schema_name: str, table_name: str) -> TableClient:
        """
        The requests on one `Table`. It sends no request itself.
        """
    def tag_policy(self, /, tag_policy_name: str) -> TagPolicyClient:
        """
        The requests on one `TagPolicy`. It sends no request itself.
        """
    def update_entity_tag_assignment(self, /, entity_type: str, entity_name: str, tag_key: str, tag_value: str | None = None, update_mask: str | None = None) -> EntityTagAssignment:
        """
        Update an entity tag assignment

        Updates the tag assignment for the specified entity and tag key.

        Args:
            tag_value: The value of the tag.
            update_mask: The fields to update, as a comma-separated list of field names. If empty or "*",
                         every field set in the request is updated. Otherwise only the listed fields are
                         updated, and a listed field that is unset in the request is cleared.

        Returns:
            The assignment of a tag to a Unity Catalog entity. Unlike a TagPolicy (a governed-tag
            *definition*), an assignment is the application of a tag to a specific securable. It has no
            identifier of its own — its identity is the composite of (entity_type, entity_name,
            tag_key). It
            is intentionally NOT a `google.api.resource`: assignments are stored as associations between
            the
            entity and its tag, not as standalone objects.
        """
    def update_model_version(self, /, full_name: str, version: SupportsIndex, comment: str | None = None) -> ModelVersion:
        """
        Update a model version

        Updates the model version that matches the supplied name and version.

        Args:
            comment: User-provided free-form text description.

        Returns:
            A model version under a registered model. A version is identified by its parent
            model's three-
            level name plus an integer `version` number assigned by the server. It carries its own
            artifact
            storage location and moves through the PENDING_REGISTRATION -> READY lifecycle via the
            finalize
            operation.
        """
    def volume(self, /, catalog_name: str, schema_name: str, volume_name: str) -> VolumeClient:
        """
        The requests on one `Volume`. It sends no request itself.
        """

@disjoint_base
class UnityCatalogError(Exception):
    """
    An error answer of the server, or a request that got no answer.
    """
    def __init__(self, /, message: str, code: str | None = None, status: SupportsIndex | None = None, details: list | None = None) -> None:
        """
        Sets `args` to the message, as `BaseException.__init__` (which takes no keyword
        arguments) would.
        """
    def __new__(cls, /, message: str, code: str | None = None, status: SupportsIndex | None = None, details: list | None = None) -> UnityCatalogError: ...
    def __reduce__(self, /) -> tuple[type, tuple[str, str | None, int | None, list]]:
        """
        For `pickle`.
        """
    def __repr__(self, /) -> str: ...
    def __str__(self, /) -> str: ...
    @property
    def code(self, /) -> str | None:
        """
        The error code of the answer, such as `NOT_FOUND`, if any.
        """
    @property
    def details(self, /) -> list:
        """
        The details of the answer.
        """
    @property
    def message(self, /) -> str:
        """
        The error message.
        """
    @property
    def status(self, /) -> int | None:
        """
        The HTTP status of the answer. `None` when the request got no answer.
        """

def parse_uc_url(url: str, /) -> tuple[str, dict[str, str]]:
    """
    Parse a Unity Catalog URL into a `(kind, payload)` tuple.

    `kind` is one of:

    * ``"volume"`` — payload has `catalog`, `schema`, `volume`, `path` keys
      (`path` is the empty string when the URL addresses the volume root).
    * ``"table"``  — payload has `catalog`, `schema`, `table` keys.
    * ``"path"``   — payload has a single `url` key carrying the raw cloud
      URL string.

    Raises a `UnityCatalogError` on invalid input.
    """
