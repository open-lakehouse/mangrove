// Public surface of `@open-lakehouse/log-query` — the ONLY entry point other
// code may import from (except the `./testing` subexport for the dev stub).
//
// This package is a LEAF like @open-lakehouse/query: it depends on apache-arrow
// (via @open-lakehouse/data-grid's ArrowResultStore) and React, but NEVER on
// Unity Catalog or any app feature. The Unity Catalog package builds on it
// (renders a Delta-log tab), never the reverse.
//
// It ships NO runtime query implementation. It provides the seam:
//   - a low-level `LogQueryRunner` a host / the wasm engine registers, and
//   - a table-oriented `LogQueryService` / `useLogPreview` / provider on top,
// feeding Arrow IPC into data-grid's `ArrowResultStore`. The dev stub that makes
// the tab render without wasm lives on the `./testing` subexport, not here.
//
// It ALSO owns the Delta-log-specific views built on data-grid's generic Arrow
// primitives: the reconciled min/max-box visualization and the rich action-row
// stream. These are schema-aware (they know `stats.minValues` / the six action
// slots), which is exactly why they live here and not in the generic grid.

// Delta-log views (built on data-grid's generic Arrow primitives). Exports are
// kept alphabetized by the formatter's organize-imports assist, so the Delta
// view/service/seam entries interleave — the grouping is by comment, not order.
//
// The rich action-row stream — replaces the flat grid for the `actions` surface.
export { ActionsLog } from "./actions-log";
// Table-oriented service surface.
export {
  createLogQueryService,
  defaultLogQueryService,
  setDefaultLogQueryService,
} from "./api";
// React injection + hook.
export { LogQueryServiceProvider, useLogQueryService } from "./context";
// `isActionsSchema` gates the action-row view; the actions surface uses it.
export { isActionsSchema } from "./lib/actionSlots";
// The min/max-box view — per-file [min,max] intervals (1D) / bounding boxes (2D)
// from the reconciled-log stats; `hasMinMaxAxes` gates whether it's offered.
export { hasMinMaxAxes } from "./lib/minMaxAxes";
export { MinMaxView } from "./min-max-view";
// Low-level runner seam (the swap point for the wasm engine / a host).
export {
  getLogQueryRunner,
  hasLogQueryRunner,
  type LogKind,
  type LogQueryChunk,
  type LogQueryRequest,
  type LogQueryRunner,
  type LogQueryRunnerCapabilities,
  logQueryRunner,
  logQueryRunnerSupports,
  NoLogQueryRunnerError,
  registerLogQueryRunner,
} from "./runner";
export type {
  LogPreviewHandle,
  LogPreviewRequest,
  LogQueryService,
  LogSupportsInput,
} from "./types";
export { type LogPreviewState, useLogPreview } from "./useLogPreview";
