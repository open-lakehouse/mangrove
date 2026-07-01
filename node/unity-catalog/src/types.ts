// Shared types for the Catalog Explorer.

export type ObjectKind = "table" | "volume" | "function" | "model";

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
