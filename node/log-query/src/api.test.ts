// Regression test for the StrictMode dead-handle trap in the log-query mirror of
// @open-lakehouse/query. See node/query/src/api.test.ts for the full rationale:
// a mount effect firing start() → cancel() → start() (React 18 StrictMode) must
// still stream rows; the original guard left the run permanently aborted, so the
// Delta-log grid showed "No rows" until a reconciled/actions toggle built a new
// handle.

import { expect, test } from "bun:test";
import { tableFromJSON, tableToIPC } from "apache-arrow";
import { createLogQueryService } from "./api";
import type { LogQueryChunk, LogQueryRunner } from "./runner";
import { registerLogQueryRunner } from "./runner";

function oneRowChunk(): LogQueryChunk {
  const table = tableFromJSON([{ id: 1 }]);
  return { arrowIpc: tableToIPC(table, "stream"), numRows: 1 };
}

function makeRunner(): LogQueryRunner {
  return async function* (_req, opts) {
    await Promise.resolve();
    if (opts.signal.aborted) return;
    yield oneRowChunk();
  };
}

async function flush(): Promise<void> {
  for (let i = 0; i < 5; i++) await Promise.resolve();
}

test("StrictMode start→cancel→start still fills the log store", async () => {
  registerLogQueryRunner(makeRunner());
  const handle = createLogQueryService().preview({
    target: "cat.sch.tbl",
    limit: 100,
    kind: "reconciled",
  });

  handle.start();
  handle.cancel();
  handle.start();
  await flush();

  expect(handle.store.rowCount).toBe(1);
  expect(handle.error).toBeNull();
  expect(handle.running).toBe(false);
});

test("cancel() with no restart leaves the log store empty, no error", async () => {
  registerLogQueryRunner(makeRunner());
  const handle = createLogQueryService().preview({
    target: "cat.sch.tbl",
    limit: 100,
    kind: "actions",
  });

  handle.start();
  handle.cancel();
  await flush();

  expect(handle.store.rowCount).toBe(0);
  expect(handle.error).toBeNull();
});

test("restart does not double-append log rows", async () => {
  registerLogQueryRunner(makeRunner());
  const handle = createLogQueryService().preview({
    target: "cat.sch.tbl",
    limit: 100,
    kind: "reconciled",
  });

  handle.start();
  await flush();
  handle.cancel();
  handle.start();
  await flush();

  expect(handle.store.rowCount).toBe(1);
});
