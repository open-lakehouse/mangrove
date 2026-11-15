// Grants vocabulary: the securable types the grants API (`/permissions/{type}/{name}`)
// takes, the privileges that apply to each, and the normalization between the
// two wire spellings of a privilege. Framework-free.
//
// The server answers in the OSS spelling (`USE CATALOG`) and accepts either it
// or the canonical underscore form (`USE_CATALOG`). Compare privileges through
// `normalizePrivilege` and send the canonical form.

/** The `securable_type` path segment of the grants API. */
export type GrantSecurableType =
  | "metastore"
  | "catalog"
  | "schema"
  | "table"
  | "function"
  | "volume"
  | "registered_model"
  | "external_location"
  | "credential";

/** The ownership pseudo-privilege. Reported, never granted: ownership moves
 *  through the securable's `owner` field. */
export const OWNER_PRIVILEGE = "OWNER";

/** The canonical (underscore, upper-case) name of a privilege in either spelling. */
export function normalizePrivilege(raw: string): string {
  return raw.trim().toUpperCase().replace(/\s+/g, "_");
}

/** A privilege for display: `USE_CATALOG` is `USE CATALOG`. */
export function displayPrivilege(raw: string): string {
  return normalizePrivilege(raw).replace(/_/g, " ");
}

// Which privileges can be granted on each securable type. Privileges inherit
// down the hierarchy, so a container lists its own privileges plus those of the
// objects it contains (granting SELECT on a schema covers its tables). The
// server does not check applicability yet, so this map is the UI's guard.
const DATA_IN_SCHEMA = [
  "USE_SCHEMA",
  "CREATE_TABLE",
  "CREATE_FUNCTION",
  "CREATE_VOLUME",
  "CREATE_MODEL",
  "CREATE_MODEL_VERSION",
  "SELECT",
  "MODIFY",
  "EXECUTE",
  "READ_VOLUME",
  "WRITE_VOLUME",
  "READ_METADATA",
  "EXTERNAL_USE_SCHEMA",
  "APPLY_TAG",
  "MANAGE",
] as const;

export const GRANTABLE_PRIVILEGES: Record<
  GrantSecurableType,
  readonly string[]
> = {
  metastore: [
    "CREATE_CATALOG",
    "CREATE_EXTERNAL_LOCATION",
    "CREATE_STORAGE_CREDENTIAL",
    "CREATE_SHARE",
    "CREATE_RECIPIENT",
    "CREATE_PROVIDER",
    "USE_SHARE",
    "USE_RECIPIENT",
    "USE_PROVIDER",
    "SET_SHARE_PERMISSION",
  ],
  catalog: [
    "ALL_PRIVILEGES",
    "BROWSE",
    "USE_CATALOG",
    "CREATE_SCHEMA",
    ...DATA_IN_SCHEMA,
  ],
  schema: ["ALL_PRIVILEGES", ...DATA_IN_SCHEMA],
  table: [
    "ALL_PRIVILEGES",
    "SELECT",
    "MODIFY",
    "READ_METADATA",
    "APPLY_TAG",
    "MANAGE",
  ],
  volume: [
    "ALL_PRIVILEGES",
    "READ_VOLUME",
    "WRITE_VOLUME",
    "APPLY_TAG",
    "MANAGE",
  ],
  function: ["ALL_PRIVILEGES", "EXECUTE", "MANAGE"],
  registered_model: [
    "ALL_PRIVILEGES",
    "EXECUTE",
    "CREATE_MODEL_VERSION",
    "APPLY_TAG",
    "MANAGE",
  ],
  external_location: [
    "ALL_PRIVILEGES",
    "BROWSE",
    "READ_FILES",
    "WRITE_FILES",
    "CREATE_EXTERNAL_TABLE",
    "CREATE_EXTERNAL_VOLUME",
    "CREATE_MANAGED_STORAGE",
    "EXTERNAL_USE_LOCATION",
    "MANAGE",
  ],
  credential: [
    "ALL_PRIVILEGES",
    "CREATE_EXTERNAL_LOCATION",
    "CREATE_EXTERNAL_TABLE",
    "READ_FILES",
    "WRITE_FILES",
    "MANAGE",
  ],
};
