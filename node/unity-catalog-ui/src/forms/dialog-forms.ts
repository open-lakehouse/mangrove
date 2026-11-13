// Per-dialog form definitions: the tailored JSON Schema, the uiSchema, and the
// form data a dialog seeds before the user types anything.
//
// These live outside the dialog components so they can be validated in tests
// without rendering. Every field hidden through `ui:widget: hidden` is still
// validated by Ajv, and its errors never show, so a hidden field the schema
// requires (or gives a non-empty constraint) must be seeded here. The tests in
// dialog-forms.test.ts check that for every dialog.

import type {
  CredentialInfo,
  ExternalLocationInfo,
} from "@open-lakehouse/unity-catalog-client";
import type { RJSFSchema, UiSchema } from "@rjsf/utils";
import { cloneSchema, formSchemas } from "./schemas.js";

type LooseSchema = Record<string, unknown>;

function properties(schema: RJSFSchema): Record<string, LooseSchema> {
  return (schema.properties ?? {}) as Record<string, LooseSchema>;
}

// ── Catalog / schema create ─────────────────────────────────────────────────

export interface NamespaceFormData {
  name?: string;
  catalog_name?: string;
  comment?: string;
}

/** The create schema for a catalog or schema, reduced to name + comment. */
export function namespaceCreateSchema(kind: "catalog" | "schema"): RJSFSchema {
  const schema = cloneSchema(
    kind === "catalog" ? formSchemas.createCatalog : formSchemas.createSchema,
  );
  // The key-value `properties` map is an object field; not surfaced yet.
  delete properties(schema).properties;
  return schema;
}

// Hidden fields are still part of the schema (so injected values validate) but
// not rendered. Storage is handled by the dedicated picker below the form, and
// the parent catalog by the catalog picker, which writes `catalog_name` into
// the form data (see `namespaceFormData`).
export const NAMESPACE_CREATE_UI: UiSchema = {
  catalog_name: { "ui:widget": "hidden" },
  storage_root: { "ui:widget": "hidden" },
  storage_location: { "ui:widget": "hidden" },
  provider_name: { "ui:widget": "hidden" },
  share_name: { "ui:widget": "hidden" },
  comment: { "ui:placeholder": "Description (optional)" },
  name: { "ui:placeholder": "my_object", "ui:autofocus": true },
  "ui:order": ["name", "comment", "*"],
};

/**
 * The form data a namespace create dialog renders: what the user typed plus,
 * for a schema, the parent catalog chosen in the picker.
 */
export function namespaceFormData(
  kind: "catalog" | "schema",
  data: NamespaceFormData,
  catalog: string,
): NamespaceFormData {
  if (kind === "catalog") return data;
  return { ...data, catalog_name: catalog };
}

// ── Storage credential ──────────────────────────────────────────────────────

const AZURE_GCP_FIELDS = [
  "azure_service_principal",
  "azure_managed_identity",
  "azure_storage_key",
  "databricks_gcp_service_account",
];

export interface CredentialFormData {
  name?: string;
  purpose?: string;
  comment?: string;
  new_name?: string;
  aws_iam_role?: { role_arn?: string };
}

/**
 * Reduce the generated credential schema to the AWS IAM role scope. In create
 * mode the role and its ARN are required. In edit mode they stay optional, so
 * a credential of another cloud can still be renamed or re-commented.
 */
export function credentialSchema(mode: "create" | "edit"): RJSFSchema {
  const schema = cloneSchema(
    mode === "create"
      ? formSchemas.createCredential
      : formSchemas.updateCredential,
  );
  const props = properties(schema);
  for (const field of AZURE_GCP_FIELDS) delete props[field];
  if (mode === "edit") return schema;

  schema.required = Array.from(
    new Set([...(schema.required ?? []), "aws_iam_role"]),
  );
  const ref = (
    props.aws_iam_role as { $ref?: string } | undefined
  )?.$ref?.replace("#/$defs/", "");
  const defs = schema.$defs as Record<string, LooseSchema> | undefined;
  const awsDef = ref ? defs?.[ref] : undefined;
  if (awsDef) {
    awsDef.required = Array.from(
      new Set([...((awsDef.required as string[]) ?? []), "role_arn"]),
    );
  }
  return schema;
}

const AWS_ROLE_UI: UiSchema = {
  "ui:title": "AWS IAM role",
  role_arn: {
    "ui:title": "IAM role ARN",
    "ui:placeholder": "arn:aws:iam::<account-id>:role/<role-name>",
  },
  region: { "ui:widget": "hidden" },
  access_key_id: { "ui:widget": "hidden" },
  secret_access_key: { "ui:widget": "hidden" },
  session_token: { "ui:widget": "hidden" },
};

