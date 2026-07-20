// A dev/stub `LogQueryRunner` that streams a rich, canned reconciled-Delta-log
// dataset as Arrow IPC — no wasm, no server. It proves the log-query seam + the
// DataGrid end-to-end and is the data the Delta-log UI is built against until the
// real `ReconciledLogProvider` wasm runner lands.
//
// Lives on the `@open-lakehouse/log-query/testing` subexport (mirroring
// query-wasm's `./stub`) so it ships with the seam it satisfies and backs both
// the app's dev wiring and Storybook stories, without polluting the main barrel.

import { type Table, tableFromJSON, tableToIPC } from "apache-arrow";
import {
  type LogKind,
  type LogQueryChunk,
  type LogQueryRunner,
  registerLogQueryRunner,
} from "../runner";
import { actionsLogFixtureRows, reconciledLogFixtureRows } from "./fixtures";

// Rows per streamed record batch. Small enough that the ~40-row fixture arrives
// in several chunks, exercising progressive render, virtualization and the
// running -> done transition.
const BATCH_ROWS = 12;

// Build the full fixture as ONE Arrow table first, so type inference sees a
// non-null example of every nested struct (deletion vector, each action slot)
// and settles the nested struct schema; then stream it out batch-by-batch.
// `tableFromJSON` maps bigint -> Int64, nested objects -> Struct, and null ->
// nullable, matching the reconciled scan-file-row / action-stream shapes.
function buildFixtureTable(kind: LogKind): Table {
  // tableFromJSON is typed for Record<string, unknown>; the fixture rows carry
  // bigints and nested structs it handles at runtime.
  const rows =
    kind === "actions" ? actionsLogFixtureRows() : reconciledLogFixtureRows();
  return tableFromJSON(rows as unknown as Record<string, unknown>[]);
}

/** One self-contained IPC stream (schema + batch + EOS) for a table slice. */
function sliceToIpc(table: Table, offset: number, length: number): Uint8Array {
  return tableToIPC(table.slice(offset, offset + length), "stream");
}

/**
 * A stub runner streaming the canned log dataset in several batches, then
 * completing. Serves the reconciled or action-stream fixture per `req.kind`
 * (ignoring `target`), and honours abort between batches.
 */
export const stubLogQueryRunner: LogQueryRunner = (req, { signal }) => ({
  async *[Symbol.asyncIterator](): AsyncIterator<LogQueryChunk> {
    signal.throwIfAborted();
    const table = buildFixtureTable(req.kind ?? "reconciled");
    for (let offset = 0; offset < table.numRows; offset += BATCH_ROWS) {
      if (signal.aborted) throw signal.reason ?? new Error("aborted");
      const length = Math.min(BATCH_ROWS, table.numRows - offset);
      yield {
        arrowIpc: sliceToIpc(table, offset, length),
        numRows: length,
      } satisfies LogQueryChunk;
    }
  },
});

/**
 * One-call wiring: register the stub as the app's log-query runner with a
 * permissive capability probe (the Delta/format gate lives in the tab). Call once
 * at startup, before the UI bootstraps. Mirrors `registerWasmPreview`.
 */
export function registerStubLogPreview(): void {
  registerLogQueryRunner(stubLogQueryRunner, { supports: () => true });
}
