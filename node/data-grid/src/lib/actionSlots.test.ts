// Tests for action-slot detection: the reconciled action stream has six
// nullable top-level struct columns with exactly one non-null per row, and we
// must detect that one slot reading only validity (zero-copy). Data goes
// through a real Arrow IPC round-trip so we exercise the decoded Vector graph.

import { expect, test } from "bun:test";
import { type Table, tableFromJSON, tableToIPC } from "apache-arrow";
import {
  ACTION_SLOTS,
  detectActionSlot,
  isActionsSchema,
  resolveSlotColumns,
} from "./actionSlots";
import { ArrowResultStore } from "./arrowResultStore";

const EMPTY = {
  add: null as { path: string; size: bigint } | null,
  remove: null as { path: string } | null,
  metaData: null as { id: string } | null,
  protocol: null as { minReaderVersion: number } | null,
  domainMetadata: null as { domain: string } | null,
  txn: null as { appId: string; version: bigint } | null,
};

function rows() {
  return [
    { ...EMPTY, add: { path: "a.parquet", size: 100n } },
    { ...EMPTY, remove: { path: "b.parquet" } },
    { ...EMPTY, metaData: { id: "tbl-1" } },
    { ...EMPTY, protocol: { minReaderVersion: 3 } },
    { ...EMPTY, domainMetadata: { domain: "delta.x" } },
    { ...EMPTY, txn: { appId: "app", version: 7n } },
  ];
}

function table(): Table {
  return tableFromJSON(rows() as unknown as Record<string, unknown>[]);
}

function store(...slices: [number, number][]): ArrowResultStore {
  const t = table();
  const s = new ArrowResultStore();
  if (slices.length === 0) {
    s.append(tableToIPC(t, "stream"));
  } else {
    for (const [off, len] of slices)
      s.append(tableToIPC(t.slice(off, off + len), "stream"));
  }
  return s;
}

test("resolveSlotColumns finds all six slots in schema order", () => {
  const s = store();
  const cols = resolveSlotColumns(s);
  expect(cols.map((c) => c.slot)).toEqual([...ACTION_SLOTS]);
  for (const c of cols) expect(c.colIndex).toBeGreaterThanOrEqual(0);
});

test("isActionsSchema is true for the action stream", () => {
  expect(isActionsSchema(store())).toBe(true);
});

test("detectActionSlot returns the single populated slot per row", () => {
  const s = store();
  const cols = resolveSlotColumns(s);
  const detected = rows().map(
    (_r, i) => detectActionSlot(s, cols, i)?.slot ?? null,
  );
  expect(detected).toEqual([
    "add",
    "remove",
    "metaData",
    "protocol",
    "domainMetadata",
    "txn",
  ]);
});

test("detectActionSlot reads the value of the detected slot zero-copy", () => {
  const s = store();
  const cols = resolveSlotColumns(s);
  const add = detectActionSlot(s, cols, 0);
  expect(add?.slot).toBe("add");
  // The add slot's nested path/size read straight from the struct child vectors.
  expect(s.getNested(0, add?.colIndex ?? -1, ["path"])).toBe("a.parquet");
  expect(s.getNested(0, add?.colIndex ?? -1, ["size"])).toBe(100n);
});

test("detection works across batch boundaries", () => {
  const s = store([0, 2], [2, 4]);
  const cols = resolveSlotColumns(s);
  expect(detectActionSlot(s, cols, 0)?.slot).toBe("add");
  expect(detectActionSlot(s, cols, 2)?.slot).toBe("metaData");
  expect(detectActionSlot(s, cols, 5)?.slot).toBe("txn");
});

test("isActionsSchema is false for a non-action table", () => {
  const s = new ArrowResultStore();
  s.append(
    tableToIPC(
      tableFromJSON([{ id: 1, name: "x" }] as Record<string, unknown>[]),
      "stream",
    ),
  );
  expect(isActionsSchema(s)).toBe(false);
});
