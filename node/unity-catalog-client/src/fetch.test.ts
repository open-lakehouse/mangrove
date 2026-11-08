// The default client's transport resolution: the `setDefaultUnityCatalogFetch`
// override, else the registered host's `unity-catalog` binding, else the
// platform fetch — resolved on every call, so late registration, an
// environment switch and a removed binding all apply to an existing client.

import { afterEach, beforeEach, expect, test } from "bun:test";
import { registerHost } from "@open-lakehouse/host";
import { createFakeHost } from "@open-lakehouse/host/testing";
import {
  createUnityCatalogFetchClient,
  setDefaultUnityCatalogFetch,
  UNITY_CATALOG_MODULE_ID,
} from "./fetch.js";

// An absolute base URL: a relative one can't build a `Request` outside a browser.
const baseUrl = "http://uc.test/api/2.1/unity-catalog";
const catalogs = (name: string) => ({ catalogs: [{ name }] });

const realFetch = globalThis.fetch;
let platformCalls: string[];

beforeEach(() => {
  platformCalls = [];
  globalThis.fetch = (async (input: RequestInfo | URL) => {
    const url = input instanceof Request ? input.url : String(input);
    platformCalls.push(url);
    return Response.json(catalogs("platform"));
  }) as typeof globalThis.fetch;
});

afterEach(() => {
  globalThis.fetch = realFetch;
  registerHost(null);
  setDefaultUnityCatalogFetch(null);
});

const hostWith = (name: string) =>
  createFakeHost({
    services: {
      [UNITY_CATALOG_MODULE_ID]: {
        "GET /api/2.1/unity-catalog/catalogs": catalogs(name),
      },
    },
  });

const listCatalogs = async (
  client = createUnityCatalogFetchClient({ baseUrl }),
) => {
  const { data, error } = await client.GET("/catalogs");
  expect(error).toBeUndefined();
  return data?.catalogs?.map((c) => c.name);
};

test("the module id is trestle's catalog id", () => {
  expect(UNITY_CATALOG_MODULE_ID).toBe("unity-catalog");
});

test("without a host or an override, requests use the platform fetch", async () => {
  expect(await listCatalogs()).toEqual(["platform"]);
  expect(platformCalls).toEqual([`${baseUrl}/catalogs`]);
});

test("a registered host's binding serves the default client", async () => {
  const host = hostWith("from-host");
  registerHost(host);
  expect(await listCatalogs()).toEqual(["from-host"]);
  expect(platformCalls).toEqual([]);
  expect(host.requests.map((r) => r.moduleId)).toEqual(["unity-catalog"]);
});

test("a host registered after the client is built still applies", async () => {
  const client = createUnityCatalogFetchClient({ baseUrl });
  expect(await listCatalogs(client)).toEqual(["platform"]);
  registerHost(hostWith("late"));
  expect(await listCatalogs(client)).toEqual(["late"]);
});

test("switching hosts reroutes an existing client", async () => {
  const client = createUnityCatalogFetchClient({ baseUrl });
  registerHost(hostWith("a"));
  expect(await listCatalogs(client)).toEqual(["a"]);
  registerHost(hostWith("b"));
  expect(await listCatalogs(client)).toEqual(["b"]);
});

test("a host without a unity-catalog binding falls back to the platform fetch", async () => {
  const host = hostWith("from-host");
  registerHost(host);
  host.setService(UNITY_CATALOG_MODULE_ID, undefined);
  expect(await listCatalogs()).toEqual(["platform"]);
});

test("the override wins over the host, and clearing it restores the host", async () => {
  registerHost(hostWith("from-host"));
  setDefaultUnityCatalogFetch(async () => Response.json(catalogs("override")));
  expect(await listCatalogs()).toEqual(["override"]);
  setDefaultUnityCatalogFetch(null);
  expect(await listCatalogs()).toEqual(["from-host"]);
});

test("an explicit fetch option bypasses both", async () => {
  registerHost(hostWith("from-host"));
  setDefaultUnityCatalogFetch(async () => Response.json(catalogs("override")));
  const client = createUnityCatalogFetchClient({
    baseUrl,
    fetch: async () => Response.json(catalogs("explicit")),
  });
  expect(await listCatalogs(client)).toEqual(["explicit"]);
});
