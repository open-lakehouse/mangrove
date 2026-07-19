// The default `QueryService` implementation and its mutable default slot.
//
// `createQueryService` is transport-agnostic: it builds preview SQL from a table
// reference, drives the low-level `queryRunner` seam (runner.ts), and streams the
// resulting Arrow IPC chunks into an `ArrowResultStore` for `<DataGrid>`. Which
// runner actually executes is the host's / a later phase's decision — this file
// never imports a transport.
//
// The default slot (`setDefaultQueryService` / `defaultQueryService`) mirrors
// `setDefaultUnityCatalogFetch` in unity-catalog-client: a host can repoint the
// app-wide service once at startup, and the context (context.tsx) falls back to
// it when no provider is mounted.

import { ArrowResultStore } from "@open-lakehouse/data-grid";
import { queryRunner, queryRunnerSupports } from "./runner";
import type {
  PreviewHandle,
  PreviewRequest,
  QueryService,
  SupportsInput,
} from "./types";

/** Default preview row cap when a request omits `limit`. */
const DEFAULT_PREVIEW_LIMIT = 100;

// Backtick-quote one identifier part (doubling embedded backticks), matching the
// DataFusion/Spark dialect the query runners speak. Guards against a table name
// with reserved words or dots-in-quotes breaking out of the identifier.
function quoteIdent(part: string): string {
  return `\`${part.replace(/`/g, "``")}\``;
}

// Build `SELECT <cols|*> FROM <quoted.fqn> LIMIT <n>` from a table reference.
// Each dot-separated part of the fully-qualified name is quoted independently.
function buildPreviewSql(req: PreviewRequest, limit: number): string {
  const from = req.tableFullName.split(".").map(quoteIdent).join(".");
  const projection =
    req.columns && req.columns.length > 0
      ? req.columns.map(quoteIdent).join(", ")
      : "*";
  return `SELECT ${projection} FROM ${from} LIMIT ${limit}`;
}

// Internal handle: owns the store, a subscriber set, and the run lifecycle. A
// single monotonic `version` is the `useSyncExternalStore` snapshot; it bumps on
// every append and on running/error transitions.
class PreviewRun implements PreviewHandle {
  readonly store = new ArrowResultStore();
  private subscribers = new Set<() => void>();
  private controller = new AbortController();
  private _version = 0;
  private _running = true;
  private _error: Error | null = null;
  private _active = false;
  private readonly sql: string;
  private readonly externalSignal?: AbortSignal;

  constructor(req: PreviewRequest, limit: number) {
    // Build the SQL now (cheap, synchronous) but defer the run: `start()` is
    // invoked from the hook's mount effect so the stream can't emit before the
    // `useSyncExternalStore` subscription is attached. Otherwise fast (warm-
    // cache) runs finish before subscribe and their bumps are lost, leaving the
    // grid holding a full store it was never told to re-read.
    this.sql = buildPreviewSql(req, limit);
    this.externalSignal = req.signal;
  }

  // Begin (or re-begin) the run. Called from the hook's mount effect; must be
  // resilient to React StrictMode's mount → cleanup → mount cycle, which invokes
  // start() → cancel() → start() on this same handle. A cancel() aborts the
  // controller, so the second start() has to spin up a FRESH controller and run
  // rather than no-op — otherwise the run stays dead and the grid shows "No rows"
  // until an unrelated dep change (table switch) builds a new handle. That dead-
  // handle trap was the real first-view-empty bug.
  start(): void {
    if (this._active) return;
    // Honour an already-aborted external signal: nothing to run.
    if (this.externalSignal?.aborted) return;
    this._active = true;
    // Fresh controller so a prior cancel() (StrictMode throwaway, or a resumed
    // mount) doesn't leave us permanently aborted. Reset per-run state; the
    // store is cleared so a restart can't double-append rows.
    this.controller = new AbortController();
    this.store.reset();
    this._error = null;
    this._running = true;
    this.externalSignal?.addEventListener("abort", () => this.cancel(), {
      once: true,
    });
    void this.drive(this.sql);
  }

  get version(): number {
    return this._version;
  }
  get running(): boolean {
    return this._running;
  }
  get error(): Error | null {
    return this._error;
  }

  subscribe(cb: () => void): () => void {
    this.subscribers.add(cb);
    return () => this.subscribers.delete(cb);
  }

  cancel(): void {
    // Clearing `_active` lets a later start() (e.g. StrictMode's second mount)
    // spin up a fresh run instead of no-opping on the aborted controller.
    this._active = false;
    if (!this.controller.signal.aborted) this.controller.abort();
  }

  private bump(): void {
    this._version += 1;
    for (const cb of this.subscribers) cb();
  }

  private async drive(sql: string): Promise<void> {
    // Capture the controller for THIS run: cancel()/start() may swap
    // `this.controller` for a later run, but this generator must observe the
    // signal it was launched with.
    const { signal } = this.controller;
    try {
      for await (const chunk of queryRunner(
        { sql, limit: undefined },
        { signal },
      )) {
        if (signal.aborted) return;
        this.store.append(chunk.arrowIpc);
        this.bump();
      }
    } catch (err) {
      // An abort is an intentional teardown, not a surfaced error.
      if (!signal.aborted) {
        this._error = err instanceof Error ? err : new Error(String(err));
      }
    } finally {
      // Only the currently-live run reports completion. If this run was aborted
      // and superseded by a fresh start() (StrictMode remount), its controller
      // is no longer `this.controller`; skip so it can't flip `running` false or
      // bump on behalf of the live run.
      if (signal === this.controller.signal) {
        this._running = false;
        this.bump();
      }
    }
  }
}

/**
 * Build a {@link QueryService} over the registered {@link queryRunner}. Stateless
 * and cheap to create; each `preview` starts an independent run.
 */
export function createQueryService(): QueryService {
  return {
    preview(req: PreviewRequest): PreviewHandle {
      const limit = req.limit ?? DEFAULT_PREVIEW_LIMIT;
      return new PreviewRun(req, limit);
    },
    // Delegate to the registered runner's capability probe (permissive when it
    // declares none): the runner knows what it can read — e.g. the wasm engine
    // registers a Delta-only probe. The feature flag + `hasQueryRunner` keep
    // preview off in the standalone build regardless.
    supports(x: SupportsInput): boolean {
      return queryRunnerSupports(x);
    },
  };
}

let currentDefault: QueryService | null = null;

/**
 * Repoint the app-wide default {@link QueryService}. Call once at startup, before
 * first use. Mirrors `setDefaultUnityCatalogFetch`.
 */
export function setDefaultQueryService(service: QueryService): void {
  currentDefault = service;
}

/**
 * The app-wide default {@link QueryService}, created lazily on first use.
 * {@link QueryServiceProvider} falls back to this when no service is supplied.
 */
export function defaultQueryService(): QueryService {
  if (!currentDefault) currentDefault = createQueryService();
  return currentDefault;
}
