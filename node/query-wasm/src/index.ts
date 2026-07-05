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
  type QueryChunk,
  type QueryRunner,
  registerQueryRunner,
  type SupportsInput,
} from "@open-lakehouse/query";
import type { RunMessage, WorkerResponse } from "./protocol";

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
 * Build a {@link QueryRunner} executing previews in the browser: one Web
 * Worker per run; aborting the request terminates the worker (the engine
 * holds no cross-run state, so teardown is just that).
 */
export function createWasmQueryRunner(options: WasmQueryOptions): QueryRunner {
  return (req, { signal }) => ({
    async *[Symbol.asyncIterator](): AsyncIterator<QueryChunk> {
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
        const run: RunMessage = {
          type: "run",
          baseUrl: options.baseUrl,
          authToken: options.authToken,
          sql: req.sql,
          limit: req.limit,
          catalog: req.catalog,
          schema: req.schema,
        };
        worker.postMessage(run);

        while (true) {
          if (signal.aborted) throw signal.reason ?? new Error("aborted");
          const message = queue.shift();
          if (!message) {
            await new Promise<void>((resolve) => {
              wake = resolve;
              // A message may have raced in between the shift and this await.
              if (queue.length > 0 || signal.aborted) resolve();
            });
            continue;
          }
          switch (message.type) {
            case "chunk":
              yield { arrowIpc: message.ipc, numRows: message.numRows };
              break;
            case "done":
              return;
            case "error": {
              const error = new Error(message.message) as WasmQueryError;
              error.name = "WasmQueryError";
              error.code = message.code;
              throw error;
            }
          }
        }
      } finally {
        signal.removeEventListener("abort", onAbort);
        worker.terminate();
      }
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
