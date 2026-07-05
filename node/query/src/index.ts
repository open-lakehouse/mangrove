// Public surface of `@open-lakehouse/query` — the ONLY entry point other code
// may import from.
//
// This package is a LEAF like data-grid: it depends on apache-arrow (via
// @open-lakehouse/data-grid's ArrowResultStore) and React, but NEVER on Unity
// Catalog or any app feature. The Unity Catalog package builds on it (renders a
// preview), never the reverse — enforced by the Biome leaf rule in
// node/biome.json.
//
// It ships NO runtime query implementation. It provides the seam:
//   - a low-level `QueryRunner` a host / the wasm engine registers, and
//   - a table-oriented `QueryService` / `usePreview` / provider on top,
// feeding Arrow IPC into data-grid's `ArrowResultStore`. See ./runner.ts for why
// there is no default runner.

// Table-oriented service surface.
export {
  createQueryService,
  defaultQueryService,
  setDefaultQueryService,
} from "./api";
// React injection + hook.
export { QueryServiceProvider, useQueryService } from "./context";
// Runner composition (wasm-first with a host fallback).
export { createFallbackQueryRunner } from "./fallback";
// Generated contract message types (proto/query) — re-exported so runner
// implementors and tests share the upstream-owned shapes.
export type {
  RunQueryRequest,
  RunQueryResponse,
} from "./gen/open_lakehouse/query/v1/svc_pb";
// Low-level runner seam (the swap point for the wasm engine / a host).
export {
  getQueryRunner,
  hasQueryRunner,
  NoQueryRunnerError,
  type QueryChunk,
  type QueryRequest,
  type QueryRunner,
  type QueryRunnerCapabilities,
  queryRunner,
  queryRunnerSupports,
  registerQueryRunner,
} from "./runner";
export type {
  PreviewHandle,
  PreviewRequest,
  QueryService,
  SupportsInput,
} from "./types";
export { type PreviewState, usePreview } from "./usePreview";
