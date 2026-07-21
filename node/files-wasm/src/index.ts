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
// ## Transport split (mirrors hydrofoil's Tauri backend)
//
// The runner drives the `portal.files.v1.FilesService` contract through a
// `@open-lakehouse/files-connect` client over a wasm-backed connect transport:
//   - unary METADATA RPCs (list / stat / delete / mkdir) go through the client →
//     the worker's `connectUnary` → the in-wasm connect Router (binary proto), so
//     int64 fields survive as `bigint`;
//   - file BYTES bypass the dispatcher — `readFile` streams the worker's native
//     `read`, and `writeFile` posts one buffered native `write`.
//
// One worker PER OPERATION (not per session): the engine holds no cross-operation
// state, so teardown is just `worker.terminate()`.

import { create } from "@bufbuild/protobuf";
import type { Transport } from "@connectrpc/connect";
import {
  type DirectoryMetadata,
  type DirectoryPage,
  type FileChunk,
  type FileMetadata,
  type FilesRunner,
  type FilesSupportsInput,
  type ListDirectoryRequest,
  type ReadFileRequest,
  registerFilesRunner,
  type WriteFileRequest,
  type WriteFileResult,
} from "@open-lakehouse/files";
import {
  CreateDirectoryRequestSchema,
  createFilesClient,
  createWasmFilesTransport,
  DeleteDirectoryRequestSchema,
  DeleteFileRequestSchema,
  type FilesBackend,
  type FilesClient,
  GetFileMetadataRequestSchema,
  ListDirectoryContentsRequestSchema,
  registerFilesTransport,
} from "@open-lakehouse/files-connect";
import type {
  WorkerRequest,
  WorkerResponse,
  WriteResultPayload,
} from "./protocol";

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
 * terminates the worker. Shared by all verbs — `connectUnary`/`write` resolve on
 * their single reply, `read` yields chunks as they arrive.
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
 * Build a {@link FilesBackend} whose three calls each spawn one Web Worker
 * (`connectUnary` for unary metadata proto; `read` / `write` for native bytes),
 * bound to a per-operation `AbortSignal`. The connect transport is built over
 * this; metadata client calls and byte reads/writes all flow through it.
 */
function createWorkerBackend(
  options: WasmFilesOptions,
  signal: AbortSignal,
): FilesBackend {
  return {
    connectUnary(path, requestBytes): Promise<Uint8Array> {
      let responseBytes: Uint8Array | undefined;
      return runViaWorker(
        {
          type: "connectUnary",
          baseUrl: options.baseUrl,
          authToken: options.authToken,
          path,
          requestBytes,
        },
        signal,
        (response) => {
          if (response.type === "unary") responseBytes = response.responseBytes;
        },
      ).then(() => {
        if (!responseBytes) {
          throw new Error(
            "wasm files worker finished without a unary response",
          );
        }
        return responseBytes;
      });
    },
    readFileBytes(path, offset, length, onChunk): Promise<void> {
      return runViaWorker(
        {
          type: "read",
          baseUrl: options.baseUrl,
          authToken: options.authToken,
          path,
          offset,
          length,
        },
        signal,
        (response) => {
          if (response.type === "chunk") onChunk(response.bytes);
        },
      );
    },
    writeFileBytes(
      path,
      bytes,
      contentType,
      ifMatchEtag,
    ): Promise<WriteResultPayload> {
      let result: WriteResultPayload | undefined;
      return runViaWorker(
        {
          type: "write",
          baseUrl: options.baseUrl,
          authToken: options.authToken,
          path,
          bytes,
          contentType,
          ifMatchEtag,
        },
        signal,
        (response) => {
          if (response.type === "writeResult") result = response.result;
        },
      ).then(() => {
        if (!result) {
          throw new Error("wasm files worker finished without a write result");
        }
        return result;
      });
    },
  };
}

/** Build a per-operation transport + client bound to `signal`, so aborting the
 *  request tears down every worker the operation spawned. */
function clientFor(
  options: WasmFilesOptions,
  signal: AbortSignal,
): { client: FilesClient; backend: FilesBackend; transport: Transport } {
  const backend = createWorkerBackend(options, signal);
  const transport = createWasmFilesTransport(backend);
  return { client: createFilesClient(transport), backend, transport };
}

/**
 * Build a {@link FilesRunner} executing volume-files operations in the browser:
 * one Web Worker per operation; aborting the request terminates the worker (the
 * engine holds no cross-operation state, so teardown is just that). Metadata
 * verbs (list/stat/delete/mkdir) go through the connect client (binary proto);
 * byte verbs (read/write) drive the worker's native calls directly.
 */
