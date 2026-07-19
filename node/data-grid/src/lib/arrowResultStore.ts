import {
  type Schema,
  type Table,
  tableFromIPC,
  type Vector,
} from "apache-arrow";
import { arrowTypeLabel } from "./arrowTypeLabel";

/** A read-only summary of what an {@link ArrowResultStore} currently holds.
 *  Cheap to produce (no row scan) — for memory accounting and a "what's in the
 *  store" affordance. */
export interface ArrowStoreInfo {
  /** Per-column name + human-readable type label (e.g. "int64"). */
  schema: { name: string; type: string }[];
  rowCount: number;
  columnCount: number;
  /** Number of appended Arrow IPC chunks (record batches). */
  batchCount: number;
  /** Approximate in-memory footprint: sum of each batch's backing buffer
   *  byte lengths. Accumulated on append, so reading it is O(1). */
  byteLength: number;
}

// Holds streamed query results in Arrow form and serves individual cells with
// zero-copy access — the opposite of eagerly materializing every row into a
// plain JS object (which copies every value out of the Arrow buffers and loses
// the columnar / logical-type advantages).
//
// Each streamed chunk is one self-contained Arrow IPC stream (one record batch),
// decoded once with `tableFromIPC` and kept as-is. We record each batch's global
// row offset so a flat row index resolves to (batch, localRow) by binary search,
// then `Vector.get(localRow)` reads the value with no copy. Appending is O(1) —
// no per-chunk re-concatenation.

interface BatchEntry {
  table: Table;
  /** Global index of this batch's first row. */
  startRow: number;
  /** Number of rows in this batch (`table.numRows`). */
  length: number;
}

export class ArrowResultStore {
  /** Schema from the first chunk; null until the first `append`. */
  schema: Schema | null = null;

  private batches: BatchEntry[] = [];
  private total = 0;
  private bytes = 0;
  // Cache column vectors per batch so repeated cell reads in the same batch
  // don't re-call `getChildAt`. Keyed by batch index, then column index.
  private vectorCache = new Map<number, Map<number, Vector | null>>();

  /** Total rows accumulated so far. */
  get rowCount(): number {
    return this.total;
  }

  /** Number of result columns (0 until the first chunk arrives). */
  get columnCount(): number {
    return this.schema?.fields.length ?? 0;
  }

  /** Decode and append one Arrow IPC chunk. Sets `schema` on the first chunk. */
  append(ipc: Uint8Array): void {
    const table = tableFromIPC(ipc);
    if (!this.schema) this.schema = table.schema;
    this.batches.push({ table, startRow: this.total, length: table.numRows });
    this.total += table.numRows;
    // Accumulate the backing-buffer footprint now so `inspect().byteLength` is
    // O(1). `Data.byteLength` recursively sums each record batch's buffers.
    for (const batch of table.batches) this.bytes += batch.data.byteLength;
  }

  /**
   * Drop all accumulated rows and reset to the pre-`append` state. Used when a
   * run restarts (e.g. a React StrictMode remount re-invokes the producer) so a
   * fresh stream can't double-append onto a store that already holds the same
   * rows. Keeps the store instance stable for `useSyncExternalStore` consumers.
   */
  reset(): void {
    this.schema = null;
    this.batches = [];
    this.total = 0;
    this.bytes = 0;
    this.vectorCache.clear();
  }

  /**
   * A cheap, read-only summary of what the store currently holds: schema, row /
   * column / batch counts, and approximate in-memory footprint. No row scan and
   * no copy — safe to call on every render or for memory accounting across
   * sessions.
   */
  inspect(): ArrowStoreInfo {
    const fields = this.schema?.fields ?? [];
    return {
      schema: fields.map((f) => ({
        name: f.name,
        type: arrowTypeLabel(f.type),
      })),
      rowCount: this.total,
      columnCount: fields.length,
      batchCount: this.batches.length,
      byteLength: this.bytes,
    };
  }

  /**
   * Read one cell by global row and column index, zero-copy. Returns `null` for
   * null slots (and for out-of-range indices); empty strings come back as `""`,
   * so null and empty string stay distinguishable.
   */
  getCell(globalRow: number, colIndex: number): unknown {
    const batchIndex = this.locate(globalRow);
    if (batchIndex < 0) return null;
    const entry = this.batches[batchIndex];
    const vec = this.columnVector(batchIndex, colIndex);
    return vec ? vec.get(globalRow - entry.startRow) : null;
  }

  /** Resolve a batch's column Vector, memoized. */
  private columnVector(batchIndex: number, colIndex: number): Vector | null {
    let cols = this.vectorCache.get(batchIndex);
    if (!cols) {
      cols = new Map();
      this.vectorCache.set(batchIndex, cols);
    }
    let vec = cols.get(colIndex);
    if (vec === undefined) {
      vec = this.batches[batchIndex].table.getChildAt(colIndex) ?? null;
      cols.set(colIndex, vec);
    }
    return vec;
  }

  /** Binary-search the batch index owning `globalRow`, or -1 if out of range. */
  private locate(globalRow: number): number {
    if (globalRow < 0 || globalRow >= this.total) return -1;
    let lo = 0;
    let hi = this.batches.length - 1;
    while (lo <= hi) {
      const mid = (lo + hi) >>> 1;
      const entry = this.batches[mid];
      if (globalRow < entry.startRow) hi = mid - 1;
      else if (globalRow >= entry.startRow + entry.length) lo = mid + 1;
      else return mid;
    }
    return -1;
  }
}
