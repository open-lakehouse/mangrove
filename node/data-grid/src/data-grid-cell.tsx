import { cn } from "@open-lakehouse/ui-kit";
import type { DataType } from "apache-arrow";
import { memo } from "react";
import { formatCell } from "./lib/cellFormatters";

interface DataGridCellProps {
  value: unknown;
  type: DataType;
}

/**
 * One result cell. Delegates to the type-aware formatter registry for the node
 * and alignment, then applies the layout classes. Memoized so re-renders during
 * streaming only touch cells whose value actually changed.
 */
export const DataGridCell = memo(function DataGridCell({
  value,
  type,
}: DataGridCellProps) {
  const { node, align } = formatCell(value, type);
  return (
    <div
      className={cn(
        "truncate px-3 py-1.5 font-mono text-xs tabular-nums",
        align === "right" ? "text-right" : "text-left",
      )}
      title={typeof node === "string" ? node : undefined}
    >
      {node}
    </div>
  );
});
