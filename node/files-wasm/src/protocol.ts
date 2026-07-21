// The message protocol between the files runner (`createWasmFilesRunner`, main
// thread) and the wasm worker. One worker per operation: the main thread posts a
// single `connectUnary` / `read` / `write`, the worker replies and finishes with
// `done` / `error` (`read` streams `chunk` messages first); cancellation is
// `Worker.terminate()` (the engine holds no cross-operation state).
//
// The split mirrors hydrofoil's Tauri backend exactly: unary METADATA RPCs go
// through the generic `connectUnary` binary-proto dispatch (the wasm connect
// Router), and file BYTES bypass proto — `read` / `write` are native byte
// transfers. Metadata request/response bodies are opaque protobuf `Uint8Array`s
// here; the connect client (in `@open-lakehouse/files-connect`) encodes/decodes
// them.

/** Main → worker: dispatch one unary FilesService RPC as binary proto. */
export interface ConnectUnaryMessage {
  type: "connectUnary";
  /** Unity Catalog REST base, e.g. `${origin}/api/2.1/unity-catalog`. */
  baseUrl: string;
  /** Optional bearer for the UC API (same-origin cookies flow regardless). */
  authToken?: string;
  /** Full RPC path, e.g. `portal.files.v1.FilesService/GetFileMetadata`. */
  path: string;
  /** Binary-proto request body. */
  requestBytes: Uint8Array;
}

/** Main → worker: read a file (or byte range) as a chunk stream (bytes bypass
 *  the connect dispatcher). */
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

/** Main → worker: write (create or overwrite) a file from one buffered body
 *  (bytes bypass the connect dispatcher). */
export interface WriteMessage {
  type: "write";
  /** Unity Catalog REST base, e.g. `${origin}/api/2.1/unity-catalog`. */
  baseUrl: string;
  /** Optional bearer for the UC API (same-origin cookies flow regardless). */
  authToken?: string;
  /** Destination file (canonical `/Volumes/<c>/<s>/<v>/<rest>`). */
  path: string;
  /** The whole file body. */
  bytes: Uint8Array;
  /** MIME type recorded as the object's Content-Type (optional). */
  contentType?: string;
  /** Optional if-match etag: a conditional overwrite (lost-update guard). */
  ifMatchEtag?: string;
}

/** Any request the worker accepts. */
export type WorkerRequest = ConnectUnaryMessage | ReadMessage | WriteMessage;

/** The post-write metadata `writeFileBytes` resolves to. */
export interface WriteResultPayload {
  path: string;
  fileSize: number;
  etag?: string;
}

/** Worker → main: the binary-proto response for a `connectUnary`. */
export interface UnaryMessage {
  type: "unary";
  responseBytes: Uint8Array;
}

/** Worker → main: one chunk of a `read` stream (transferred, not copied). */
export interface ChunkMessage {
  type: "chunk";
  bytes: Uint8Array;
}

/** Worker → main: the resolved write metadata (answers a `write`). */
export interface WriteResultMessage {
  type: "writeResult";
  result: WriteResultPayload;
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
  | UnaryMessage
  | ChunkMessage
  | WriteResultMessage
  | DoneMessage
  | ErrorMessage;
