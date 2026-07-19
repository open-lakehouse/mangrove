// Live rendered-markdown preview for the active `.md` tab.
//
// Reads the live content straight from the tab's Monaco model (the model is the
// source of truth) on a debounced content-change subscription, renders it with
// `marked`, and sanitizes the HTML with DOMPurify before injecting it — markdown
// can embed raw HTML, so sanitizing is the responsible default. Styled with
// Tailwind's `prose` (the consuming app supplies @tailwindcss/typography).

import DOMPurify from "dompurify";
import { marked } from "marked";
import { useEffect, useState } from "react";
import { getEntry } from "../core/models";
import type { TabId } from "./sessionReducer";

const DEBOUNCE_MS = 150;

function render(markdown: string): string {
  // `marked.parse` is sync for our usage (no async extensions configured).
  const html = marked.parse(markdown, { async: false }) as string;
  return DOMPurify.sanitize(html);
}

export function MarkdownPreview({ activePath }: { activePath: TabId }) {
  const [html, setHtml] = useState("");

  useEffect(() => {
    const entry = getEntry(activePath);
    if (!entry || entry.model.isDisposed()) {
      setHtml("");
      return;
    }
    const { model } = entry;

    let timer: ReturnType<typeof setTimeout> | null = null;
    const update = () => setHtml(render(model.getValue()));

    // Initial render, then debounced updates on edits.
    update();
    const sub = model.onDidChangeContent(() => {
      if (timer) clearTimeout(timer);
      timer = setTimeout(update, DEBOUNCE_MS);
    });

    return () => {
      if (timer) clearTimeout(timer);
      sub.dispose();
    };
  }, [activePath]);

  return (
    <div className="h-full overflow-auto border-l bg-background p-6">
      <div
        className="prose prose-sm dark:prose-invert max-w-none"
        // biome-ignore lint/security/noDangerouslySetInnerHtml: sanitized by DOMPurify in render().
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </div>
  );
}
