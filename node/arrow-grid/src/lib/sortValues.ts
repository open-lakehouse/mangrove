import { type DataType, Type } from "apache-arrow";

// Type-aware comparator for sorting result rows by an index permutation. Reads
// raw values straight from the Arrow store (no materialization) and compares
// them per the column's logical type. Nulls sort last in ascending order.

/** Compare two raw Arrow cell values for the given column type. */
export function compareCells(a: unknown, b: unknown, type: DataType): number {
  // Nulls last (ascending); the caller negates the whole result for descending.
  const aNull = a === null || a === undefined;
  const bNull = b === null || b === undefined;
  if (aNull && bNull) return 0;
  if (aNull) return 1;
  if (bNull) return -1;

  switch (type.typeId) {
    case Type.Int:
    case Type.Float:
    case Type.Decimal:
    case Type.Date:
    case Type.Time:
    case Type.Timestamp:
    case Type.Duration:
      return compareNumeric(a, b);
    case Type.Bool:
      return Number(a as boolean) - Number(b as boolean);
    default:
      return String(a).localeCompare(String(b));
  }
}

/** Compare two numeric-ish values that may be `number` or `bigint`. */
function compareNumeric(a: unknown, b: unknown): number {
  if (typeof a === "bigint" || typeof b === "bigint") {
    const ab = toBigInt(a);
    const bb = toBigInt(b);
    return ab < bb ? -1 : ab > bb ? 1 : 0;
  }
  const an = Number(a);
  const bn = Number(b);
  return an < bn ? -1 : an > bn ? 1 : 0;
}

function toBigInt(v: unknown): bigint {
  if (typeof v === "bigint") return v;
  if (typeof v === "number") return BigInt(Math.trunc(v));
  return BigInt(0);
}
