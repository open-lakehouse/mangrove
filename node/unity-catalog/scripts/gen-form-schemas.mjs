// Generates the JSON Schemas that back the UI's schema-driven forms directly
// from this repo's own Unity Catalog protobuf definitions (proto/unitycatalog,
// the source of truth the server also builds from).
//
// Pipeline:
//   1. `buf generate` runs bufbuild's protoschema-jsonschema plugin against the
//      remote git ref (see buf.gen.jsonschema.yaml), emitting one self-contained
//      `*.schema.bundle.json` per request message (proto/snake_case field names,
//      all dependencies inlined under `$defs`).
//   2. We post-process each bundle into a clean, rjsf-friendly schema:
//        - inline the root `$ref` so the schema has a concrete root object,
//        - drop the camelCase `patternProperties` aliases (the REST API is
//          snake_case),
//        - collapse the proto scalar `anyOf` unions (e.g. enum int|string) to
//          their human-facing branch,
//        - keep a single draft 2020-12 `$schema` at the root.
//   3. Output is checked in under unity-catalog/src/forms/schemas/ so the UI build needs
//      no network access.
//
// Regenerate with `npm run gen:form-schemas` (or `just gen-forms`). Requires the
// `buf` CLI and network access to the BSR + GitHub.

import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const pkgDir = path.resolve(here, "..");
const tmpDir = path.join(pkgDir, ".gen-jsonschema");
const outDir = path.resolve(pkgDir, "src/forms/schemas");

/** Output file name (without extension) -> fully-qualified proto message. */
const TARGETS = {
  "create-catalog": "unitycatalog.catalogs.v1.CreateCatalogRequest",
  "create-schema": "unitycatalog.schemas.v1.CreateSchemaRequest",
  "create-credential": "unitycatalog.credentials.v1.CreateCredentialRequest",
  "update-credential": "unitycatalog.credentials.v1.UpdateCredentialRequest",
  "create-external-location":
    "unitycatalog.external_locations.v1.CreateExternalLocationRequest",
  "update-external-location":
    "unitycatalog.external_locations.v1.UpdateExternalLocationRequest",
};

const JSON_SCHEMA_DIALECT = "https://json-schema.org/draft/2020-12/schema";

// The plugin maps `buf.validate` field constraints but not `google.api.field_behavior`
// REQUIRED, so some API-required fields aren't marked required in the generated
// schema. We re-assert the well-known required fields per form so the UI enforces
// them client-side (the server enforces them regardless).
const REQUIRED = {
  "create-schema": ["name", "catalog_name"],
  "create-external-location": ["name", "url", "credential_name"],
};

// Recursively normalize a generated schema node into something clean for forms:
// strip per-subschema `$schema`/`$id`, drop camelCase `patternProperties`
// aliases, and collapse proto scalar `anyOf` unions to their nicest branch
// (a string enum, else a numeric branch, else the first).
function clean(node) {
  if (Array.isArray(node)) return node.map(clean);
  if (!node || typeof node !== "object") return node;

  const out = {};
  for (const [key, value] of Object.entries(node)) {
    if (key === "$schema" || key === "$id" || key === "patternProperties")
      continue;
    out[key] = clean(value);
  }

  if (Array.isArray(out.anyOf)) {
    const branches = out.anyOf;
    const chosen =
      branches.find((b) => b && typeof b === "object" && "enum" in b) ??
      branches.find(
        (b) =>
          b &&
          typeof b === "object" &&
          (b.type === "number" || b.type === "integer"),
      ) ??
      branches[0];
    delete out.anyOf;
    if (chosen && typeof chosen === "object") {
      for (const [bk, bv] of Object.entries(chosen)) {
        if (!(bk in out)) out[bk] = bv;
      }
    }
  }

  return out;
}

function bundleToFormSchema(bundle) {
  const defs = bundle.$defs ?? {};
  const rootRef = typeof bundle.$ref === "string" ? bundle.$ref : "";
  const rootKey = rootRef.replace("#/$defs/", "");
  const root = defs[rootKey];
  if (!root) {
    throw new Error(`Could not resolve bundle root $ref: ${rootRef}`);
  }

  // Inline the root definition so the schema has a concrete object at the top,
  // while keeping `$defs` intact for nested `$ref`s (e.g. aws_iam_role).
  const schema = clean({ ...root, $defs: defs });
  schema.$schema = JSON_SCHEMA_DIALECT;
  return schema;
}

console.log("buf generate (protoschema-jsonschema)…");
execFileSync("buf", ["generate", "--template", "buf.gen.jsonschema.yaml"], {
  cwd: pkgDir,
  stdio: "inherit",
});

fs.mkdirSync(outDir, { recursive: true });

for (const [file, type] of Object.entries(TARGETS)) {
  const bundlePath = path.join(tmpDir, `${type}.schema.bundle.json`);
  if (!fs.existsSync(bundlePath)) {
    throw new Error(`Expected bundle not generated: ${bundlePath}`);
  }
  const bundle = JSON.parse(fs.readFileSync(bundlePath, "utf8"));
  const schema = bundleToFormSchema(bundle);
  if (REQUIRED[file]) {
    schema.required = Array.from(
      new Set([...(schema.required ?? []), ...REQUIRED[file]]),
    );
  }
  const dest = path.join(outDir, `${file}.json`);
  fs.writeFileSync(dest, `${JSON.stringify(schema, null, 2)}\n`);
  console.log(`wrote ${path.relative(process.cwd(), dest)}`);
}

// The buf output is an intermediate artifact; the checked-in schemas live in the UI.
fs.rmSync(tmpDir, { recursive: true, force: true });
