import { cn } from "@open-lakehouse/ui-kit";
import type { ReactNode } from "react";

import { CopyButton } from "../CopyButton";

export function Meta({
  label,
  value,
  /** Span the full grid width on its own line (for long values). */
  wide,
  /** Render the value monospaced and break on any character (paths/URLs/ARNs). */
  mono,
  /** Show a subtle inline copy icon after the value (IDs, names, …). */
  copyable,
  /** Custom value content (pill, icon+text, …). Overrides `value` rendering. */
  children,
}: {
  label: string;
  value?: string;
  wide?: boolean;
  mono?: boolean;
  copyable?: boolean;
  children?: ReactNode;
}) {
  return (
    <div className={cn("group min-w-0", wide && "col-span-full")}>
      <dt className="font-mono text-[11px] uppercase tracking-wider text-muted-foreground">
        {label}
      </dt>
      <dd
        className="flex min-w-0 items-center gap-1.5"
        title={children ? undefined : value}
      >
        {children ?? (
          <>
            <span
              className={cn(
                "min-w-0",
                wide ? "break-words" : "truncate",
                mono && "break-all font-mono text-xs",
              )}
            >
              {value || "—"}
            </span>
            {copyable && value && <CopyButton value={value} label={label} />}
          </>
        )}
      </dd>
    </div>
  );
}

export function MetaGrid({ children }: { children: React.ReactNode }) {
  return (
    <dl className="grid grid-cols-2 gap-x-6 gap-y-2 text-sm sm:grid-cols-3">
      {children}
    </dl>
  );
}

// Coerce an epoch-millis field to a usable number. Proto3 JSON encodes int64 as
// a *quoted string* (e.g. "1709640300000"), so these fields arrive as strings at
// runtime even though the generated TS types say `number` — and `new Date(str)`
// on a bare epoch string yields "Invalid Date". Parse defensively and reject
// missing / non-finite / non-positive values (caller shows "—").
export function toEpochMillis(ms?: number | string | null): number | undefined {
  if (ms === undefined || ms === null || ms === "") return undefined;
  const n = typeof ms === "number" ? ms : Number(ms);
  return Number.isFinite(n) && n > 0 ? n : undefined;
}

// Epoch-millis → localized "Mar 05, 2026, 12:05 PM". Returns undefined for
// missing/invalid timestamps so <Meta> renders its "—" placeholder.
export function formatTimestamp(
  ms?: number | string | null,
): string | undefined {
  const epoch = toEpochMillis(ms);
  if (epoch === undefined) return undefined;
  return new Date(epoch).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "numeric",
    minute: "2-digit",
  });
}
