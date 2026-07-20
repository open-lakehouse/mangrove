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
//
// Everything here is SCHEMA-AGNOSTIC. Delta-log-specific views (min/max boxes,
// action rows) that know about `stats.minValues` / the six action slots live in
// @open-lakehouse/log-query, built ON these generic primitives — not here.
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
// Generic Arrow cell formatting — a React node + alignment for the grid, and a
// compact plain-string form for dense inline contexts (used by log-query views).
export { formatCell, formatScalarText } from "./lib/cellFormatters";
// Zero-copy nested-struct navigation + ordered-value coercion. Generic Arrow
// helpers the Arrow-backed visualizations (in log-query) build on.
export {
  resolveChildPath,
  structChildren,
  structFieldByName,
} from "./lib/nestedAccess";
export {
  isOrderableType,
  timestampToEpochMs,
  toAxisNumber,
} from "./lib/temporal";
