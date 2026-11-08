// `UnityCatalogProvider` picks its client from, in order: the `client` prop,
// the nearest host's `unity-catalog` binding (`<HostProvider>` over
// `registerHost`), else the default client. `useOptionalEnvironmentKey` reads
// the nearest host's key without throwing when there is none.

import { afterEach, expect, test } from "bun:test";
import { type LakehouseHost, registerHost } from "@open-lakehouse/host";
import { HostProvider } from "@open-lakehouse/host/react";
import { createFakeHost } from "@open-lakehouse/host/testing";
import type { ReactNode } from "react";
import { renderToString } from "react-dom/server";
import {
  createUnityCatalogClient,
  defaultUnityCatalogClient,
  type UnityCatalogClient,
} from "../api.js";
import {
  setDefaultUnityCatalogFetch,
  UNITY_CATALOG_MODULE_ID,
} from "../fetch.js";
import { UnityCatalogProvider, useUnityCatalog } from "./context.js";
import { useOptionalEnvironmentKey } from "./host.js";

const baseUrl = "http://uc.test/api/2.1/unity-catalog";
const catalogs = (name: string) => ({ catalogs: [{ name }] });

afterEach(() => {
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

/** Render `tree` and return the client `useUnityCatalog()` saw inside it. */
function clientIn(wrap: (probe: ReactNode) => ReactNode): UnityCatalogClient {
  let seen: UnityCatalogClient | undefined;
  function Probe() {
    seen = useUnityCatalog();
    return null;
  }
  renderToString(wrap(<Probe />));
  if (!seen) throw new Error("probe did not render");
  return seen;
}

const listCatalogs = async (client: UnityCatalogClient) => {
  // Per-request base URL: the provider's client keeps the default relative one.
  const { data } = await client.fetchClient.GET("/catalogs", { baseUrl });
  return data?.catalogs?.map((c) => c.name);
};

test("without a provider, the default client is used", () => {
  expect(clientIn((probe) => probe)).toBe(defaultUnityCatalogClient);
});

test("without a client or a host, the provider uses the default client", () => {
  const client = clientIn((probe) => (
    <UnityCatalogProvider>{probe}</UnityCatalogProvider>
  ));
  expect(client).toBe(defaultUnityCatalogClient);
});

test("an explicit client wins over the host", () => {
  const explicit = createUnityCatalogClient({ baseUrl });
  const client = clientIn((probe) => (
    <HostProvider host={hostWith("from-host")}>
      <UnityCatalogProvider client={explicit}>{probe}</UnityCatalogProvider>
    </HostProvider>
  ));
  expect(client).toBe(explicit);
});

test("the provider serves the HostProvider host's binding", async () => {
  const host = hostWith("provided");
  const client = clientIn((probe) => (
    <HostProvider host={host}>
      <UnityCatalogProvider>{probe}</UnityCatalogProvider>
    </HostProvider>
  ));
  expect(await listCatalogs(client)).toEqual(["provided"]);
  expect(host.requests).toHaveLength(1);
});

test("a HostProvider host wins over the registered host", async () => {
  registerHost(hostWith("registered"));
  const client = clientIn((probe) => (
    <HostProvider host={hostWith("provided")}>
      <UnityCatalogProvider>{probe}</UnityCatalogProvider>
    </HostProvider>
  ));
  expect(await listCatalogs(client)).toEqual(["provided"]);
});

test("the provider serves the registered host's binding", async () => {
  registerHost(hostWith("registered"));
  const client = clientIn((probe) => (
    <UnityCatalogProvider>{probe}</UnityCatalogProvider>
  ));
  expect(await listCatalogs(client)).toEqual(["registered"]);
});

test("the provider's client looks the binding up on every request", async () => {
  const host = hostWith("first");
  const client = clientIn((probe) => (
    <HostProvider host={host}>
      <UnityCatalogProvider>{probe}</UnityCatalogProvider>
    </HostProvider>
  ));
  expect(await listCatalogs(client)).toEqual(["first"]);
  host.setService(UNITY_CATALOG_MODULE_ID, {
    "GET /api/2.1/unity-catalog/catalogs": catalogs("second"),
  });
  expect(await listCatalogs(client)).toEqual(["second"]);
});

test("the setDefaultUnityCatalogFetch override wins over a provided host", async () => {
  setDefaultUnityCatalogFetch(async () => Response.json(catalogs("override")));
  const client = clientIn((probe) => (
    <HostProvider host={hostWith("provided")}>
      <UnityCatalogProvider>{probe}</UnityCatalogProvider>
    </HostProvider>
  ));
  expect(await listCatalogs(client)).toEqual(["override"]);
});

/** Render and return what `useOptionalEnvironmentKey()` saw. */
function keyIn(host: LakehouseHost | null): string | null {
  let seen: string | null | undefined;
  function Probe() {
    seen = useOptionalEnvironmentKey();
    return null;
  }
  renderToString(
    host ? (
      <HostProvider host={host}>
        <Probe />
      </HostProvider>
    ) : (
      <Probe />
    ),
  );
  if (seen === undefined) throw new Error("probe did not render");
  return seen;
}

test("useOptionalEnvironmentKey is null without a host", () => {
  expect(keyIn(null)).toBeNull();
});

test("useOptionalEnvironmentKey follows the host's environment key", () => {
  const host = createFakeHost();
  const first = keyIn(host);
  expect(first).toBe(host.environment.key);
  host.setEnvironment({ id: "other", name: "other" });
  const second = keyIn(host);
  expect(second).toBe(host.environment.key);
  expect(second).not.toBe(first);
});

test("useOptionalEnvironmentKey reads a registered host", () => {
  const host = createFakeHost();
  registerHost(host);
  expect(keyIn(null)).toBe(host.environment.key);
});
