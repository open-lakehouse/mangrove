/**
 * Three-level fully-qualified name (`catalog.schema.object`). Several OSS list
 * payloads omit `full_name` from the typed schema (the server populates it at
 * runtime), so we derive it deterministically from the namespace parts. This is
 * the single source of truth for both detail-cache keys and display.
 */
export function objectFullName(parts: {
  catalog_name?: string;
  schema_name?: string;
  name?: string;
}): string {
  return [parts.catalog_name, parts.schema_name, parts.name]
    .filter(Boolean)
    .join(".");
}

/** Back-compat alias; prefer `objectFullName`. */
export const tableFullName = objectFullName;
