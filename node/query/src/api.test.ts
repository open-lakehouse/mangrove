// Regression test for the StrictMode dead-handle trap: a preview handle whose
// mount effect fires start() → cancel() → start() (React 18 StrictMode double-
// invoke) must still stream rows into the store. The original deferred-start
// guard (`if (started || aborted) return`) left the run permanently dead after
// the cleanup abort, so the grid showed "No rows" until a dep change built a new
// handle. See node/query/src/api.ts.

import { expect, test } from "bun:test";
import { tableFromJSON, tableToIPC } from "apache-arrow";
import { createQueryService } from "./api";
import type { QueryChunk, QueryRunner } from "./runner";
import { registerQueryRunner } from "./runner";

// A one-row Arrow IPC chunk so `store.append` has something real to decode.
function oneRowChunk(): QueryChunk {
  const table = tableFromJSON([{ id: 1 }]);
  return { arrowIpc: tableToIPC(table, "stream"), numRows: 1 };
}

// A runner that yields one chunk after a microtask, and records how many times
// it was invoked and whether each invocation was aborted mid-flight.
function makeCountingRunner(): {
  runner: QueryRunner;
  calls: () => number;
  abortedCalls: () => number;
} {
  let calls = 0;
  let abortedCalls = 0;
  const runner: QueryRunner = async function* (_req, opts) {
    calls += 1;
    // Yield to the microtask queue so a synchronous cancel() (StrictMode
    // cleanup) can abort this run before it appends, exactly as in the browser.
    await Promise.resolve();
    if (opts.signal.aborted) {
      abortedCalls += 1;
      return;
    }
    yield oneRowChunk();
  };
  return { runner, calls: () => calls, abortedCalls: () => abortedCalls };
}

// Await enough microtasks that any dispatched drive() generators settle.
async function flush(): Promise<void> {
  for (let i = 0; i < 5; i++) await Promise.resolve();
}

test("StrictMode start→cancel→start still fills the store", async () => {
  const { runner } = makeCountingRunner();
  registerQueryRunner(runner);

  const handle = createQueryService().preview({
    tableFullName: "cat.sch.tbl",
    limit: 100,
  });

  // Simulate React 18 StrictMode's mount → cleanup → mount on the SAME handle.
  handle.start();
  handle.cancel();
  handle.start();

  await flush();

  // The second start() must have produced a live run that appended the row.
  expect(handle.store.rowCount).toBe(1);
  expect(handle.error).toBeNull();
  expect(handle.running).toBe(false);
});

test("a single start() (no StrictMode) fills the store", async () => {
  const { runner } = makeCountingRunner();
  registerQueryRunner(runner);

  const handle = createQueryService().preview({
    tableFullName: "cat.sch.tbl",
    limit: 100,
  });
  handle.start();
  await flush();

  expect(handle.store.rowCount).toBe(1);
});

test("cancel() with no restart leaves the store empty and does not error", async () => {
  const { runner } = makeCountingRunner();
  registerQueryRunner(runner);

  const handle = createQueryService().preview({
    tableFullName: "cat.sch.tbl",
    limit: 100,
  });
  handle.start();
  handle.cancel();
  await flush();

  // A genuine unmount: aborted run appends nothing, surfaces no error.
  expect(handle.store.rowCount).toBe(0);
  expect(handle.error).toBeNull();
});

test("restart does not double-append rows", async () => {
  const { runner } = makeCountingRunner();
  registerQueryRunner(runner);

  const handle = createQueryService().preview({
    tableFullName: "cat.sch.tbl",
    limit: 100,
  });
  handle.start();
  await flush(); // first run completes, store has 1 row
  handle.cancel();
  handle.start(); // restart: store.reset() then re-stream
  await flush();

  expect(handle.store.rowCount).toBe(1);
});
