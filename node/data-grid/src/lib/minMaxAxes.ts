// Enumerate the columns that can serve as a min/max-box axis, from the
// reconciled-log schema. Each surviving file carries a nested
// `stats.minValues.<col>` / `stats.maxValues.<col>` pair; a column is a usable
// axis iff it appears as an orderable leaf under BOTH minValues and maxValues.
// Pure schema walk — no data touched.

import type { DataType, Field } from "apache-arrow";
import { structChildren, structFieldByName } from "./nestedAccess";
import { isOrderableType } from "./temporal";

/** The top-level column holding the per-file stats struct. */
export const STATS_COLUMN = "stats";
const MIN_VALUES = "minValues";
const MAX_VALUES = "maxValues";

/** One column usable as a min/max axis: its name and leaf Arrow type. */
export interface MinMaxAxis {
  /** Leaf column name (logical), e.g. "amount". */
  name: string;
  /** The leaf's Arrow type (min and max share it). */
  type: DataType;
}

/**
 * List the orderable columns for which BOTH `stats.minValues.<col>` and
 * `stats.maxValues.<col>` exist as orderable leaves, in schema order. Returns
 * `[]` when the schema has no `stats` struct, or no `minValues`/`maxValues`
 * sub-struct (a table with no skipping-eligible columns), or none of the shared
 * leaves are orderable — the empty-axes case the controls surface as a message.
 */
export function enumerateMinMaxAxes(
  fields: readonly Field[] | undefined,
): MinMaxAxis[] {
  const stats = fields?.find((f) => f.name === STATS_COLUMN);
  if (!stats) return [];

  const minField = structFieldByName(stats.type, MIN_VALUES);
  const maxField = structFieldByName(stats.type, MAX_VALUES);
  if (!minField || !maxField) return [];

  const maxLeaves = new Map(
    structChildren(maxField.type).map((c) => [c.name, c] as const),
  );

  const axes: MinMaxAxis[] = [];
  for (const minLeaf of structChildren(minField.type)) {
    const maxLeaf = maxLeaves.get(minLeaf.name);
    if (!maxLeaf) continue; // present in min but not max — skip
    // Use the min leaf's type as the axis type (min and max are the same type).
    if (isOrderableType(minLeaf.type)) {
      axes.push({ name: minLeaf.name, type: minLeaf.type });
    }
  }
  return axes;
}

/** Whether the schema has at least one orderable min/max axis — the gate the
 *  Delta-log tab reads to decide whether to offer the "Boxes" sub-view. */
export function hasMinMaxAxes(fields: readonly Field[] | undefined): boolean {
  return enumerateMinMaxAxes(fields).length > 0;
}

/** The struct path to a column's min leaf under the `stats` column. */
export function minPath(column: string): string[] {
  return [MIN_VALUES, column];
}

/** The struct path to a column's max leaf under the `stats` column. */
export function maxPath(column: string): string[] {
  return [MAX_VALUES, column];
}
