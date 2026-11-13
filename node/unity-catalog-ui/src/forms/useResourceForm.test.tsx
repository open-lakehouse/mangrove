import "../testing/dom.js";

import { describe, expect, test } from "bun:test";
import { act, renderHook } from "@testing-library/react";
import { updateCatalog, updateExternalLocation } from "./request-fields.gen.js";
import { useResourceForm } from "./useResourceForm.js";

const loaded = {
  new_name: "sales",
  comment: "Revenue",
  properties: { team: "data" },
};

function editForm(options = {}) {
  return renderHook(() =>
    useResourceForm<Record<string, unknown>>(updateCatalog, loaded, {
      patch: true,
      ...options,
    }),
  );
}

describe("changes", () => {
  test("only changed fields are sent", () => {
    const { result } = editForm();
    act(() => result.current.set("comment", "Revenue, all regions"));
    expect(
      result.current.changes(["new_name", "comment", "properties"]),
    ).toEqual({ comment: "Revenue, all regions" });
  });

  test("a cleared comment is sent as an empty string", () => {
    const { result } = editForm();
    act(() => result.current.set("comment", undefined));
    expect(result.current.changes(["comment"])).toEqual({ comment: "" });
  });

  test("cleared properties are sent as an empty map", () => {
    const { result } = editForm();
    act(() => result.current.set("properties", undefined));
    expect(result.current.changes(["properties"])).toEqual({ properties: {} });
  });

  test("nothing changed means an empty body", () => {
    const { result } = editForm();
    expect(result.current.changes(["new_name", "comment"])).toEqual({});
  });
});

describe("validation", () => {
  test("errors show only after touch or submit", () => {
    const { result } = renderHook(() =>
      useResourceForm<Record<string, unknown>>(updateExternalLocation, {}),
    );
    act(() => result.current.set("url", "not a url?#"));
    expect(result.current.errors.url).toBeDefined();
    expect(result.current.error("url")).toBeUndefined();
    act(() => result.current.touch("url"));
    expect(result.current.error("url")).toBeDefined();
  });

  test("patch mode ignores fields that weren't changed", () => {
    // A loaded value the spec would reject (e.g. a legacy name) must not
    // block saving another field.
    const { result } = renderHook(() =>
      useResourceForm<Record<string, unknown>>(
        updateCatalog,
        { new_name: "Legacy-Name" },
        { patch: true },
      ),
    );
    expect(result.current.valid).toBe(true);
    act(() => result.current.set("new_name", "Still-Bad"));
    expect(result.current.valid).toBe(false);
  });

  test("submit runs only when valid, and shows every error", () => {
    const { result } = renderHook(() =>
      useResourceForm<Record<string, unknown>>(
        updateCatalog,
        {},
        { validate: (v) => (v.new_name ? {} : { new_name: "Required." }) },
      ),
    );
    let submitted = false;
    act(() =>
      result.current.submit(() => {
        submitted = true;
      })(),
    );
    expect(submitted).toBe(false);
    expect(result.current.error("new_name")).toBe("Required.");
  });

  test("a server error naming a field lands on that field", () => {
    const { result } = editForm();
    act(() =>
      result.current.fail({
        error_code: "INVALID_PARAMETER_VALUE",
        message: "new_name must be unique",
      }),
    );
    expect(result.current.error("new_name")).toContain("must be unique");
    expect(result.current.formError).toBeUndefined();
  });

  test("other server errors go to the form", () => {
    const { result } = editForm();
    act(() => result.current.fail({ message: "permission denied" }));
    expect(result.current.formError).toBe("permission denied");
  });

  test("seed only applies once", () => {
    const { result } = renderHook(() =>
      useResourceForm<Record<string, unknown>>(updateCatalog, {}),
    );
    act(() => result.current.seed({ comment: "first" }));
    act(() => result.current.set("comment", "typed"));
    act(() => result.current.seed({ comment: "refetched" }));
    expect(result.current.values.comment).toBe("typed");
  });
});
