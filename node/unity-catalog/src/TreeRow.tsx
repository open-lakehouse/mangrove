import { Button, cn } from "@open-lakehouse/ui-kit";
import { ChevronDown, ChevronRight, Plus } from "lucide-react";
import type { ReactNode } from "react";

const INDENT_REM = 0.875;

export function rowPadding(depth: number) {
  return `${depth * INDENT_REM + 0.5}rem`;
}

export function TreeRow({
  depth,
  icon,
  label,
  expandable,
  open,
  selected,
  action,
  onToggle,
  onSelect,
  onMouseEnter,
}: {
  depth: number;
  icon: ReactNode;
  label?: string;
  expandable?: boolean;
  open?: boolean;
  selected?: boolean;
  action?: ReactNode;
  onToggle?: () => void;
  /** When provided, clicking the row body selects the node. */
  onSelect?: () => void;
  onMouseEnter?: () => void;
}) {
  // Body click selects when the node is selectable; otherwise (group containers)
  // it falls back to toggling expansion so the whole row stays interactive.
  const onBodyClick = onSelect ?? (expandable ? onToggle : undefined);

  return (
    <div
      className={cn(
        "group flex items-center rounded pr-1 hover:bg-accent",
        selected && "bg-accent text-accent-foreground",
      )}
    >
      <div
        className="flex min-w-0 flex-1 items-center"
        style={{ paddingLeft: rowPadding(depth) }}
      >
        {expandable ? (
          <button
            type="button"
            aria-label={open ? "Collapse" : "Expand"}
            onClick={(e) => {
              e.stopPropagation();
              onToggle?.();
            }}
            className="flex h-6 w-4 shrink-0 items-center justify-center text-muted-foreground hover:text-foreground"
          >
            {open ? (
              <ChevronDown className="h-3.5 w-3.5" />
            ) : (
              <ChevronRight className="h-3.5 w-3.5" />
            )}
          </button>
        ) : (
          <span className="h-6 w-4 shrink-0" />
        )}
        <button
          type="button"
          onClick={onBodyClick}
          onMouseEnter={onMouseEnter}
          className="flex min-w-0 flex-1 items-center gap-1.5 px-1 py-1.5 text-left text-sm"
        >
          {icon}
          <span className="min-w-0 flex-1 truncate font-medium">{label}</span>
        </button>
      </div>
      {action && (
        <span className="flex items-center opacity-0 transition-opacity focus-within:opacity-100 group-hover:opacity-100">
          {action}
        </span>
      )}
    </div>
  );
}

export function CreateAction({
  title,
  onClick,
}: {
  title: string;
  onClick: () => void;
}) {
  return (
    <Button
      type="button"
      variant="ghost"
      size="sm"
      title={title}
      aria-label={title}
      className="h-6 w-6 p-0"
      onClick={onClick}
    >
      <Plus className="h-3.5 w-3.5" />
    </Button>
  );
}

export function ListStates({
  depth = 0,
  isLoading,
  error,
  isEmpty,
  hasNextPage,
  isFetchingNextPage,
  onLoadMore,
  children,
}: {
  depth?: number;
  isLoading: boolean;
  error: unknown;
  isEmpty: boolean;
  hasNextPage?: boolean;
  isFetchingNextPage?: boolean;
  onLoadMore?: () => void;
  children: ReactNode;
}) {
  const pad = { paddingLeft: `${depth * INDENT_REM + 1.75}rem` } as const;
  if (isLoading) {
    return (
      <div style={pad} className="py-1.5 text-sm text-muted-foreground">
        <span className="inline-block h-3.5 w-3.5 animate-spin rounded-full border-2 border-muted border-t-primary align-middle" />
      </div>
    );
  }
  if (error) {
    return (
      <div style={pad} className="py-1.5 text-sm text-destructive">
        Failed to load.
      </div>
    );
  }
  if (isEmpty) {
    return (
      <div style={pad} className="py-1.5 text-sm text-muted-foreground">
        Empty.
      </div>
    );
  }
  return (
    <>
      {children}
      {hasNextPage && (
        <Button
          variant="ghost"
          size="sm"
          style={pad}
          className="w-full justify-start text-xs"
          disabled={isFetchingNextPage}
          onClick={onLoadMore}
        >
          {isFetchingNextPage ? "Loading…" : "Load more"}
        </Button>
      )}
    </>
  );
}
