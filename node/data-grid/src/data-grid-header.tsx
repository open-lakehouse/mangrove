import { Badge, cn } from "@open-lakehouse/ui-kit";
import type { Header } from "@tanstack/react-table";
import { ChevronDown, ChevronsUpDown, ChevronUp } from "lucide-react";
import { arrowTypeLabel } from "./lib/arrowTypeLabel";
import type { ArrowColumnMeta } from "./lib/useArrowTable";

interface DataGridHeaderProps {
  // biome-ignore lint/suspicious/noExplicitAny: TanStack Header is generic over row data we don't model
  header: Header<any, unknown>;
  column: ArrowColumnMeta;
  sortEnabled: boolean;
}

/**
 * One header cell: column name, an Arrow-type badge, a sort toggle (when sorting
 * is enabled), and a drag handle for resizing. Sorting and resizing are pure
 * state changes — neither touches the underlying Arrow data.
 */
export function DataGridHeader({
  header,
  column,
  sortEnabled,
}: DataGridHeaderProps) {
  const sorted = header.column.getIsSorted();
  return (
    <div className="relative flex items-center gap-1.5 border-b border-r bg-muted px-3 py-2">
      <button
        type="button"
        disabled={!sortEnabled}
        onClick={
          sortEnabled ? header.column.getToggleSortingHandler() : undefined
        }
        className={cn(
          "flex min-w-0 items-center gap-1.5 text-left font-medium",
          sortEnabled && "cursor-pointer hover:text-foreground",
          !sortEnabled && "cursor-default",
        )}
        title={column.name}
      >
        <span className="truncate">{column.name}</span>
        {sortEnabled &&
          (sorted === "asc" ? (
            <ChevronUp className="h-3 w-3 shrink-0" />
          ) : sorted === "desc" ? (
            <ChevronDown className="h-3 w-3 shrink-0" />
          ) : (
            <ChevronsUpDown className="h-3 w-3 shrink-0 opacity-40" />
          ))}
      </button>
      <Badge variant="outline" className="shrink-0 normal-case tracking-normal">
        {arrowTypeLabel(column.type)}
      </Badge>
      {header.column.getCanResize() && (
        // Drag handle on the right edge; TanStack tracks the size delta.
        // biome-ignore lint/a11y/noStaticElementInteractions: resize handle is a pointer-only affordance
        <div
          onMouseDown={header.getResizeHandler()}
          onTouchStart={header.getResizeHandler()}
          className={cn(
            "absolute right-0 top-0 h-full w-1 cursor-col-resize select-none touch-none",
            "hover:bg-primary/40",
            header.column.getIsResizing() && "bg-primary",
          )}
        />
      )}
    </div>
  );
}
