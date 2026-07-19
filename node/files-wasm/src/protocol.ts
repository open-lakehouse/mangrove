// The message protocol between the files runner (`createWasmFilesRunner`, main
// thread) and the wasm worker. One worker per operation: the main thread posts a
// single `list` / `read` / `stat`, the worker replies and finishes with `done` /
// `error` (`read` streams `chunk` messages first); cancellation is
// `Worker.terminate()` (the engine holds no cross-operation state). Mirrors
// @open-lakehouse/query-wasm's protocol.ts, adapted for the files verbs — list
// and stat are request→single-response, read is a chunk stream.

/** Main → worker: list one bounded page of a directory. */
export interface ListMessage {
  type: "list";
  /** Unity Catalog REST base, e.g. `${origin}/api/2.1/unity-catalog`. */
  baseUrl: string;
  /** Optional bearer for the UC API (same-origin cookies flow regardless). */
  authToken?: string;
  /** Directory to list (canonical `/Volumes/<c>/<s>/<v>/<rest>`). */
  path: string;
  /** Page size; the engine applies its own cap when omitted. */
  maxResults?: number;
  /** Opaque continuation token from a previous page. */
  pageToken?: string;
}

/** Main → worker: read a file (or byte range) as a chunk stream. */
export interface ReadMessage {
  type: "read";
  /** Unity Catalog REST base, e.g. `${origin}/api/2.1/unity-catalog`. */
  baseUrl: string;
  /** Optional bearer for the UC API (same-origin cookies flow regardless). */
  authToken?: string;
  /** File to read (canonical `/Volumes/<c>/<s>/<v>/<rest>`). */
  path: string;
  /** Byte offset to start from (defaults to 0). */
  offset?: number;
  /** Number of bytes to read (defaults to the rest of the file). */
  length?: number;
}

/** Main → worker: stat a single file (HTTP HEAD analog). */
export interface StatMessage {
  type: "stat";
  /** Unity Catalog REST base, e.g. `${origin}/api/2.1/unity-catalog`. */
  baseUrl: string;
  /** Optional bearer for the UC API (same-origin cookies flow regardless). */
  authToken?: string;
  /** File to stat (canonical `/Volumes/<c>/<s>/<v>/<rest>`). */
  path: string;
}

/** Any request the worker accepts. */
export type WorkerRequest = ListMessage | ReadMessage | StatMessage;

/** The directory-page shape the engine's `listDirectory` serializes to (the
 *  @open-lakehouse/files `DirectoryPage`). */
export interface DirectoryPagePayload {
  entries: Array<{
    path: string;
    isDirectory: boolean;
    fileSize: number;
    lastModified: number;
  }>;
  nextPageToken?: string;
}

/** The file-metadata shape the engine's `stat` serializes to (the
 *  @open-lakehouse/files `FileMetadata`). */
export interface FileMetadataPayload {
  path: string;
  fileSize: number;
  lastModified: number;
  contentType?: string;
  etag?: string;
}

/** Worker → main: the resolved directory page (answers a `list`). */
export interface PageMessage {
  type: "page";
  page: DirectoryPagePayload;
}

/** Worker → main: the resolved file metadata (answers a `stat`). */
export interface MetaMessage {
  type: "meta";
  meta: FileMetadataPayload;
}

/** Worker → main: one chunk of a `read` stream (transferred, not copied). */
export interface ChunkMessage {
  type: "chunk";
  bytes: Uint8Array;
}

/** Worker → main: the operation finished successfully. */
export interface DoneMessage {
  type: "done";
}

/** Worker → main: the operation failed. `code` is the engine's machine-readable
 *  class: "UNSUPPORTED" | "NETWORK" | "FAILED" (fallback composition treats the
 *  first two as retry-on-fallback signals). */
export interface ErrorMessage {
  type: "error";
  message: string;
  code: string;
}

export type WorkerResponse =
  | PageMessage
  | MetaMessage
  | ChunkMessage
  | DoneMessage
  | ErrorMessage;
