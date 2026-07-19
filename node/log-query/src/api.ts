// The default `LogQueryService` implementation and its mutable default slot.
//
// `createLogQueryService` is transport-agnostic: it addresses the log surface by
// `target` (the physical table) + `kind` (reconciled files vs. the action stream),
// drives the low-level `logQueryRunner` seam (runner.ts), and streams the resulting
// Arrow IPC chunks into an `ArrowResultStore` for `<DataGrid>`. Which runner
// actually executes is the host's / a later phase's decision — this file never
// imports a transport. Mirrors @open-lakehouse/query's api.ts.
//
// The default slot (`setDefaultLogQueryService` / `defaultLogQueryService`)
// mirrors `setDefaultQueryService`: a host can repoint the app-wide service once
// at startup, and the context (context.tsx) falls back to it when no provider is
// mounted.

import { ArrowResultStore } from "@open-lakehouse/data-grid";
import { type LogKind, logQueryRunner, logQueryRunnerSupports } from "./runner";
import type {
  LogPreviewHandle,
  LogPreviewRequest,
  LogQueryService,
  LogSupportsInput,
} from "./types";

/** Default row cap when a request omits `limit`. */
const DEFAULT_LOG_LIMIT = 100;

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
  private _active = false;
  private readonly target: string;
  private readonly kind: LogKind;
  private readonly limit: number;
  private readonly externalSignal?: AbortSignal;

  constructor(req: LogPreviewRequest, kind: LogKind, limit: number) {
    // Defer the run: `start()` is invoked from the hook's mount effect so the
    // stream can't emit before the `useSyncExternalStore` subscription is
    // attached. Otherwise fast (warm-cache) runs finish before subscribe and
    // their bumps are lost, leaving the grid holding a full store it was never
    // told to re-read.
    this.target = req.target;
    this.kind = kind;
    this.limit = limit;
    this.externalSignal = req.signal;
  }

  // Begin (or re-begin) the run. Called from the hook's mount effect; must be
  // resilient to React StrictMode's mount → cleanup → mount cycle, which invokes
  // start() → cancel() → start() on this same handle. A cancel() aborts the
  // controller, so the second start() has to spin up a FRESH controller and run
  // rather than no-op — otherwise the run stays dead and the grid shows "No rows"
  // until an unrelated dep change (kind toggle) builds a new handle. That dead-
  // handle trap was the real first-view-empty bug.
  start(): void {
    if (this._active) return;
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
    void this.drive(this.target, this.kind, this.limit);
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

  private async drive(
    target: string,
    kind: LogKind,
    limit: number,
  ): Promise<void> {
    // Capture the controller for THIS run: cancel()/start() may swap
    // `this.controller` for a later run, but this generator must observe the
    // signal it was launched with.
    const { signal } = this.controller;
    try {
      // The surface is addressed by `target` + `kind`; the runner synthesizes the
      // execution (the wasm engine builds a `delta_*_log('target')` UDTF query).
      for await (const chunk of logQueryRunner(
        { target, kind, limit },
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
 * Build a {@link LogQueryService} over the registered {@link logQueryRunner}.
 * Stateless and cheap to create; each `preview` starts an independent run.
 */
export function createLogQueryService(): LogQueryService {
  return {
    preview(req: LogPreviewRequest): LogPreviewHandle {
      const limit = req.limit ?? DEFAULT_LOG_LIMIT;
      const kind = req.kind ?? "reconciled";
      return new LogPreviewRun(req, kind, limit);
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