export function credentialUiSchema(
  mode: "create" | "edit",
  name?: string,
): UiSchema {
  const base: UiSchema = {
    purpose: { "ui:widget": "hidden" },
    read_only: { "ui:widget": "hidden" },
    skip_validation: { "ui:widget": "hidden" },
    force: { "ui:widget": "hidden" },
    owner: { "ui:widget": "hidden" },
    comment: { "ui:placeholder": "Description (optional)" },
    aws_iam_role: AWS_ROLE_UI,
  };
  if (mode === "create") {
    return {
      ...base,
      "ui:order": ["name", "aws_iam_role", "comment", "*"],
      name: { "ui:placeholder": "my_credential", "ui:autofocus": true },
    };
  }
  return {
    ...base,
    "ui:order": ["new_name", "aws_iam_role", "comment", "*"],
    // `name` identifies the credential (path param); rename uses `new_name`.
    name: { "ui:widget": "hidden" },
    new_name: { "ui:title": "Name", "ui:placeholder": name },
  };
}

/** Create seed: `purpose` is required but fixed to STORAGE (and hidden). */
export const CREDENTIAL_CREATE_SEED: CredentialFormData = {
  name: "",
  purpose: "STORAGE",
};

/**
 * Edit seed from the loaded credential. `name` is hidden but required by the
 * update schema. `purpose` is not part of the update request, so it is not
 * seeded. The AWS role is seeded only when the credential has one.
 */
export function credentialEditSeed(loaded: CredentialInfo): CredentialFormData {
  const roleArn = loaded.aws_iam_role?.role_arn;
  return {
    name: loaded.name,
    new_name: loaded.name,
    comment: loaded.comment,
    ...(roleArn ? { aws_iam_role: { role_arn: roleArn } } : {}),
  };
}

// ── External location ───────────────────────────────────────────────────────

export interface ExternalLocationFormData {
  name?: string;
  new_name?: string;
  url?: string;
  credential_name?: string;
  comment?: string;
}

/** The external-location schema, with credential names injected as an enum. */
export function externalLocationSchema(
  mode: "create" | "edit",
  credentialNames: string[],
): RJSFSchema {
  const schema = cloneSchema(
    mode === "create"
      ? formSchemas.createExternalLocation
      : formSchemas.updateExternalLocation,
  );
  const props = properties(schema);
  if (props.credential_name && credentialNames.length > 0) {
    props.credential_name = { ...props.credential_name, enum: credentialNames };
  }
  return schema;
}

export function externalLocationUiSchema(
  mode: "create" | "edit",
  name?: string,
): UiSchema {
  const base: UiSchema = {
    read_only: { "ui:widget": "hidden" },
    skip_validation: { "ui:widget": "hidden" },
    force: { "ui:widget": "hidden" },
    owner: { "ui:widget": "hidden" },
    url: { "ui:placeholder": "s3://bucket/path" },
    credential_name: { "ui:placeholder": "Select a storage credential" },
    comment: { "ui:placeholder": "Description (optional)" },
  };
  if (mode === "create") {
    return {
      ...base,
      "ui:order": ["name", "url", "credential_name", "comment", "*"],
      name: {
        "ui:placeholder": "my_external_location",
        "ui:autofocus": true,
      },
    };
  }
  return {
    ...base,
    "ui:order": ["new_name", "url", "credential_name", "comment", "*"],
    // `name` identifies the location (path param); rename uses `new_name`.
    name: { "ui:widget": "hidden" },
    new_name: { "ui:title": "Name", "ui:placeholder": name },
  };
}

export const EXTERNAL_LOCATION_CREATE_SEED: ExternalLocationFormData = {
  name: "",
  url: "",
};

/** Edit seed from the loaded location. `name` is hidden but validated. */
export function externalLocationEditSeed(
  loaded: ExternalLocationInfo,
): ExternalLocationFormData {
  return {
    name: loaded.name,
    new_name: loaded.name,
    url: loaded.url,
    credential_name: loaded.credential_name,
    comment: loaded.comment,
  };
}

// ── Comment updates ─────────────────────────────────────────────────────────

/**
 * The `comment` to send in an update. An omitted field means "unchanged" to the
 * server, so a comment the user cleared must be sent as `""`. An unchanged
 * comment is left out, so it can't overwrite a concurrent edit.
 */
export function commentPatch(
  next: string | undefined,
  previous: string | undefined,
): string | undefined {
  const value = next ?? "";
  if (value === (previous ?? "")) return undefined;
  return value;
}
