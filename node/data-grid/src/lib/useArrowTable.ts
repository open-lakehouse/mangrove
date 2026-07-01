import {
  type ColumnSizingState,
  getCoreRowModel,
  type SortingState,
  useReactTable,
} from "@tanstack/react-table";
import type { DataType, Type } from "apache-arrow";
import { useMemo, useState } from "react";
import type { ArrowResultStore } from "./arrowResultStore";
import { compareCells } from "./sortValues";

/** Metadata for one result column, derived from the Arrow schema. */
export interface ArrowColumnMeta {
  name: string;
  /** Column index into the Arrow record batches. */
  index: number;
  type: DataType;
  typeId: Type;
}

export interface UseArrowTableResult {
  columns: ArrowColumnMeta[];
  rowCount: number;
  /** Per-column pixel widths (from resizing); falls back to a default. */
  columnSizing: ColumnSizingState;
  setColumnSizing: React.Dispatch<React.SetStateAction<ColumnSizingState>>;
  sorting: SortingState;
  setSorting: React.Dispatch<React.SetStateAction<SortingState>>;
  /** Whether sorting is currently applied (disabled while streaming). */
  sortEnabled: boolean;
  /** Read a cell by *display* row (resolves through the sort order). */
  getCell: (displayRow: number, colIndex: number) => unknown;
  /** TanStack header groups for rendering the header row. */
  headerGroups: ReturnType<ReturnType<typeof useReactTable>["getHeaderGroups"]>;
}

const DEFAULT_COLUMN_WIDTH = 160;

/**
 * Bridge an `ArrowResultStore` to TanStack Table state without ever handing the
 * table the row data: TanStack manages the column model, sorting state, and
 * column-resizing state, while row access stays zero-copy through the store.
 *
 * Sorting is applied as an index permutation (`rowOrder`) over an `Int32Array`,
 * so the Arrow data is never copied or rebuilt — only a small index array is
 * sorted. Sorting is disabled while `running` (per the stream-end sort policy);
 * the permutation is recomputed when sorting, schema, or `version` changes.
 */
export function useArrowTable(
  store: ArrowResultStore | null,
  version: number,
  running: boolean,
): UseArrowTableResult {
  const [sorting, setSorting] = useState<SortingState>([]);
  const [columnSizing, setColumnSizing] = useState<ColumnSizingState>({});

  // Columns only change on the first chunk; memoize on schema identity + version.
  // version covers the first-chunk schema arrival; store identity is stable.
  // biome-ignore lint/correctness/useExhaustiveDependencies: store is mutated in place; `version` is the re-read signal
  const columns = useMemo<ArrowColumnMeta[]>(
    () =>
      store?.schema?.fields.map((f, index) => ({
        name: f.name,
        index,
        type: f.type,
        typeId: f.type.typeId as Type,
      })) ?? [],
    [store, version],
  );

  const rowCount = store?.rowCount ?? 0;
  const sortEnabled = !running;

  // TanStack Table drives column + sort + resize STATE only — no row data.
  const table = useReactTable({
    data: EMPTY_ROWS,
    columns: useMemo(
      () =>
        columns.map((c) => ({
          id: c.name,
          accessorKey: c.name,
          size: DEFAULT_COLUMN_WIDTH,
        })),
      [columns],
    ),
    state: { sorting, columnSizing },
    onSortingChange: setSorting,
    onColumnSizingChange: setColumnSizing,
    manualSorting: true,
    enableColumnResizing: true,
    columnResizeMode: "onChange",
    getCoreRowModel: getCoreRowModel(),
  });

  // Sort permutation: display index -> global row. null === identity order.
  // biome-ignore lint/correctness/useExhaustiveDependencies: store is mutated in place; `version` is the re-sort signal
  const rowOrder = useMemo<Int32Array | null>(() => {
    if (!store || !sortEnabled || sorting.length === 0 || rowCount === 0) {
      return null;
    }
    const order = new Int32Array(rowCount);
    for (let i = 0; i < rowCount; i++) order[i] = i;
    // Resolve each sort entry to (column index, type, direction).
    const keys = sorting
      .map((s) => {
        const col = columns.find((c) => c.name === s.id);
        return col ? { index: col.index, type: col.type, desc: s.desc } : null;
      })
      .filter((k): k is NonNullable<typeof k> => k !== null);
    if (keys.length === 0) return null;

    const arr = Array.from(order);
    arr.sort((a, b) => {
      for (const k of keys) {
        const cmp = compareCells(
          store.getCell(a, k.index),
          store.getCell(b, k.index),
          k.type,
        );
        if (cmp !== 0) return k.desc ? -cmp : cmp;
      }
      return 0;
    });
    return Int32Array.from(arr);
    // version: re-sort when new rows arrive (only matters after stream end here).
  }, [store, sorting, columns, rowCount, sortEnabled, version]);

  const getCell = useMemo(
    () =>
      (displayRow: number, colIndex: number): unknown => {
        if (!store) return null;
        const globalRow = rowOrder
          ? (rowOrder[displayRow] ?? displayRow)
          : displayRow;
        return store.getCell(globalRow, colIndex);
      },
    [store, rowOrder],
  );

  return {
    columns,
    rowCount,
    columnSizing,
    setColumnSizing,
    sorting,
    setSorting,
    sortEnabled,
    getCell,
    headerGroups: table.getHeaderGroups(),
  };
}

// Stable empty array so TanStack's core row model stays trivial — we never feed
// it real rows.
const EMPTY_ROWS: never[] = [];
