// Public surface of `@open-lakehouse/query-wasm`: the in-browser wasm query
// engine packaged as a `QueryRunner` for the `@open-lakehouse/query` seam
// (WASM_QUERY_PREVIEW.md Phase B).
//
// `registerWasmPreview` is the one-call app wiring: it registers a runner that
// spawns a Web Worker per preview run (crates/query-wasm inside), with a
// Delta-only capability probe so `TablePreview` never offers previews the
// engine cannot serve. Default builds alias THIS PACKAGE to ./stub.ts (see
// node/app/vite.config.ts), so the gitignored wasm artifact is only resolved
// when VITE_ENABLE_WASM_QUERY=true.

import {
  type LogQueryChunk,
  type LogQueryRunner,
  type LogSupportsInput,
  registerLogQueryRunner,
} from "@open-lakehouse/log-query";
import {
  type QueryChunk,
  type QueryRunner,
  registerQueryRunner,
  type SupportsInput,
} from "@open-lakehouse/query";
import type { LogRunMessage, RunMessage, WorkerResponse } from "./protocol";

export interface WasmQueryOptions {
  /** Unity Catalog REST base, e.g. `${origin}/api/2.1/unity-catalog`. */
  baseUrl: string;
  /** Optional bearer for the UC API (same-origin cookies flow regardless). */
  authToken?: string;
}

/** An `Error` carrying the engine's machine-readable failure class. */
export interface WasmQueryError extends Error {
  /** "UNSUPPORTED" (outside the engine's envelope) | "NETWORK" (CORS/
   *  connectivity on direct storage fetch) | "FAILED". */
  code: string;
}

/** True when a failed wasm run should transparently fall back to another
 *  runner (pair with `createFallbackQueryRunner`'s `shouldFallBack`). */
export function isFallbackWorthy(error: unknown): boolean {
  const code = (error as { code?: unknown } | null)?.code;
  return code === "UNSUPPORTED" || code === "NETWORK";
}

/**
 * Spawn one Web Worker, post `message`, and yield each Arrow IPC chunk as
 * `{ arrowIpc, numRows }` until the worker signals `done` (or throws on
 * `error`). Aborting `signal` terminates the worker (the engine holds no
 * cross-run state, so teardown is just that). Shared by the table-query and
 * log-query runners — both post a single message and drain the same chunk
 * stream.
 */
async function* runViaWorker(
  message: RunMessage | LogRunMessage,
  signal: AbortSignal,
): AsyncGenerator<{ arrowIpc: Uint8Array; numRows: number }> {
  signal.throwIfAborted();
  const worker = new Worker(new URL("./worker.ts", import.meta.url), {
    type: "module",
  });

  // Pump worker messages into a pull queue the async iterator drains.
  const queue: WorkerResponse[] = [];
  let wake = () => {};
  const arrived = () => {
    wake();
    wake = () => {};
  };
  worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
    queue.push(event.data);
    arrived();
  };
  worker.onerror = (event) => {
    queue.push({
      type: "error",
      message: event.message || "wasm query worker crashed",
      code: "FAILED",
    });
    arrived();
  };
  const onAbort = () => arrived();
  signal.addEventListener("abort", onAbort, { once: true });

  try {
    worker.postMessage(message);

    while (true) {
      if (signal.aborted) throw signal.reason ?? new Error("aborted");
      const response = queue.shift();
      if (!response) {
        await new Promise<void>((resolve) => {
          wake = resolve;
          // A message may have raced in between the shift and this await.
          if (queue.length > 0 || signal.aborted) resolve();
        });
        continue;
      }
      switch (response.type) {
        case "chunk":
          yield { arrowIpc: response.ipc, numRows: response.numRows };
          break;
        case "done":
          return;
        case "error": {
          const error = new Error(response.message) as WasmQueryError;
          error.name = "WasmQueryError";
          error.code = response.code;
          throw error;
        }
      }
    }
  } finally {
    signal.removeEventListener("abort", onAbort);
    worker.terminate();
  }
}

/**
 * Build a {@link QueryRunner} executing previews in the browser: one Web
 * Worker per run; aborting the request terminates the worker (the engine
 * holds no cross-run state, so teardown is just that).
 */
export function createWasmQueryRunner(options: WasmQueryOptions): QueryRunner {
  return (req, { signal }) => ({
    async *[Symbol.asyncIterator](): AsyncIterator<QueryChunk> {
      const run: RunMessage = {
        type: "run",
        baseUrl: options.baseUrl,
        authToken: options.authToken,
        sql: req.sql,
        limit: req.limit,
        catalog: req.catalog,
        schema: req.schema,
      };
      yield* runViaWorker(run, signal);
    },
  });
}

/**
 * Build a {@link LogQueryRunner} executing reconciled-Delta-log previews in the
 * browser, mirroring {@link createWasmQueryRunner}. The physical table rides on
 * `req.target` (the SQL references a fixed logical name), and `req.kind` selects
 * the log surface (reconciled files vs. the reconciled action stream).
 */
export function createWasmLogQueryRunner(
  options: WasmQueryOptions,
): LogQueryRunner {
  return (req, { signal }) => ({
    async *[Symbol.asyncIterator](): AsyncIterator<LogQueryChunk> {
      const run: LogRunMessage = {
        type: "run-log",
        baseUrl: options.baseUrl,
        authToken: options.authToken,
        sql: req.sql,
        limit: req.limit,
        target: req.target ?? "",
        kind: req.kind ?? "reconciled",
      };
      yield* runViaWorker(run, signal);
    },
  });
}

/** Delta tables only — the engine's capability probe for `supports()`. The
 *  finer gates (deletion vectors, zstd, unbackfilled commits) are runtime
 *  UNSUPPORTED errors, surfaced for fallback composition instead. */
export function supportsWasmPreview(x: SupportsInput): boolean {
  return (x.format ?? "").toUpperCase() === "DELTA";
}

/**
 * One-call wiring: register the wasm engine as the app's query runner with the
 * Delta-only capability probe. Call once at startup, before the UI bootstraps.
 */
export function registerWasmPreview(options: WasmQueryOptions): void {
  registerQueryRunner(createWasmQueryRunner(options), {
    supports: supportsWasmPreview,
  });
}

/** Delta tables only — the engine's capability probe for the log-query
 *  `supports()`, mirroring {@link supportsWasmPreview}. */
export function supportsWasmLogPreview(x: LogSupportsInput): boolean {
  return (x.format ?? "").toUpperCase() === "DELTA";
}

/**
 * One-call wiring: register the wasm engine as the app's reconciled-Delta-log
 * runner with the Delta-only capability probe. Call once at startup, before the
 * UI bootstraps. Mirrors {@link registerWasmPreview} for the log-query seam.
 */
export function registerWasmLogPreview(options: WasmQueryOptions): void {
  registerLogQueryRunner(createWasmLogQueryRunner(options), {
    supports: supportsWasmLogPreview,
  });
}
