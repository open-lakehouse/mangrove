// Tests for min/max axis enumeration and the zero-copy interval extraction. The
// reconciled log carries per-file stats.minValues/maxValues nested structs; an
// axis is usable iff a column is an orderable leaf under BOTH. Data goes through
// a real Arrow IPC round-trip.

import { expect, test } from "bun:test";
import { ArrowResultStore } from "@open-lakehouse/data-grid";
import { type Table, tableFromJSON, tableToIPC } from "apache-arrow";
import { enumerateMinMaxAxes, hasMinMaxAxes } from "./minMaxAxes";

// A reconciled-shaped fixture with numeric min/max leaves (id, amount) plus a
// string leaf (note) that must NOT become an axis.
interface Row {
  path: string;
  stats: {
    numRecords: bigint;
    minValues: { id: number; amount: number; note: string };
    maxValues: { id: number; amount: number; note: string };
  };
}

function rows(count = 4): Row[] {
  return Array.from({ length: count }, (_v, i) => ({
    path: `part-${i}.parquet`,
    stats: {
      numRecords: BigInt(100 + i),
      minValues: { id: i * 10, amount: i * 1.5, note: "a" },
      maxValues: { id: i * 10 + 9, amount: i * 1.5 + 5, note: "z" },
    },
  }));
}

function table(rs: object[]): Table {
  return tableFromJSON(rs as unknown as Record<string, unknown>[]);
}

function storeFrom(rs: object[]): ArrowResultStore {
  const s = new ArrowResultStore();
  s.append(tableToIPC(table(rs), "stream"));
  return s;
}

test("enumerateMinMaxAxes returns orderable leaves present in both min and max", () => {
  const s = storeFrom(rows());
  const axes = enumerateMinMaxAxes(s.schema?.fields);
  const names = axes.map((a) => a.name);
  expect(names).toContain("id");
  expect(names).toContain("amount");
  // "note" is a string leaf — not an axis.
  expect(names).not.toContain("note");
});

test("hasMinMaxAxes is true when orderable axes exist, false when absent", () => {
  expect(hasMinMaxAxes(storeFrom(rows()).schema?.fields)).toBe(true);

  // A table whose stats has numRecords but no minValues/maxValues sub-structs.
  const noStats = storeFrom([{ path: "a.parquet", stats: { numRecords: 1n } }]);
  expect(hasMinMaxAxes(noStats.schema?.fields)).toBe(false);
});

test("enumerateMinMaxAxes returns [] when there is no stats column", () => {
  const s = storeFrom([{ path: "a.parquet", size: 10n } as unknown as Row]);
  expect(enumerateMinMaxAxes(s.schema?.fields)).toEqual([]);
});

// useMinMaxBoxes is a hook (useMemo); its extraction logic is pure. Invoke the
// underlying computation by calling it inside a trivial "render" via the
// same code path is overkill for bun:test, so we validate the zero-copy read
// path directly against the store instead.
test("min/max intervals read zero-copy match the fixture values", () => {
  const rs = rows(3);
  const s = storeFrom(rs);
  const fields = s.schema?.fields ?? [];
  const statsCol = fields.findIndex((f) => f.name === "stats");
  expect(statsCol).toBeGreaterThanOrEqual(0);

  for (let i = 0; i < rs.length; i++) {
    expect(s.getNested(i, statsCol, ["minValues", "id"])).toBe(
      rs[i].stats.minValues.id,
    );
    expect(s.getNested(i, statsCol, ["maxValues", "id"])).toBe(
      rs[i].stats.maxValues.id,
    );
    expect(s.getNested(i, statsCol, ["minValues", "amount"])).toBeCloseTo(
      rs[i].stats.minValues.amount,
    );
  }
});
