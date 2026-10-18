// React surface of the Unity Catalog client package
// (`@open-lakehouse/unity-catalog-client/react`): the TanStack Query bindings,
// the client injection context, and the read/mutate hooks bound to the UC
// entity/resource/securable model. Re-exports the framework-free root entry so
// React code needs a single import. Requires the `react` and
// `@tanstack/react-query` peers.

// Client with query bindings.
export {
  $api,
  createUnityCatalogClient,
  defaultUnityCatalogClient,
  type UnityCatalogClient,
} from "./api.js";
export * from "./index.js";
// Client injection context.
export { UnityCatalogProvider, useUnityCatalog } from "./uc/context.js";
// Mutations + cache invalidators.
export * from "./uc/mutations.js";
// Read hooks + query helpers + list keys.
export * from "./uc/queries.js";
