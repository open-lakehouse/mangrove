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
  private _started = false;
  private readonly sql: string;

  constructor(req: PreviewRequest, limit: number) {
    // Chain an external abort signal into our controller. Handle the
    // already-aborted case explicitly: `addEventListener` never fires for an
    // event that has already passed.
    if (req.signal?.aborted) {
      this.cancel();
    } else {
      req.signal?.addEventListener("abort", () => this.cancel(), {
        once: true,
      });
    }
    // Build the SQL now (cheap, synchronous) but defer the run: `start()` is
    // invoked from the hook's mount effect so the stream can't emit before the
    // `useSyncExternalStore` subscription is attached. Otherwise fast (warm-
    // cache) runs finish before subscribe and their bumps are lost, leaving the
    // grid holding a full store it was never told to re-read.
    this.sql = buildPreviewSql(req, limit);
  }

  start(): void {
    if (this._started || this.controller.signal.aborted) return;
    this._started = true;
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
    if (!this.controller.signal.aborted) this.controller.abort();
  }

  private bump(): void {
    this._version += 1;
    for (const cb of this.subscribers) cb();
  }

  private async drive(sql: string): Promise<void> {
    try {
      for await (const chunk of queryRunner(
        { sql, limit: undefined },
        { signal: this.controller.signal },
      )) {
        this.store.append(chunk.arrowIpc);
        this.bump();
      }
    } catch (err) {
      // An abort is an intentional teardown, not a surfaced error.
      if (!this.controller.signal.aborted) {
        this._error = err instanceof Error ? err : new Error(String(err));
      }
    } finally {
      this._running = false;
      this.bump();
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
