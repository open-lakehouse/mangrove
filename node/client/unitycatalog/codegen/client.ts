// @generated — do not edit by hand.
import { fromBinary, toBinary } from "@bufbuild/protobuf";
import {
  type Agent,
  AgentSchema,
  type AgentSkill,
  AgentSkillSchema,
  type Catalog,
  CatalogSchema,
  type Credential,
  CredentialSchema,
  type EntityTagAssignment,
  EntityTagAssignmentSchema,
  type ExternalLocation,
  ExternalLocationSchema,
  type Function as Function$,
  FunctionSchema,
  type GetCommitsResponse,
  GetCommitsResponseSchema,
  type ListEntityTagAssignmentsResponse,
  ListEntityTagAssignmentsResponseSchema,
  type Provider,
  ProviderSchema,
  type Recipient,
  RecipientSchema,
  type Schema,
  SchemaSchema,
  type Share,
  ShareSchema,
  type StagingTable,
  StagingTableSchema,
  type Table,
  TableSchema,
  type TagPolicy,
  TagPolicySchema,
  type TemporaryCredential,
  TemporaryCredentialSchema,
  type Volume,
  VolumeSchema,
} from "./models";
import {
  type NapiAgentClient as NativeAgentClient,
  type NapiAgentSkillClient as NativeAgentSkillClient,
  type NapiCatalogClient as NativeCatalogClient,
  NapiUnityCatalogClient as NativeClient,
  type NapiCredentialClient as NativeCredentialClient,
  type NapiExternalLocationClient as NativeExternalLocationClient,
  type NapiFunctionClient as NativeFunctionClient,
  type NapiProviderClient as NativeProviderClient,
  type NapiRecipientClient as NativeRecipientClient,
  type NapiSchemaClient as NativeSchemaClient,
  type NapiShareClient as NativeShareClient,
  type NapiStagingTableClient as NativeStagingTableClient,
  type NapiTableClient as NativeTableClient,
  type NapiTagPolicyClient as NativeTagPolicyClient,
  type NapiVolumeClient as NativeVolumeClient,
} from "./native";

// ── UnityCatalogError error hierarchy ────────────────────────────────────────────────────────

/** Base class for all UnityCatalogError errors. */
export class UnityCatalogError extends Error {
  readonly errorCode: string;
  constructor(message: string, errorCode: string) {
    super(message);
    this.name = "UnityCatalogError";
    this.errorCode = errorCode;
  }
}

export class NotFoundError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "RESOURCE_NOT_FOUND");
    this.name = "NotFoundError";
  }
}

export class AlreadyExistsError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "RESOURCE_ALREADY_EXISTS");
    this.name = "AlreadyExistsError";
  }
}

export class PermissionDeniedError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "PERMISSION_DENIED");
    this.name = "PermissionDeniedError";
  }
}

export class UnauthenticatedError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "UNAUTHENTICATED");
    this.name = "UnauthenticatedError";
  }
}

export class InvalidParameterError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "INVALID_PARAMETER_VALUE");
    this.name = "InvalidParameterError";
  }
}

export class RequestLimitError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "REQUEST_LIMIT_EXCEEDED");
    this.name = "RequestLimitError";
  }
}

export class InternalServerError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "INTERNAL_ERROR");
    this.name = "InternalServerError";
  }
}

export class ServiceUnavailableError extends UnityCatalogError {
  constructor(message: string) {
    super(message, "TEMPORARILY_UNAVAILABLE");
    this.name = "ServiceUnavailableError";
  }
}

type ErrorConstructor = new (message: string) => UnityCatalogError;

const ERROR_MAP: Record<string, ErrorConstructor> = {
  RESOURCE_NOT_FOUND: NotFoundError,
  RESOURCE_ALREADY_EXISTS: AlreadyExistsError,
  PERMISSION_DENIED: PermissionDeniedError,
  UNAUTHENTICATED: UnauthenticatedError,
  INVALID_PARAMETER_VALUE: InvalidParameterError,
  REQUEST_LIMIT_EXCEEDED: RequestLimitError,
  INTERNAL_ERROR: InternalServerError,
  TEMPORARILY_UNAVAILABLE: ServiceUnavailableError,
};

/**
 * Parse a native NAPI error that may carry a `UC:<CODE>:<message>` prefix
 * and re-throw as the appropriate typed subclass of `UnityCatalogError`.
 */
function parseNativeError(e: unknown): never {
  if (e instanceof Error) {
    const match = e.message.match(/^UC:([^:]+):([\s\S]*)$/);
    if (match) {
      const [, code, message] = match;
      const Ctor = ERROR_MAP[code] ?? UnityCatalogError;
      throw new Ctor(message);
    }
  }
  throw e;
}

// ── end UnityCatalogError error hierarchy ─────────────────────────────────────────────────────

export interface ListAgentSkillsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include agent skills in the response for which the principal can
   *  only access selective metadata for. */
  includeBrowse?: boolean;
}

export interface CreateAgentSkillOptions {
  /** The storage location of the skill directory on the cloud.
   *
   *  Required for EXTERNAL skills; ignored (server-derived) for MANAGED skills. */
  storageLocation?: string;
  /** A human-readable description of what the skill does and when to use it. */
  description?: string;
  /** SPDX license identifier or free-form license text for the skill. */
  license?: string;
  /** The tools the skill is permitted to use. */
  allowedTools?: string[];
  /** Arbitrary additional metadata declared by the skill. */
  metadata?: Record<string, string>;
  /** User-provided free-form text description. */
  comment?: string;
}

export interface GetAgentSkillOptions {
  /** Whether to include agent skills in the response for which the principal can
   *  only access selective metadata for. */
  includeBrowse?: boolean;
}

export interface UpdateAgentSkillOptions {
  /** New name for the agent skill. */
  newName?: string;
  /** Updated description of what the skill does and when to use it. */
  description?: string;
  /** Updated tools the skill is permitted to use. */
  allowedTools?: string[];
  /** The comment attached to the agent skill. */
  comment?: string;
  /** The identifier of the user who owns the agent skill. */
  owner?: string;
}

export interface ListAgentsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include agents in the response for which the principal can only
   *  access selective metadata for. */
  includeBrowse?: boolean;
}

export interface CreateAgentOptions {
  /** An LLM-readable description of what the agent does and the inputs it expects. */
  description?: string;
  /** Capability identifiers advertised by the agent. */
  capabilities?: string[];
  /** A JSON Schema (encoded as a JSON string) describing the expected input. */
  inputSchema?: string;
  /** User-provided free-form text description. */
  comment?: string;
}

export interface GetAgentOptions {
  /** Whether to include agents in the response for which the principal can only
   *  access selective metadata for. */
  includeBrowse?: boolean;
}

