import { cn } from "@open-lakehouse/ui-kit";
import { type CSSProperties, useCallback, useEffect, useRef } from "react";

import { CatalogTree } from "./CatalogTree";
import { DetailPane } from "./DetailPane";
import { CatalogDialogsProvider } from "./dialogs";
import { ExpansionProvider, nodeId, useExpansion } from "./ExpansionContext";
import { useCatalogSelection } from "./selection";
import { isObjectKind } from "./types";
import { useSidebarWidth } from "./useSidebarWidth";

export function CatalogExplorer() {
  return (
    <ExpansionProvider>
      <CatalogDialogsProvider>
        <ExplorerLayout />
      </CatalogDialogsProvider>
    </ExpansionProvider>
  );
}

function ExplorerLayout() {
  useExpandToSelection();
  const { width, setWidth, clamp, min, max, isDragging, setDragging } =
    useSidebarWidth();
  const containerRef = useRef<HTMLDivElement>(null);

  const startDrag = useCallback(
    (e: React.PointerEvent) => {
      e.preventDefault();
      const container = containerRef.current;
      if (!container) return;
      const { left } = container.getBoundingClientRect();
      setDragging(true);

      const onMove = (ev: PointerEvent) => setWidth(clamp(ev.clientX - left));
      const onUp = () => {
        setDragging(false);
        window.removeEventListener("pointermove", onMove);
        window.removeEventListener("pointerup", onUp);
      };
      window.addEventListener("pointermove", onMove);
      window.addEventListener("pointerup", onUp);
    },
    [clamp, setWidth, setDragging],
  );

  const onHandleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      const step = e.shiftKey ? 48 : 16;
      if (e.key === "ArrowLeft") {
        e.preventDefault();
        setWidth((w) => clamp(w - step));
      } else if (e.key === "ArrowRight") {
        e.preventDefault();
        setWidth((w) => clamp(w + step));
      }
    },
    [clamp, setWidth],
  );

  return (
    <div className="flex h-[calc(100vh-3rem)] flex-col">
      <div
        ref={containerRef}
        className={cn(
          "grid min-h-0 flex-1 grid-cols-1 overflow-hidden md:grid-cols-[var(--sidebar-w)_minmax(0,1fr)]",
          // While dragging, freeze the whole surface: no text selection and a
          // resize cursor everywhere (the pointer often leaves the thin handle).
          isDragging && "cursor-col-resize select-none",
        )}
        style={{ "--sidebar-w": `${width}px` } as CSSProperties}
      >
        <div className="relative flex min-h-0 flex-col border-r bg-sidebar">
          <CatalogTree />
          {/* Resize handle straddling the column's right border. Hidden on the
              single-column mobile layout, where the panes stack. */}
          {/* biome-ignore lint/a11y/useSemanticElements: the ARIA window-splitter
              pattern needs an interactive, focusable role="separator" (with
              aria-valuenow/keyboard control); <hr> can't carry those handlers. */}
          <div
            role="separator"
            aria-orientation="vertical"
            aria-label="Resize sidebar"
            aria-valuenow={Math.round(width)}
            aria-valuemin={min}
            aria-valuemax={max}
            tabIndex={0}
            onPointerDown={startDrag}
            onKeyDown={onHandleKeyDown}
            className={cn(
              "absolute inset-y-0 right-0 z-20 hidden w-1.5 translate-x-1/2 cursor-col-resize transition-colors md:block",
              "hover:bg-primary/40 focus-visible:bg-primary/60 focus-visible:outline-none",
              isDragging && "bg-primary/60",
            )}
          />
        </div>
        <DetailPane />
      </div>
    </div>
  );
}

/**
 * Expand the *ancestors* of the selected node so a deep-linked object becomes
 * visible. We never expand the selected node itself — selecting a catalog or
 * schema must not toggle it (that's the chevron's job).
 */
function useExpandToSelection() {
  const { selection } = useCatalogSelection();
  const { expand } = useExpansion();

  // Depend on the primitive selection fields, NOT the `selection` object:
  // it is re-created on every render (decodeSelection), so using it as a dep
  // would re-run this effect on every render and re-expand ancestors — which
  // would fight the user trying to collapse a parent while a child is selected.
  const kind = selection?.kind;
  const fullName = selection?.fullName;

  useEffect(() => {
    if (!kind || !fullName) return;
    const [catalog, schema] = fullName.split(".");
    const ids: string[] = [];
    // A schema (or deeper) needs its catalog open; a leaf also needs its schema
    // and the matching group open.
    if (catalog && schema) ids.push(nodeId.catalog(catalog));
    if (catalog && schema && isObjectKind(kind)) {
      ids.push(nodeId.schema(catalog, schema));
      ids.push(nodeId.group(catalog, schema, kind));
    }
    if (ids.length) expand(ids);
  }, [kind, fullName, expand]);
}
