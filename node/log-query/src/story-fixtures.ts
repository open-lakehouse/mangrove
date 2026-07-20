// Self-contained Arrow-IPC fixtures for the Delta-log view stories (ActionsLog,
// MinMaxView). These views read zero-copy from a data-grid `ArrowResultStore`
// built from real Arrow IPC — the same bytes the streaming LogQueryService
// delivers — so the showcase data is real Arrow tables serialized to IPC and
// decoded back through the store's live path.
//
// The shapes here are Delta-log-specific (six action slots; the reconciled
// stats struct), which is why they live in this package rather than in
// data-grid's generic story fixtures.

import { ArrowResultStore } from "@open-lakehouse/data-grid";
import { type Table, tableFromJSON, tableToIPC } from "apache-arrow";

/** Serialize an Arrow table to a self-contained IPC stream (schema + batch). */
function tableToIpc(table: Table): Uint8Array {
  return tableToIPC(table, "stream");
}

/** Build an {@link ArrowResultStore} from one or more IPC chunks — the shape the
 *  streaming LogQueryService produces, ready to hand to a Delta-log view. */
export function storeFromIpc(...chunks: Uint8Array[]): ArrowResultStore {
  const store = new ArrowResultStore();
  for (const chunk of chunks) store.append(chunk);
  return store;
}

// --- Actions-log fixture (six nullable slots, one non-null per row) ----------
// Mirrors the reconciled action stream: seed one row per slot first so
// tableFromJSON settles every struct schema, then a realistic mix.

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

/** IPC for the actions-log fixture. */
export const actionsIpc = tableToIpc(tableFromJSON(actionRows()));

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
