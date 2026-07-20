import { useMemo, useState } from "react";
import type { ArrowResultStore } from "./lib/arrowResultStore";
import { enumerateMinMaxAxes, type MinMaxAxis } from "./lib/minMaxAxes";
import { MinMaxBoxes } from "./min-max-boxes";
import { type BoxDim, MinMaxControls } from "./min-max-controls";

// The complete min/max-box view: enumerates the orderable axes from the store
// schema, holds the dimension + axis-selection state, and renders the controls
// over the SVG box plot. Self-contained so DeltaLogTab can drop it in for the
// reconciled "Boxes" sub-view.

interface MinMaxViewProps {
  store: ArrowResultStore;
  version: number;
  className?: string;
}

export function MinMaxView({ store, version, className }: MinMaxViewProps) {
  // Orderable axes only change when the schema arrives (first chunk); re-derive
  // on version so the first-chunk schema is picked up. store mutates in place.
  // biome-ignore lint/correctness/useExhaustiveDependencies: version is the re-read signal
  const available = useMemo(
    () => enumerateMinMaxAxes(store.schema?.fields),
    [store, version],
  );

  const [dim, setDim] = useState<BoxDim>("2d");
  // Default selections: first two axes (or the first twice as a fallback).
  const [selected, setSelected] = useState<string[]>([]);

  // Seed / repair selection whenever the available set changes.
  const effectiveSelected = useMemo(() => {
    if (available.length === 0) return [];
    const names = available.map((a) => a.name);
    const x = names.includes(selected[0]) ? selected[0] : names[0];
    const y = names.includes(selected[1])
      ? selected[1]
      : (names[1] ?? names[0]);
    return [x, y];
  }, [available, selected]);

  const onSelect = (index: number, name: string) => {
    const next = [...effectiveSelected];
    next[index] = name;
    setSelected(next);
  };

  if (available.length === 0) {
    return (
      <div
        className={
          "flex min-h-0 flex-1 items-center justify-center rounded border bg-background px-3 py-10 text-center text-sm text-muted-foreground"
        }
      >
        No orderable min/max stats in this table — nothing to plot.
      </div>
    );
  }

  const axes: MinMaxAxis[] = resolveAxes(
    available,
    dim === "2d" ? effectiveSelected : effectiveSelected.slice(0, 1),
  );

  return (
    <div className={className}>
      <MinMaxControls
        available={available}
        dim={dim}
        onDim={setDim}
        selected={effectiveSelected}
        onSelect={onSelect}
      />
      <MinMaxBoxes store={store} version={version} axes={axes} />
    </div>
  );
}

/** Map selected names to their axis metadata, dropping any that vanished. */
function resolveAxes(available: MinMaxAxis[], names: string[]): MinMaxAxis[] {
  const byName = new Map(available.map((a) => [a.name, a] as const));
  const out: MinMaxAxis[] = [];
  for (const n of names) {
    const a = byName.get(n);
    if (a) out.push(a);
  }
  return out;
}
