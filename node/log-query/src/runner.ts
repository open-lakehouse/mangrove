// Pluggable log-query-execution seam — the single swap point that lets a host or
// a later phase run reconciled-Delta-log SQL somewhere (an in-browser wasm
// engine, a downstream service) WITHOUT the UI depending on that mechanism.
// Mirrors @open-lakehouse/query's runner.ts, but for the reconciled-log surface.
//
// Mangrove ships NO runtime runner here either. Until one is registered,
// `logQueryRunner` throws `NoLogQueryRunnerError`, and the UI keeps its Delta-log
// tab gated off (`hasLogQueryRunner()` reads false).
//
// The eventual runner registers the async-native `ReconciledLogProvider`
// (crates/olai-delta-df) under a fixed logical table name and streams Arrow IPC.
// There is no generated proto contract for this surface yet, so the request
// shape is defined locally rather than derived from protobuf.

import type { LogSupportsInput } from "./types";

/** Which reconciled-Delta-log surface to scan. `reconciled` = the surviving
 *  scan-file rows after log replay; `actions` = the reconciled full action
 *  stream (add/remove/metaData/protocol/domainMetadata/txn). */
export type LogKind = "reconciled" | "actions";

/**
 * A reconciled-log query to execute. `sql` targets the fixed logical table name
 * the runner binds the provider to (see api.ts, one name per `kind`); `target`
 * carries the physical table (fully-qualified name or storage path) the runner
 * resolves and binds — it travels out-of-band from the SQL. A later
 * log-query-wasm impl may swap this for a generated contract type.
 */
export interface LogQueryRequest {
  /** SQL over the fixed logical table the runner registers. */
  sql: string;
  /** Row cap (narrowed for the UI); the service applies a default. */
  limit?: number;
  /** The table whose reconciled log to scan — resolved by the runner. */
  target?: string;
  /** Which log surface to project (default `reconciled`). */
  kind?: LogKind;
}

/**
 * One streamed result chunk: a self-contained Arrow IPC stream for one record
 * batch (schema + batch + EOS) plus its row count. `arrowIpc` feeds straight
 * into `ArrowResultStore.append`.
 */
export interface LogQueryChunk {
  arrowIpc: Uint8Array;
  numRows: number;
}

/**
 * Executes a reconciled-log query and yields Arrow IPC chunks as they are
 * produced. The implementation is host-chosen; aborting `opts.signal` must tear
 * the execution down.
 */
export type LogQueryRunner = (
  req: LogQueryRequest,
  opts: { signal: AbortSignal },
) => AsyncIterable<LogQueryChunk>;

/** Thrown by the default (unregistered) runner. Callers surface this as the
 *  reason the Delta-log tab is unavailable; a registered runner never throws it. */
export class NoLogQueryRunnerError extends Error {
  constructor() {
    super(
      "No log-query runner registered. The Delta-log explorer needs a runner " +
        "installed via registerLogQueryRunner (the in-browser wasm engine, a " +
        "host runner, or the dev stub). The standalone build ships none, so the " +
        "tab stays hidden.",
    );
    this.name = "NoLogQueryRunnerError";
  }
}

/** The default runner: there is none. Returns an async iterable that throws on
 *  iteration, so an accidentally-enabled tab fails loudly rather than hanging. */
const noopLogQueryRunner: LogQueryRunner = () => ({
  [Symbol.asyncIterator](): AsyncIterator<LogQueryChunk> {
    return {
      next(): Promise<IteratorResult<LogQueryChunk>> {
        return Promise.reject(new NoLogQueryRunnerError());
      },
    };
  },
});

/**
 * Optional capabilities a runner declares at registration. `supports` is the
 * table-level probe the default `LogQueryService` consults (e.g. a wasm engine
 * reads Delta only); omitted means "everything the UI asks about".
 */
export interface LogQueryRunnerCapabilities {
  supports?(x: LogSupportsInput): boolean;
}

let current: LogQueryRunner = noopLogQueryRunner;
let currentCaps: LogQueryRunnerCapabilities = {};

/** Install a custom runner (with optional capabilities). Hosts / later phases
 *  call this once, before the UI bootstraps (late binding tolerates any order). */
export function registerLogQueryRunner(
  runner: LogQueryRunner,
  caps: LogQueryRunnerCapabilities = {},
): void {
  current = runner;
  currentCaps = caps;
}

/** The registered runner's capability probe (permissive when undeclared).
 *  Consulted by the default `LogQueryService.supports`. */
export function logQueryRunnerSupports(x: LogSupportsInput): boolean {
  return currentCaps.supports?.(x) ?? true;
}

/** The runner currently in effect (the registered one, or the throwing default). */
export function getLogQueryRunner(): LogQueryRunner {
  return current;
}

/** True once a real runner has been registered — the capability probe the
 *  Delta-log tab reads so it never shows a view that can only error. */
export function hasLogQueryRunner(): boolean {
  return current !== noopLogQueryRunner;
}

// Stable reference the data layer always calls. It dereferences `current` on
// every call (late binding), so a host can register its runner before OR after
// this module is evaluated and still take effect — no ordering constraint.
export const logQueryRunner: LogQueryRunner = (req, opts) => current(req, opts);
