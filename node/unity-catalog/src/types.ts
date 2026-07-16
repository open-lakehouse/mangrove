// Shared types for the Catalog Explorer.

import type { CredentialInfo } from "@open-lakehouse/unity-catalog-client";

export type ObjectKind = "table" | "volume" | "function" | "model";

/**
 * AWS IAM trust material shown on a credential (the external ID + the managed
 * Unity Catalog IAM ARN the user wires into their role's trust policy).
 *
 * TODO(#85): the canonical proto-derived spec types `Credential.aws_iam_role` as
 * the internal `AwsIamRoleConfig` (role_arn + static base creds) rather than the
 * public `AwsIamRole` response shape, so these two output-only trust fields are
 * absent from the generated type. Read them through this narrow view until #85
 * splits the response model and populates them server-side; then delete this.
 */
export type AwsIamRoleTrust = {
  external_id?: string;
  unity_catalog_iam_arn?: string;
};

/** Read the (currently un-typed — see #85) AWS IAM trust material off a credential. */
export function awsIamRoleTrust(
  credential: Pick<CredentialInfo, "aws_iam_role"> | null | undefined,
): AwsIamRoleTrust {
  return (credential?.aws_iam_role ?? {}) as AwsIamRoleTrust;
}

export const OBJECT_KINDS: ObjectKind[] = [
  "table",
  "volume",
  "function",
  "model",
];

/**
 * Metastore-level securables that live outside the three-level namespace:
 * storage credentials and external locations.
 */
export type StorageKind = "credential" | "external_location";

export const STORAGE_KINDS: StorageKind[] = ["external_location", "credential"];

/**
 * Everything that can be selected and shown in the detail pane: the two
 * namespace containers (catalog, schema), the leaf objects, and the
 * metastore-level storage securables. The type-level group rows
 * (Tables/Volumes/...) are intentionally NOT here — they only expand.
 */
export type SelectableKind = "catalog" | "schema" | ObjectKind | StorageKind;

export const SELECTABLE_KINDS: SelectableKind[] = [
  "catalog",
  "schema",
  ...OBJECT_KINDS,
  ...STORAGE_KINDS,
];

export function isObjectKind(kind: SelectableKind): kind is ObjectKind {
  return (OBJECT_KINDS as string[]).includes(kind);
}

/**
 * A selected node. We deliberately store only the kind + fully-qualified name
 * here (not the payload): the name is enough to address the object in the URL
 * and to look its details up from the query cache. See selection.ts.
 */
export interface Selection {
  kind: SelectableKind;
  fullName: string;
}

/** Split a `catalog.schema.object` name into its namespace parts. */
export function splitFullName(fullName: string): {
  catalog?: string;
  schema?: string;
  object?: string;
} {
  const [catalog, schema, object] = fullName.split(".");
  return { catalog, schema, object };
}
