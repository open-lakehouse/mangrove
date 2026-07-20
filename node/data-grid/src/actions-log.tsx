import { cn } from "@open-lakehouse/ui-kit";
import { useVirtualizer } from "@tanstack/react-virtual";
import { useCallback, useLayoutEffect, useMemo, useState } from "react";
import { ActionRow } from "./action-row";
import {
  detectActionSlot,
  resolveSlotColumns,
  SLOT_SPECS,
} from "./lib/actionSlots";
import type { ArrowResultStore } from "./lib/arrowResultStore";

// The rich Delta-log actions view. Replaces the flat DataGrid (which shows 5/6
// null cells per row) for the `actions` log surface: one console-style row per
// reconciled action, color+glyph+label per action type, expandable for full
// detail. Virtualized like DataGrid so very long logs stay at 60fps — the
// populated slot and its fields are read (zero-copy) ONLY for rows the
// virtualizer renders, never scanned up front. Rows have dynamic height
// (collapsed 30px, expanded taller), so the virtualizer measures each element.

interface ActionsLogProps {
  store: ArrowResultStore;
  /** Bumped as chunks arrive; re-renders as rows stream in. */
  version: number;
  /** True while streaming (kept for API parity with DataGrid; unused here). */
  running?: boolean;
  className?: string;
}

const COLLAPSED_ROW_HEIGHT = 30;

export function ActionsLog({ store, version, className }: ActionsLogProps) {
  // See DataGrid: a state-backed callback ref re-renders the instant the scroll
  // element mounts, so the virtualizer gets a live element to measure (a plain
  // ref wouldn't trigger the render the virtualizer needs to emit items).
  const [scrollEl, setScrollEl] = useState<HTMLDivElement | null>(null);
  const [expanded, setExpanded] = useState<ReadonlySet<number>>(
    () => new Set(),
  );

  // Slot columns resolve from the schema once (re-derived on version in case the
  // schema arrives with the first chunk). store is mutated in place.
  // biome-ignore lint/correctness/useExhaustiveDependencies: version is the re-read signal
  const slotColumns = useMemo(
    () => resolveSlotColumns(store),
    [store, version],
  );
  const rowCount = store.rowCount;

  const toggle = useCallback((row: number) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(row)) next.delete(row);
      else next.add(row);
      return next;
    });
  }, []);

  const rowVirtualizer = useVirtualizer({
    count: rowCount,
    getScrollElement: () => scrollEl,
    estimateSize: () => COLLAPSED_ROW_HEIGHT,
    overscan: 12,
    // Measure each rendered row so expanded rows (taller) size correctly.
    measureElement: (el) => el.getBoundingClientRect().height,
  });

  // Re-measure when rows arrive, the scroll element attaches, or an expand
  // toggles — mirrors DataGrid's first-paint fix (measure before paint).
  // biome-ignore lint/correctness/useExhaustiveDependencies: scrollEl/expanded/version are re-run triggers
  useLayoutEffect(() => {
    rowVirtualizer.measure();
  }, [rowVirtualizer, rowCount, scrollEl, expanded, version]);

  const virtualRows = rowVirtualizer.getVirtualItems();

  return (
    <div
      ref={setScrollEl}
      className={cn(
        "min-h-0 flex-1 overflow-auto rounded border bg-background",
        className,
      )}
    >
      <div
        className="relative w-full"
        style={{ height: rowVirtualizer.getTotalSize() }}
      >
        {virtualRows.map((vRow) => {
          const hit = detectActionSlot(store, slotColumns, vRow.index);
          return (
            <div
              key={vRow.key}
              data-index={vRow.index}
              ref={rowVirtualizer.measureElement}
              className="absolute left-0 top-0 w-full"
              style={{ transform: `translateY(${vRow.start}px)` }}
            >
              {hit ? (
                <ActionRow
                  store={store}
                  spec={SLOT_SPECS[hit.slot]}
                  colIndex={hit.colIndex}
                  row={vRow.index}
                  expanded={expanded.has(vRow.index)}
                  onToggle={toggle}
                />
              ) : (
                // A well-formed reconciled action stream always has one slot;
                // render an unobtrusive placeholder rather than crash if not.
                <div className="border-b px-9 py-1.5 font-mono text-xs text-muted-foreground/60">
                  (no action)
                </div>
              )}
            </div>
          );
        })}
      </div>

      {rowCount === 0 && (
        <div className="px-3 py-6 text-center text-sm text-muted-foreground">
          No actions
        </div>
      )}
    </div>
  );
}
