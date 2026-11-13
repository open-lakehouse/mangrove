import { describe, expect, test } from "bun:test";
import { QueryClient, type QueryKey } from "@tanstack/react-query";
import { optimisticRemove, restore } from "./cache.js";
import {
  removeCatalogDescendants,
  removeSchemaDescendants,
} from "./mutations.js";

// Keys in the shape openapi-react-query derives: ["get", path, init].
const tablesKey = (catalog: string, schema: string): QueryKey => [
  "get",
  "/tables",
  { params: { query: { catalog_name: catalog, schema_name: schema } } },
];
const schemasKey = (catalog: string): QueryKey => [
  "get",
  "/schemas",
  { params: { query: { catalog_name: catalog } } },
];
const detailKey = (path: string, id: string): QueryKey => [
  "get",
  path,
  { params: { path: { full_name: id } } },
];

const page = (names: string[], prefix: string) => ({
  pages: [
    { tables: names.map((name) => ({ name, full_name: `${prefix}.${name}` })) },
  ],
  pageParams: [undefined],
});

function names(client: QueryClient, key: QueryKey): string[] {
  const data = client.getQueryData<ReturnType<typeof page>>(key);
  return data?.pages.flatMap((p) => p.tables.map((t) => t.name)) ?? [];
}

function seeded() {
  const client = new QueryClient();
  client.setQueryData(tablesKey("a", "s"), page(["t", "u"], "a.s"));
  client.setQueryData(tablesKey("b", "s"), page(["t"], "b.s"));
  return client;
}

describe("optimisticRemove", () => {
  test("removes the row only from the lists inside the scope", () => {
    const client = seeded();
    optimisticRemove(
      client,
      "/tables",
      "tables",
      { catalog_name: "a", schema_name: "s" },
      (item) => item.full_name === "a.s.t" || item.name === "t",
    );
    expect(names(client, tablesKey("a", "s"))).toEqual(["u"]);
    // Same leaf name, different catalog: untouched.
    expect(names(client, tablesKey("b", "s"))).toEqual(["t"]);
  });

  test("restore puts the removed rows back", () => {
    const client = seeded();
    const snapshots = optimisticRemove(
      client,
      "/tables",
      "tables",
      { catalog_name: "a", schema_name: "s" },
      (item) => item.name === "t",
    );
    restore(client, snapshots);
    expect(names(client, tablesKey("a", "s"))).toEqual(["t", "u"]);
  });
});

describe("descendant purges", () => {
  test("removeSchemaDescendants drops leaf lists and details under the schema", () => {
    const client = seeded();
    client.setQueryData(detailKey("/tables/{full_name}", "a.s.t"), {});
    client.setQueryData(detailKey("/tables/{full_name}", "a.st.x"), {});
    removeSchemaDescendants(client, "a.s");

    expect(client.getQueryData(tablesKey("a", "s"))).toBeUndefined();
    expect(
      client.getQueryData(detailKey("/tables/{full_name}", "a.s.t")),
    ).toBeUndefined();
    // A sibling schema whose name shares a prefix is kept.
    expect(
      client.getQueryData(detailKey("/tables/{full_name}", "a.st.x")),
    ).toBeDefined();
    expect(client.getQueryData(tablesKey("b", "s"))).toBeDefined();
  });

  test("removeCatalogDescendants drops schema lists, leaf lists and details", () => {
    const client = seeded();
    client.setQueryData(schemasKey("a"), { pages: [], pageParams: [] });
    client.setQueryData(detailKey("/schemas/{full_name}", "a.s"), {});
    removeCatalogDescendants(client, "a");

    expect(client.getQueryData(schemasKey("a"))).toBeUndefined();
    expect(client.getQueryData(tablesKey("a", "s"))).toBeUndefined();
    expect(
      client.getQueryData(detailKey("/schemas/{full_name}", "a.s")),
    ).toBeUndefined();
    expect(client.getQueryData(tablesKey("b", "s"))).toBeDefined();
  });
});
