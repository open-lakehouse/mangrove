// Numeric coercions for Arrow values, shared by the cell formatters and the
// Arrow-backed visualizations (min/max boxes). Kept dependency-free and
// side-effect-free so any consumer can map an ordered Arrow value onto a number
// line without pulling in React or the formatter registry.

import { type DataType, type Timestamp, TimeUnit, Type } from "apache-arrow";

/**
 * Convert an Arrow timestamp value (interpreted per its `unit`) to epoch
 * milliseconds. Values arrive as `number` or `bigint` depending on the unit and
 * precision; both are handled.
 */
export function timestampToEpochMs(value: unknown, type: Timestamp): number {
  const asBig =
    typeof value === "bigint" ? value : BigInt(Math.trunc(Number(value)));
  switch (type.unit) {
    case TimeUnit.SECOND:
      return Number(asBig) * 1000;
    case TimeUnit.MILLISECOND:
      return Number(asBig);
    case TimeUnit.MICROSECOND:
      return Number(asBig / 1000n);
    case TimeUnit.NANOSECOND:
      return Number(asBig / 1_000_000n);
    default:
      return Number(asBig);
  }
}

/**
 * Whether a column's Arrow type has a natural ordering that maps to a single
 * number line — the precondition for using it as a min/max box axis. Numbers,
 * decimals and temporal types qualify; strings, booleans, and nested types do
 * not.
 */
export function isOrderableType(type: DataType): boolean {
  switch (type.typeId) {
    case Type.Int:
    case Type.Float:
    case Type.Decimal:
    case Type.Date:
    case Type.Timestamp:
    case Type.Time:
      return true;
    default:
      return false;
  }
}

/**
 * Coerce an ordered Arrow value to a JS `number` for plotting on a linear axis.
 * Returns `NaN` for null/undefined or anything that can't be placed on a number
 * line, so callers can use `NaN` as a uniform "no value" sentinel.
 *
 * - Int/Float: numeric (bigint widened to Number — safe for the display ranges
 *   we plot, at the cost of precision past 2^53).
 * - Decimal: unscaled integer placed by the type's `scale`.
 * - Date: apache-arrow's get visitor already normalizes both Date units to epoch
 *   milliseconds, so the raw value is ms — do not scale again.
 * - Timestamp: converted from its unit via {@link timestampToEpochMs}.
 * - Time: numeric ticks (unit-relative); usable as a relative axis.
 */
export function toAxisNumber(value: unknown, type: DataType): number {
  if (value === null || value === undefined) return Number.NaN;
  switch (type.typeId) {
    case Type.Int:
    case Type.Float:
      return typeof value === "bigint" ? Number(value) : Number(value);
    case Type.Decimal:
      return decimalToNumber(value, type);
    case Type.Date:
      return value instanceof Date ? value.getTime() : Number(value);
    case Type.Timestamp:
      return timestampToEpochMs(value, type as Timestamp);
    case Type.Time:
      return typeof value === "bigint" ? Number(value) : Number(value);
    default:
      return Number.NaN;
  }
}

/** Place a Decimal's unscaled integer by its `scale`, returning a JS number. */
function decimalToNumber(value: unknown, type: DataType): number {
  const scale = (type as unknown as { scale?: number }).scale ?? 0;
  let unscaled: number;
  if (typeof value === "bigint") {
    unscaled = Number(value);
  } else if (typeof value === "number") {
    unscaled = value;
  } else {
    return Number.NaN;
  }
  return scale > 0 ? unscaled / 10 ** scale : unscaled;
}
