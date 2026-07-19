// The editor tab strip: one chip per open file, with a dirty/saving indicator
// and a close button. Click selects; drag reorders. State comes from the session
// reducer (OpenTab[]), so the strip re-renders on status changes without
// touching Monaco.

import { cn } from "@open-lakehouse/ui-kit";
import { Loader2, X } from "lucide-react";
import { useState } from "react";
import { useEditorSession } from "./EditorSessionContext";
import type { OpenTab } from "./sessionReducer";

export function TabStrip() {
  const { tabs, activeId, activate, close, reorder } = useEditorSession();
  const [dragIndex, setDragIndex] = useState<number | null>(null);

  if (tabs.length === 0) return null;

  return (
    <div className="flex items-stretch overflow-x-auto border-b bg-sidebar">
      {tabs.map((tab, index) => (
        <Tab
          key={tab.id}
          tab={tab}
          active={tab.id === activeId}
          onSelect={() => activate(tab.id)}
          onClose={() => void close(tab.id)}
          draggable
          onDragStart={() => setDragIndex(index)}
          onDragOver={(e) => e.preventDefault()}
          onDrop={() => {
            if (dragIndex !== null && dragIndex !== index)
              reorder(dragIndex, index);
            setDragIndex(null);
          }}
        />
      ))}
    </div>
  );
}

function Tab({
  tab,
  active,
  onSelect,
  onClose,
  ...drag
}: {
  tab: OpenTab;
  active: boolean;
  onSelect: () => void;
  onClose: () => void;
  draggable?: boolean;
  onDragStart?: () => void;
  onDragOver?: (e: React.DragEvent) => void;
  onDrop?: () => void;
}) {
  const dirty = tab.saveStatus === "dirty";
  const saving = tab.saveStatus === "saving";
  const errored = tab.saveStatus === "error";

  return (
    <div
      {...drag}
      className={cn(
        "group flex max-w-[12rem] shrink-0 items-center gap-1.5 border-r px-3 py-1.5 text-sm",
        active
          ? "bg-background text-foreground"
          : "text-muted-foreground hover:bg-accent/50",
      )}
      title={errored ? tab.error : tab.path}
    >
      <button
        type="button"
        onClick={onSelect}
        className="flex min-w-0 items-center gap-1.5"
      >
        <span className={cn("truncate", errored && "text-destructive")}>
          {tab.name}
        </span>
      </button>
      {/* Status / close affordance: spinner while saving, dot when dirty,
          close (×) otherwise — and × on hover even when dirty. */}
      <span className="flex h-4 w-4 shrink-0 items-center justify-center">
        {saving ? (
          <Loader2 className="h-3 w-3 animate-spin text-muted-foreground" />
        ) : (
          <>
            <button
              type="button"
              aria-label={`Close ${tab.name}`}
              onClick={onClose}
              className={cn(
                "hidden h-4 w-4 items-center justify-center rounded hover:bg-accent group-hover:flex",
                !dirty && !errored && "flex",
              )}
            >
              <X className="h-3 w-3" />
            </button>
            {(dirty || errored) && (
              <span
                className={cn(
                  "h-2 w-2 rounded-full group-hover:hidden",
                  errored ? "bg-destructive" : "bg-foreground/70",
                )}
              />
            )}
          </>
        )}
      </span>
    </div>
  );
}