export function createWasmFilesRunner(options: WasmFilesOptions): FilesRunner {
  return {
    async listDirectory(
      req: ListDirectoryRequest,
      { signal }: { signal: AbortSignal },
    ): Promise<DirectoryPage> {
      const { client } = clientFor(options, signal);
      const res = await client.listDirectoryContents(
        create(ListDirectoryContentsRequestSchema, {
          path: req.path,
          maxResults: req.maxResults,
          pageToken: req.pageToken,
        }),
        { signal },
      );
      return {
        entries: res.contents.map((e) => ({
          path: e.path,
          isDirectory: e.isDirectory,
          fileSize: Number(e.fileSize),
          lastModified: Number(e.lastModified),
        })),
        nextPageToken: res.nextPageToken,
      };
    },

    readFile(
      req: ReadFileRequest,
      { signal }: { signal: AbortSignal },
    ): AsyncIterable<FileChunk> {
      // Bytes bypass the connect dispatcher: drive the worker's native `read`
      // directly for true chunk streaming (the transport buffers; the runner
      // must not for large files).
      const backend = createWorkerBackend(options, signal);
      return {
        async *[Symbol.asyncIterator](): AsyncIterator<FileChunk> {
          const queue: FileChunk[] = [];
          let wake = () => {};
          const arrived = () => {
            wake();
            wake = () => {};
          };
          let done = false;
          let failure: unknown;
          const run = backend
            .readFileBytes(req.path, req.offset, req.length, (bytes) => {
              queue.push({ bytes });
              arrived();
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
                if (queue.length > 0 || done) resolve();
              });
            }
          } finally {
            await run;
          }
        },
      };
    },

    async stat(
      path: string,
      { signal }: { signal: AbortSignal },
    ): Promise<FileMetadata> {
      const { client } = clientFor(options, signal);
      const m = await client.getFileMetadata(
        create(GetFileMetadataRequestSchema, { path }),
        { signal },
      );
      return {
        path: m.path,
        fileSize: Number(m.fileSize),
        lastModified: Number(m.lastModified),
        contentType: m.contentType || undefined,
        etag: m.etag || undefined,
      };
    },

    async writeFile(
      req: WriteFileRequest,
      { signal }: { signal: AbortSignal },
    ): Promise<WriteFileResult> {
      // Bytes bypass the dispatcher: one buffered native `write`.
      const backend = createWorkerBackend(options, signal);
      const result = await backend.writeFileBytes(
        req.path,
        req.bytes,
        req.contentType,
        undefined,
      );
      return {
        path: result.path,
        fileSize: result.fileSize,
        etag: result.etag,
      };
    },

    async createDir(
      path: string,
      { signal }: { signal: AbortSignal },
    ): Promise<DirectoryMetadata> {
      const { client } = clientFor(options, signal);
      const m = await client.createDirectory(
        create(CreateDirectoryRequestSchema, { path }),
        { signal },
      );
      return { path: m.path, lastModified: Number(m.lastModified) };
    },

    async delete(
      path: string,
      { signal }: { signal: AbortSignal },
    ): Promise<void> {
      // A trailing slash marks a directory delete (recursive prefix sweep);
      // otherwise a single-file delete. Both are unary metadata RPCs.
      const { client } = clientFor(options, signal);
      if (path.endsWith("/")) {
        await client.deleteDirectory(
          create(DeleteDirectoryRequestSchema, {
            path: path.replace(/\/+$/, ""),
          }),
          { signal },
        );
      } else {
        await client.deleteFile(create(DeleteFileRequestSchema, { path }), {
          signal,
        });
      }
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
 * Azure/GCP-only capability probe, declaring write support. Call once at startup,
 * before the UI bootstraps. Mirrors @open-lakehouse/query-wasm's
 * `registerWasmPreview`.
 *
 * Also registers a `@open-lakehouse/files-connect` transport so a consumer that
 * uses the ConnectRPC client directly (the separately-updated editor) reaches the
 * same wasm backend. The transport is stateless — it builds a per-call worker
 * backend from the same options — so the two registrations share no state.
 */
export function registerWasmFiles(options: WasmFilesOptions): void {
  // Late-binding transport for the raw ConnectRPC client surface: each call
  // spawns a per-call worker backend with a fresh never-aborting signal (the
  // client's own AbortSignal, when passed, still terminates the worker via the
  // per-operation runner path; this transport path is the client-direct fallback).
  registerFilesTransport(
    createWasmFilesTransport(
      createWorkerBackend(options, new AbortController().signal),
    ),
  );
  registerFilesRunner(createWasmFilesRunner(options), {
    supports: supportsWasmFiles,
    canWrite: true,
  });
}
