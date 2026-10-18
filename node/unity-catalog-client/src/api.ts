import createQueryClient from "openapi-react-query";
import {
  type CreateUnityCatalogClientOptions,
  createUnityCatalogFetchClient,
  fetchClient,
  type UnityCatalogFetchClient,
} from "./fetch.js";
import type { paths } from "./uc-types.js";

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
  fetchClient: UnityCatalogFetchClient;
}

/**
 * Construct a {@link UnityCatalogClient}. The default instance
 * ({@link defaultUnityCatalogClient}) is built from the same config the singleton
 * used historically, so default behavior is unchanged.
 */
export function createUnityCatalogClient(
  opts: CreateUnityCatalogClientOptions = {},
): UnityCatalogClient {
  return withQueryBindings(createUnityCatalogFetchClient(opts));
}

// `$api` wraps the fetch client with TanStack Query bindings. It auto-derives a
// query key of the form ["get", path, init] for every request. We treat that as
// the canonical key for a resource everywhere (reads, prefetch, invalidation),
// so keys never drift — see uc/queries.ts for the conventions.
function withQueryBindings(
  fetchClient: UnityCatalogFetchClient,
): UnityCatalogClient {
  return { $api: createQueryClient(fetchClient), fetchClient };
}

/**
 * The default, app-wide client instance — wraps the root package's default
 * `fetchClient`, so both share the transport set by
 * `setDefaultUnityCatalogFetch`. `UnityCatalogProvider` falls back to this when
 * no client is supplied, and the non-hook query helpers (`*DetailQuery`,
 * `prefetch*`) bind to it directly (query-key derivation is client-independent,
 * so caches still align).
 */
export const defaultUnityCatalogClient = withQueryBindings(fetchClient);

// Back-compat singleton export: the default instance's query bindings. Prefer
// reading the client from `useUnityCatalog()` in React code.
export const $api = defaultUnityCatalogClient.$api;
