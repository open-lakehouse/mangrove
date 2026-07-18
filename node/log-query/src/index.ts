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

// Table-oriented service surface.
export {
  createLogQueryService,
  defaultLogQueryService,
  setDefaultLogQueryService,
} from "./api";
// React injection + hook.
export { LogQueryServiceProvider, useLogQueryService } from "./context";
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
