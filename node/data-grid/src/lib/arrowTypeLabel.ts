import { type DataType, type Timestamp, TimeUnit, Type } from "apache-arrow";

// Human-readable type label for a column header badge, e.g. "int64",
// "timestamp[ms, UTC]", "list<utf8>". Kept compact — it's a chip, not a tooltip.

const TIME_UNIT_LABEL: Record<number, string> = {
  [TimeUnit.SECOND]: "s",
  [TimeUnit.MILLISECOND]: "ms",
  [TimeUnit.MICROSECOND]: "µs",
  [TimeUnit.NANOSECOND]: "ns",
};

export function arrowTypeLabel(type: DataType): string {
  switch (type.typeId) {
    case Type.Null:
      return "null";
    case Type.Int: {
      const t = type as unknown as { bitWidth?: number; isSigned?: boolean };
      const prefix = t.isSigned === false ? "uint" : "int";
      return `${prefix}${t.bitWidth ?? ""}`;
    }
    case Type.Float: {
      const t = type as unknown as { precision?: number };
      const bits = t.precision === 0 ? 16 : t.precision === 1 ? 32 : 64;
      return `float${bits}`;
    }
    case Type.Decimal: {
      const t = type as unknown as { precision?: number; scale?: number };
      return `decimal(${t.precision ?? "?"},${t.scale ?? "?"})`;
    }
    case Type.Bool:
      return "bool";
    case Type.Utf8:
      return "utf8";
    case Type.LargeUtf8:
      return "large_utf8";
    case Type.Binary:
      return "binary";
    case Type.Date:
      return "date";
    case Type.Time:
      return "time";
    case Type.Timestamp: {
      const t = type as Timestamp;
      const unit = TIME_UNIT_LABEL[t.unit] ?? "";
      const tz = t.timezone ? `, ${t.timezone}` : ", UTC";
      return `timestamp[${unit}${tz}]`;
    }
    case Type.List:
    case Type.FixedSizeList: {
      const child = (type as unknown as { children?: { type: DataType }[] })
        .children?.[0];
      return `list<${child ? arrowTypeLabel(child.type) : "?"}>`;
    }
    case Type.Struct:
      return "struct";
    case Type.Map:
      return "map";
    default:
      return Type[type.typeId]?.toLowerCase() ?? "?";
  }
}
