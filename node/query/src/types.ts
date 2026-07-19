// The table-oriented query surface the UC UI consumes. It sits on top of the
// low-level `QueryRunner` seam (runner.ts): a `QueryService` turns a table
// reference into a `PreviewHandle` that streams Arrow IPC into an
// `ArrowResultStore` for `<DataGrid>`. Nothing here knows the transport — the
// registered runner does.

import type { ArrowResultStore } from "@open-lakehouse/data-grid";

/** A request to preview a table's rows. */
export interface PreviewRequest {
  /** Fully-qualified `catalog.schema.table`. */
  tableFullName: string;
  /** Row cap (default applied by the service). */
  limit?: number;
  /** Optional column projection; omitted / empty means `SELECT *`. */
  columns?: string[];
  /** Aborts the underlying run when triggered. */
  signal?: AbortSignal;
}

/**
 * A live handle to a running (or finished) preview. The store accumulates Arrow
 * IPC as chunks arrive; `version` bumps on every append/state change so a
 * `useSyncExternalStore` consumer re-renders. `subscribe` returns an
 * unsubscribe function.
 */
export interface PreviewHandle {
  /** The grid's input contract — chunks are appended here as they stream in. */
  readonly store: ArrowResultStore;
  /** Begin the run. Idempotent and deferred: the hook calls this from a mount
   *  effect — i.e. *after* `subscribe` is attached — so no chunk can be appended
   *  (and bumped) before a subscriber exists to observe it. A no-op if already
   *  started or cancelled. */
  start(): void;
  /** Register a change callback; returns an unsubscribe. Fires per append and
   *  on running/error transitions. */
  subscribe(cb: () => void): () => void;
  /** Monotonic counter bumped on every change — the `getSnapshot` value. */
  get version(): number;
  /** True while the stream is open. */
  get running(): boolean;
  /** The terminal error, if the run failed; null otherwise. */
  get error(): Error | null;
  /** Abort the run (idempotent). */
  cancel(): void;
}

/** Capability probe input: what the UI knows about a table before previewing. */
export interface SupportsInput {
  format?: string;
  tableType?: string;
}

/**
 * The swappable preview service. `preview` starts a run and returns a handle;
 * `supports` gates whether a preview should be offered at all (e.g. Delta only,
 * later excluding deletion-vector / zstd tables once the wasm engine lands).
 */
export interface QueryService {
  preview(req: PreviewRequest): PreviewHandle;
  supports(x: SupportsInput): boolean;
}
