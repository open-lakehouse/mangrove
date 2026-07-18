// The default `LogQueryService` implementation and its mutable default slot.
//
// `createLogQueryService` is transport-agnostic: it builds SQL over the fixed
// logical reconciled-log table, drives the low-level `logQueryRunner` seam
// (runner.ts), and streams the resulting Arrow IPC chunks into an
// `ArrowResultStore` for `<DataGrid>`. Which runner actually executes is the
// host's / a later phase's decision — this file never imports a transport.
// Mirrors @open-lakehouse/query's api.ts.
//
// The default slot (`setDefaultLogQueryService` / `defaultLogQueryService`)
// mirrors `setDefaultQueryService`: a host can repoint the app-wide service once
// at startup, and the context (context.tsx) falls back to it when no provider is
// mounted.

import { ArrowResultStore } from "@open-lakehouse/data-grid";
import { logQueryRunner, logQueryRunnerSupports } from "./runner";
import type {
  LogPreviewHandle,
  LogPreviewRequest,
  LogQueryService,
  LogSupportsInput,
} from "./types";

/** Default row cap when a request omits `limit`. */
const DEFAULT_LOG_LIMIT = 100;

// The fixed logical table name the runner binds the reconciled-log provider to.
// The target table is NOT interpolated into the FROM clause — it rides on the
// request (see LogQueryRequest.target) so the runner registers the provider for
// that table under this name.
const RECONCILED_LOG_TABLE = "reconciled_log";

// Build `SELECT * FROM reconciled_log LIMIT <n>`.
function buildLogSql(limit: number): string {
  return `SELECT * FROM ${RECONCILED_LOG_TABLE} LIMIT ${limit}`;
}

// Internal handle: owns the store, a subscriber set, and the run lifecycle. A
// single monotonic `version` is the `useSyncExternalStore` snapshot; it bumps on
// every append and on running/error transitions.
class LogPreviewRun implements LogPreviewHandle {
  readonly store = new ArrowResultStore();
  private subscribers = new Set<() => void>();
  private controller = new AbortController();
  private _version = 0;
  private _running = true;
  private _error: Error | null = null;

  constructor(req: LogPreviewRequest, limit: number) {
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
    void this.drive(buildLogSql(limit), req.target);
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

  private async drive(sql: string, target: string): Promise<void> {
    try {
      for await (const chunk of logQueryRunner(
        { sql, limit: undefined, target },
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
 * Build a {@link LogQueryService} over the registered {@link logQueryRunner}.
 * Stateless and cheap to create; each `preview` starts an independent run.
 */
export function createLogQueryService(): LogQueryService {
  return {
    preview(req: LogPreviewRequest): LogPreviewHandle {
      const limit = req.limit ?? DEFAULT_LOG_LIMIT;
      return new LogPreviewRun(req, limit);
    },
    // Delegate to the registered runner's capability probe (permissive when it
    // declares none): the runner knows what it can read — e.g. a wasm engine
    // registers a Delta-only probe. `hasLogQueryRunner` keeps the tab off in the
    // standalone build regardless.
    supports(x: LogSupportsInput): boolean {
      return logQueryRunnerSupports(x);
    },
  };
}

let currentDefault: LogQueryService | null = null;

/**
 * Repoint the app-wide default {@link LogQueryService}. Call once at startup,
 * before first use. Mirrors `setDefaultQueryService`.
 */
export function setDefaultLogQueryService(service: LogQueryService): void {
  currentDefault = service;
}

/**
 * The app-wide default {@link LogQueryService}, created lazily on first use.
 * {@link LogQueryServiceProvider} falls back to this when no service is supplied.
 */
export function defaultLogQueryService(): LogQueryService {
  if (!currentDefault) currentDefault = createLogQueryService();
  return currentDefault;
}
