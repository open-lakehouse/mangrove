// The fixtures back both the stories and the dev app. Guard the two shapes the
// visualizations depend on: the reconciled stats sub-structs (present + absent)
// and the actions six-slot "exactly one non-null per row" invariant. Everything
// goes through a real Arrow IPC round-trip so we validate the inferred schema,
// not the plain objects.

import { expect, test } from "bun:test";
import { tableFromJSON, tableToIPC } from "apache-arrow";
import {
  type ActionLogRow,
  actionsLogFixtureRows,
  reconciledLogFixtureRows,
} from "./fixtures";

test("reconciled fixture carries typed minValues/maxValues sub-structs", () => {
  const table = tableFromJSON(
    reconciledLogFixtureRows(8) as unknown as Record<string, unknown>[],
  );
  const stats = table.schema.fields.find((f) => f.name === "stats");
  expect(stats).toBeDefined();
  const children =
    (stats?.type as { children?: { name: string }[] } | undefined)?.children ??
    [];
  const names = children.map((c) => c.name);
  expect(names).toContain("minValues");
  expect(names).toContain("maxValues");
});

test("reconciled fixture withStats=false omits minValues/maxValues", () => {
  const table = tableFromJSON(
    reconciledLogFixtureRows(8, false) as unknown as Record<string, unknown>[],
  );
  const stats = table.schema.fields.find((f) => f.name === "stats");
  const children =
    (stats?.type as { children?: { name: string }[] } | undefined)?.children ??
    [];
  const names = children.map((c) => c.name);
  expect(names).not.toContain("minValues");
  expect(names).not.toContain("maxValues");
  expect(names).toContain("numRecords");
});

test("actions fixture has the six action slots and exactly one non-null per row", () => {
  const rows = actionsLogFixtureRows(24);
  const table = tableFromJSON(rows as unknown as Record<string, unknown>[]);
  const slotNames = [
    "add",
    "remove",
    "metaData",
    "protocol",
    "domainMetadata",
    "txn",
  ];
  const fieldNames = table.schema.fields.map((f) => f.name);
  for (const s of slotNames) expect(fieldNames).toContain(s);

  // Exactly one populated slot per row (the reconciled action-stream invariant).
  for (const row of rows) {
    const populated = slotNames.filter(
      (s) => (row as unknown as Record<string, unknown>)[s] != null,
    );
    expect(populated.length).toBe(1);
  }
});

test("actions fixture round-trips through Arrow IPC", () => {
  const rows: ActionLogRow[] = actionsLogFixtureRows(12);
  const table = tableFromJSON(rows as unknown as Record<string, unknown>[]);
  const ipc = tableToIPC(table, "stream");
  expect(ipc.byteLength).toBeGreaterThan(0);
});
