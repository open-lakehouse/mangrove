// The fixture handler answers the client's own requests: lists filtered by
// their parent, details by full name, and 404 for unknown entities and writes.

import { afterEach, expect, test } from "bun:test";
import { registerHost } from "@open-lakehouse/host";
import { createFakeHost } from "@open-lakehouse/host/testing";
import {
  createUnityCatalogFetchClient,
  UNITY_CATALOG_MODULE_ID,
} from "./fetch.js";
import { ucFixtureHandler } from "./testing.js";

afterEach(() => registerHost(null));

const client = () => {
  registerHost(
    createFakeHost({
      services: { [UNITY_CATALOG_MODULE_ID]: ucFixtureHandler() },
    }),
  );
  return createUnityCatalogFetchClient({
    baseUrl: "http://uc.test/api/2.1/unity-catalog",
  });
};

test("lists catalogs, and schemas and tables by parent", async () => {
  const uc = client();
  const catalogs = await uc.GET("/catalogs");
  expect(catalogs.data?.catalogs?.map((c) => c.name)).toEqual([
    "main",
    "scratch",
  ]);
  const schemas = await uc.GET("/schemas", {
    params: { query: { catalog_name: "main" } },
  });
  expect(schemas.data?.schemas?.map((s) => s.full_name)).toEqual([
    "main.sales",
  ]);
  const empty = await uc.GET("/schemas", {
    params: { query: { catalog_name: "scratch" } },
  });
  expect(empty.data?.schemas).toEqual([]);
  const tables = await uc.GET("/tables", {
    params: { query: { catalog_name: "main", schema_name: "sales" } },
  });
  expect(tables.data?.tables?.map((t) => t.name)).toEqual([
    "orders",
    "orders_summary",
  ]);
});

test("serves details by full name and 404s unknown ones", async () => {
  const uc = client();
  const table = await uc.GET("/tables/{full_name}", {
    params: { path: { full_name: "main.sales.orders" } },
  });
  expect(table.data?.columns?.map((c) => c.name)).toContain("order_id");
  const volume = await uc.GET("/volumes/{name}", {
    params: { path: { name: "main.sales.raw_files" } },
  });
  expect(volume.data?.volume_type).toBe("EXTERNAL");
  const missing = await uc.GET("/tables/{full_name}", {
    params: { path: { full_name: "main.sales.nope" } },
  });
  expect(missing.response.status).toBe(404);
});

test("rejects writes", async () => {
  const uc = client();
  const created = await uc.POST("/catalogs", { body: { name: "new" } });
  expect(created.response.status).toBe(404);
});
