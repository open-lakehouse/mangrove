// Foundation tests for the zero-copy nested-struct access layer: the store's
// getNested / getLeafVector / isSlotValid and the resolveChildPath walk. Data
// goes through a real Arrow IPC round-trip (the same path the streaming service
// uses) so we exercise the decoded Vector graph, not hand-built objects.

import { expect, test } from "bun:test";
import { type Table, tableFromJSON, tableToIPC } from "apache-arrow";
import { ArrowResultStore } from "./arrowResultStore";

// One reconciled-log-shaped row: a top-level `stats` struct with nested
// minValues/maxValues leaves, plus a sibling nullable `dv` struct. Build the
// whole set as ONE table first so arrow infers a stable nested schema, then
// slice it into batches to prove cross-batch resolution.
interface Row {
  path: string;
  stats: {
    numRecords: bigint;
    minValues: { id: number; amount: number };
    maxValues: { id: number; amount: number };
  } | null;
  dv: { cardinality: number } | null;
}

function rows(): Row[] {
  return [
    {
      path: "a.parquet",
      stats: {
        numRecords: 100n,
        minValues: { id: 0, amount: 1.5 },
        maxValues: { id: 99, amount: 42.25 },
      },
      dv: { cardinality: 3 },
    },
    {
      path: "b.parquet",
      stats: {
        numRecords: 200n,
        minValues: { id: 100, amount: 10.0 },
        maxValues: { id: 299, amount: 88.0 },
      },
      dv: null,
    },
    {
      path: "c.parquet",
      stats: null,
      dv: { cardinality: 7 },
    },
  ];
}

function table(): Table {
  return tableFromJSON(rows() as unknown as Record<string, unknown>[]);
}

function storeFrom(...slices: [number, number][]): ArrowResultStore {
  const t = table();
  const store = new ArrowResultStore();
  for (const [off, len] of slices) {
    store.append(tableToIPC(t.slice(off, off + len), "stream"));
  }
  return store;
}

function colIndex(store: ArrowResultStore, name: string): number {
  const fields = store.schema?.fields ?? [];
  return fields.findIndex((f) => f.name === name);
}

test("getNested reads typed leaf values from a nested struct path", () => {
  const store = storeFrom([0, 3]);
  const stats = colIndex(store, "stats");
  expect(store.getNested(0, stats, ["minValues", "id"])).toBe(0);
  expect(store.getNested(0, stats, ["maxValues", "id"])).toBe(99);
  expect(store.getNested(0, stats, ["minValues", "amount"])).toBe(1.5);
  expect(store.getNested(1, stats, ["maxValues", "amount"])).toBe(88.0);
});

test("getNested returns null when the parent struct slot is null", () => {
  const store = storeFrom([0, 3]);
  const stats = colIndex(store, "stats");
  // Row 2 has stats === null: leaf reads under it are null, not a throw.
  expect(store.getNested(2, stats, ["minValues", "id"])).toBeNull();
});

test("getNested returns null for an absent path segment", () => {
  const store = storeFrom([0, 3]);
  const stats = colIndex(store, "stats");
  expect(store.getNested(0, stats, ["notAField", "id"])).toBeNull();
  expect(store.getNested(0, stats, ["minValues", "nope"])).toBeNull();
});

test("isSlotValid distinguishes populated from null top-level struct slots", () => {
  const store = storeFrom([0, 3]);
  const dv = colIndex(store, "dv");
  expect(store.isSlotValid(0, dv)).toBe(true);
  expect(store.isSlotValid(1, dv)).toBe(false); // dv === null
  expect(store.isSlotValid(2, dv)).toBe(true);
});

test("nested resolution works across batch boundaries", () => {
  // Two batches: rows 0 in the first, rows 1-2 in the second. A global row must
  // resolve to the right batch's leaf vector.
  const store = storeFrom([0, 1], [1, 2]);
  const stats = colIndex(store, "stats");
  expect(store.getNested(0, stats, ["minValues", "id"])).toBe(0);
  expect(store.getNested(1, stats, ["minValues", "id"])).toBe(100);
  expect(store.getNested(2, stats, ["minValues", "id"])).toBeNull(); // stats null
});

test("getLeafVector binds once for a tight per-batch loop", () => {
  const store = storeFrom([0, 3]);
  const stats = colIndex(store, "stats");
  const seen: (number | null)[] = [];
  store.forEachBatch((batchIndex, startRow, length) => {
    const leaf = store.getLeafVector(batchIndex, stats, ["minValues", "id"]);
    for (let r = 0; r < length; r++) {
      // Row 2 (stats null) -> leaf.get returns null even though the leaf vector
      // itself exists for the batch.
      seen[startRow + r] = leaf ? (leaf.get(r) as number | null) : null;
    }
  });
  expect(seen).toEqual([0, 100, null]);
});

test("out-of-range rows are null / invalid, not throws", () => {
  const store = storeFrom([0, 3]);
  const stats = colIndex(store, "stats");
  expect(store.getNested(99, stats, ["minValues", "id"])).toBeNull();
  expect(store.isSlotValid(99, stats)).toBe(false);
});
