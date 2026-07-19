// The volume file editor — the "Files" tab of VolumeDetail.
//
// Composes two seam packages the app wires up, keeping UC decoupled from both:
//   - @open-lakehouse/files — browse the volume (useDirectory) and read file
//     bytes (FilesService.readFile). Read-only in v1 (canWrite() is false).
//   - @open-lakehouse/editor — the Monaco editor + tab session. SQL highlighting,
//     catalog-aware completion (via the UC-backed CatalogProvider registered in
//     main.tsx), live pgsql diagnostics, and a markdown preview for `.md`.
//
// Because the files backend is read-only, the editor session runs in read-only
// mode: files open and are editable in the buffer, but nothing is persisted
// (autosave disabled) until the files seam gains write verbs. When a query runner
// is registered, running a SQL buffer streams results into a grid.
//
// The editor's FileStore seam is satisfied by a thin read-only adapter over the
// FilesService — no writeFile, so the session is read-only by construction.

import { ArrowResultStore, DataGrid } from "@open-lakehouse/data-grid";
import { MonacoHost } from "@open-lakehouse/editor";
import {
  EditorSessionProvider,
  type FileStore,
  MarkdownPreview,
  type RunRequest,
  TabStrip,
  useEditorSession,
} from "@open-lakehouse/editor/session";
import {
  type DirectoryEntry,
  type FilesService,
  formatVolumePath,
  hasFilesRunner,
  useDirectory,
  useFilesService,
} from "@open-lakehouse/files";
import { hasQueryRunner, queryRunner } from "@open-lakehouse/query";
import { cn } from "@open-lakehouse/ui-kit";
import { FileText, Folder } from "lucide-react";
import { useCallback, useMemo, useRef, useState } from "react";

/** Adapt the files seam's FilesService to the editor's (read-only) FileStore:
 *  readFile drains the file bytes; no writeFile, so the editor session is
 *  read-only (autosave disabled) — matching the files backend's canWrite()=false. */
function filesReadOnlyStore(svc: FilesService): FileStore {
  return {
    async readFile(path) {
      const bytes = await svc.readFile({ path });
      // The files seam doesn't surface an etag on read; read-only needs none.
      return { bytes, stat: { etag: "" } };
    },
  };
}

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
  // The Files tab is only offered when a files runner is registered (the app
  // wires the dev stub / wasm backend); otherwise there's nothing to browse.
  if (!hasFilesRunner()) {
    return (
      <p className="p-4 text-sm text-muted-foreground">
        File browsing isn't available in this build.
      </p>
    );
  }
  return <VolumeEditorInner fullName={fullName} />;
}

function VolumeEditorInner({ fullName }: { fullName: string }) {
  // The volume root: /Volumes/<catalog>/<schema>/<volume> from the dotted name.
  const [catalog, schema, volume] = fullName.split(".");
  const volumeRoot = useMemo(
    () => formatVolumePath({ catalog, schema, volume, relativePath: "" }),
    [catalog, schema, volume],
  );

  const svc = useFilesService();
  // Read-only FileStore over the files service (no writeFile → session is
  // read-only). Rebuilt only when the service instance changes.
  const fileStore = useMemo(() => filesReadOnlyStore(svc), [svc]);
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
        <FileList root={volumeRoot} />
        <div className="flex min-w-0 flex-1 flex-col">
          <TabStrip />
          <EditorSurface results={results} />
        </div>
      </div>
    </EditorSessionProvider>
  );
}

function FileList({ root }: { root: string }) {
  const { openFile, activeId } = useEditorSession();
  const { entries, isLoading, error, hasMore, loadMore } = useDirectory(root);

  return (
    <nav className="w-56 shrink-0 overflow-y-auto border-r bg-sidebar p-1 text-sm">
      <div
        className="truncate px-2 py-1 font-mono text-xs text-muted-foreground"
        title={root}
      >
        {root}
      </div>
      {error && (
        <p className="px-2 py-1 text-xs text-destructive">{error.message}</p>
      )}
      {entries.map((entry) => (
        <FileRow
          key={entry.path}
          entry={entry}
          active={activeId === entry.path}
          onOpen={() => {
            // Files open in the editor; directories are inert in this first cut.
            if (!entry.isDirectory) void openFile(entry.path);
          }}
        />
      ))}
      {isLoading && (
        <p className="px-2 py-1 text-xs text-muted-foreground">Loading…</p>
      )}
      {hasMore && !isLoading && (
        <button
          type="button"
          onClick={loadMore}
          className="w-full rounded px-2 py-1 text-left text-xs text-muted-foreground hover:bg-accent/50"
        >
          Load more…
        </button>
      )}
    </nav>
  );
}

function FileRow({
  entry,
  active,
  onOpen,
}: {
  entry: DirectoryEntry;
  active: boolean;
  onOpen: () => void;
}) {
  const name = entry.path.replace(/\/+$/, "").split("/").pop() ?? entry.path;
  const Icon = entry.isDirectory ? Folder : FileText;
  return (
    <button
      type="button"
      onClick={onOpen}
      className={cn(
        "flex w-full items-center gap-1.5 rounded px-2 py-1 text-left",
        active
          ? "bg-accent text-foreground"
          : "text-muted-foreground hover:bg-accent/50",
        entry.isDirectory && "cursor-default",
      )}
      title={entry.path}
    >
      <Icon className="h-3.5 w-3.5 shrink-0" />
      <span className="truncate">{name}</span>
    </button>
  );
}

function EditorSurface({
  results,
}: {
  results: ReturnType<typeof useRunResults>;
}) {
  const { activeId, runActive, attachMonaco, readOnly } = useEditorSession();
  const isMarkdown = activeId?.toLowerCase().endsWith(".md") ?? false;
  const showResults = hasQueryRunner();

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {readOnly && activeId && (
        <div className="border-b bg-muted/40 px-3 py-1 text-xs text-muted-foreground">
          Read-only — edits aren't saved (volume writes aren't available yet).
        </div>
      )}
      <div className="flex min-h-0 flex-1">
        <div className="min-w-0 flex-1">
          <MonacoHost
            activeId={activeId}
            onRun={runActive}
            onEditorMount={attachMonaco}
            emptyState="Open a file from the list to view it."
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
    </div>
  );
}
