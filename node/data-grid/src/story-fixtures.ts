// Self-contained Arrow-IPC fixtures for the DataGrid stories. The grid reads
// zero-copy from an ArrowResultStore built from real Arrow IPC — the same bytes
// the streaming QueryService would deliver — so these are real Arrow tables
// serialized to IPC and decoded back through the store's live path.
//
// This package cannot depend on the app's fixture world (that world depends on
// this package), so the grid's own showcase data lives here.

import {
  type Table,
  tableFromArrays,
  tableFromJSON,
  tableToIPC,
} from "apache-arrow";
import { ArrowResultStore } from "./index";

/** Serialize an Arrow table to a self-contained IPC stream (schema + batch). */
function tableToIpc(table: Table): Uint8Array {
  return tableToIPC(table, "stream");
}

/** Build an {@link ArrowResultStore} from one or more IPC chunks — the same
 *  shape the streaming QueryService produces, ready to hand to <DataGrid>. */
export function storeFromIpc(...chunks: Uint8Array[]): ArrowResultStore {
  const store = new ArrowResultStore();
  for (const chunk of chunks) store.append(chunk);
  return store;
}

/** A small, mixed-type result — the everyday case for the grid showcase. */
const topCustomersTable: Table = tableFromArrays({
  customer_id: Int32Array.from([1001, 1002, 1003, 1004, 1005]),
  full_name: [
    "Ada Lovelace",
    "Alan Turing",
    "Grace Hopper",
    "Edsger Dijkstra",
    "Barbara Liskov",
  ],
  orders: Int32Array.from([42, 37, 51, 29, 64]),
  revenue_usd: Float64Array.from([12450.5, 9870.0, 15320.75, 7600.0, 20110.25]),
});

/** An empty result (valid schema, zero rows) — exercises the empty state. */
const emptyTable: Table = tableFromArrays({
  customer_id: Int32Array.from([]),
  full_name: [] as string[],
  revenue_usd: Float64Array.from([]),
});

/** A wider result with more columns — exercises horizontal scroll / sizing. */
const tripsTable: Table = tableFromArrays({
  vendor_id: Int32Array.from([1, 2, 1, 2, 1, 2]),
  passenger_count: Int32Array.from([1, 2, 1, 3, 1, 4]),
  trip_distance: Float64Array.from([1.2, 3.8, 0.9, 7.4, 2.1, 5.5]),
  fare_amount: Float64Array.from([7.5, 14.0, 6.0, 28.5, 9.0, 22.25]),
  tip_amount: Float64Array.from([1.5, 2.0, 0.0, 5.0, 1.8, 4.0]),
  payment_type: ["card", "cash", "card", "card", "cash", "card"],
});

/** IPC byte streams for each canned table (the wire form). */
export const topCustomersIpc = tableToIpc(topCustomersTable);
export const emptyIpc = tableToIpc(emptyTable);
export const tripsIpc = tableToIpc(tripsTable);

// --- Actions-log fixture (six nullable slots, one non-null per row) ----------
// data-grid is a leaf package and cannot depend on @open-lakehouse/log-query's
// fixtures (that dependency runs the other way), so the ActionsLog showcase data
// lives here. The shape mirrors the reconciled action stream: seed one row per
// slot first so tableFromJSON settles every struct schema, then a realistic mix.

const EMPTY_SLOTS = {
  add: null as {
    path: string;
    size: bigint;
    stats_parsed: { numRecords: bigint };
  } | null,
  remove: null as { path: string; deletionTimestamp: bigint } | null,
  metaData: null as { id: string; name: string | null } | null,
  protocol: null as {
    minReaderVersion: number;
    minWriterVersion: number;
  } | null,
  domainMetadata: null as { domain: string; removed: boolean } | null,
  txn: null as { appId: string; version: bigint } | null,
};

function actionRows(count = 30): Record<string, unknown>[] {
  const rows: (typeof EMPTY_SLOTS)[] = [
    {
      ...EMPTY_SLOTS,
      add: {
        path: "part-00000.parquet",
        size: 48_000n,
        stats_parsed: { numRecords: 1_000n },
      },
    },
    {
      ...EMPTY_SLOTS,
      remove: {
        path: "part-00001.parquet",
        deletionTimestamp: 1_714_000_000_000n,
      },
    },
    { ...EMPTY_SLOTS, metaData: { id: "b1e6-table", name: "events" } },
    { ...EMPTY_SLOTS, protocol: { minReaderVersion: 3, minWriterVersion: 7 } },
    {
      ...EMPTY_SLOTS,
      domainMetadata: { domain: "delta.liquid", removed: false },
    },
    { ...EMPTY_SLOTS, txn: { appId: "streaming-app", version: 12n } },
  ];
  for (let i = rows.length; i < count; i++) {
    if (i % 3 === 0) {
      rows.push({
        ...EMPTY_SLOTS,
        remove: {
          path: `part-${String(i).padStart(5, "0")}.parquet`,
          deletionTimestamp: BigInt(1_714_000_000_000 + i * 1000),
        },
      });
    } else {
      rows.push({
        ...EMPTY_SLOTS,
        add: {
          path: `part-${String(i).padStart(5, "0")}.parquet`,
          size: BigInt(48_000 + i * 900),
          stats_parsed: { numRecords: BigInt(1_000 + i * 137) },
        },
      });
    }
  }
  return rows as unknown as Record<string, unknown>[];
}

const actionsTable: Table = tableFromJSON(actionRows());

/** IPC for the actions-log fixture. */
export const actionsIpc = tableToIpc(actionsTable);

// --- Reconciled-with-stats fixture (for the min/max boxes) -------------------
// A handful of files with a nested `stats.minValues`/`stats.maxValues` struct on
// numeric columns, ranges overlapping so the 2D boxes visibly overlap.

interface StatsRow {
  path: string;
  size: bigint;
  stats: {
    numRecords: bigint;
    minValues: { id: number; amount: number };
    maxValues: { id: number; amount: number };
  };
}

function statsRows(count = 24): Record<string, unknown>[] {
  const rows: StatsRow[] = [];
  for (let i = 0; i < count; i++) {
    const idMin = i * 400;
    const idMax = idMin + 1000;
    const amountMin = 0.5 + (i % 6) * 12.5;
    const amountMax = amountMin + 140 + (i % 5) * 20;
    rows.push({
      path: `part-${String(i).padStart(5, "0")}.parquet`,
      size: BigInt(48_000 + i * 900),
      stats: {
        numRecords: BigInt(1_000 + i * 137),
        minValues: { id: idMin, amount: Number(amountMin.toFixed(2)) },
        maxValues: { id: idMax, amount: Number(amountMax.toFixed(2)) },
      },
    });
  }
  return rows as unknown as Record<string, unknown>[];
}

/** IPC for the reconciled-with-stats fixture (min/max boxes). */
export const reconciledStatsIpc = tableToIpc(tableFromJSON(statsRows()));
