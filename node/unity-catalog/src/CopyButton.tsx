import { cn } from "@open-lakehouse/ui-kit";
import { Check, Copy } from "lucide-react";
import { useEffect, useState } from "react";

// A small, subtle inline "copy to clipboard" affordance meant to sit directly
// after a value (an ID, a fully-qualified name, …). It stays dim until the row
// is hovered, then brightens; on click it briefly flips to a check. Kept as a
// bare <button> (not the ui-kit Button) so it can render inline mid-text
// without the padding/height of a real button.
export function CopyButton({
  value,
  label,
  className,
}: {
  value: string;
  /** Accessible name, e.g. "Volume ID". Falls back to a generic label. */
  label?: string;
  className?: string;
}) {
  const [copied, setCopied] = useState(false);

  // Reset the confirmation after a short delay; clean up so a quick unmount
  // (e.g. selecting another object) doesn't fire setState on a dead component.
  useEffect(() => {
    if (!copied) return;
    const t = setTimeout(() => setCopied(false), 1200);
    return () => clearTimeout(t);
  }, [copied]);

  function copy(e: React.MouseEvent) {
    // Values often live inside a clickable row/header; don't trigger selection.
    e.stopPropagation();
    e.preventDefault();
    navigator.clipboard?.writeText(value).then(() => setCopied(true));
  }

  return (
    <button
      type="button"
      onClick={copy}
      aria-label={copied ? "Copied" : `Copy ${label ?? "value"}`}
      title={copied ? "Copied" : "Copy"}
      className={cn(
        "inline-flex shrink-0 items-center justify-center align-middle text-muted-foreground/50 opacity-0 transition group-hover:opacity-100 hover:text-foreground focus-visible:opacity-100 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring",
        copied && "text-success opacity-100",
        className,
      )}
    >
      {copied ? (
        <Check className="h-3.5 w-3.5" />
      ) : (
        <Copy className="h-3.5 w-3.5" />
      )}
    </button>
  );
}