export interface UpdateAgentOptions {
  /** New name for the agent. */
  newName?: string;
  /** The protocol a recipient uses to invoke the agent. */
  invocationProtocol?: number;
  /** The agent's invocation endpoint URL. */
  endpoint?: string;
  /** Updated LLM-readable description. */
  description?: string;
  /** Updated capability identifiers advertised by the agent. */
  capabilities?: string[];
  /** Updated JSON Schema (encoded as a JSON string) describing the expected input. */
  inputSchema?: string;
  /** The comment attached to the agent. */
  comment?: string;
  /** The identifier of the user who owns the agent. */
  owner?: string;
}

export interface ListCatalogsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface CreateCatalogOptions {
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable. */
  properties?: Record<string, string>;
  /** Storage root URL for managed tables within catalog. */
  storageRoot?: string;
  /** The name of delta sharing provider.
   *
   *  A Delta Sharing catalog is a catalog that is based on a Delta share on a remote sharing server. */
  providerName?: string;
  /** The name of the share under the share provider. */
  shareName?: string;
}

export interface GetCatalogOptions {
  /** Whether to include catalogs in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface UpdateCatalogOptions {
  /** Username of new owner of catalog. */
  owner?: string;
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable.
   *
   *  When provided in update request, the specified properties will override the existing properties.
   *  To add and remove properties, one would need to perform a read-modify-write. */
  properties?: Record<string, string>;
  /** Name of catalog. */
  newName?: string;
}

export interface DeleteCatalogOptions {
  /** Force deletion even if the catalog is not empty. */
  force?: boolean;
}

export interface ListCredentialsOptions {
  /** Return only credentials for the specified purpose. */
  purpose?: number;
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface CreateCredentialOptions {
  /** Comment associated with the credential. */
  comment?: string;
  /** Whether the credential is usable only for read operations. Only applicable when purpose is STORAGE. */
  readOnly?: boolean;
  /** Optional. Supplying true to this argument skips validation of the created set of credentials. */
  skipValidation?: boolean;
}

export interface UpdateCredentialOptions {
  /** New name of the credential. */
  newName?: string;
  /** Comment associated with the credential. */
  comment?: string;
  /** Whether the credential is usable only for read operations. Only applicable when purpose is STORAGE. */
  readOnly?: boolean;
  /** Username of current owner of credential. */
  owner?: string;
  /** Supply true to this argument to skip validation of the updated credential. */
  skipValidation?: boolean;
  /** Force an update even if there are dependent services (when purpose is SERVICE)
   *  or dependent external locations and external tables (when purpose is STORAGE). */
  force?: boolean;
}

export interface CommitOptions {
  /** Notify the catalog that commits up to and including this version have been
   *  published (backfilled) to the Delta log. The catalog prunes ratified
   *  commits accordingly. */
  latestBackfilledVersion?: number;
}

export interface GetCommitsOptions {
  /** The highest version to return (inclusive). When set, must be
   *  `>= start_version`. Defaults to the latest version. */
  endVersion?: number;
}

export interface ListEntityTagAssignmentsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface UpdateEntityTagAssignmentOptions {
  /** The list of fields to update, as a comma-separated string. */
  updateMask?: string;
}

export interface ListExternalLocationsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include schemas in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface CreateExternalLocationOptions {
  /** Indicates whether the external location is read-only. */
  readOnly?: boolean;
  /** User-provided free-form text description. */
  comment?: string;
  /** Skips validation of the storage credential associated with the external location. */
  skipValidation?: boolean;
}

export interface UpdateExternalLocationOptions {
  /** Path URL of the external location. */
  url?: string;
  /** Name of the storage credential used with this location. */
  credentialName?: string;
  /** Indicates whether the external location is read-only. */
  readOnly?: boolean;
  /** owner of the external location. */
  owner?: string;
  /** User-provided free-form text description. */
  comment?: string;
  /** new name of the external location. */
  newName?: string;
  /** force update of the external location. */
  force?: boolean;
  /** Skips validation of the storage credential associated with the external location. */
  skipValidation?: boolean;
}

export interface DeleteExternalLocationOptions {
  /** Force deletion even if the external location is not empty. */
  force?: boolean;
}

export interface ListFunctionsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include functions in the response for which the principal can only access selective metadata for. */
  includeBrowse?: boolean;
}

export interface CreateFunctionOptions {
  /** Function body. */
  routineDefinition?: string;
  /** The language of the function routine body. */
  routineBodyLanguage?: string;
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable. */
  properties?: Record<string, string>;
}

export interface UpdateFunctionOptions {
  /** Username of new owner of the function. */
  owner?: string;
}

export interface DeleteFunctionOptions {
  /** Force deletion even if the function is not empty. */
  force?: boolean;
}

export interface ListProvidersOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface CreateProviderOptions {
  /** Username of the provider owner. */
  owner?: string;
  /** Description about the provider. */
  comment?: string;
  /** The recipient profile (credential file contents) used to connect to the
   *  sharing server, required for TOKEN authentication. */
  recipientProfileStr?: string;
  /** Provider properties as map of string key-value pairs. */
  properties?: Record<string, string>;
}

export interface UpdateProviderOptions {
  /** New name for the provider. */
  newName?: string;
  /** Username of the provider owner. */
  owner?: string;
  /** Description about the provider. */
  comment?: string;
  /** The recipient profile (credential file contents) used to connect to the
   *  sharing server. */
  recipientProfileStr?: string;
  /** Provider properties as map of string key-value pairs.
   *
   *  When provided in update request, the specified properties will override the existing properties.
   *  To add and remove properties, one would need to perform a read-modify-write. */
  properties?: Record<string, string>;
}

export interface ListRecipientsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface CreateRecipientOptions {
  /** Description about the recipient. */
  comment?: string;
  /** Recipient properties as map of string key-value pairs.
   *
   *  When provided in update request, the specified properties will override the existing properties.
   *  To add and remove properties, one would need to perform a read-modify-write. */
  properties?: Record<string, string>;
  /** Expiration timestamp of the token, in epoch milliseconds. */
  expirationTime?: number;
}

export interface UpdateRecipientOptions {
  /** New name for the recipient */
  newName?: string;
  /** Username of the recipient owner. */
  owner?: string;
  /** Description about the recipient. */
  comment?: string;
  /** Recipient properties as map of string key-value pairs.
   *
   *  When provided in update request, the specified properties will override the existing properties.
   *  To add and remove properties, one would need to perform a read-modify-write. */
  properties?: Record<string, string>;
  /** Expiration timestamp of the token, in epoch milliseconds. */
  expirationTime?: number;
}

export interface ListSchemasOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include schemas in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface CreateSchemaOptions {
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable. */
  properties?: Record<string, string>;
  /** Storage root URL for managed storage location of the schema.
   *
   *  If not set, managed securables under this schema fall back to the parent
   *  catalog's storage location. Example: `s3://bucket/ucroot`. */
  storageRoot?: string;
}

