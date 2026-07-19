// Public surface of `@open-lakehouse/files-wasm`: the in-browser wasm volume-
// files engine packaged as a `FilesRunner` for the `@open-lakehouse/files` seam.
//
// `registerWasmFiles` is the one-call wiring: it registers a runner that spawns a
// Web Worker per operation (crates/query-wasm's `UcFilesEngine` inside), with an
// Azure/GCP-only capability probe so a Files tab is never offered for storage the
// engine cannot read direct-to-cloud (AWS/R2 need SigV4, unsupported in-browser).
// Default builds alias THIS PACKAGE to ./stub.ts (see node/app/vite.config.ts),
// so the gitignored wasm artifact is only resolved when the wasm build is enabled.
//
// One worker PER OPERATION (not per session): `listDirectory`/`stat` post one
// message and await one reply; `readFile` streams chunks. The engine holds no
// cross-operation state, so teardown is just `worker.terminate()`.

import {
  type DirectoryPage,
  type FileChunk,
  type FileMetadata,
  type FilesRunner,
  type FilesSupportsInput,
  type ListDirectoryRequest,
  type ReadFileRequest,
  registerFilesRunner,
} from "@open-lakehouse/files";
import type { ReadMessage, WorkerRequest, WorkerResponse } from "./protocol";

export interface WasmFilesOptions {
  /** Unity Catalog REST base, e.g. `${origin}/api/2.1/unity-catalog`. */
  baseUrl: string;
  /** Optional bearer for the UC API (same-origin cookies flow regardless). */
  authToken?: string;
}

/** An `Error` carrying the engine's machine-readable failure class. */
export interface WasmFilesError extends Error {
  /** "UNSUPPORTED" (outside the engine's envelope — e.g. AWS/R2 storage) |
   *  "NETWORK" (CORS/connectivity on the direct storage/list fetch) | "FAILED". */
  code: string;
}

/** True when a failed wasm operation should transparently fall back to another
 *  runner (a downstream ConnectRPC service, a host runner). Mirrors
 *  @open-lakehouse/query-wasm's `isFallbackWorthy`. */
export function isFallbackWorthy(error: unknown): boolean {
  const code = (error as { code?: unknown } | null)?.code;
  return code === "UNSUPPORTED" || code === "NETWORK";
}

/**
 * Spawn one Web Worker for `message`, then drive it to completion. Every
 * `WorkerResponse` is pushed to `onResponse` until the worker signals `done`
 * (resolve) or `error` (reject with a {@link WasmFilesError}). Aborting `signal`
 * terminates the worker. Shared by all three verbs — list/stat resolve on their
 * single reply, read yields chunks as they arrive.
 */
function runViaWorker(
  message: WorkerRequest,
  signal: AbortSignal,
  onResponse: (response: WorkerResponse) => void,
): Promise<void> {
  signal.throwIfAborted();
  return new Promise<void>((resolve, reject) => {
    const worker = new Worker(new URL("./worker.ts", import.meta.url), {
      type: "module",
    });
    const cleanup = () => {
      signal.removeEventListener("abort", onAbort);
      worker.terminate();
    };
    const onAbort = () => {
      cleanup();
      reject(signal.reason ?? new Error("aborted"));
    };
    signal.addEventListener("abort", onAbort, { once: true });

    worker.onmessage = (event: MessageEvent<WorkerResponse>) => {
      const response = event.data;
      switch (response.type) {
        case "done":
          cleanup();
          resolve();
          break;
        case "error": {
          cleanup();
          const error = new Error(response.message) as WasmFilesError;
          error.name = "WasmFilesError";
          error.code = response.code;
          reject(error);
          break;
        }
        default:
          onResponse(response);
      }
    };
    worker.onerror = (event) => {
      cleanup();
      const error = new Error(
        event.message || "wasm files worker crashed",
      ) as WasmFilesError;
      error.name = "WasmFilesError";
      error.code = "FAILED";
      reject(error);
    };

    worker.postMessage(message);
  });
}

/**
 * Build a {@link FilesRunner} executing volume-files operations in the browser:
 * one Web Worker per operation; aborting the request terminates the worker (the
 * engine holds no cross-operation state, so teardown is just that).
 */
