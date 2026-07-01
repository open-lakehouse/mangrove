// Public surface of the `data-grid` package — the ONLY entry point other code
// may import from. Internal files (cells, header, the Arrow-support modules
// under ./lib, story-fixtures) are not exported by the package `exports` map and
// must not be imported from outside.
//
// `data-grid` is the reusable, virtualized, Arrow-backed table primitive. It is
// a LEAF: it depends only on apache-arrow, @tanstack/*, and the shared
// @open-lakehouse/ui-kit — never on Unity Catalog or any app feature. Both the
// SQL editor result pane and the import preview build on it today; the Unity
// Catalog package may build on it in the future, never the reverse.
// See ../README.md.

export { DataGrid } from "./data-grid";
// The grid's input contract. Consumers (the editor's run controller, the import
// preview, Storybook fixtures) construct an `ArrowResultStore` and hand it to
// `<DataGrid store={…}>`. Kept Arrow-specific by design — generalizing the grid
// beyond Arrow is a separate, behavior-changing refactor.
export {
  ArrowResultStore,
  type ArrowStoreInfo,
} from "./lib/arrowResultStore";
