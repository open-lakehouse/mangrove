import { describe, expect, test } from "bun:test";
import { isSelfOrDescendant, renamedSelection } from "./selection.js";

describe("isSelfOrDescendant", () => {
  test("matches the object itself", () => {
    expect(
      isSelfOrDescendant({ kind: "schema", fullName: "a.b" }, "schema", "a.b"),
    ).toBe(true);
  });

  test("matches objects under a catalog or schema", () => {
    expect(
      isSelfOrDescendant({ kind: "schema", fullName: "a.b" }, "catalog", "a"),
    ).toBe(true);
    expect(
      isSelfOrDescendant(
        { kind: "volume", fullName: "a.b.v" },
        "schema",
        "a.b",
      ),
    ).toBe(true);
  });

  test("does not match siblings sharing a name prefix", () => {
    expect(
      isSelfOrDescendant({ kind: "catalog", fullName: "ab" }, "catalog", "a"),
    ).toBe(false);
    expect(
      isSelfOrDescendant({ kind: "schema", fullName: "ab.c" }, "catalog", "a"),
    ).toBe(false);
  });

  test("storage securables never live under a catalog", () => {
    expect(
      isSelfOrDescendant({ kind: "credential", fullName: "a" }, "catalog", "a"),
    ).toBe(false);
  });
});

describe("renamedSelection", () => {
  test("renames the selected object", () => {
    expect(
      renamedSelection(
        { kind: "schema", fullName: "a.b" },
        "schema",
        "a.b",
        "c",
      ),
    ).toEqual({ kind: "schema", fullName: "a.c" });
  });

  test("renames the path of a selected descendant", () => {
    expect(
      renamedSelection(
        { kind: "table", fullName: "a.b.t" },
        "catalog",
        "a",
        "z",
      ),
    ).toEqual({ kind: "table", fullName: "z.b.t" });
  });

  test("leaves unrelated selections alone", () => {
    expect(
      renamedSelection(
        { kind: "schema", fullName: "x.b" },
        "catalog",
        "a",
        "z",
      ),
    ).toBeUndefined();
  });
});