export function createWasmFilesRunner(options: WasmFilesOptions): FilesRunner {
  return {
    listDirectory(
      req: ListDirectoryRequest,
      { signal }: { signal: AbortSignal },
    ): Promise<DirectoryPage> {
      let page: DirectoryPage | undefined;
      return runViaWorker(
        {
          type: "list",
          baseUrl: options.baseUrl,
          authToken: options.authToken,
          path: req.path,
          maxResults: req.maxResults,
          pageToken: req.pageToken,
        },
        signal,
        (response) => {
          if (response.type === "page") page = response.page;
        },
      ).then(() => {
        if (!page) {
          throw new Error(
            "wasm files worker finished without a directory page",
          );
        }
        return page;
      });
    },

    stat(
      path: string,
      { signal }: { signal: AbortSignal },
    ): Promise<FileMetadata> {
      let meta: FileMetadata | undefined;
      return runViaWorker(
        {
          type: "stat",
          baseUrl: options.baseUrl,
          authToken: options.authToken,
          path,
        },
        signal,
        (response) => {
          if (response.type === "meta") meta = response.meta;
        },
      ).then(() => {
        if (!meta) {
          throw new Error("wasm files worker finished without file metadata");
        }
        return meta;
      });
    },

    readFile(
      req: ReadFileRequest,
      { signal }: { signal: AbortSignal },
    ): AsyncIterable<FileChunk> {
      const message: ReadMessage = {
        type: "read",
        baseUrl: options.baseUrl,
        authToken: options.authToken,
        path: req.path,
        offset: req.offset,
        length: req.length,
      };
      return {
        async *[Symbol.asyncIterator](): AsyncIterator<FileChunk> {
          // Pump worker chunk messages into a pull queue the iterator drains;
          // `runViaWorker`'s promise settles the stream (resolve → end, reject →
          // throw).
          const queue: FileChunk[] = [];
          let wake = () => {};
          const arrived = () => {
            wake();
            wake = () => {};
          };
          let done = false;
          let failure: unknown;
          const run = runViaWorker(message, signal, (response) => {
            if (response.type === "chunk") {
              queue.push({ bytes: response.bytes });
              arrived();
            }
          })
            .then(() => {
              done = true;
              arrived();
            })
            .catch((error) => {
              failure = error;
              done = true;
              arrived();
            });

          try {
            while (true) {
              const chunk = queue.shift();
              if (chunk) {
                yield chunk;
                continue;
              }
              if (failure) throw failure;
              if (done) return;
              await new Promise<void>((resolve) => {
                wake = resolve;
                // A message may have raced in between the shift and this await.
                if (queue.length > 0 || done) resolve();
              });
            }
          } finally {
            // Ensure the worker is torn down even if the consumer breaks early.
            await run;
          }
        },
      };
    },
  };
}

/** Azure + GCP volumes only — the engine's capability probe for `supports()`.
 *  AWS/R2 need SigV4 request signing the in-browser engine does not do, so they
 *  are gated off here (surfaced as UNSUPPORTED at runtime otherwise). A volume
 *  whose `storageScheme` is unknown is optimistically allowed; the finer gates
 *  are runtime UNSUPPORTED/NETWORK errors, surfaced for fallback composition. */
export function supportsWasmFiles(x: FilesSupportsInput): boolean {
  const scheme = (x.storageScheme ?? "").toLowerCase();
  if (scheme === "") return true;
  return (
    scheme === "abfss" ||
    scheme === "abfs" ||
    scheme === "az" ||
    scheme === "wasbs" ||
    scheme === "wasb" ||
    scheme === "azurite" ||
    scheme === "gs"
  );
}

/**
 * One-call wiring: register the wasm engine as the app's files runner with the
 * Azure/GCP-only capability probe. Call once at startup, before the UI
 * bootstraps. Mirrors @open-lakehouse/query-wasm's `registerWasmPreview`. v1 is
 * read-only, so `canWrite` is left undeclared (false).
 */
export function registerWasmFiles(options: WasmFilesOptions): void {
  registerFilesRunner(createWasmFilesRunner(options), {
    supports: supportsWasmFiles,
  });
}
