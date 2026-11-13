// Generates src/forms/request-fields.gen.ts: one field-spec map per
// create/update request the UI's resource forms send, read from the canonical
// OpenAPI spec (openapi/openapi.yaml, generated from the protos and enriched
// by trestle with validation and field behavior).
//
// What the spec carries and the forms rely on:
//   - `required`: from `google.api.field_behavior = REQUIRED`,
//   - `readOnly`: from OUTPUT_ONLY. These fields are dropped, so no form can
//     send them,
//   - `minLength` / `maxLength` / `pattern` / `enum`: from buf.validate,
//   - titles and descriptions: from the proto comments.
// Path parameters (a resource's `name` / `full_name` on update) are not part of
// the request bodies, so they need no special casing.
//
// Run with `bun run gen:request-fields` in this package (no network needed).
// `just codegen generate-node-openapi` runs it after regenerating the client
// types, and node-ci fails when the checked-in output is stale.

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { parse } from "yaml";

const here = path.dirname(fileURLToPath(import.meta.url));
const pkgDir = path.resolve(here, "..");
const specPath = path.resolve(pkgDir, "../../openapi/openapi.yaml");
const outPath = path.join(pkgDir, "src/forms/request-fields.gen.ts");

/** The operations the resource forms send, by OpenAPI operationId. */
const OPERATIONS = [
  "CreateCatalog",
  "UpdateCatalog",
  "CreateSchema",
  "UpdateSchema",
  "CreateVolume",
  "UpdateVolume",
  "CreateRegisteredModel",
  "UpdateRegisteredModel",
  "CreateCredential",
  "UpdateCredential",
  "CreateExternalLocation",
  "UpdateExternalLocation",
];

// biome-ignore lint/suspicious/noExplicitAny: untyped YAML document
type Json = any;

const doc: Json = parse(fs.readFileSync(specPath, "utf8"));
const components: Record<string, Json> = doc.components.schemas;

function resolve(schema: Json): Json {
  if (schema?.$ref) {
    const name = schema.$ref.replace("#/components/schemas/", "");
    const target = components[name];
    if (!target) throw new Error(`unresolved $ref ${schema.$ref}`);
    return resolve(target);
  }
  // `allOf: [{ $ref }]` wraps a message field to attach its own docs and
  // constraints; merge the wrapper over the referenced schema.
  if (Array.isArray(schema?.allOf) && schema.allOf.length === 1) {
    const { allOf, ...rest } = schema;
    return { ...resolve(allOf[0]), ...rest };
  }
  return schema;
}

/** Collapse proto comment line breaks (" \n " continuation lines). */
function text(value: unknown): string | undefined {
  if (typeof value !== "string") return undefined;
  const out = value.replace(/\s*\n\s*/g, " ").trim();
  return out || undefined;
}

function fieldSpec(raw: Json, required: boolean): Json | undefined {
  const schema = resolve(raw);
  if (schema.readOnly || raw.readOnly) return undefined;

  const base: Json = {};
  if (required) base.required = true;
  const title = text(schema.title);
  const description = text(schema.description);
  if (title) base.title = title;
  if (description) base.description = description;

  if (Array.isArray(schema.enum)) {
    return {
      kind: "enum",
      ...base,
      // The proto zero value is never a valid choice.
      values: schema.enum.filter((v: string) => !v.endsWith("_UNSPECIFIED")),
    };
  }
  switch (schema.type) {
    case "string": {
      const spec: Json = { kind: "string", ...base };
      for (const key of ["minLength", "maxLength", "pattern"]) {
        if (schema[key] !== undefined) spec[key] = schema[key];
      }
      return spec;
    }
    case "boolean":
      return { kind: "boolean", ...base };
    case "integer":
    case "number": {
      const spec: Json = { kind: "number", ...base };
      for (const key of ["minimum", "maximum"]) {
        if (schema[key] !== undefined) spec[key] = schema[key];
      }
      return spec;
    }
    case "object":
      if (schema.properties) {
        return { kind: "object", ...base, fields: objectFields(schema) };
      }
      if (schema.additionalProperties?.type === "string") {
        return { kind: "map", ...base };
      }
      break;
  }
  throw new Error(`unsupported schema: ${JSON.stringify(schema)}`);
}

function objectFields(schema: Json): Json {
  const required = new Set<string>(schema.required ?? []);
  const fields: Json = {};
  for (const [name, raw] of Object.entries<Json>(schema.properties ?? {})) {
    const spec = fieldSpec(raw, required.has(name));
    if (spec) fields[name] = spec;
  }
  return fields;
}

interface Target {
  key: string;
  component: string;
  fields: Json;
}

function findOperation(operationId: string): Json {
  for (const item of Object.values<Json>(doc.paths)) {
    for (const op of Object.values<Json>(item)) {
      if (op?.operationId === operationId) return op;
    }
  }
  throw new Error(`operation ${operationId} not found in ${specPath}`);
}

const targets: Target[] = OPERATIONS.map((operationId) => {
  const op = findOperation(operationId);
  const ref = op.requestBody?.content?.["application/json"]?.schema?.$ref;
  if (!ref) throw new Error(`${operationId} has no JSON request body $ref`);
  const component = ref.replace("#/components/schemas/", "");
  return {
    key: operationId.charAt(0).toLowerCase() + operationId.slice(1),
    component,
    fields: objectFields(resolve({ $ref: ref })),
  };
});

const header = `// @generated by scripts/gen-request-fields.ts from openapi/openapi.yaml.
// Do not edit; run \`bun run gen:request-fields\` in node/unity-catalog-ui.

import type { components } from "@open-lakehouse/unity-catalog-client";
import type { FieldSpecs } from "./spec.js";

type Schemas = components["schemas"];
`;

const body = targets
  .map(
    ({ key, component, fields }) =>
      `\n/** Fields of \`${component}\`. */\nexport const ${key} = ${JSON.stringify(fields, null, 2)} as const satisfies FieldSpecs<Schemas["${component}"]>;\n`,
  )
  .join("");

fs.writeFileSync(outPath, header + body);
// Format like the rest of the repo, so the CI drift check compares like with like.
execFileSync("bunx", ["biome", "format", "--write", outPath], {
  stdio: "inherit",
});
console.log(`wrote ${path.relative(process.cwd(), outPath)}`);
