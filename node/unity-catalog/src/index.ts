// Public surface of the Unity Catalog PRESENTATIONAL package — the ONLY entry
// point other code may import from. Everything else (tree internals, detail
// panes, dialog wiring, selection, ExpansionContext, the *EntityDialog and
// storage dialogs, the forms renderer) is package-internal; the `exports` map
// reaches only this barrel.
//
// This package owns the catalog browser, the per-entity detail panes, the
// create/edit/delete + storage dialogs, and the schema-driven form renderer. It
// depends on shared primitives (@open-lakehouse/ui-kit) and the UC data layer
// (@open-lakehouse/unity-catalog-client) for all fetching/hooks/types, and has
// exactly one host edge — the environment scope id, fed via
// EnvironmentScopeProvider (./env-seam). See ../README.md.
//
// Client concerns (UnityCatalogProvider, the hooks, setDefaultUnityCatalogFetch,
// the UC OpenAPI types, parseUcError) are NOT re-exported here — import them
// directly from @open-lakehouse/unity-catalog-client so the two concerns stay
// decoupled.
//
// Dependency direction: the package may build on @open-lakehouse/data-grid in
// the future; data-grid never builds on it. (Today StorageTable hand-rolls its
// own table and does not use data-grid — porting it is a separate refactor.)

// ── Route-level UI ───────────────────────────────────────────────────────────
export { CatalogExplorer } from "./CatalogExplorer";
// ── Shared presentational primitives reused by the host ──────────────────────
// EnvironmentDetail renders entity metadata; the editor's file browser reuses
// the tree row + list-state primitives.
export { Meta, MetaGrid } from "./detail/Meta";
// Dialog orchestration the environment manager mounts around catalog actions.
export { CatalogDialogsProvider } from "./dialogs";
export { ExternalDataPage } from "./ExternalDataPage";
// The host feeds the environment scope id (namespaces per-env tree expansion).
export { EnvironmentScopeProvider } from "./env-seam";
export { StorageLocationPicker } from "./storage/StorageLocationPicker";
// ── Storage admin surfaces (used by the environment manager) ─────────────────
export { StorageTable } from "./storage/StorageTable";
export { ListStates, TreeRow } from "./TreeRow";
export type { StorageKind } from "./types";
