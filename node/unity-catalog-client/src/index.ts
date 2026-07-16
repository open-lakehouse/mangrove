// Public surface of the Unity Catalog CLIENT package — the data-fetching half of
// the UC UI, kept separate from the presentational `@open-lakehouse/unity-catalog`
// package so the two concerns are not bundled together.
//
// What lives here: the generated UC OpenAPI types, the injectable client factory
// + its transport seam (`setDefaultUnityCatalogFetch`), the React-Query context
// provider, and the read/mutate hooks bound to the UC entity/resource/securable
// model. The presentational package (and hosts) depend on this package for all
// UC data access.
//
// Pluggability: UC *coverage* is fixed — these hooks encode the UC API surface
// the components are written against — so the swappable seam is the transport
// UNDERNEATH the client, not the client API. A host redirects every request by
// registering a fetch via `setDefaultUnityCatalogFetch` (or by passing a client
// built with a custom `fetch` to `UnityCatalogProvider`). A future in-process
// (wasm) UC client plugs in as a fetch-shaped adapter here, with no change to the
// hooks or the components. See ../README.md.

// Client factory + transport seam.
export {
  $api,
  type CreateUnityCatalogClientOptions,
  createUnityCatalogClient,
  defaultUnityCatalogClient,
  fetchClient,
  setDefaultUnityCatalogFetch,
  type UnityCatalogClient,
} from "./api";
// Client injection context.
export { UnityCatalogProvider, useUnityCatalog } from "./uc/context";
// Error helper.
export { parseUcError } from "./uc/errors";
// Mutations + cache invalidators.
export * from "./uc/mutations";
// Read hooks + query helpers + list keys.
export * from "./uc/queries";
// Generated UC OpenAPI types (from the canonical ../../openapi/openapi.yaml via `gen:api`).
export * from "./uc-types";
