// The volume file editor — the "Files" tab of VolumeDetail.
//
// Composes the reusable editor package: a file list on the left, the persistent
// Monaco editor + tab strip in the middle (SQL highlighting, catalog-aware
// completion via the UC-backed provider registered in main.tsx, live pgsql
// diagnostics), a live markdown preview for `.md`, and — when a query runner is
// registered — a results grid driven by running the buffer's SQL.
//
// This is where the editor's seams are pushed in: the FileStore is injected as a
// prop, and `onRun` drives @open-lakehouse/query. The editor package itself
// never imports query / data-grid / unity-catalog — that composition lives here.
//
// ⚠️ The UC volume Files API is not yet implemented server-side (see
// ./editor/ucFileStore), so this seeds an in-memory store with a couple of demo
// files. Swap `memoryFileStore(...)` for `createUcFileStore(...)` once the Files
// API lands.

import { ArrowResultStore, DataGrid } from "@open-lakehouse/data-grid";
import { MonacoHost } from "@open-lakehouse/editor";
import {
  EditorSessionProvider,
  MarkdownPreview,
  type RunRequest,
  TabStrip,
  useEditorSession,
} from "@open-lakehouse/editor/session";
import { hasQueryRunner, queryRunner } from "@open-lakehouse/query";
import { cn } from "@open-lakehouse/ui-kit";
import { FileText } from "lucide-react";
import { useCallback, useMemo, useRef, useState } from "react";
import { memoryFileStore } from "../editor/ucFileStore";

// Demo files until the volume Files API lands. Keyed by an in-volume path.
const DEMO_FILES: Record<string, string> = {
  "query.sql": `-- Catalog-aware completion + live validation.
select id, email
from main.default.users
where events > 10;
`,
  "README.md": `# Volume files\n\nThis is a **markdown** preview rendered from the editor buffer.\n`,
  "notes.txt": "Plain text opens in a Monaco buffer.\n",
};

/** A tiny run/results controller: streams the buffer's SQL through the query
 *  runner into an ArrowResultStore. Kept minimal and local — the reusable query
 *  seam owns the transport. */
function useRunResults() {
  const storeRef = useRef(new ArrowResultStore());
  const [version, setVersion] = useState(0);
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);

  const run = useCallback(async (sql: string) => {
    abortRef.current?.abort();
    const controller = new AbortController();
    abortRef.current = controller;
    storeRef.current.reset();
    setError(null);
    setRunning(true);
    setVersion((v) => v + 1);
    try {
      for await (const chunk of queryRunner(
        { sql },
        { signal: controller.signal },
      )) {
        storeRef.current.append(chunk.arrowIpc);
        setVersion((v) => v + 1);
      }
    } catch (err) {
      if (!controller.signal.aborted) {
        setError(err instanceof Error ? err.message : String(err));
      }
    } finally {
      if (!controller.signal.aborted) setRunning(false);
    }
  }, []);

  return { store: storeRef.current, version, running, error, run };
}

export function VolumeEditor({ fullName }: { fullName: string }) {
  // The Volumes root this editor is rooted at. A real FileStore would address
  // `/Volumes/<catalog>/<schema>/<volume>/...` (via createUcFileStore); the demo
  // store ignores the root, but we surface it so the intent is visible.
  const volumeRoot = `/Volumes/${fullName.split(".").join("/")}`;
  // One in-memory file store per mounted volume editor (until the Files API lands).
  const fileStore = useMemo(() => memoryFileStore(DEMO_FILES), []);
  const results = useRunResults();

  const onRun = useCallback(
    async (req: RunRequest) => {
      if (req.language !== "sql" || !hasQueryRunner()) return;
      await results.run(req.text);
    },
    [results],
  );

  return (
    <EditorSessionProvider fileStore={fileStore} onRun={onRun}>
      <div className="flex h-[32rem] overflow-hidden rounded-md border">
        <FileList paths={Object.keys(DEMO_FILES)} root={volumeRoot} />
        <div className="flex min-w-0 flex-1 flex-col">
          <TabStrip />
          <EditorSurface results={results} />
        </div>
      </div>
    </EditorSessionProvider>
  );
}

function FileList({ paths, root }: { paths: string[]; root: string }) {
  const { openFile, activeId } = useEditorSession();
  return (
    <nav className="w-48 shrink-0 overflow-y-auto border-r bg-sidebar p-1 text-sm">
      <div
        className="truncate px-2 py-1 font-mono text-xs text-muted-foreground"
        title={root}
      >
        {root}
      </div>
      {paths.map((path) => (
        <button
          key={path}
          type="button"
          onClick={() => void openFile(path)}
          className={cn(
            "flex w-full items-center gap-1.5 rounded px-2 py-1 text-left",
            activeId === path
              ? "bg-accent text-foreground"
              : "text-muted-foreground hover:bg-accent/50",
          )}
        >
          <FileText className="h-3.5 w-3.5 shrink-0" />
          <span className="truncate">{path}</span>
        </button>
      ))}
    </nav>
  );
}

function EditorSurface({
  results,
}: {
  results: ReturnType<typeof useRunResults>;
}) {
  const { activeId, runActive, attachMonaco } = useEditorSession();
  const isMarkdown = activeId?.toLowerCase().endsWith(".md") ?? false;
  const showResults = hasQueryRunner();

  return (
    <div className="flex min-h-0 flex-1">
      <div className="min-w-0 flex-1">
        <MonacoHost
          activeId={activeId}
          onRun={runActive}
          onEditorMount={attachMonaco}
          emptyState="Open a file from the list to start editing."
        />
      </div>
      {isMarkdown && activeId && (
        <div className="w-1/2 min-w-0">
          <MarkdownPreview activePath={activeId} />
        </div>
      )}
      {!isMarkdown && showResults && (
        <div className="flex w-1/2 min-w-0 flex-col border-l">
          <div className="border-b px-3 py-1.5 text-xs font-semibold uppercase tracking-wide text-muted-foreground">
            Results
          </div>
          <div className="min-h-0 flex-1">
            {results.error ? (
              <p className="p-3 text-sm text-destructive">{results.error}</p>
            ) : (
              <DataGrid
                store={results.store}
                version={results.version}
                running={results.running}
              />
            )}
          </div>
        </div>
      )}
    </div>
  );
}
