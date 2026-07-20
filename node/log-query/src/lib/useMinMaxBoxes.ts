// Extract per-file [min,max] intervals for the selected axes from the reconciled
// log store, zero-copy: for each axis and each batch we bind the min/max leaf
// child vectors ONCE (store.getLeafVector) and then read them per local row.
// Values are coerced to a number line via toAxisNumber (bigint→Number,
// temporal→epoch ms, decimal→scaled). Null / absent leaves become NaN so a file
// with no stats on an axis is simply not plotted on it.

import { type ArrowResultStore, toAxisNumber } from "@open-lakehouse/data-grid";
import { useMemo } from "react";
import type { MinMaxAxis } from "./minMaxAxes";
import { maxPath, minPath, STATS_COLUMN } from "./minMaxAxes";

/** Per-axis extracted intervals plus its numeric domain. */
export interface AxisData {
  axis: MinMaxAxis;
  /** `min[i]` / `max[i]` are file i's interval on this axis (NaN if absent). */
  min: Float64Array;
  max: Float64Array;
  /** Finite [lo, hi] domain across all files, or null if no finite values. */
  domain: [number, number] | null;
}

export interface MinMaxBoxes {
  /** Number of files (rows). */
  count: number;
  /** File paths for tooltips/labels (empty string if the `path` column absent). */
  paths: string[];
  /** One entry per selected axis, in the given order. */
  axes: AxisData[];
}

/**
 * Read the selected axes' per-file min/max from the store. Recomputes when the
 * store version, the selected axes, or the row count changes. `selected` is the
 * ordered list of axes to extract (1 for the number-line view, 2 for the 2D
 * view).
 */
export function useMinMaxBoxes(
  store: ArrowResultStore | null,
  version: number,
  selected: MinMaxAxis[],
): MinMaxBoxes {
  // biome-ignore lint/correctness/useExhaustiveDependencies: store mutates in place; version is the re-read signal
  return useMemo(() => {
    if (!store || store.rowCount === 0 || selected.length === 0) {
      return { count: 0, paths: [], axes: [] };
    }
    const count = store.rowCount;
    const fields = store.schema?.fields ?? [];
    const statsCol = fields.findIndex((f) => f.name === STATS_COLUMN);
    const pathCol = fields.findIndex((f) => f.name === "path");

    const paths: string[] = new Array(count).fill("");
    const axes: AxisData[] = selected.map((axis) => ({
      axis,
      min: new Float64Array(count).fill(Number.NaN),
      max: new Float64Array(count).fill(Number.NaN),
      domain: null,
    }));

    if (statsCol < 0) {
      return { count, paths, axes };
    }

    // Per batch: bind each axis's min/max leaf vector once, then read local rows.
    store.forEachBatch((batchIndex, startRow, length) => {
      const pathLeaf =
        pathCol >= 0 ? store.getLeafVector(batchIndex, pathCol, []) : null;
      const bound = axes.map((a) => ({
        minVec: store.getLeafVector(batchIndex, statsCol, minPath(a.axis.name)),
        maxVec: store.getLeafVector(batchIndex, statsCol, maxPath(a.axis.name)),
      }));
      for (let r = 0; r < length; r++) {
        const g = startRow + r;
        if (pathLeaf) {
          const p = pathLeaf.get(r);
          if (p != null) paths[g] = String(p);
        }
        for (let ai = 0; ai < axes.length; ai++) {
          const { minVec, maxVec } = bound[ai];
          const t = axes[ai].axis.type;
          axes[ai].min[g] = minVec
            ? toAxisNumber(minVec.get(r), t)
            : Number.NaN;
          axes[ai].max[g] = maxVec
            ? toAxisNumber(maxVec.get(r), t)
            : Number.NaN;
        }
      }
    });

    // Compute each axis's finite domain (spanning both endpoints).
    for (const a of axes) {
      let lo = Number.POSITIVE_INFINITY;
      let hi = Number.NEGATIVE_INFINITY;
      for (let i = 0; i < count; i++) {
        const mn = a.min[i];
        const mx = a.max[i];
        if (Number.isFinite(mn)) {
          if (mn < lo) lo = mn;
          if (mn > hi) hi = mn;
        }
        if (Number.isFinite(mx)) {
          if (mx < lo) lo = mx;
          if (mx > hi) hi = mx;
        }
      }
      a.domain = lo <= hi ? [lo, hi] : null;
    }

    return { count, paths, axes };
  }, [store, version, selected]);
}
