import { cn } from "@open-lakehouse/ui-kit";
import type { ReactNode } from "react";

// A console-style section kicker: a small solid accent square (a "status dot"
// borrowed from developer-hub / terminal UIs) followed by a monospace uppercase
// label. Doubles as a VS Code / Zed panel header ("EXPLORER", "OUTLINE"), so it
// reads as tooling chrome rather than marketing. Used for sidebar and detail
// section headings; the caller supplies trailing controls as `children` of the
// surrounding row when needed.
export function SectionLabel({
  children,
  className,
}: {
  children: ReactNode;
  className?: string;
}) {
  return (
    <span
      className={cn(
        "flex items-center gap-2 font-mono text-[11px] font-medium uppercase tracking-[0.08em] text-muted-foreground",
        className,
      )}
    >
      <span
        aria-hidden
        className="size-1.5 shrink-0 rounded-[1px] bg-primary/70"
      />
      {children}
    </span>
  );
}