export interface UpdateSchemaOptions {
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable.
   *
   *  When provided in update request, the specified properties will override the existing properties.
   *  To add and remove properties, one would need to perform a read-modify-write. */
  properties?: Record<string, string>;
  /** Name of schema. */
  newName?: string;
}

export interface DeleteSchemaOptions {
  /** Force deletion even if the schema is not empty. */
  force?: boolean;
}

export interface ListSharesOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface CreateShareOptions {
  /** User-provided free-form text description. */
  comment?: string;
}

export interface GetShareOptions {
  /** Query for data to include in the share. */
  includeSharedData?: boolean;
}

export interface UpdateShareOptions {
  /** A new name for the share. */
  newName?: string;
  /** Owner of the share. */
  owner?: string;
  /** User-provided free-form text description. */
  comment?: string;
}

export interface GetPermissionsOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface UpdatePermissionsOptions {
  /** Whether to return the latest permissions list of the share in the response. */
  omitPermissionsList?: boolean;
}

export interface ListTableSummariesOptions {
  /** A sql LIKE pattern (% and _) for schema names. All schemas will be returned if not set or empty. */
  schemaNamePattern?: string;
  /** A sql LIKE pattern (% and _) for table names. All tables will be returned if not set or empty. */
  tableNamePattern?: string;
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include a manifest containing capabilities the table has. */
  includeManifestCapabilities?: boolean;
}

export interface ListTablesOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether delta metadata should be included in the response. */
  includeDeltaMetadata?: boolean;
  /** Whether to omit the columns of the table from the response or not. */
  omitColumns?: boolean;
  /** Whether to omit the properties of the table from the response or not. */
  omitProperties?: boolean;
  /** Whether to omit the username of the table (e.g. owner, updated_by, created_by) from the response or not. */
  omitUsername?: boolean;
  /** Whether to include tables in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
  /** Whether to include a manifest containing capabilities the table has. */
  includeManifestCapabilities?: boolean;
}

export interface CreateTableOptions {
  /** Storage root URL for external table. */
  storageLocation?: string;
  /** User-provided free-form text description. */
  comment?: string;
  /** A map of key-value properties attached to the securable. */
  properties?: Record<string, string>;
  /** Definition text for view-like table types (VIEW, MATERIALIZED_VIEW,
   *  STREAMING_TABLE, METRIC_VIEW). The format depends on the table type:
   *  SQL for views, YAML for metric views. Required for METRIC_VIEW. */
  viewDefinition?: string;
}

export interface GetTableOptions {
  /** Whether delta metadata should be included in the response. */
  includeDeltaMetadata?: boolean;
  /** Whether to include tables in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
  /** Whether to include a manifest containing capabilities the table has. */
  includeManifestCapabilities?: boolean;
}

export interface ListTagPoliciesOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
}

export interface UpdateTagPolicyOptions {
  /** The list of fields to update, as a comma-separated string. */
  updateMask?: string;
}

export interface GenerateTemporaryPathCredentialsOptions {
  /** When set to true, the service will not validate that the generated
   *  credentials can perform write operations, therefore no new paths will be
   *  created and the response will not contain valid credentials. Defaults to false. */
  dryRun?: boolean;
}

export interface ListVolumesOptions {
  /** The maximum number of results per page that should be returned. */
  maxResults?: number;
  /** Opaque pagination token to go to next page based on previous query. */
  pageToken?: string;
  /** Whether to include schemas in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface CreateVolumeOptions {
  /** The storage location on the cloud */
  storageLocation?: string;
  /** The storage location on the cloud */
  comment?: string;
}

export interface GetVolumeOptions {
  /** Whether to include schemas in the response for which the principal can only access selective metadata for */
  includeBrowse?: boolean;
}

export interface UpdateVolumeOptions {
  /** New name for the volume. */
  newName?: string;
  /** The comment attached to the volume */
  comment?: string;
  /** The identifier of the user who owns the volume */
  owner?: string;
}

export class AgentSkillClient {
  private readonly inner: NativeAgentSkillClient;

  /** @internal */
  constructor(inner: NativeAgentSkillClient) {
    this.inner = inner;
  }

