import { getHost, type LakehouseHost } from "@open-lakehouse/host";
import createFetchClient from "openapi-fetch";
import type { paths } from "./uc-types.js";

/**
 * The module id Unity Catalog is bound under in an `@open-lakehouse/host`
 * service directory (trestle's catalog module id).
 */
export const UNITY_CATALOG_MODULE_ID = "unity-catalog";

/** The explicit override set by {@link setDefaultUnityCatalogFetch}, if any. */
let override: typeof globalThis.fetch | null = null;

/** The platform fetch, read per call so a patched `globalThis.fetch` applies. */
const platformFetch: typeof globalThis.fetch = (input, init) =>
  globalThis.fetch(input, init);

/**
 * A fetch that resolves the Unity Catalog transport on every call:
 *
 * 1. the override set by {@link setDefaultUnityCatalogFetch},
 * 2. else the `unity-catalog` binding of the host `currentHost()` returns,
 * 3. else the platform `fetch` (a UI running standalone on its own origin).
 *
 * Resolving per call means neither the order of host registration and client
 * construction nor an environment switch needs the client to be rebuilt.
 *
 * @internal Used by the default client and `UnityCatalogProvider`.
 */
export function unityCatalogFetchFor(
  currentHost: () => LakehouseHost | null,
): typeof globalThis.fetch {
  return (input, init) => {
    if (override) return override(input, init);
    const binding = currentHost()?.services.get(UNITY_CATALOG_MODULE_ID);
    return binding ? binding.fetch(input, init) : platformFetch(input, init);
  };
}

/**
 * The fetch the default client uses: the {@link setDefaultUnityCatalogFetch}
 * override, else the registered host's `unity-catalog` binding
 * (`registerHost` from `@open-lakehouse/host`), else the platform `fetch`.
 */
const defaultFetch = unityCatalogFetchFor(getHost);

/**
 * Repoint the default Unity Catalog client at an explicit fetch, over any
 * registered host. Pass `null` to clear it and go back to the host binding (or
 * the platform `fetch`).
 *
 * Hosts that implement the `@open-lakehouse/host` contract don't need this:
 * registering the host is enough. It stays as the explicit override for hosts
 * that route UC calls through their own fetch (auth, IPC, …).
 */
export function setDefaultUnityCatalogFetch(
  fetch: typeof globalThis.fetch | null,
) {
  override = fetch;
}

/** The raw typed `openapi-fetch` client for the Unity Catalog REST API. */
export type UnityCatalogFetchClient = ReturnType<
  typeof createFetchClient<paths>
>;

/** Options for {@link createUnityCatalogFetchClient}. */
export interface CreateUnityCatalogClientOptions {
  /**
   * Base URL for the Unity Catalog REST API. Defaults to `VITE_API_URL` when
   * the host is built with Vite, else the Databricks-parallel root path the
   * Envoy gateway routes to the UC server (see
   * environments/docker/envoy/envoy.yaml).
   */
  baseUrl?: string;
  /**
   * Fetch implementation. Defaults to the package's default fetch (the
   * {@link setDefaultUnityCatalogFetch} override, else the registered host's
   * `unity-catalog` binding, else the platform `fetch`). Pass an explicit fetch
   * to build a client with a fixed transport.
   */
  fetch?: typeof globalThis.fetch;
}

/**
 * Construct a framework-free {@link UnityCatalogFetchClient}. Works in any
 * runtime with a `fetch` (browsers, Node ≥ 18, Bun, Deno); React hosts usually
 * want `createUnityCatalogClient` from `@open-lakehouse/unity-catalog-client/react`
 * instead, which wraps this with TanStack Query bindings.
 */
export function createUnityCatalogFetchClient(
  opts: CreateUnityCatalogClientOptions = {},
): UnityCatalogFetchClient {
  return createFetchClient<paths>({
    // `import.meta.env` only exists under Vite; guard it so the package also
    // loads in plain Node / other bundlers.
    baseUrl:
      opts.baseUrl ?? import.meta.env?.VITE_API_URL ?? "/api/2.1/unity-catalog",
    fetch: opts.fetch ?? defaultFetch,
  });
}

/**
 * The default, app-wide fetch client over the default fetch. The React
 * default client (`defaultUnityCatalogClient`) wraps this same instance, so
 * both share one transport.
 */
export const fetchClient = createUnityCatalogFetchClient();
