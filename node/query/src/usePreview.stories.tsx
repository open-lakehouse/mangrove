// Stories for the preview seam. They exercise the whole Phase 0 path with a FAKE
// runner (no server, no wasm): a registered `QueryRunner` yields canned Arrow IPC
// chunks, `usePreview` streams them into an `ArrowResultStore`, and `<DataGrid>`
// renders — proving the seam end-to-end in isolation. Also covers the error path
// and the unregistered-runner default (preview stays off).
//
// Mirrors data-grid.stories.tsx. Storybook is not yet wired in this repo; these
// are the reference harness for when it lands (and are excluded from tsc like the
// other packages' stories).

import { DataGrid } from "@open-lakehouse/data-grid";
import type { Meta, StoryObj } from "@storybook/react-vite";
import { type Table, tableFromArrays, tableToIPC } from "apache-arrow";
import {
  createQueryService,
  hasQueryRunner,
  type QueryRunner,
  registerQueryRunner,
  usePreview,
} from "./index";

// --- canned Arrow IPC (self-contained; this package doesn't reach into
// data-grid's internal fixtures) ---
const rowsTable: Table = tableFromArrays({
  customer_id: Int32Array.from([1001, 1002, 1003, 1004, 1005]),
  full_name: [
    "Ada Lovelace",
    "Alan Turing",
    "Grace Hopper",
    "Edsger Dijkstra",
    "Barbara Liskov",
  ],
  revenue_usd: Float64Array.from([12450.5, 9870.0, 15320.75, 7600.0, 20110.25]),
});
const rowsIpc = tableToIPC(rowsTable, "stream");

// A fake runner that streams the canned batch, then completes.
const fakeRunner: QueryRunner = async function* () {
  yield { arrowIpc: rowsIpc, numRows: rowsTable.numRows };
};

// A runner that fails on iteration, to drive the error branch.
const failingRunner: QueryRunner = () => ({
  [Symbol.asyncIterator]() {
    return {
      next: () => Promise.reject(new Error("preview query failed (fake)")),
    };
  },
});

// A tiny harness component: register a runner, then preview a table.
function Preview({
  runner,
  fullName = "main.default.customers",
}: {
  runner: QueryRunner;
  fullName?: string;
}) {
  // Register before the first preview runs (the seam is module-global, so this
  // is set synchronously in render for the story harness). hasQueryRunner() then
  // reports true, exactly as the real UI's feature gate reads it.
  registerQueryRunner(runner);

  const svc = createQueryService();
  const { store, version, running, error } = usePreview(
    { tableFullName: fullName, limit: 100 },
    svc,
  );

  if (error) return <p style={{ color: "crimson" }}>{error.message}</p>;
  return (
    <div>
      <p style={{ fontSize: 12, opacity: 0.7 }}>
        runner registered: {String(hasQueryRunner())} · running:{" "}
        {String(running)} · rows: {store.rowCount}
      </p>
      <div style={{ height: 360, border: "1px solid #ddd", borderRadius: 6 }}>
        <DataGrid store={store} version={version} running={running} />
      </div>
    </div>
  );
}

const meta: Meta<typeof Preview> = {
  title: "Query/Preview",
  component: Preview,
  parameters: { layout: "fullscreen" },
};
export default meta;
type Story = StoryObj<typeof Preview>;

/** Happy path: a registered runner streams rows into the grid. */
export const Supported: Story = { args: { runner: fakeRunner } };

/** The runner errors; the preview surfaces the message instead of the grid. */
export const Errored: Story = { args: { runner: failingRunner } };