  async get(options?: GetAgentSkillOptions): Promise<AgentSkill> {
    const { includeBrowse } = options || {};
    try {
      return fromBinary(AgentSkillSchema, await this.inner.get(includeBrowse));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async update(options?: UpdateAgentSkillOptions): Promise<AgentSkill> {
    const { newName, description, allowedTools, comment, owner } =
      options || {};
    try {
      return fromBinary(
        AgentSkillSchema,
        await this.inner.update(
          newName,
          description,
          allowedTools,
          comment,
          owner,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class AgentClient {
  private readonly inner: NativeAgentClient;

  /** @internal */
  constructor(inner: NativeAgentClient) {
    this.inner = inner;
  }

  async get(options?: GetAgentOptions): Promise<Agent> {
    const { includeBrowse } = options || {};
    try {
      return fromBinary(AgentSchema, await this.inner.get(includeBrowse));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async update(options?: UpdateAgentOptions): Promise<Agent> {
    const {
      newName,
      invocationProtocol,
      endpoint,
      description,
      capabilities,
      inputSchema,
      comment,
      owner,
    } = options || {};
    try {
      return fromBinary(
        AgentSchema,
        await this.inner.update(
          newName,
          invocationProtocol,
          endpoint,
          description,
          capabilities,
          inputSchema,
          comment,
          owner,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class CatalogClient {
  private readonly inner: NativeCatalogClient;

  /** @internal */
  constructor(inner: NativeCatalogClient) {
    this.inner = inner;
  }

  /**
   * Get a catalog
   *
   * Gets the specified catalog in a metastore. The caller must be a metastore admin,
   * the owner of the catalog, or a user that has the USE_CATALOG privilege set for their account.
   */
  async get(options?: GetCatalogOptions): Promise<Catalog> {
    const { includeBrowse } = options || {};
    try {
      return fromBinary(CatalogSchema, await this.inner.get(includeBrowse));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Update a catalog
   *
   * Updates the catalog that matches the supplied name. The caller must be either
   * the owner of the catalog, or a metastore admin (when changing the owner field of the catalog).
   */
  async update(options?: UpdateCatalogOptions): Promise<Catalog> {
    const { owner, comment, properties, newName } = options || {};
    try {
      return fromBinary(
        CatalogSchema,
        await this.inner.update(owner, comment, properties, newName),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Delete a catalog
   *
   * Deletes the catalog that matches the supplied name. The caller must
   * be a metastore admin or the owner of the catalog.
   */
  async delete(options?: DeleteCatalogOptions): Promise<void> {
    const { force } = options || {};
    try {
      await this.inner.delete(force);
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class CredentialClient {
  private readonly inner: NativeCredentialClient;

  /** @internal */
  constructor(inner: NativeCredentialClient) {
    this.inner = inner;
  }

  async get(): Promise<Credential> {
    try {
      return fromBinary(CredentialSchema, await this.inner.get());
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async update(options?: UpdateCredentialOptions): Promise<Credential> {
    const { newName, comment, readOnly, owner, skipValidation, force } =
      options || {};
    try {
      return fromBinary(
        CredentialSchema,
        await this.inner.update(
          newName,
          comment,
          readOnly,
          owner,
          skipValidation,
          force,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class ExternalLocationClient {
  private readonly inner: NativeExternalLocationClient;

  /** @internal */
  constructor(inner: NativeExternalLocationClient) {
    this.inner = inner;
  }

  /**
   * Get an external location
   */
  async get(): Promise<ExternalLocation> {
    try {
      return fromBinary(ExternalLocationSchema, await this.inner.get());
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Update an external location
   */
  async update(
    options?: UpdateExternalLocationOptions,
  ): Promise<ExternalLocation> {
    const {
      url,
      credentialName,
      readOnly,
      owner,
      comment,
      newName,
      force,
      skipValidation,
    } = options || {};
    try {
      return fromBinary(
        ExternalLocationSchema,
        await this.inner.update(
          url,
          credentialName,
          readOnly,
          owner,
          comment,
          newName,
          force,
          skipValidation,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Delete an external location
   */
  async delete(options?: DeleteExternalLocationOptions): Promise<void> {
    const { force } = options || {};
    try {
      await this.inner.delete(force);
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class FunctionClient {
  private readonly inner: NativeFunctionClient;

  /** @internal */
  constructor(inner: NativeFunctionClient) {
    this.inner = inner;
  }

  /**
   * Get a function
   *
   * Gets a function from within a parent catalog and schema. For the fetch to succeed,
   * the caller must be a metastore admin, the owner of the function, or have SELECT on
   * the function.
   */
  async get(): Promise<Function$> {
    try {
      return fromBinary(FunctionSchema, await this.inner.get());
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Update a function
   *
   * Updates the function that matches the supplied name. Only the owner of the function
   * can be updated.
   */
  async update(options?: UpdateFunctionOptions): Promise<Function$> {
    const { owner } = options || {};
    try {
      return fromBinary(FunctionSchema, await this.inner.update(owner));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Delete a function
   *
   * Deletes the function that matches the supplied name. For the deletion to succeed,
   * the caller must be the owner of the function.
   */
  async delete(options?: DeleteFunctionOptions): Promise<void> {
    const { force } = options || {};
    try {
      await this.inner.delete(force);
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class ProviderClient {
  private readonly inner: NativeProviderClient;

  /** @internal */
  constructor(inner: NativeProviderClient) {
    this.inner = inner;
  }

  /**
   * Get a provider by name.
   */
  async get(): Promise<Provider> {
    try {
      return fromBinary(ProviderSchema, await this.inner.get());
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Update a provider.
   */
  async update(options?: UpdateProviderOptions): Promise<Provider> {
    const { newName, owner, comment, recipientProfileStr, properties } =
      options || {};
    try {
      return fromBinary(
        ProviderSchema,
        await this.inner.update(
          newName,
          owner,
          comment,
          recipientProfileStr,
          properties,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Delete a provider.
   */
  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class RecipientClient {
  private readonly inner: NativeRecipientClient;

  /** @internal */
  constructor(inner: NativeRecipientClient) {
    this.inner = inner;
  }

  /**
   * Get a recipient by name.
   */
  async get(): Promise<Recipient> {
    try {
      return fromBinary(RecipientSchema, await this.inner.get());
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Update a recipient.
   */
  async update(options?: UpdateRecipientOptions): Promise<Recipient> {
    const { newName, owner, comment, properties, expirationTime } =
      options || {};
    try {
      return fromBinary(
        RecipientSchema,
        await this.inner.update(
          newName,
          owner,
          comment,
          properties,
          expirationTime,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Delete a recipient.
   */
  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class SchemaClient {
  private readonly inner: NativeSchemaClient;

  /** @internal */
  constructor(inner: NativeSchemaClient) {
    this.inner = inner;
  }

  /**
   * Gets the specified schema within the metastore.
   * The caller must be a metastore admin, the owner of the schema,
   * or a user that has the USE_SCHEMA privilege on the schema.
   */
  async get(): Promise<Schema> {
    try {
      return fromBinary(SchemaSchema, await this.inner.get());
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Updates a schema for a catalog. The caller must be the owner of the schema or a metastore admin.
   * If the caller is a metastore admin, only the owner field can be changed in the update.
   * If the name field must be updated, the caller must be a metastore admin or have the CREATE_SCHEMA
   * privilege on the parent catalog.
   */
  async update(options?: UpdateSchemaOptions): Promise<Schema> {
    const { comment, properties, newName } = options || {};
    try {
      return fromBinary(
        SchemaSchema,
        await this.inner.update(comment, properties, newName),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Deletes the specified schema from the parent catalog. The caller must be the owner
   * of the schema or an owner of the parent catalog.
   */
  async delete(options?: DeleteSchemaOptions): Promise<void> {
    const { force } = options || {};
    try {
      await this.inner.delete(force);
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class ShareClient {
  private readonly inner: NativeShareClient;

  /** @internal */
  constructor(inner: NativeShareClient) {
    this.inner = inner;
  }

  /**
   * Get a share by name.
   */
  async get(options?: GetShareOptions): Promise<Share> {
    const { includeSharedData } = options || {};
    try {
      return fromBinary(ShareSchema, await this.inner.get(includeSharedData));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Update a share.
   */
  async update(options?: UpdateShareOptions): Promise<Share> {
    const { newName, owner, comment } = options || {};
    try {
      return fromBinary(
        ShareSchema,
        await this.inner.update(newName, owner, comment),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Deletes a share.
   */
  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class StagingTableClient {
  /** @internal */
  constructor(readonly inner: NativeStagingTableClient) {}
}

export class TableClient {
  private readonly inner: NativeTableClient;

  /** @internal */
  constructor(inner: NativeTableClient) {
    this.inner = inner;
  }

  /**
   * Get a table
   */
  async get(options?: GetTableOptions): Promise<Table> {
    const { includeDeltaMetadata, includeBrowse, includeManifestCapabilities } =
      options || {};
    try {
      return fromBinary(
        TableSchema,
        await this.inner.get(
          includeDeltaMetadata,
          includeBrowse,
          includeManifestCapabilities,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Delete a table
   */
  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class TagPolicyClient {
  private readonly inner: NativeTagPolicyClient;

  /** @internal */
  constructor(inner: NativeTagPolicyClient) {
    this.inner = inner;
  }

  /**
   * Get a tag policy
   *
   * Gets the governed tag definition for the specified tag key.
   */
  async get(): Promise<TagPolicy> {
    try {
      return fromBinary(TagPolicySchema, await this.inner.get());
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Update a tag policy
   *
   * Updates the governed tag definition that matches the supplied tag key.
   */
  async update(
    tagPolicy: TagPolicy,
    options?: UpdateTagPolicyOptions,
  ): Promise<TagPolicy> {
    const { updateMask } = options || {};
    try {
      return fromBinary(
        TagPolicySchema,
        await this.inner.update(
          Buffer.from(toBinary(TagPolicySchema, tagPolicy)),
          updateMask,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Delete a tag policy
   *
   * Deletes the governed tag definition that matches the supplied tag key.
   */
  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class VolumeClient {
  private readonly inner: NativeVolumeClient;

  /** @internal */
  constructor(inner: NativeVolumeClient) {
    this.inner = inner;
  }

  async get(options?: GetVolumeOptions): Promise<Volume> {
    const { includeBrowse } = options || {};
    try {
      return fromBinary(VolumeSchema, await this.inner.get(includeBrowse));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async update(options?: UpdateVolumeOptions): Promise<Volume> {
    const { newName, comment, owner } = options || {};
    try {
      return fromBinary(
        VolumeSchema,
        await this.inner.update(newName, comment, owner),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async delete(): Promise<void> {
    try {
      await this.inner.delete();
    } catch (e) {
      throw parseNativeError(e);
    }
  }
}

export class UnityCatalogClient {
  private readonly inner: NativeClient;

  constructor(url: string, token?: string) {
    this.inner = NativeClient.fromUrl(url, token);
  }

  /**
   * Lists agent skills.
   */
  async listAgentSkills(
    catalogName: string,
    schemaName: string,
    options?: ListAgentSkillsOptions,
  ): Promise<AgentSkill[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (
        await this.inner.listAgentSkills(
          catalogName,
          schemaName,
          maxResults,
          includeBrowse,
        )
      ).map((data) => fromBinary(AgentSkillSchema, data));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Lists agent skills.
   */
  async *listAgentSkillsStream(
    catalogName: string,
    schemaName: string,
    options?: ListAgentSkillsOptions,
  ): AsyncIterable<AgentSkill> {
    const { maxResults, includeBrowse } = options || {};
    try {
      for await (const data of this.inner.listAgentSkillsStream(
        catalogName,
        schemaName,
        maxResults,
        includeBrowse,
      )) {
        yield fromBinary(AgentSkillSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async createAgentSkill(
    catalogName: string,
    schemaName: string,
    name: string,
    agentSkillType: number,
    options?: CreateAgentSkillOptions,
  ): Promise<AgentSkill> {
    const {
      storageLocation,
      description,
      license,
      allowedTools,
      metadata,
      comment,
    } = options || {};
    try {
      return fromBinary(
        AgentSkillSchema,
        await this.inner.createAgentSkill(
          catalogName,
          schemaName,
          name,
          agentSkillType,
          storageLocation,
          description,
          license,
          allowedTools,
          metadata,
          comment,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  agentSkill(
    catalogName: string,
    schemaName: string,
    agentSkillName: string,
  ): AgentSkillClient {
    return new AgentSkillClient(
      this.inner.agentSkill(catalogName, schemaName, agentSkillName),
    );
  }

  /**
   * Lists agents.
   */
  async listAgents(
    catalogName: string,
    schemaName: string,
    options?: ListAgentsOptions,
  ): Promise<Agent[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (
        await this.inner.listAgents(
          catalogName,
          schemaName,
          maxResults,
          includeBrowse,
        )
      ).map((data) => fromBinary(AgentSchema, data));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Lists agents.
   */
  async *listAgentsStream(
    catalogName: string,
    schemaName: string,
    options?: ListAgentsOptions,
  ): AsyncIterable<Agent> {
    const { maxResults, includeBrowse } = options || {};
    try {
      for await (const data of this.inner.listAgentsStream(
        catalogName,
        schemaName,
        maxResults,
        includeBrowse,
      )) {
        yield fromBinary(AgentSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async createAgent(
    catalogName: string,
    schemaName: string,
    name: string,
    invocationProtocol: number,
    endpoint: string,
    options?: CreateAgentOptions,
  ): Promise<Agent> {
    const { description, capabilities, inputSchema, comment } = options || {};
    try {
      return fromBinary(
        AgentSchema,
        await this.inner.createAgent(
          catalogName,
          schemaName,
          name,
          invocationProtocol,
          endpoint,
          description,
          capabilities,
          inputSchema,
          comment,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  agent(
    catalogName: string,
    schemaName: string,
    agentName: string,
  ): AgentClient {
    return new AgentClient(
      this.inner.agent(catalogName, schemaName, agentName),
    );
  }

  /**
   * List catalogs
   *
   * Gets an array of catalogs in the metastore. If the caller is the metastore admin,
   * all catalogs will be retrieved. Otherwise, only catalogs owned by the caller
   * (or for which the caller has the USE_CATALOG privilege) will be retrieved.
   * There is no guarantee of a specific ordering of the elements in the array.
   */
  async listCatalogs(options?: ListCatalogsOptions): Promise<Catalog[]> {
    const { maxResults } = options || {};
    try {
      return (await this.inner.listCatalogs(maxResults)).map((data) =>
        fromBinary(CatalogSchema, data),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * List catalogs
   *
   * Gets an array of catalogs in the metastore. If the caller is the metastore admin,
   * all catalogs will be retrieved. Otherwise, only catalogs owned by the caller
   * (or for which the caller has the USE_CATALOG privilege) will be retrieved.
   * There is no guarantee of a specific ordering of the elements in the array.
   */
  async *listCatalogsStream(
    options?: ListCatalogsOptions,
  ): AsyncIterable<Catalog> {
    const { maxResults } = options || {};
    try {
      for await (const data of this.inner.listCatalogsStream(maxResults)) {
        yield fromBinary(CatalogSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Create a new catalog
   *
   * Creates a new catalog instance in the parent metastore if the caller
   * is a metastore admin or has the CREATE_CATALOG privilege.
   */
  async createCatalog(
    name: string,
    options?: CreateCatalogOptions,
  ): Promise<Catalog> {
    const { comment, properties, storageRoot, providerName, shareName } =
      options || {};
    try {
      return fromBinary(
        CatalogSchema,
        await this.inner.createCatalog(
          name,
          comment,
          properties,
          storageRoot,
          providerName,
          shareName,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  catalog(catalogName: string): CatalogClient {
    return new CatalogClient(this.inner.catalog(catalogName));
  }

  async listCredentials(
    options?: ListCredentialsOptions,
  ): Promise<Credential[]> {
    const { purpose, maxResults } = options || {};
    try {
      return (await this.inner.listCredentials(purpose, maxResults)).map(
        (data) => fromBinary(CredentialSchema, data),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async *listCredentialsStream(
    options?: ListCredentialsOptions,
  ): AsyncIterable<Credential> {
    const { purpose, maxResults } = options || {};
    try {
      for await (const data of this.inner.listCredentialsStream(
        purpose,
        maxResults,
      )) {
        yield fromBinary(CredentialSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async createCredential(
    name: string,
    purpose: number,
    options?: CreateCredentialOptions,
  ): Promise<Credential> {
    const { comment, readOnly, skipValidation } = options || {};
    try {
      return fromBinary(
        CredentialSchema,
        await this.inner.createCredential(
          name,
          purpose,
          comment,
          readOnly,
          skipValidation,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  credential(credentialName: string): CredentialClient {
    return new CredentialClient(this.inner.credential(credentialName));
  }

  /**
   * Ratify a staged commit at the requested version (first-writer-wins), and/or
   * notify the catalog that commits have been backfilled to the Delta log.
   */
  async commit(
    tableId: string,
    tableUri: string,
    options?: CommitOptions,
  ): Promise<void> {
    const { latestBackfilledVersion } = options || {};
    try {
      await this.inner.commit(tableId, tableUri, latestBackfilledVersion);
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Return ratified-but-unpublished commits for a table, plus the latest
   * version the catalog tracks.
   */
  async getCommits(
    tableId: string,
    tableUri: string,
    startVersion: number,
    options?: GetCommitsOptions,
  ): Promise<GetCommitsResponse> {
    const { endVersion } = options || {};
    try {
      return fromBinary(
        GetCommitsResponseSchema,
        await this.inner.getCommits(
          tableId,
          tableUri,
          startVersion,
          endVersion,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * List entity tag assignments
   *
   * Gets the tag assignments for the specified entity.
   */
  async listEntityTagAssignments(
    entityType: string,
    entityName: string,
    options?: ListEntityTagAssignmentsOptions,
  ): Promise<ListEntityTagAssignmentsResponse> {
    const { maxResults, pageToken } = options || {};
    try {
      return fromBinary(
        ListEntityTagAssignmentsResponseSchema,
        await this.inner.listEntityTagAssignments(
          entityType,
          entityName,
          maxResults,
          pageToken,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Create an entity tag assignment
   *
   * Assigns a tag to a Unity Catalog entity.
   */
  async createEntityTagAssignment(
    tagAssignment: EntityTagAssignment,
  ): Promise<EntityTagAssignment> {
    try {
      return fromBinary(
        EntityTagAssignmentSchema,
        await this.inner.createEntityTagAssignment(
          Buffer.from(toBinary(EntityTagAssignmentSchema, tagAssignment)),
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Get an entity tag assignment
   *
   * Gets the tag assignment for the specified entity and tag key.
   */
  async getEntityTagAssignment(
    entityType: string,
    entityName: string,
    tagKey: string,
  ): Promise<EntityTagAssignment> {
    try {
      return fromBinary(
        EntityTagAssignmentSchema,
        await this.inner.getEntityTagAssignment(entityType, entityName, tagKey),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Update an entity tag assignment
   *
   * Updates the tag assignment for the specified entity and tag key.
   */
  async updateEntityTagAssignment(
    entityType: string,
    entityName: string,
    tagKey: string,
    tagAssignment: EntityTagAssignment,
    options?: UpdateEntityTagAssignmentOptions,
  ): Promise<EntityTagAssignment> {
    const { updateMask } = options || {};
    try {
      return fromBinary(
        EntityTagAssignmentSchema,
        await this.inner.updateEntityTagAssignment(
          entityType,
          entityName,
          tagKey,
          Buffer.from(toBinary(EntityTagAssignmentSchema, tagAssignment)),
          updateMask,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Delete an entity tag assignment
   *
   * Deletes the tag assignment for the specified entity and tag key.
   */
  async deleteEntityTagAssignment(
    entityType: string,
    entityName: string,
    tagKey: string,
  ): Promise<void> {
    try {
      await this.inner.deleteEntityTagAssignment(
        entityType,
        entityName,
        tagKey,
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * List external locations
   */
  async listExternalLocations(
    options?: ListExternalLocationsOptions,
  ): Promise<ExternalLocation[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (
        await this.inner.listExternalLocations(maxResults, includeBrowse)
      ).map((data) => fromBinary(ExternalLocationSchema, data));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * List external locations
   */
  async *listExternalLocationsStream(
    options?: ListExternalLocationsOptions,
  ): AsyncIterable<ExternalLocation> {
    const { maxResults, includeBrowse } = options || {};
    try {
      for await (const data of this.inner.listExternalLocationsStream(
        maxResults,
        includeBrowse,
      )) {
        yield fromBinary(ExternalLocationSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Create a new external location
   */
  async createExternalLocation(
    name: string,
    url: string,
    credentialName: string,
    options?: CreateExternalLocationOptions,
  ): Promise<ExternalLocation> {
    const { readOnly, comment, skipValidation } = options || {};
    try {
      return fromBinary(
        ExternalLocationSchema,
        await this.inner.createExternalLocation(
          name,
          url,
          credentialName,
          readOnly,
          comment,
          skipValidation,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  externalLocation(externalLocationName: string): ExternalLocationClient {
    return new ExternalLocationClient(
      this.inner.externalLocation(externalLocationName),
    );
  }

  /**
   * List functions
   *
   * List functions within the specified parent catalog and schema. If the caller is the metastore
   * admin, all functions are returned in the response. Otherwise, the caller must have USE_CATALOG
   * on the parent catalog and USE_SCHEMA on the parent schema, and the function must either be
   * owned by the caller or have SELECT on the function.
   */
  async listFunctions(
    catalogName: string,
    schemaName: string,
    options?: ListFunctionsOptions,
  ): Promise<Function$[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (
        await this.inner.listFunctions(
          catalogName,
          schemaName,
          maxResults,
          includeBrowse,
        )
      ).map((data) => fromBinary(FunctionSchema, data));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * List functions
   *
   * List functions within the specified parent catalog and schema. If the caller is the metastore
   * admin, all functions are returned in the response. Otherwise, the caller must have USE_CATALOG
   * on the parent catalog and USE_SCHEMA on the parent schema, and the function must either be
   * owned by the caller or have SELECT on the function.
   */
  async *listFunctionsStream(
    catalogName: string,
    schemaName: string,
    options?: ListFunctionsOptions,
  ): AsyncIterable<Function$> {
    const { maxResults, includeBrowse } = options || {};
    try {
      for await (const data of this.inner.listFunctionsStream(
        catalogName,
        schemaName,
        maxResults,
        includeBrowse,
      )) {
        yield fromBinary(FunctionSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Create a function
   *
   * Creates a new function. The caller must be a metastore admin or have the CREATE_FUNCTION
   * privilege on the parent catalog and schema.
   */
  async createFunction(
    name: string,
    catalogName: string,
    schemaName: string,
    dataType: string,
    fullDataType: string,
    parameterStyle: number,
    isDeterministic: boolean,
    sqlDataAccess: number,
    isNullCall: boolean,
    securityType: number,
    routineBody: number,
    options?: CreateFunctionOptions,
  ): Promise<Function$> {
    const { routineDefinition, routineBodyLanguage, comment, properties } =
      options || {};
    try {
      return fromBinary(
        FunctionSchema,
        await this.inner.createFunction(
          name,
          catalogName,
          schemaName,
          dataType,
          fullDataType,
          parameterStyle,
          isDeterministic,
          sqlDataAccess,
          isNullCall,
          securityType,
          routineBody,
          routineDefinition,
          routineBodyLanguage,
          comment,
          properties,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  function(
    catalogName: string,
    schemaName: string,
    functionName: string,
  ): FunctionClient {
    return new FunctionClient(
      this.inner.function(catalogName, schemaName, functionName),
    );
  }

  /**
   * List providers.
   */
  async listProviders(options?: ListProvidersOptions): Promise<Provider[]> {
    const { maxResults } = options || {};
    try {
      return (await this.inner.listProviders(maxResults)).map((data) =>
        fromBinary(ProviderSchema, data),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * List providers.
   */
  async *listProvidersStream(
    options?: ListProvidersOptions,
  ): AsyncIterable<Provider> {
    const { maxResults } = options || {};
    try {
      for await (const data of this.inner.listProvidersStream(maxResults)) {
        yield fromBinary(ProviderSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Create a new provider.
   */
  async createProvider(
    name: string,
    authenticationType: number,
    options?: CreateProviderOptions,
  ): Promise<Provider> {
    const { owner, comment, recipientProfileStr, properties } = options || {};
    try {
      return fromBinary(
        ProviderSchema,
        await this.inner.createProvider(
          name,
          authenticationType,
          owner,
          comment,
          recipientProfileStr,
          properties,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  provider(providerName: string): ProviderClient {
    return new ProviderClient(this.inner.provider(providerName));
  }

  /**
   * List recipients.
   */
  async listRecipients(options?: ListRecipientsOptions): Promise<Recipient[]> {
    const { maxResults } = options || {};
    try {
      return (await this.inner.listRecipients(maxResults)).map((data) =>
        fromBinary(RecipientSchema, data),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * List recipients.
   */
  async *listRecipientsStream(
    options?: ListRecipientsOptions,
  ): AsyncIterable<Recipient> {
    const { maxResults } = options || {};
    try {
      for await (const data of this.inner.listRecipientsStream(maxResults)) {
        yield fromBinary(RecipientSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Create a new recipient.
   */
  async createRecipient(
    name: string,
    authenticationType: number,
    owner: string,
    options?: CreateRecipientOptions,
  ): Promise<Recipient> {
    const { comment, properties, expirationTime } = options || {};
    try {
      return fromBinary(
        RecipientSchema,
        await this.inner.createRecipient(
          name,
          authenticationType,
          owner,
          comment,
          properties,
          expirationTime,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  recipient(recipientName: string): RecipientClient {
    return new RecipientClient(this.inner.recipient(recipientName));
  }

  /**
   * Gets an array of schemas for a catalog in the metastore. If the caller is the metastore
   * admin or the owner of the parent catalog, all schemas for the catalog will be retrieved.
   * Otherwise, only schemas owned by the caller (or for which the caller has the USE_SCHEMA privilege)
   * will be retrieved. There is no guarantee of a specific ordering of the elements in the array.
   */
  async listSchemas(
    catalogName: string,
    options?: ListSchemasOptions,
  ): Promise<Schema[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (
        await this.inner.listSchemas(catalogName, maxResults, includeBrowse)
      ).map((data) => fromBinary(SchemaSchema, data));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Gets an array of schemas for a catalog in the metastore. If the caller is the metastore
   * admin or the owner of the parent catalog, all schemas for the catalog will be retrieved.
   * Otherwise, only schemas owned by the caller (or for which the caller has the USE_SCHEMA privilege)
   * will be retrieved. There is no guarantee of a specific ordering of the elements in the array.
   */
  async *listSchemasStream(
    catalogName: string,
    options?: ListSchemasOptions,
  ): AsyncIterable<Schema> {
    const { maxResults, includeBrowse } = options || {};
    try {
      for await (const data of this.inner.listSchemasStream(
        catalogName,
        maxResults,
        includeBrowse,
      )) {
        yield fromBinary(SchemaSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Creates a new schema for catalog in the Metatastore. The caller must be a metastore admin,
   * or have the CREATE_SCHEMA privilege in the parent catalog.
   */
  async createSchema(
    name: string,
    catalogName: string,
    options?: CreateSchemaOptions,
  ): Promise<Schema> {
    const { comment, properties, storageRoot } = options || {};
    try {
      return fromBinary(
        SchemaSchema,
        await this.inner.createSchema(
          name,
          catalogName,
          comment,
          properties,
          storageRoot,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  schema(catalogName: string, schemaName: string): SchemaClient {
    return new SchemaClient(this.inner.schema(catalogName, schemaName));
  }

  /**
   * List shares.
   */
  async listShares(options?: ListSharesOptions): Promise<Share[]> {
    const { maxResults } = options || {};
    try {
      return (await this.inner.listShares(maxResults)).map((data) =>
        fromBinary(ShareSchema, data),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * List shares.
   */
  async *listSharesStream(options?: ListSharesOptions): AsyncIterable<Share> {
    const { maxResults } = options || {};
    try {
      for await (const data of this.inner.listSharesStream(maxResults)) {
        yield fromBinary(ShareSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Create a new share.
   */
  async createShare(
    name: string,
    options?: CreateShareOptions,
  ): Promise<Share> {
    const { comment } = options || {};
    try {
      return fromBinary(
        ShareSchema,
        await this.inner.createShare(name, comment),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  share(shareName: string): ShareClient {
    return new ShareClient(this.inner.share(shareName));
  }

  /**
   * Creates a new staging table, allocating an immutable table id and a storage
   * location under the parent schema/catalog managed storage root. The caller
   * must have the CREATE privilege on the parent schema.
   */
  async createStagingTable(
    name: string,
    catalogName: string,
    schemaName: string,
  ): Promise<StagingTable> {
    try {
      return fromBinary(
        StagingTableSchema,
        await this.inner.createStagingTable(name, catalogName, schemaName),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  stagingTable(stagingTableName: string): StagingTableClient {
    return new StagingTableClient(this.inner.stagingTable(stagingTableName));
  }

  /**
   * Gets an array of all tables for the current metastore under the parent catalog and schema.
   *
   * The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table.
   * For the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the
   * parent catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a
   * specific ordering of the elements in the array.
   */
  async listTables(
    catalogName: string,
    schemaName: string,
    options?: ListTablesOptions,
  ): Promise<Table[]> {
    const {
      maxResults,
      includeDeltaMetadata,
      omitColumns,
      omitProperties,
      omitUsername,
      includeBrowse,
      includeManifestCapabilities,
    } = options || {};
    try {
      return (
        await this.inner.listTables(
          catalogName,
          schemaName,
          maxResults,
          includeDeltaMetadata,
          omitColumns,
          omitProperties,
          omitUsername,
          includeBrowse,
          includeManifestCapabilities,
        )
      ).map((data) => fromBinary(TableSchema, data));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Gets an array of all tables for the current metastore under the parent catalog and schema.
   *
   * The caller must be a metastore admin or an owner of (or have the SELECT privilege on) the table.
   * For the latter case, the caller must also be the owner or have the USE_CATALOG privilege on the
   * parent catalog and the USE_SCHEMA privilege on the parent schema. There is no guarantee of a
   * specific ordering of the elements in the array.
   */
  async *listTablesStream(
    catalogName: string,
    schemaName: string,
    options?: ListTablesOptions,
  ): AsyncIterable<Table> {
    const {
      maxResults,
      includeDeltaMetadata,
      omitColumns,
      omitProperties,
      omitUsername,
      includeBrowse,
      includeManifestCapabilities,
    } = options || {};
    try {
      for await (const data of this.inner.listTablesStream(
        catalogName,
        schemaName,
        maxResults,
        includeDeltaMetadata,
        omitColumns,
        omitProperties,
        omitUsername,
        includeBrowse,
        includeManifestCapabilities,
      )) {
        yield fromBinary(TableSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Create a table
   */
  async createTable(
    name: string,
    schemaName: string,
    catalogName: string,
    tableType: number,
    dataSourceFormat: number,
    options?: CreateTableOptions,
  ): Promise<Table> {
    const { storageLocation, comment, properties, viewDefinition } =
      options || {};
    try {
      return fromBinary(
        TableSchema,
        await this.inner.createTable(
          name,
          schemaName,
          catalogName,
          tableType,
          dataSourceFormat,
          storageLocation,
          comment,
          properties,
          viewDefinition,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  table(
    catalogName: string,
    schemaName: string,
    tableName: string,
  ): TableClient {
    return new TableClient(
      this.inner.table(catalogName, schemaName, tableName),
    );
  }

  /**
   * List tag policies
   *
   * Gets an array of tag policies. There is no guarantee of a specific ordering
   * of the elements in the array.
   */
  async listTagPolicies(
    options?: ListTagPoliciesOptions,
  ): Promise<TagPolicy[]> {
    const { maxResults } = options || {};
    try {
      return (await this.inner.listTagPolicies(maxResults)).map((data) =>
        fromBinary(TagPolicySchema, data),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * List tag policies
   *
   * Gets an array of tag policies. There is no guarantee of a specific ordering
   * of the elements in the array.
   */
  async *listTagPoliciesStream(
    options?: ListTagPoliciesOptions,
  ): AsyncIterable<TagPolicy> {
    const { maxResults } = options || {};
    try {
      for await (const data of this.inner.listTagPoliciesStream(maxResults)) {
        yield fromBinary(TagPolicySchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Create a new tag policy
   *
   * Creates a new governed tag definition.
   */
  async createTagPolicy(tagPolicy: TagPolicy): Promise<TagPolicy> {
    try {
      return fromBinary(
        TagPolicySchema,
        await this.inner.createTagPolicy(
          Buffer.from(toBinary(TagPolicySchema, tagPolicy)),
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  tagPolicy(tagPolicyName: string): TagPolicyClient {
    return new TagPolicyClient(this.inner.tagPolicy(tagPolicyName));
  }

  /**
   * Generate a new set of credentials for a table.
   */
  async generateTemporaryTableCredentials(
    tableId: string,
    operation: number,
  ): Promise<TemporaryCredential> {
    try {
      return fromBinary(
        TemporaryCredentialSchema,
        await this.inner.generateTemporaryTableCredentials(tableId, operation),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Generate a new set of credentials for a path.
   */
  async generateTemporaryPathCredentials(
    url: string,
    operation: number,
    options?: GenerateTemporaryPathCredentialsOptions,
  ): Promise<TemporaryCredential> {
    const { dryRun } = options || {};
    try {
      return fromBinary(
        TemporaryCredentialSchema,
        await this.inner.generateTemporaryPathCredentials(
          url,
          operation,
          dryRun,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Generate a new set of credentials for a volume.
   *
   * The metastore must have the `external_access_enabled` flag set to true
   * (default false). The caller must have the `EXTERNAL_USE_SCHEMA`
   * privilege on the parent schema (granted by a catalog owner).
   */
  async generateTemporaryVolumeCredentials(
    volumeId: string,
    operation: number,
  ): Promise<TemporaryCredential> {
    try {
      return fromBinary(
        TemporaryCredentialSchema,
        await this.inner.generateTemporaryVolumeCredentials(
          volumeId,
          operation,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Lists volumes.
   */
  async listVolumes(
    catalogName: string,
    schemaName: string,
    options?: ListVolumesOptions,
  ): Promise<Volume[]> {
    const { maxResults, includeBrowse } = options || {};
    try {
      return (
        await this.inner.listVolumes(
          catalogName,
          schemaName,
          maxResults,
          includeBrowse,
        )
      ).map((data) => fromBinary(VolumeSchema, data));
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  /**
   * Lists volumes.
   */
  async *listVolumesStream(
    catalogName: string,
    schemaName: string,
    options?: ListVolumesOptions,
  ): AsyncIterable<Volume> {
    const { maxResults, includeBrowse } = options || {};
    try {
      for await (const data of this.inner.listVolumesStream(
        catalogName,
        schemaName,
        maxResults,
        includeBrowse,
      )) {
        yield fromBinary(VolumeSchema, data);
      }
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  async createVolume(
    catalogName: string,
    schemaName: string,
    name: string,
    volumeType: number,
    options?: CreateVolumeOptions,
  ): Promise<Volume> {
    const { storageLocation, comment } = options || {};
    try {
      return fromBinary(
        VolumeSchema,
        await this.inner.createVolume(
          catalogName,
          schemaName,
          name,
          volumeType,
          storageLocation,
          comment,
        ),
      );
    } catch (e) {
      throw parseNativeError(e);
    }
  }

  volume(
    catalogName: string,
    schemaName: string,
    volumeName: string,
  ): VolumeClient {
    return new VolumeClient(
      this.inner.volume(catalogName, schemaName, volumeName),
    );
  }
}
