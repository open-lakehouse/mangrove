// Public surface of the Unity Catalog package — the ONLY entry point other code
// may import from. Everything else (tree internals, detail panes, dialog wiring,
// selection, ExpansionContext, the *EntityDialog and storage dialogs, the forms
// renderer, the uc/ data layer internals) is package-internal; the `exports` map
// reaches only this barrel.
//
// The package owns the catalog browser, the per-entity detail panes, the
// create/edit/delete + storage dialogs, the schema-driven form renderer, and the
// React-Query data layer that talks to the Unity Catalog REST API. It depends on
// shared primitives (@open-lakehouse/ui-kit), its own generated UC OpenAPI types
// (./uc-types, from openapi/unity-catalog.yaml), and exactly one host edge — the
// environment scope id, fed via EnvironmentScopeProvider (./env-seam). See
// ./README.md.
//
// Dependency direction: the package may build on @open-lakehouse/data-grid in
// the future; data-grid never builds on it. (Today StorageTable hand-rolls its
// own table and does not use data-grid — porting it is a separate refactor.)

// The default client + its transport seam. The host wires its own fetch into the
// default client via `setDefaultUnityCatalogFetch` (so the back-compat singletons
// and prefetch helpers route through it), and builds custom clients with
// `createUnityCatalogClient`. `fetchClient` is the default client's raw typed
// fetch, for non-hook query functions in the host.
export {
  type CreateUnityCatalogClientOptions,
  createUnityCatalogClient,
  defaultUnityCatalogClient,
  fetchClient,
  setDefaultUnityCatalogFetch,
  type UnityCatalogClient,
} from "./api";
// ── Route-level UI ───────────────────────────────────────────────────────────
export { CatalogExplorer } from "./CatalogExplorer";
// ── Shared presentational primitives reused by the host ──────────────────────
// EnvironmentDetail renders entity metadata; the editor's file browser reuses
// the tree row + list-state primitives.
export { Meta, MetaGrid } from "./detail/Meta";
// Dialog orchestration the environment manager mounts around catalog actions.
export { CatalogDialogsProvider } from "./dialogs";
// The host feeds the environment scope id (namespaces per-env tree expansion).
export { EnvironmentScopeProvider } from "./env-seam";
export { StorageLocationPicker } from "./storage/StorageLocationPicker";
// ── Storage admin surfaces (used by the environment manager) ─────────────────
export { StorageTable } from "./storage/StorageTable";
export { ListStates, TreeRow } from "./TreeRow";
export type { StorageKind } from "./types";
// ── Provider / client injection ─────────────────────────────────────────────
export {
  UnityCatalogProvider,
  useUnityCatalog,
} from "./uc/context";
// ── Error helper ─────────────────────────────────────────────────────────────
export { parseUcError } from "./uc/errors";

// ── Invalidators ─────────────────────────────────────────────────────────────
export { invalidateTables } from "./uc/mutations";
// ── Read hooks ───────────────────────────────────────────────────────────────
export {
  prefetchCatalogs,
  useCatalogs,
  useCredentials,
  useExternalLocations,
  useSchemas,
  useVolumes,
} from "./uc/queries";
