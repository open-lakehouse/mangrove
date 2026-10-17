// Public surface of the Unity Catalog CLIENT package — the data-fetching half of
// the UC UI, kept separate from the presentational `@open-lakehouse/unity-catalog-ui`
// package so the two concerns are not bundled together.
//
// This root entry is FRAMEWORK-FREE: the typed `openapi-fetch` client, its
// transport seam (`setDefaultUnityCatalogFetch`), the error/name helpers, and
// the generated UC OpenAPI types. It runs in any runtime with a `fetch` and
// needs no React. The React-Query provider and hooks live in the `./react`
// subpath (`@open-lakehouse/unity-catalog-client/react`), which re-exports
// everything here too.
//
// Pluggability: UC *coverage* is fixed — the hooks encode the UC API surface the
// components are written against — so the swappable seam is the transport
// UNDERNEATH the client, not the client API. A host redirects every request by
// registering a fetch via `setDefaultUnityCatalogFetch` (or by passing a custom
// `fetch` when constructing a client). A future in-process (wasm) UC client
// plugs in as a fetch-shaped adapter here, with no change to the hooks or the
// components. See ../README.md.

// Client factory + transport seam.
export {
  type CreateUnityCatalogClientOptions,
  createUnityCatalogFetchClient,
  fetchClient,
  setDefaultUnityCatalogFetch,
  type UnityCatalogFetchClient,
} from "./fetch.js";
// Name helpers.
export { objectFullName, tableFullName } from "./names.js";
// Error helper.
export { parseUcError } from "./uc/errors.js";
// Generated UC OpenAPI types (from the canonical ../../openapi/openapi.yaml via `gen:api`).
export * from "./uc-types.js";
