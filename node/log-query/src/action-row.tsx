import {
  type ArrowResultStore,
  formatScalarText,
  structChildren,
} from "@open-lakehouse/data-grid";
import { cn } from "@open-lakehouse/ui-kit";
import type { DataType, Field } from "apache-arrow";
import { ChevronRight } from "lucide-react";
import { memo, useMemo } from "react";
import type { SlotSpec } from "./lib/actionSlots";

// One rendered action row. The collapsed row keeps the console look — monospace,
// tabular figures, single line — prefixed by a color-and-glyph badge that names
// the action type (color never carries meaning alone: glyph + label are always
// present). Key fields are read zero-copy from the populated slot's struct
// leaves. Expanding reads *every* leaf of that slot, lazily, only when open.

/** Tailwind class fragments per slot color, keyed by the `--color-action-*`
 *  token. Static strings so Tailwind's source scan can see them. */
const COLOR_CLASSES: Record<
  string,
  { text: string; bg: string; ring: string }
> = {
  "action-add": {
    text: "text-action-add",
    bg: "bg-action-add/12",
    ring: "ring-action-add/30",
  },
  "action-remove": {
    text: "text-action-remove",
    bg: "bg-action-remove/12",
    ring: "ring-action-remove/30",
  },
  "action-metadata": {
    text: "text-action-metadata",
    bg: "bg-action-metadata/12",
    ring: "ring-action-metadata/30",
  },
  "action-protocol": {
    text: "text-action-protocol",
    bg: "bg-action-protocol/12",
    ring: "ring-action-protocol/30",
  },
  "action-txn": {
    text: "text-action-txn",
    bg: "bg-action-txn/12",
    ring: "ring-action-txn/30",
  },
  "action-domain": {
    text: "text-action-domain",
    bg: "bg-action-domain/12",
    ring: "ring-action-domain/30",
  },
};

export interface ActionRowProps {
  store: ArrowResultStore;
  spec: SlotSpec;
  /** Top-level column index of the populated slot. */
  colIndex: number;
  /** Global row index into the store. */
  row: number;
  expanded: boolean;
  onToggle: (row: number) => void;
}

/** Read one inline field's value as display text, or null if absent. */
function fieldText(
  store: ArrowResultStore,
  colIndex: number,
  row: number,
  path: readonly string[],
  leafType: DataType | null,
): string | null {
  if (!leafType) return null;
  const raw = store.getNested(row, colIndex, path);
  if (raw === null || raw === undefined) return null;
  return formatScalarText(raw, leafType);
}

/** The leaf DataType for a struct path under a slot column, from the schema. */
function leafTypeFor(
  slotField: Field | undefined,
  path: readonly string[],
): DataType | null {
  let children = slotField ? structChildren(slotField.type) : [];
  let type: DataType | null = null;
  for (const name of path) {
    const child = children.find((c) => c.name === name);
    if (!child) return null;
    type = child.type;
    children = structChildren(child.type);
  }
  return type;
}

export const ActionRow = memo(function ActionRow({
  store,
  spec,
  colIndex,
  row,
  expanded,
  onToggle,
}: ActionRowProps) {
  const colors = COLOR_CLASSES[spec.color] ?? COLOR_CLASSES["action-metadata"];
  const slotField = store.schema?.fields[colIndex];

  // Inline key fields (collapsed row): label=value pairs, absent ones skipped.
  const inline = useMemo(() => {
    return spec.fields
      .map((f) => {
        const leafType = leafTypeFor(slotField, f.path);
        const text = fieldText(store, colIndex, row, f.path, leafType);
        return text === null ? null : { label: f.label, text };
      })
      .filter((x): x is { label: string; text: string } => x !== null);
    // row is the read key; store is mutated in place but a given row is stable.
  }, [store, spec, colIndex, row, slotField]);

  return (
    <div
      className={cn(
        "border-b font-mono text-xs tabular-nums last:border-b-0 hover:bg-accent/40",
        expanded && "bg-accent/20",
      )}
    >
      {/* Collapsed summary line */}
      <button
        type="button"
        onClick={() => onToggle(row)}
        className="flex w-full items-center gap-2 px-2 py-1.5 text-left"
        aria-expanded={expanded}
      >
        <ChevronRight
          className={cn(
            "h-3.5 w-3.5 shrink-0 text-muted-foreground transition-transform",
            expanded && "rotate-90",
          )}
        />
        <span
          className={cn(
            "inline-flex shrink-0 items-center gap-1 rounded px-1.5 py-0.5 font-medium ring-1 ring-inset",
            colors.text,
            colors.bg,
            colors.ring,
          )}
        >
          <span aria-hidden>{spec.glyph}</span>
          {spec.label}
        </span>
        <span className="flex min-w-0 items-center gap-3 truncate text-muted-foreground">
          {inline.map((f) => (
            <span key={f.label} className="truncate">
              <span className="text-muted-foreground/60">{f.label}=</span>
              <span className="text-foreground">{f.text}</span>
            </span>
          ))}
        </span>
      </button>

      {expanded && <ActionDetail store={store} colIndex={colIndex} row={row} />}
    </div>
  );
});

/** The full-detail panel: every leaf field of the populated slot struct, read
 *  lazily (only when the row is expanded) and zero-copy. */
function ActionDetail({
  store,
  colIndex,
  row,
}: {
  store: ArrowResultStore;
  colIndex: number;
  row: number;
}) {
  const slotField = store.schema?.fields[colIndex];
  // colIndex already identifies the slot (each slot is a distinct column), so no
  // separate slot key is needed to invalidate this memo.
  const rows = useMemo(() => {
    const leaves = flattenLeaves(
      slotField ? structChildren(slotField.type) : [],
    );
    return leaves.map((leaf) => ({
      key: leaf.path.join("."),
      value: fieldText(store, colIndex, row, leaf.path, leaf.type) ?? "null",
    }));
  }, [store, colIndex, row, slotField]);

  return (
    <dl className="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-0.5 border-t bg-muted/20 px-9 py-2">
      {rows.map((r) => (
        <div key={r.key} className="contents">
          <dt className="text-muted-foreground/70">{r.key}</dt>
          <dd className="truncate text-foreground">{r.value}</dd>
        </div>
      ))}
    </dl>
  );
}

/** Depth-first flatten of a struct's leaf fields to (path, type) pairs. Structs
 *  recurse; everything else is a leaf (rendered via formatScalarText, which
 *  JSON-stringifies lists/maps). */
function flattenLeaves(
  fields: readonly Field[],
  prefix: readonly string[] = [],
): { path: string[]; type: DataType }[] {
  const out: { path: string[]; type: DataType }[] = [];
  for (const f of fields) {
    const path = [...prefix, f.name];
    const children = structChildren(f.type);
    if (children.length > 0) {
      out.push(...flattenLeaves(children, path));
    } else {
      out.push({ path, type: f.type });
    }
  }
  return out;
}
