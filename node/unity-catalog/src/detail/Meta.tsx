import { cn } from "@open-lakehouse/ui-kit";

export function Meta({
  label,
  value,
  /** Span the full grid width on its own line (for long values). */
  wide,
  /** Render the value monospaced and break on any character (paths/URLs/ARNs). */
  mono,
}: {
  label: string;
  value?: string;
  wide?: boolean;
  mono?: boolean;
}) {
  return (
    <div className={cn("min-w-0", wide && "col-span-full")}>
      <dt className="text-xs text-muted-foreground">{label}</dt>
      <dd
        className={cn(
          wide ? "break-words" : "truncate",
          mono && "break-all font-mono text-xs",
        )}
        title={value}
      >
        {value || "—"}
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

// Epoch-millis → localized "Mar 05, 2026, 12:05 PM". Returns undefined for
// missing/zero timestamps so <Meta> renders its "—" placeholder.
export function formatTimestamp(ms?: number): string | undefined {
  if (!ms) return undefined;
  return new Date(ms).toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "numeric",
    minute: "2-digit",
  });
}
