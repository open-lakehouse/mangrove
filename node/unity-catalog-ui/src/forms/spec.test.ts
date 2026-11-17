import { describe, expect, test } from "bun:test";
import * as fields from "./request-fields.gen.js";
import {
  type FieldSpec,
  fieldLabel,
  validateField,
  validateRequest,
} from "./spec.js";

const name: FieldSpec = fields.createCatalog.name;

describe("validateField", () => {
  test("required fields reject empty values", () => {
    expect(validateField(name, undefined)).toBe("Required.");
    expect(validateField(name, "")).toBe("Required.");
  });

  test("optional fields accept empty values", () => {
    expect(validateField({ kind: "string" }, undefined)).toBeUndefined();
  });

  test("securable names accept what Unity Catalog accepts", () => {
    for (const ok of ["a", "Sales-2026", "team@acme", "sales_2026", "Ümlaut"]) {
      expect(validateField(name, ok)).toBeUndefined();
    }
    for (const bad of ["a.b", "has space", "a/b", "tab\tname"]) {
      expect(validateField(name, bad)).toContain("periods, spaces, slashes");
    }
    expect(validateField(name, "x".repeat(256))).toContain("at most 255");
  });

  test("three-level names need three non-empty parts", () => {
    const fullName: FieldSpec = {
      kind: "string",
      required: true,
      pattern:
        "^[^./\\x00-\\x20\\x7F]+\\.[^./\\x00-\\x20\\x7F]+\\.[^./\\x00-\\x20\\x7F]+$",
    };
    expect(validateField(fullName, "Cat.sch.Tbl-1")).toBeUndefined();
    expect(validateField(fullName, "cat.sch")).toContain("three-level");
    expect(validateField(fullName, "cat..tbl")).toContain("three-level");
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
    const errors = validateRequest(fields.createSchema, { name: "has space" });
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
