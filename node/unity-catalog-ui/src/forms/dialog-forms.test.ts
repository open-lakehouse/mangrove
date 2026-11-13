// Every dialog's seeded form data must pass the same validation the rendered
// form runs on submit. A hidden field the schema rejects blocks submit with no
// visible error, which is how schema create, credential edit and
// external-location edit used to fail silently.

import { describe, expect, test } from "bun:test";
import type { RJSFSchema, UiSchema } from "@rjsf/utils";
import {
  CREDENTIAL_CREATE_SEED,
  commentPatch,
  credentialEditSeed,
  credentialSchema,
  credentialUiSchema,
  EXTERNAL_LOCATION_CREATE_SEED,
  externalLocationEditSeed,
  externalLocationSchema,
  externalLocationUiSchema,
  NAMESPACE_CREATE_UI,
  namespaceCreateSchema,
  namespaceFormData,
} from "./dialog-forms.js";
import { invisibleErrors, validateFormData } from "./SchemaForm.js";

function messages(schema: RJSFSchema, data: unknown): string[] {
  return validateFormData(schema, data).map((e) => e.stack);
}

/** Errors on fields the user can't see, so can't fix. */
function hiddenErrors(
  schema: RJSFSchema,
  uiSchema: UiSchema,
  data: unknown,
): string[] {
  return invisibleErrors(validateFormData(schema, data), uiSchema).map(
    (e) => e.stack,
  );
}

describe("catalog and schema create", () => {
  test("a catalog with a valid name validates", () => {
    const data = namespaceFormData("catalog", { name: "sales" }, "");
    expect(messages(namespaceCreateSchema("catalog"), data)).toEqual([]);
  });

  test("a schema validates once a catalog is picked", () => {
    const data = namespaceFormData(
      "schema",
      { name: "bronze", comment: "raw" },
      "sales",
    );
    expect(data.catalog_name).toBe("sales");
    expect(messages(namespaceCreateSchema("schema"), data)).toEqual([]);
  });

  test("an empty form only reports errors on visible fields", () => {
    for (const kind of ["catalog", "schema"] as const) {
      const data = namespaceFormData(kind, {}, "sales");
      expect(
        hiddenErrors(namespaceCreateSchema(kind), NAMESPACE_CREATE_UI, data),
      ).toEqual([]);
    }
  });

  test("an invalid name is rejected on the visible name field", () => {
    const errors = validateFormData(
      namespaceCreateSchema("schema"),
      namespaceFormData("schema", { name: "Bad-Name" }, "sales"),
    );
    expect(errors.map((e) => e.property)).toContain(".name");
  });
});

describe("storage credential", () => {
  test("the create seed plus an ARN validates", () => {
    const data = {
      ...CREDENTIAL_CREATE_SEED,
      name: "my_role",
      aws_iam_role: { role_arn: "arn:aws:iam::123456789012:role/uc" },
    };
    expect(messages(credentialSchema("create"), data)).toEqual([]);
  });

  test("an AWS credential's edit seed validates", () => {
    const seed = credentialEditSeed({
      name: "my_role",
      purpose: "STORAGE",
      comment: "c",
      aws_iam_role: { role_arn: "arn:aws:iam::123456789012:role/uc" },
    } as never);
    expect(seed).not.toHaveProperty("purpose");
    expect(messages(credentialSchema("edit"), seed)).toEqual([]);
    expect(
      hiddenErrors(credentialSchema("edit"), credentialUiSchema("edit"), seed),
    ).toEqual([]);
  });

  test("a non-AWS credential's edit seed validates without a role", () => {
    const seed = credentialEditSeed({
      name: "azure_cred",
      purpose: "STORAGE",
      azure_managed_identity: { access_connector_id: "x" },
    } as never);
    expect(seed).not.toHaveProperty("aws_iam_role");
    expect(messages(credentialSchema("edit"), seed)).toEqual([]);
  });
});

describe("external location", () => {
  test("a filled create form validates", () => {
    const data = {
      ...EXTERNAL_LOCATION_CREATE_SEED,
      name: "landing",
      url: "s3://bucket/landing",
      credential_name: "my_role",
    };
    expect(
      messages(externalLocationSchema("create", ["my_role"]), data),
    ).toEqual([]);
  });

  test("the edit seed validates", () => {
    const seed = externalLocationEditSeed({
      name: "landing",
      url: "s3://bucket/landing",
      credential_name: "my_role",
    } as never);
    const schema = externalLocationSchema("edit", ["my_role"]);
    expect(messages(schema, seed)).toEqual([]);
    expect(
      hiddenErrors(schema, externalLocationUiSchema("edit"), seed),
    ).toEqual([]);
  });
});

describe("commentPatch", () => {
  test("an unchanged comment is omitted", () => {
    expect(commentPatch("same", "same")).toBeUndefined();
    expect(commentPatch(undefined, undefined)).toBeUndefined();
    expect(commentPatch("", undefined)).toBeUndefined();
  });

  test("a cleared comment is sent as an empty string", () => {
    expect(commentPatch("", "old")).toBe("");
    expect(commentPatch(undefined, "old")).toBe("");
  });

  test("a changed comment is sent", () => {
    expect(commentPatch("new", "old")).toBe("new");
  });
});

describe("invisibleErrors", () => {
  // These are the seeds the dialogs used to send; the form-level alert must
  // surface them instead of blocking submit silently.
  test("reports an error on a hidden field", () => {
    const errors = hiddenErrors(
      namespaceCreateSchema("schema"),
      NAMESPACE_CREATE_UI,
      { name: "bronze" },
    );
    expect(errors.some((e) => e.includes("catalog_name"))).toBe(true);
  });

  test("reports a root-level additionalProperties error", () => {
    const errors = hiddenErrors(
      credentialSchema("edit"),
      credentialUiSchema("edit"),
      { name: "my_role", new_name: "my_role", purpose: "STORAGE" },
    );
    expect(errors.some((e) => e.includes("additional properties"))).toBe(true);
  });

  test("leaves errors on visible fields to the field itself", () => {
    const errors = hiddenErrors(
      namespaceCreateSchema("catalog"),
      NAMESPACE_CREATE_UI,
      { name: "x" },
    );
    expect(errors).toEqual([]);
  });
});
