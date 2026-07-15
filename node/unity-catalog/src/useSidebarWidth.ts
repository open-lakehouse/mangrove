import { useCallback, useEffect, useState } from "react";

// Draggable sidebar column width (px). Bounds roughly track the previous static
// `minmax(18rem, 24rem)` but widen the ceiling so the tree can show deep paths.
const MIN = 240;
const MAX = 640;
const DEFAULT = 288; // 18rem
const STORAGE_KEY = "uc.explorer.sidebar-width";

function readStored(): number {
  if (typeof window === "undefined") return DEFAULT;
  const stored = Number(window.localStorage.getItem(STORAGE_KEY));
  return Number.isFinite(stored) && stored > 0 ? stored : DEFAULT;
}

/**
 * Persisted, clamped width for the catalog explorer's sidebar column, plus the
 * transient drag flag. The width survives reloads via localStorage; `clamp`
 * keeps both drag and keyboard adjustments within [MIN, MAX].
 */
export function useSidebarWidth() {
  const clamp = useCallback(
    (px: number) => Math.min(MAX, Math.max(MIN, px)),
    [],
  );
  const [width, setWidth] = useState(() => clamp(readStored()));
  const [isDragging, setDragging] = useState(false);

  useEffect(() => {
    window.localStorage.setItem(STORAGE_KEY, String(Math.round(width)));
  }, [width]);

  return {
    width,
    setWidth,
    clamp,
    min: MIN,
    max: MAX,
    isDragging,
    setDragging,
  };
}
