import { type DataType, type Timestamp, Type } from "apache-arrow";
import type { ReactNode } from "react";
import { timestampToEpochMs } from "./temporal";

// Type-aware cell rendering driven by the Arrow schema. Each formatter takes a
// raw value read zero-copy from the store plus the column's `DataType`, and
// returns a React node and an alignment hint, so styling stays in the existing
// Tailwind idiom. Numbers right-align with tabular figures; temporal types
// render in human time format; null is a distinct muted token.

export type CellAlign = "left" | "right";

export interface CellRender {
  node: ReactNode;
  align: CellAlign;
}

const numberFmt = new Intl.NumberFormat();
const dateTimeFmt = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeStyle: "medium",
});
// Timestamps without a timezone are tz-naive — format them in UTC and label as
// such rather than silently applying the browser's local zone.
const utcDateTimeFmt = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeStyle: "medium",
  timeZone: "UTC",
});
// Date-only values are tz-naive points; format in UTC to avoid day-shift.
const utcDateFmt = new Intl.DateTimeFormat(undefined, {
  dateStyle: "medium",
  timeZone: "UTC",
});

const NULL_TOKEN: ReactNode = (
  <span className="italic text-muted-foreground/60">null</span>
);

/** Format one raw Arrow cell value for display. */
export function formatCell(value: unknown, type: DataType): CellRender {
  if (value === null || value === undefined) {
    return { node: NULL_TOKEN, align: "left" };
  }
  switch (type.typeId) {
    case Type.Int:
    case Type.Float:
      return { node: formatNumber(value), align: "right" };
    case Type.Decimal:
      return { node: formatDecimal(value, type), align: "right" };
    case Type.Bool:
      return { node: <BoolToken value={value as boolean} />, align: "left" };
    case Type.Timestamp:
      return { node: formatTimestamp(value, type as Timestamp), align: "left" };
    case Type.Date:
      return { node: formatDate(value, type), align: "left" };
    case Type.Time:
      return { node: String(value), align: "left" };
    case Type.Utf8:
    case Type.LargeUtf8:
      return { node: String(value), align: "left" };
    case Type.List:
    case Type.FixedSizeList:
    case Type.Struct:
    case Type.Map:
    case Type.Union:
      return { node: <JsonToken value={value} />, align: "left" };
    default:
      return { node: String(value), align: "left" };
  }
}

/**
 * A compact plain-string rendering of a scalar Arrow value, for dense inline
 * contexts (e.g. a single-line row summary) where a ReactNode + alignment is
 * overkill. Nulls become the literal `"null"`. Nested types are
 * JSON-stringified. Reuses the same number / decimal / temporal logic as
 * {@link formatCell}.
 */
export function formatScalarText(value: unknown, type: DataType): string {
  if (value === null || value === undefined) return "null";
  switch (type.typeId) {
    case Type.Int:
    case Type.Float:
      return formatNumber(value);
    case Type.Decimal:
      return formatDecimal(value, type);
    case Type.Bool:
      return value ? "true" : "false";
    case Type.Timestamp: {
      const ms = timestampToEpochMs(value, type as Timestamp);
      if (!Number.isFinite(ms)) return String(value);
      const d = new Date(ms);
      return Number.isNaN(d.getTime()) ? String(value) : d.toISOString();
    }
    case Type.List:
    case Type.FixedSizeList:
    case Type.Struct:
    case Type.Map:
    case Type.Union:
      return stringifyArrow(value);
    default:
      return String(value);
  }
}

function formatNumber(value: unknown): string {
  if (typeof value === "bigint") return value.toLocaleString();
  if (typeof value === "number") return numberFmt.format(value);
  return String(value);
}

/**
 * Arrow decimals arrive as a bigint (or bigint-like) unscaled integer. Place the
 * decimal point using the column's scale. apache-arrow stores the SQL scale on
 * the `scale` property.
 */
function formatDecimal(value: unknown, type: DataType): string {
  const scale = (type as unknown as { scale?: number }).scale ?? 0;
  let unscaled: bigint;
  if (typeof value === "bigint") {
    unscaled = value;
  } else if (typeof value === "number") {
    unscaled = BigInt(Math.trunc(value));
  } else {
    // Fall back to string for anything unexpected (e.g. Uint32Array words).
    return String(value);
  }
  if (scale <= 0) return unscaled.toString();
  const neg = unscaled < 0n;
  const digits = (neg ? -unscaled : unscaled)
    .toString()
    .padStart(scale + 1, "0");
  const intPart = digits.slice(0, digits.length - scale);
  const fracPart = digits.slice(digits.length - scale);
  return `${neg ? "-" : ""}${intPart}.${fracPart}`;
}

function formatTimestamp(value: unknown, type: Timestamp): ReactNode {
  const ms = timestampToEpochMs(value, type);
  if (!Number.isFinite(ms)) return String(value);
  const date = new Date(ms);
  // A finite-but-out-of-range ms yields an Invalid Date, which `format()` rejects.
  if (Number.isNaN(date.getTime())) return String(value);
  if (type.timezone) {
    return dateTimeFmt.format(date);
  }
  // tz-naive: render in UTC and label it so the value isn't misread as local.
  return (
    <span>
      {utcDateTimeFmt.format(date)}
      <span className="ml-1 text-muted-foreground/60">UTC</span>
    </span>
  );
}

function formatDate(value: unknown, _type: DataType): string {
  // apache-arrow's get visitor already normalizes both Date units to epoch
  // milliseconds (DateDay multiplies days×86_400_000; DateMillisecond returns ms
  // directly — see visitor/get.js getDateDay/getDateMillisecond). So the raw value
  // here is already ms; do NOT scale it again (doing so overflows the JS Date
  // range and makes Intl.DateTimeFormat throw "date value is not finite").
  const ms = value instanceof Date ? value.getTime() : Number(value);
  if (!Number.isFinite(ms)) return String(value);
  const date = new Date(ms);
  // A finite-but-out-of-range ms yields an Invalid Date, which `format()` rejects;
  // guard on the resolved time so we degrade to the raw value instead of throwing.
  if (Number.isNaN(date.getTime())) return String(value);
  return utcDateFmt.format(date);
}

function BoolToken({ value }: { value: boolean }): ReactNode {
  return (
    <span className={value ? "text-primary" : "text-muted-foreground"}>
      {value ? "true" : "false"}
    </span>
  );
}

function JsonToken({ value }: { value: unknown }): ReactNode {
  return <span className="text-muted-foreground">{stringifyArrow(value)}</span>;
}

/** JSON-stringify a nested Arrow value, handling bigint and Arrow vectors. */
function stringifyArrow(value: unknown): string {
  try {
    return JSON.stringify(value, (_k, v) => {
      if (typeof v === "bigint") return v.toString();
      // Arrow vectors / structs expose toJSON / toArray for plain rendering.
      if (
        v &&
        typeof v === "object" &&
        typeof (v as { toJSON?: unknown }).toJSON === "function"
      ) {
        return (v as { toJSON: () => unknown }).toJSON();
      }
      return v;
    });
  } catch {
    return String(value);
  }
}
