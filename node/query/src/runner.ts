// Pluggable query-execution seam — the single swap point that lets a host or a
// later phase run SQL somewhere (an in-browser wasm engine, a Tauri `invoke`, a
// downstream ConnectRPC service) WITHOUT the UI depending on that mechanism.
//
// Mangrove ships NO runtime runner. This is deliberate: the in-browser query
// layer's whole purpose is a default the website can ship needing only a running
// Unity Catalog and no extra service, and that default is the wasm engine
// (Phase B) — not yet landed. Until a runner is registered, `queryRunner` throws
// `NoQueryRunnerError`, and the UI keeps its preview affordance gated off.
//
// The rest of the data layer (the query service, the Arrow store, the grid)
// depends ONLY on the `QueryRunner` type and the late-binding `queryRunner`
// below — never on a transport. A host installs its runner via
// `registerQueryRunner` before the UI bootstraps; the future
// `createWasmQueryService` does the same.
//
// The request/chunk shapes are typed against the generated `open_lakehouse.
// query.v1` contract (proto/query) so every implementing runner — the wasm
// engine, a downstream host — speaks the same upstream-owned shape.

import type {
  RunQueryRequest,
  RunQueryResponse,
} from "./gen/open_lakehouse/query/v1/svc_pb";

/**
 * A query to execute. Structurally the generated `RunQueryRequest` minus its
 * protobuf-`Message` brand, so plain object literals satisfy it without
 * `create(...)`.
 */
export type QueryRequest = Omit<RunQueryRequest, "$typeName" | "$unknown">;

/**
 * One streamed result chunk: a self-contained Arrow IPC stream for one record
 * batch (schema + batch + EOS) plus its row count. `arrowIpc` feeds straight
 * into `ArrowResultStore.append`; `numRows` is a `number` (narrowed from the
 * contract's `uint64`/`bigint`) for the UI's progress display.
 */
export interface QueryChunk {
  arrowIpc: RunQueryResponse["arrowIpc"];
  numRows: number;
}

/**
 * Executes a query and yields Arrow IPC chunks as they are produced. The
 * implementation is host-chosen; aborting `opts.signal` must tear the execution
 * down.
 */
export type QueryRunner = (
  req: QueryRequest,
  opts: { signal: AbortSignal },
) => AsyncIterable<QueryChunk>;

/** Thrown by the default (unregistered) runner. Callers surface this as the
 *  reason preview is unavailable; a registered runner never throws it. */
export class NoQueryRunnerError extends Error {
  constructor() {
    super(
      "No query runner registered. Table preview needs a runner installed via " +
        "registerQueryRunner (the in-browser wasm engine or a host runner). " +
        "The standalone build ships none, so preview stays disabled.",
    );
    this.name = "NoQueryRunnerError";
  }
}

/** The default runner: there is none. Returns an async iterable that throws on
 *  iteration, so an accidentally-enabled preview fails loudly and legibly rather
 *  than hanging on an empty stream. */
const noopQueryRunner: QueryRunner = () => ({
  [Symbol.asyncIterator](): AsyncIterator<QueryChunk> {
    return {
      next(): Promise<IteratorResult<QueryChunk>> {
        return Promise.reject(new NoQueryRunnerError());
      },
    };
  },
});

let current: QueryRunner = noopQueryRunner;

/** Install a custom runner. Hosts / later phases call this once, before the UI
 *  bootstraps (though late binding below tolerates any ordering). */
export function registerQueryRunner(runner: QueryRunner): void {
  current = runner;
}

/** The runner currently in effect (the registered one, or the throwing default). */
export function getQueryRunner(): QueryRunner {
  return current;
}

/** True once a real runner has been registered — the capability probe the UI's
 *  feature gate reads so it never shows a preview that can only error. */
export function hasQueryRunner(): boolean {
  return current !== noopQueryRunner;
}

// Stable reference the data layer always calls. It dereferences `current` on
// every call (late binding), so a host can register its runner before OR after
// this module is evaluated and still take effect — no ordering constraint.
export const queryRunner: QueryRunner = (req, opts) => current(req, opts);
