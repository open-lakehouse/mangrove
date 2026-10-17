import createFetchClient from "openapi-fetch";
import type { paths } from "./uc-types";

/**
 * The fetch the default client uses. It is a stable indirection over a mutable
 * slot so the default client (and its back-compat `fetchClient` singleton,
 * captured at module init) can be repointed at a host-supplied fetch AFTER
 * construction — without rebuilding the client. Defaults to the platform
 * `fetch`; a host (e.g. the Tauri desktop shell, which speaks over its own IPC
 * fetch) calls {@link setDefaultUnityCatalogFetch} once at startup.
 */
let currentDefaultFetch: typeof globalThis.fetch = (...args) =>
  globalThis.fetch(...args);
const defaultFetch: typeof globalThis.fetch = (...args) =>
  currentDefaultFetch(...args);

/**
 * Repoint the default Unity Catalog client at a host-supplied fetch. Call once
 * at startup, before the first request. This is the transport seam the package
 * exposes so a host can route UC calls through its own fetch (auth, IPC, …)
 * without the package depending on the host's transport registry.
 */
export function setDefaultUnityCatalogFetch(fetch: typeof globalThis.fetch) {
  currentDefaultFetch = fetch;
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
   * Fetch implementation. Defaults to the package's mutable default fetch (the
   * platform `fetch`, unless a host called {@link setDefaultUnityCatalogFetch}).
   * Pass an explicit fetch to build a client with a fixed transport.
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
      opts.baseUrl ??
      import.meta.env?.VITE_API_URL ??
      "/api/2.1/unity-catalog",
    fetch: opts.fetch ?? defaultFetch,
  });
}

/**
 * The default, app-wide fetch client over the mutable default fetch. The React
 * default client (`defaultUnityCatalogClient`) wraps this same instance, so
 * both share one transport.
 */
export const fetchClient = createUnityCatalogFetchClient();
