// The table-oriented log-query surface the UC UI consumes. It sits on top of the
// low-level `LogQueryRunner` seam (runner.ts): a `LogQueryService` turns a target
// table into a `LogPreviewHandle` that streams Arrow IPC into an
// `ArrowResultStore` for `<DataGrid>`. Nothing here knows the transport — the
// registered runner does. Mirrors @open-lakehouse/query's types.ts.

import type { ArrowResultStore } from "@open-lakehouse/data-grid";

/** A request to explore a table's reconciled Delta log. */
export interface LogPreviewRequest {
  /** The target table whose reconciled log to scan — fully-qualified
   *  `catalog.schema.table`, or a storage path. Carried opaquely to the runner,
   *  which resolves it and binds the log provider. */
  target: string;
  /** Row cap (default applied by the service). */
  limit?: number;
  /** Aborts the underlying run when triggered. */
  signal?: AbortSignal;
}

/**
 * A live handle to a running (or finished) log-preview. The store accumulates
 * Arrow IPC as chunks arrive; `version` bumps on every append/state change so a
 * `useSyncExternalStore` consumer re-renders. `subscribe` returns an
 * unsubscribe function.
 */
export interface LogPreviewHandle {
  /** The grid's input contract — chunks are appended here as they stream in. */
  readonly store: ArrowResultStore;
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

/** Capability probe input: what the UI knows about a table before exploring. */
export interface LogSupportsInput {
  format?: string;
  tableType?: string;
}

/**
 * The swappable log-query service. `preview` starts a run and returns a handle;
 * `supports` gates whether the Delta-log tab should be offered at all (e.g. Delta
 * only).
 */
export interface LogQueryService {
  preview(req: LogPreviewRequest): LogPreviewHandle;
  supports(x: LogSupportsInput): boolean;
}
