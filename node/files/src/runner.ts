// Pluggable files-execution seam — the single swap point that lets a host or a
// later phase browse volume files somewhere (an in-browser wasm engine listing
// direct-to-cloud over CORS, a downstream ConnectRPC service, a Tauri `invoke`)
// WITHOUT the UI depending on that mechanism. Mirrors @open-lakehouse/query's
// runner.ts, but for the volume-files surface.
//
// Mangrove ships NO runtime runner here either. Until one is registered,
// `filesRunner` throws `NoFilesRunnerError`, and the UI keeps its Files tab gated
// off (`hasFilesRunner()` reads false).
//
// DELIBERATE DIVERGENCE from `QueryRunner`/`LogQueryRunner`: the files surface has
// MANY verbs (list, read, stat, and — Phase 2 — write/mkdir/delete), so
// `FilesRunner` is an OBJECT OF METHODS rather than a single callable. Everything
// else — the module-level registry, `hasFilesRunner()`, the late-binding stable
// `filesRunner` ref, and the `FilesRunnerCapabilities{supports?, canWrite?}`
// machinery — mirrors query/runner.ts exactly.
//
// There is no generated proto contract for this surface yet, so the request /
// chunk shapes are defined locally (types.ts), structurally aligned with the
// portal `files/v1/svc.proto` messages so a generated contract can drop in later.

import type {
  DirectoryMetadata,
  DirectoryPage,
  FileChunk,
  FileMetadata,
  FilesSupportsInput,
  ListDirectoryRequest,
  ReadFileRequest,
  WriteFileRequest,
  WriteFileResult,
} from "./types";

/**
 * Executes volume-files operations. Unlike the single-callable `QueryRunner`,
 * this is an object of methods (files has many verbs). The read verbs
 * (`listDirectory`, `readFile`, `stat`) are the v1 surface; the write verbs
 * (`writeFile`, `createDir`, `delete`) are optional and absent until Phase 2.
 * Aborting `opts.signal` must tear the operation down.
 */
export interface FilesRunner {
  /** List one bounded page of a directory's immediate children. */
  listDirectory(
    req: ListDirectoryRequest,
    opts: { signal: AbortSignal },
  ): Promise<DirectoryPage>;
  /** Read a file (or byte range) as a stream of chunks, in file order. */
  readFile(
    req: ReadFileRequest,
    opts: { signal: AbortSignal },
  ): AsyncIterable<FileChunk>;
  /** Metadata for a single file (the proto analog of an HTTP HEAD). */
  stat(path: string, opts: { signal: AbortSignal }): Promise<FileMetadata>;
  /** Phase 2 — write (upload) a file. Optional; absent in v1. */
  writeFile?(
    req: WriteFileRequest,
    opts: { signal: AbortSignal },
  ): Promise<WriteFileResult>;
  /** Phase 2 — create a directory. Optional; absent in v1. */
  createDir?(
    path: string,
    opts: { signal: AbortSignal },
  ): Promise<DirectoryMetadata>;
  /** Phase 2 — delete a file or directory. Optional; absent in v1. */
  delete?(path: string, opts: { signal: AbortSignal }): Promise<void>;
}

/**
 * Optional capabilities a runner declares at registration. `supports` is the
 * volume-level probe the default `FilesService` consults (e.g. the wasm engine
 * reads Azure/GCP only); omitted means "everything the UI asks about".
 * `canWrite` declares whether the write verbs are usable (false / undeclared in
 * v1 — writes are deferred).
 */
export interface FilesRunnerCapabilities {
  supports?(x: FilesSupportsInput): boolean;
  canWrite?: boolean;
}

/** Thrown by the default (unregistered) runner. Callers surface this as the
 *  reason the Files tab is unavailable; a registered runner never throws it. */
export class NoFilesRunnerError extends Error {
  constructor() {
    super(
      "No files runner registered. Volume file browsing needs a runner " +
        "installed via registerFilesRunner (the in-browser wasm engine, a host " +
        "runner, or the dev stub). The standalone build ships none, so the tab " +
        "stays hidden.",
    );
    this.name = "NoFilesRunnerError";
  }
}

/** The default runner: there is none. `listDirectory`/`stat` reject with
 *  `NoFilesRunnerError`; `readFile` returns an async iterable that throws on
 *  iteration (mirroring `noopQueryRunner`), so an accidentally-enabled tab fails
 *  loudly and legibly rather than hanging on an empty stream. */
const noopFilesRunner: FilesRunner = {
  listDirectory(): Promise<DirectoryPage> {
    return Promise.reject(new NoFilesRunnerError());
  },
  readFile(): AsyncIterable<FileChunk> {
    return {
      [Symbol.asyncIterator](): AsyncIterator<FileChunk> {
        return {
          next(): Promise<IteratorResult<FileChunk>> {
            return Promise.reject(new NoFilesRunnerError());
          },
        };
      },
    };
  },
  stat(): Promise<FileMetadata> {
    return Promise.reject(new NoFilesRunnerError());
  },
};

let current: FilesRunner = noopFilesRunner;
let currentCaps: FilesRunnerCapabilities = {};

/** Install a custom runner (with optional capabilities). Hosts / later phases
 *  call this once, before the UI bootstraps (late binding tolerates any order). */
export function registerFilesRunner(
  runner: FilesRunner,
  caps: FilesRunnerCapabilities = {},
): void {
  current = runner;
  currentCaps = caps;
}

/** The registered runner's capability probe (permissive when undeclared).
 *  Consulted by the default `FilesService.supports`. */
export function filesRunnerSupports(x: FilesSupportsInput): boolean {
  return currentCaps.supports?.(x) ?? true;
}

/** Whether the registered runner declares write support (false when undeclared).
 *  Consulted by the default `FilesService.canWrite`. */
export function filesRunnerCanWrite(): boolean {
  return currentCaps.canWrite ?? false;
}

/** The runner currently in effect (the registered one, or the throwing default). */
export function getFilesRunner(): FilesRunner {
  return current;
}

/** True once a real runner has been registered — the capability probe the Files
 *  tab reads so it never shows a view that can only error. */
export function hasFilesRunner(): boolean {
  return current !== noopFilesRunner;
}

// Stable reference the data layer always calls. Each method dereferences
// `current` on every call (late binding), so a host can register its runner
// before OR after this module is evaluated and still take effect — no ordering
// constraint. Optional write verbs are forwarded through when the current runner
// defines them; otherwise they are absent (undefined), matching v1's read-only
// surface.
export const filesRunner: FilesRunner = {
  listDirectory(req, opts) {
    return current.listDirectory(req, opts);
  },
  readFile(req, opts) {
    return current.readFile(req, opts);
  },
  stat(path, opts) {
    return current.stat(path, opts);
  },
  writeFile(req, opts) {
    if (!current.writeFile) throw new NoFilesRunnerError();
    return current.writeFile(req, opts);
  },
  createDir(path, opts) {
    if (!current.createDir) throw new NoFilesRunnerError();
    return current.createDir(path, opts);
  },
  delete(path, opts) {
    if (!current.delete) throw new NoFilesRunnerError();
    return current.delete(path, opts);
  },
};
