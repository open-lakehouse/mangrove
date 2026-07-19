// The default `FilesService` implementation and its mutable default slot.
//
// `createFilesService` is transport-agnostic: it drives the low-level
// `filesRunner` seam (runner.ts) — listing directory pages, reading file bytes,
// stat — and never imports a transport. Which runner actually executes is the
// host's / a later phase's decision. Mirrors @open-lakehouse/query's api.ts and
// @open-lakehouse/log-query's api.ts, minus the `ArrowResultStore` (files aren't
// Arrow — the read surface is plain bytes / metadata, so the service is a thin
// stateless delegator rather than a streaming-into-a-store handle).
//
// The default slot (`setDefaultFilesService` / `defaultFilesService`) mirrors
// `setDefaultQueryService`: a host can repoint the app-wide service once at
// startup, and the context (context.tsx) falls back to it when no provider is
// mounted.

import {
  filesRunner,
  filesRunnerCanWrite,
  filesRunnerSupports,
} from "./runner";
import type {
  FileChunk,
  FileMetadata,
  FilesService,
  FilesSupportsInput,
  ListDirectoryRequest,
  ReadFileRequest,
} from "./types";

// A never-aborting signal for calls that omit one, so the runner always receives
// a real `AbortSignal` (its contract) without every caller having to build one.
function orNeverAbort(signal?: AbortSignal): AbortSignal {
  return signal ?? new AbortController().signal;
}

// Drain a `FileChunk` stream into a single contiguous buffer, concatenating in
// arrival (file) order — the `readFile` (vs `readFileStream`) convenience.
async function drain(stream: AsyncIterable<FileChunk>): Promise<Uint8Array> {
  const parts: Uint8Array[] = [];
  let total = 0;
  for await (const chunk of stream) {
    parts.push(chunk.bytes);
    total += chunk.bytes.length;
  }
  const out = new Uint8Array(total);
  let offset = 0;
  for (const part of parts) {
    out.set(part, offset);
    offset += part.length;
  }
  return out;
}

/**
 * Build a {@link FilesService} over the registered {@link filesRunner}. Stateless
 * and cheap to create; each call drives one runner operation.
 */
export function createFilesService(): FilesService {
  return {
    listDirectory(req: ListDirectoryRequest, signal?: AbortSignal) {
      return filesRunner.listDirectory(req, { signal: orNeverAbort(signal) });
    },
    readFile(req: ReadFileRequest, signal?: AbortSignal) {
      return drain(filesRunner.readFile(req, { signal: orNeverAbort(signal) }));
    },
    readFileStream(req: ReadFileRequest, signal?: AbortSignal) {
      return filesRunner.readFile(req, { signal: orNeverAbort(signal) });
    },
    stat(path: string, signal?: AbortSignal): Promise<FileMetadata> {
      return filesRunner.stat(path, { signal: orNeverAbort(signal) });
    },
    // Delegate to the registered runner's capability probe (permissive when it
    // declares none): the runner knows what it can read — e.g. the wasm engine
    // registers an Azure/GCP-only probe. `hasFilesRunner` keeps the tab off in
    // the standalone build regardless.
    supports(x: FilesSupportsInput): boolean {
      return filesRunnerSupports(x);
    },
    // Delegate to the registered runner's write capability (false when
    // undeclared): writes are deferred to Phase 2, so v1 runners declare none.
    canWrite(): boolean {
      return filesRunnerCanWrite();
    },
  };
}

let currentDefault: FilesService | null = null;

/**
 * Repoint the app-wide default {@link FilesService}. Call once at startup, before
 * first use. Mirrors `setDefaultQueryService`.
 */
export function setDefaultFilesService(service: FilesService): void {
  currentDefault = service;
}

/**
 * The app-wide default {@link FilesService}, created lazily on first use.
 * {@link FilesServiceProvider} falls back to this when no service is supplied.
 */
export function defaultFilesService(): FilesService {
  if (!currentDefault) currentDefault = createFilesService();
  return currentDefault;
}
