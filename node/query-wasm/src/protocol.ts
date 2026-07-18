// The message protocol between the runners (`createWasmQueryRunner` /
// `createWasmLogQueryRunner`, main thread) and the wasm worker. One worker per
// run: the main thread posts a single `run` or `run-log`, the worker streams
// `chunk` messages and finishes with `done` or `error`; cancellation is
// `Worker.terminate()` (the engine holds no cross-run state).

/** Main → worker: start the (single) table query this worker exists for. */
export interface RunMessage {
  type: "run";
  /** Unity Catalog REST base, e.g. `${origin}/api/2.1/unity-catalog`. */
  baseUrl: string;
  /** Optional bearer for the UC API (same-origin cookies flow regardless). */
  authToken?: string;
  sql: string;
  limit?: number;
  catalog?: string;
  schema?: string;
}

/** Main → worker: start the (single) reconciled-Delta-log query this worker
 *  exists for. Unlike `RunMessage`, there is no SQL — the physical table rides on
 *  `target` and `kind` selects the log surface; the engine synthesizes its own
 *  `delta_*_log('target')` table-function query. */
export interface LogRunMessage {
  type: "run-log";
  /** Unity Catalog REST base, e.g. `${origin}/api/2.1/unity-catalog`. */
  baseUrl: string;
  /** Optional bearer for the UC API (same-origin cookies flow regardless). */
  authToken?: string;
  limit?: number;
  /** Default catalog for a bare `target` (a fully-qualified target ignores it). */
  catalog?: string;
  /** Default schema for a bare `target` (a fully-qualified target ignores it). */
  schema?: string;
  /** The physical table whose log to scan. */
  target: string;
  /** Which log surface to project. */
  kind: "reconciled" | "actions";
}

/** Worker → main: one self-contained Arrow IPC chunk (transferred, not copied). */
export interface ChunkMessage {
  type: "chunk";
  ipc: Uint8Array;
  numRows: number;
}

/** Worker → main: the run finished; `stats` echoes the engine's summary. */
export interface DoneMessage {
  type: "done";
  stats: { chunks: number; rows: number; tableVersion: number };
}

/** Worker → main: the run failed. `code` is the engine's machine-readable
 *  class: "UNSUPPORTED" | "NETWORK" | "FAILED" (fallback composition treats
 *  the first two as retry-on-fallback signals). */
export interface ErrorMessage {
  type: "error";
  message: string;
  code: string;
}

export type WorkerResponse = ChunkMessage | DoneMessage | ErrorMessage;
