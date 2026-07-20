// The Delta reconciled-action-stream model for the ActionsLog view. The actions
// log has six nullable top-level struct columns — add / remove / metaData /
// protocol / domainMetadata / txn — with EXACTLY ONE non-null per row. A flat
// grid shows 5/6 null cells on every row; instead we detect the one populated
// slot per row (reading only struct validity, zero-copy) and render a compact,
// type-colored summary of its key fields.

import type { ArrowResultStore } from "./arrowResultStore";

/** The six reconciled action slots, in canonical (schema) order. */
export const ACTION_SLOTS = [
  "add",
  "remove",
  "metaData",
  "protocol",
  "domainMetadata",
  "txn",
] as const;

export type ActionSlot = (typeof ACTION_SLOTS)[number];

/** One key field to surface inline for a slot: its label and the struct path to
 *  read under the slot column (relative to the slot struct). */
export interface SlotField {
  label: string;
  /** Path of struct-child names under the slot, e.g. ["stats_parsed","numRecords"]. */
  path: readonly string[];
}

/** Per-slot presentation: the theme color token, an icon glyph, and the handful
 *  of key fields shown inline in the collapsed row. The `color` is a Tailwind
 *  color name backed by a `--color-action-*` theme token (see app globals.css). */
export interface SlotSpec {
  slot: ActionSlot;
  /** Human label shown in the badge (matches the slot name, camelCase kept). */
  label: string;
  /** Tailwind color key: `action-add`, `action-remove`, … */
  color: string;
  /** A short monospace glyph prefix, distinct per slot (icon + color + label so
   *  identity never rests on hue alone). */
  glyph: string;
  /** Whether this slot is a reserved status (good/critical) vs. categorical. */
  reserved?: "good" | "critical";
  fields: readonly SlotField[];
}

/** The inline field spec per slot — the key columns worth showing collapsed.
 *  Full detail (every leaf) is read lazily on expand, not from this list. */
export const SLOT_SPECS: Record<ActionSlot, SlotSpec> = {
  add: {
    slot: "add",
    label: "add",
    color: "action-add",
    glyph: "+",
    reserved: "good",
    fields: [
      { label: "path", path: ["path"] },
      { label: "size", path: ["size"] },
      { label: "rows", path: ["stats_parsed", "numRecords"] },
    ],
  },
  remove: {
    slot: "remove",
    label: "remove",
    color: "action-remove",
    glyph: "−",
    reserved: "critical",
    fields: [{ label: "path", path: ["path"] }],
  },
  metaData: {
    slot: "metaData",
    label: "metaData",
    color: "action-metadata",
    glyph: "◆",
    fields: [
      { label: "id", path: ["id"] },
      { label: "name", path: ["name"] },
    ],
  },
  protocol: {
    slot: "protocol",
    label: "protocol",
    color: "action-protocol",
    glyph: "▲",
    fields: [
      { label: "reader", path: ["minReaderVersion"] },
      { label: "writer", path: ["minWriterVersion"] },
    ],
  },
  domainMetadata: {
    slot: "domainMetadata",
    label: "domainMetadata",
    color: "action-domain",
    glyph: "◇",
    fields: [{ label: "domain", path: ["domain"] }],
  },
  txn: {
    slot: "txn",
    label: "txn",
    color: "action-txn",
    glyph: "⇄",
    fields: [
      { label: "appId", path: ["appId"] },
      { label: "version", path: ["version"] },
    ],
  },
};

/** Resolve each action slot to its top-level column index in the store schema,
 *  once. Returns entries only for slots the schema actually has (a subset is
 *  tolerated). */
export function resolveSlotColumns(
  store: ArrowResultStore,
): { slot: ActionSlot; colIndex: number }[] {
  const fields = store.schema?.fields ?? [];
  const out: { slot: ActionSlot; colIndex: number }[] = [];
  for (const slot of ACTION_SLOTS) {
    const idx = fields.findIndex((f) => f.name === slot);
    if (idx >= 0) out.push({ slot, colIndex: idx });
  }
  return out;
}

/** Whether a store's schema looks like the actions log (has ≥1 action slot). */
export function isActionsSchema(store: ArrowResultStore): boolean {
  return resolveSlotColumns(store).length > 0;
}

/**
 * Detect the single populated action slot at a global row by reading only the
 * slot columns' validity (zero-copy — no value materialization). Returns the
 * first valid slot and its column index, or null if none is set (shouldn't
 * happen for a well-formed reconciled action stream). Call this ONLY for rows
 * the virtualizer is about to render, never as an up-front scan.
 */
export function detectActionSlot(
  store: ArrowResultStore,
  slotColumns: { slot: ActionSlot; colIndex: number }[],
  globalRow: number,
): { slot: ActionSlot; colIndex: number } | null {
  for (const entry of slotColumns) {
    if (store.isSlotValid(globalRow, entry.colIndex)) return entry;
  }
  return null;
}
