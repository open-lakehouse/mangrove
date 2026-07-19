// Unit tests for the catalog-completion narrowing logic — the dotted-path →
// catalog/schema/table/column resolution and the parser-context fallback — run
// against the fixture catalog provider. No Monaco worker or live editor needed:
// `buildItems` takes a `Suggestions` object and a dotted prefix directly.

import { beforeAll, describe, expect, test } from "bun:test";
import { EntityContextType, type Suggestions } from "dt-sql-parser";
import { fixtureCatalogProvider } from "../fixtures";
import { registerCatalogProvider } from "./catalogProvider";
import { __test } from "./completionService";

const { dottedPrefix, buildItems, tablesInStatement } = __test;

beforeAll(() => {
  registerCatalogProvider(fixtureCatalogProvider);
});

const labels = (items: Awaited<ReturnType<typeof buildItems>>) =>
  items.map((i) => i.label).sort();

describe("dottedPrefix", () => {
  test("non-dotted text → null (only the service coerces to [])", () => {
    expect(dottedPrefix("select * from foo")).toBeNull();
  });
  test("trailing dot → segments before the (empty) partial", () => {
    expect(dottedPrefix("select * from main.")).toEqual(["main"]);
  });
  test("partial word after dot is dropped", () => {
    expect(dottedPrefix("from main.def")).toEqual(["main"]);
  });
  test("two dots → two segments", () => {
    expect(dottedPrefix("from main.default.")).toEqual(["main", "default"]);
  });
  test("non-dotted identifier → null", () => {
    expect(dottedPrefix("foo")).toBeNull();
  });
});

describe("buildItems narrowing", () => {
  test("catalog. → schemas of that catalog", async () => {
    const items = await buildItems(null, ["main"]);
    expect(labels(items)).toEqual(["analytics", "default"]);
  });

  test("catalog.schema. → tables of that schema", async () => {
    const items = await buildItems(null, ["main", "default"]);
    expect(labels(items)).toEqual(["events", "users"]);
  });

  test("catalog.schema.table. → columns of that table", async () => {
    const items = await buildItems(null, ["main", "default", "users"]);
    expect(labels(items)).toEqual(["created_at", "email", "events", "id"]);
    // detail carries the column type.
    const email = items.find((i) => i.label === "email");
    expect(email?.detail).toContain("string");
  });

  test("no prefix + TABLE context → catalogs offered", async () => {
    const suggestions: Suggestions = {
      syntax: [{ syntaxContextType: EntityContextType.TABLE, wordRanges: [] }],
      keywords: [],
    };
    const items = await buildItems(suggestions, []);
    expect(labels(items)).toEqual(["main", "samples"]);
  });

  test("unknown catalog → empty (no throw)", async () => {
    const items = await buildItems(null, ["nope"]);
    expect(items).toEqual([]);
  });
});

describe("tablesInStatement", () => {
  test("collects 3-part table refs from TABLE syntax context", () => {
    // dt-sql-parser yields one wordRange per identifier segment (no dot entries).
    const wr = (text: string) => ({
      text,
      startColumn: 1,
      endColumn: 1,
      startLine: 1,
      endLine: 1,
    });
    const suggestions = {
      syntax: [
        {
          syntaxContextType: EntityContextType.TABLE,
          wordRanges: [wr("main"), wr("default"), wr("users")],
        },
      ],
      keywords: [],
    } as unknown as Suggestions;
    expect(tablesInStatement(suggestions)).toEqual(["main.default.users"]);
  });
});
