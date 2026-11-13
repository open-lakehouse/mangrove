import { describe, expect, test } from "bun:test";
import * as fields from "./request-fields.gen.js";
import {
  type FieldSpec,
  fieldLabel,
  validateField,
  validateRequest,
} from "./spec.js";

const name: FieldSpec = {
  kind: "string",
  required: true,
  minLength: 3,
  pattern: "^[a-z][0-9a-z_]*[0-9a-z]$",
};

describe("validateField", () => {
  test("required fields reject empty values", () => {
    expect(validateField(name, undefined)).toBe("Required.");
    expect(validateField(name, "")).toBe("Required.");
  });

  test("optional fields accept empty values", () => {
    expect(validateField({ kind: "string" }, undefined)).toBeUndefined();
  });

  test("string length and identifier pattern", () => {
    expect(validateField(name, "ab")).toContain("at least 3");
    expect(validateField(name, "Bad-Name")).toContain("lowercase");
    expect(validateField(name, "sales_2026")).toBeUndefined();
  });

  test("the protovalidate URI pattern compiles and checks", () => {
    const url = fields.createCatalog.storage_root;
    expect(validateField(url, "s3://bucket/path")).toBeUndefined();
    expect(validateField(url, "not a url?#")).toBeDefined();
  });

  test("enums accept only their values", () => {
    const spec: FieldSpec = { kind: "enum", values: ["MANAGED", "EXTERNAL"] };
    expect(validateField(spec, "MANAGED")).toBeUndefined();
    expect(validateField(spec, "OTHER")).toContain("one of");
  });

  test("objects report their first invalid sub-field", () => {
    const role = fields.createCredential.aws_iam_role;
    expect(validateField(role, {})).toBe("role_arn: Required.");
    expect(validateField(role, { role_arn: "arn:x" })).toBeUndefined();
  });

  test("maps reject blank keys", () => {
    expect(validateField({ kind: "map" }, { " ": "v" })).toContain("empty");
    expect(validateField({ kind: "map" }, { k: "v" })).toBeUndefined();
  });
});

describe("validateRequest", () => {
  test("collects one error per invalid field", () => {
    const errors = validateRequest(fields.createSchema, { name: "x" });
    expect(Object.keys(errors).sort()).toEqual(["catalog_name", "name"]);
  });

  test("a complete create-schema body is valid", () => {
    expect(
      validateRequest(fields.createSchema, {
        name: "bronze",
        catalog_name: "sales",
      }),
    ).toEqual({});
  });
});

describe("generated request fields", () => {
  const all = Object.entries(fields) as [string, Record<string, FieldSpec>][];

  test("covers create and update for every base resource", () => {
    expect(all.map(([key]) => key).sort()).toEqual(
      [
        "createCatalog",
        "createCredential",
        "createExternalLocation",
        "createRegisteredModel",
        "createSchema",
        "createVolume",
        "updateCatalog",
        "updateCredential",
        "updateExternalLocation",
        "updateRegisteredModel",
        "updateSchema",
        "updateVolume",
      ].sort(),
    );
  });

  test("never includes output-only fields", () => {
    // Fields the protos mark OUTPUT_ONLY on these resources.
    const outputOnly = [
      "full_name",
      "created_at",
      "created_by",
      "updated_at",
      "updated_by",
      "metastore_id",
      "credential_id",
      "email",
      "private_key_id",
      "external_id",
      "unity_catalog_iam_arn",
    ];
    function walk(specs: Record<string, FieldSpec>, path: string) {
      for (const [key, spec] of Object.entries(specs)) {
        expect(outputOnly).not.toContain(key);
        if (spec.kind === "object") {
          walk(spec.fields as Record<string, FieldSpec>, `${path}.${key}`);
        }
      }
    }
    for (const [key, specs] of all) walk(specs, key);
  });

  test("carries required from field behavior", () => {
    expect(fields.createSchema.name.required).toBe(true);
    expect(fields.createSchema.catalog_name.required).toBe(true);
    expect(fields.createVolume.volume_type.required).toBe(true);
    expect("required" in fields.createSchema.comment).toBe(false);
  });

  test("update bodies don't carry the path identifier", () => {
    expect("full_name" in fields.updateSchema).toBe(false);
    expect("name" in fields.updateCatalog).toBe(false);
  });
});

describe("fieldLabel", () => {
  test("uses a short spec title, else the field name", () => {
    expect(fieldLabel("volume_type", fields.createVolume.volume_type)).toBe(
      "Volume Type",
    );
    expect(fieldLabel("storage_root", fields.createSchema.storage_root)).toBe(
      "Storage root",
    );
  });
});
