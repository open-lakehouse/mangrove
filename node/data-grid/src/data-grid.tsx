import { cn } from "@open-lakehouse/ui-kit";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useLayoutEffect, useState } from "react";
import { DataGridCell } from "./data-grid-cell";
import { DataGridHeader } from "./data-grid-header";
import type { ArrowResultStore } from "./lib/arrowResultStore";
import { useArrowTable } from "./lib/useArrowTable";

interface DataGridProps {
  store: ArrowResultStore;
  /** Bumped as chunks arrive; re-renders the grid as rows stream in. */
  version: number;
  /** True while the stream is open (gates sorting per the stream-end policy). */
  running: boolean;
  className?: string;
}

const ROW_HEIGHT = 30;

/**
 * Virtualized, type-aware result grid. Rows are windowed with TanStack Virtual
 * so 100k+ rows render at 60fps; only visible rows + overscan exist in the DOM.
 * Cells are read zero-copy from the Arrow store and rendered by the type-aware
 * formatter registry. Layout is CSS grid (not <table>) so columns stay aligned
 * under absolute-positioned virtual rows.
 */
export function DataGrid({
  store,
  version,
  running,
  className,
}: DataGridProps) {
  // Track the scroll element in state (via a callback ref) rather than a plain
  // ref. A ref is null on the mount render and populating it does NOT trigger a
  // re-render — so the virtualizer, first constructed with a null scroll element
  // (and often count 0, since the grid frequently mounts only once the first
  // chunk lands and the stream has already ended), never gets the render tick it
  // needs to emit virtual items once the element attaches. That left the grid
  // blank until an unrelated re-render (switching tables, toggling the log mode)
  // happened to flush it. A state-backed callback ref re-renders the instant the
  // node mounts, giving the virtualizer a live scroll element to measure.
  const [scrollEl, setScrollEl] = useState<HTMLDivElement | null>(null);
  const {
    columns,
    rowCount,
    columnSizing,
    getCell,
    headerGroups,
    sortEnabled,
  } = useArrowTable(store, version, running);

  const rowVirtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollEl,
    estimateSize: () => ROW_HEIGHT,
    overscan: 12,
  });

  // Re-measure whenever the row count grows or the scroll element (re)attaches.
  // Covers the case where rows arrive after the virtualizer first measured an
  // empty / detached element: `measure()` invalidates its size cache so the next
  // paint windows the now-present rows. `useLayoutEffect` runs before the browser
  // paints, so there is no visible empty flash.
  // biome-ignore lint/correctness/useExhaustiveDependencies: `scrollEl` is a re-run trigger (remeasure when the element attaches), not read in the body
  useLayoutEffect(() => {
    rowVirtualizer.measure();
  }, [rowVirtualizer, rowCount, scrollEl]);

  // Build the CSS grid column template from the (resizable) column sizes.
  const headers = headerGroups[0]?.headers ?? [];
  const template = headers
    .map((h) => `${columnSizing[h.column.id] ?? h.getSize()}px`)
    .join(" ");

  const virtualRows = rowVirtualizer.getVirtualItems();

  return (
    <div
      ref={setScrollEl}
      className={cn(
        "min-h-0 flex-1 overflow-auto rounded border bg-background",
        className,
      )}
    >
      {/* Sticky header row */}
      <div
        className="sticky top-0 z-10 grid w-max min-w-full"
        style={{ gridTemplateColumns: template }}
      >
        {headers.map((header) => {
          const col = columns.find((c) => c.name === header.column.id);
          if (!col) return null;
          return (
            <DataGridHeader
              key={header.id}
              header={header}
              column={col}
              sortEnabled={sortEnabled}
            />
          );
        })}
      </div>

      {/* Virtualized body: a spacer sized to all rows, with absolutely
          positioned visible rows translated into view. */}
      <div
        className="relative w-max min-w-full"
        style={{ height: rowVirtualizer.getTotalSize() }}
      >
        {virtualRows.map((vRow) => (
          <div
            key={vRow.key}
            className="absolute left-0 top-0 grid w-full border-b last:border-b-0 odd:bg-muted/20 hover:bg-accent/40"
            style={{
              height: vRow.size,
              transform: `translateY(${vRow.start}px)`,
              gridTemplateColumns: template,
            }}
          >
            {columns.map((col) => (
              <DataGridCell
                key={col.name}
                value={getCell(vRow.index, col.index)}
                type={col.type}
              />
            ))}
          </div>
        ))}
      </div>

      {rowCount === 0 && (
        <div className="px-3 py-6 text-center text-sm text-muted-foreground">
          No rows
        </div>
      )}
    </div>
  );
}
