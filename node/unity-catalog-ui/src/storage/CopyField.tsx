import { Button, Label } from "@open-lakehouse/ui-kit";
import { Check, Copy } from "lucide-react";
import { useState } from "react";

/** A read-only, monospaced value with a copy-to-clipboard button. */
export function CopyField({ label, value }: { label: string; value?: string }) {
  const [copied, setCopied] = useState(false);

  if (!value) return null;

  function copy() {
    navigator.clipboard?.writeText(value ?? "").then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  }

  return (
    <div className="space-y-1">
      <Label>{label}</Label>
      <div className="flex items-center gap-2">
        <code className="min-w-0 flex-1 truncate rounded-md border bg-muted px-2 py-1.5 text-xs">
          {value}
        </code>
        <Button
          type="button"
          variant="outline"
          size="icon"
          className="h-8 w-8 shrink-0"
          aria-label={`Copy ${label}`}
          onClick={copy}
        >
          {copied ? (
            <Check className="h-4 w-4" />
          ) : (
            <Copy className="h-4 w-4" />
          )}
        </Button>
      </div>
    </div>
  );
}
