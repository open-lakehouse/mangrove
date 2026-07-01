import createFetchClient from "openapi-fetch";
import createQueryClient from "openapi-react-query";
import type { paths } from "./uc-types";

/**
 * The fetch the default client uses. It is a stable indirection over a mutable
 * slot so the default client (and its back-compat `$api` / `fetchClient`
 * singletons, captured at module init) can be repointed at a host-supplied fetch
 * AFTER construction — without rebuilding the client. Defaults to the platform
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

/**
 * A constructed Unity Catalog client: the openapi-fetch transport plus its
 * TanStack Query binding. This is the unit the UC data layer depends on — it is
 * passed in via `UnityCatalogProvider` rather than imported as a singleton, so
 * the base URL / transport / auth are the host's decision, not the component's.
 *
 * Inverting the client this way is the seam that lets the OpenAPI client be
 * swapped for the proto-generated WASM client later with no change to the hooks
 * (see docs/portable-uc-components.md, decision 2). Keep it injectable.
 */
export interface UnityCatalogClient {
  /** TanStack Query bindings; auto-derives `["get", path, init]` query keys. */
  $api: ReturnType<typeof createQueryClient<paths>>;
  /** The raw typed fetch client, for non-hook query functions (prefetch). */
  fetchClient: ReturnType<typeof createFetchClient<paths>>;
}

/** Options for {@link createUnityCatalogClient}. */
export interface CreateUnityCatalogClientOptions {
  /**
   * Base URL for the Unity Catalog REST API. Defaults to the Databricks-parallel
   * root path the Envoy gateway routes to the UC server (see
   * environments/docker/envoy/envoy.yaml); the Vite dev proxy forwards /api to
   * the gateway (see vite.config.ts).
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
 * Construct a {@link UnityCatalogClient}. The default instance
 * ({@link defaultUnityCatalogClient}) is built from the same config the singleton
 * used historically, so default behavior is unchanged.
 */
export function createUnityCatalogClient(
  opts: CreateUnityCatalogClientOptions = {},
): UnityCatalogClient {
  const fetchClient = createFetchClient<paths>({
    baseUrl:
      opts.baseUrl ?? import.meta.env.VITE_API_URL ?? "/api/2.1/unity-catalog",
    fetch: opts.fetch ?? defaultFetch,
  });
  // `$api` wraps the fetch client with TanStack Query bindings. It auto-derives
  // a query key of the form ["get", path, init] for every request. We treat that
  // as the canonical key for a resource everywhere (reads, prefetch,
  // invalidation), so keys never drift — see lib/uc/queries.ts for the conventions.
  const $api = createQueryClient(fetchClient);
  return { $api, fetchClient };
}

/**
 * The default, app-wide client instance — identical config to the historical
 * module singleton. `UnityCatalogProvider` falls back to this when no client is
 * supplied, and the non-hook query helpers (`*DetailQuery`, `prefetch*`) bind to
 * it directly (query-key derivation is client-independent, so caches still align).
 */
export const defaultUnityCatalogClient = createUnityCatalogClient();

// Back-compat singleton exports. These are the default instance's members, so
// any code still importing them keeps identical behavior. Prefer reading the
// client from `useUnityCatalog()` in React code.
export const $api = defaultUnityCatalogClient.$api;
const fetchClient = defaultUnityCatalogClient.fetchClient;

export { fetchClient };
