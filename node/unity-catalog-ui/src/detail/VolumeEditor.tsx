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
  joinVolumePath,
  parseVolumePath,
  useDirectory,
  useFilesService,
} from "@open-lakehouse/files";
import { hasQueryRunner, queryRunner } from "@open-lakehouse/query";
import { cn } from "@open-lakehouse/ui-kit";
import {
  ChevronRight,
  Download,
  FileText,
  Folder,
  FolderUp,
} from "lucide-react";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { formatFileSize } from "./fileSize";
import { formatTimestamp } from "./Meta";

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

  // The directory currently shown in the file list. Navigating folders swaps
  // this; useDirectory is keyed on it so each change re-lists (and aborts the
  // prior in-flight page). Re-seed to the root when the volume changes — via an
  // effect rather than a `key` remount so open editor tabs survive.
  const [currentPath, setCurrentPath] = useState(volumeRoot);
  useEffect(() => setCurrentPath(volumeRoot), [volumeRoot]);

  const onRun = useCallback(
    async (req: RunRequest) => {
      if (req.language !== "sql" || !hasQueryRunner()) return;
      await results.run(req.text);
    },
    [results],
  );

  // Download the given file's bytes via the read-only files service (no write
  // verbs touched). One in-flight download at a time; a new download or unmount
  // aborts the previous, mirroring useRunResults' abort discipline.
  const downloadAbortRef = useRef<AbortController | null>(null);
  const download = useCallback(
    async (path: string) => {
      downloadAbortRef.current?.abort();
      const controller = new AbortController();
      downloadAbortRef.current = controller;
      // NOTE: readFile drains the whole file into one buffer. Fine for browsing;
      // readFileStream is the future path for very large files.
      const bytes = await svc.readFile({ path }, controller.signal);
      if (controller.signal.aborted) return;
      const name = basename(path);
      // Copy into a plain ArrayBuffer-backed view so the Blob part type is
      // concrete (readFile's Uint8Array is over ArrayBufferLike).
      const url = URL.createObjectURL(new Blob([bytes.slice()]));
      try {
        const anchor = document.createElement("a");
        anchor.href = url;
        anchor.download = name;
        anchor.click();
      } finally {
        URL.revokeObjectURL(url);
      }
    },
    [svc],
  );

  return (
    <EditorSessionProvider fileStore={fileStore} onRun={onRun}>
      <div className="flex h-[32rem] overflow-hidden rounded-md border">
        <FileList
          root={volumeRoot}
          currentPath={currentPath}
          onNavigate={setCurrentPath}
        />
        <div className="flex min-w-0 flex-1 flex-col">
          <TabStrip />
          <EditorSurface results={results} onDownload={download} />
        </div>
      </div>
    </EditorSessionProvider>
  );
}

/** The trailing path segment (a file/dir name), tolerant of trailing slashes. */
function basename(path: string): string {
  return path.replace(/\/+$/, "").split("/").pop() ?? path;
}

/** Breadcrumb for the file list: the volume name then each directory segment,
 *  every crumb clickable to jump back. Targets are built with joinVolumePath so
 *  there's no hand-rolled path math. The last crumb (current dir) is inert. */
function Breadcrumb({
  root,
  currentPath,
  onNavigate,
}: {
  root: string;
  currentPath: string;
  onNavigate: (path: string) => void;
}) {
  const vp = parseVolumePath(currentPath);
  // Defensive: a non-/Volumes path shouldn't reach here (root is canonical), but
  // fall back to a single inert crumb rather than throwing.
  const segments = vp ? vp.relativePath.split("/").filter(Boolean) : [];
  const atRoot = segments.length === 0;

  // Each crumb: a label and the path it navigates to. The volume name → root.
  const crumbs = [
    { label: vp?.volume ?? currentPath, target: root },
    ...segments.map((seg, i) => ({
      label: seg,
      target: joinVolumePath(root, ...segments.slice(0, i + 1)),
    })),
  ];

  return (
    <div className="flex items-center gap-1 border-b px-2 py-1">
      <button
        type="button"
        onClick={() =>
          onNavigate(joinVolumePath(root, ...segments.slice(0, -1)))
        }
        disabled={atRoot}
        title="Up one level"
        className="shrink-0 rounded p-0.5 text-muted-foreground hover:bg-accent/50 disabled:opacity-40 disabled:hover:bg-transparent"
      >
        <FolderUp className="h-3.5 w-3.5" />
      </button>
      <nav className="flex min-w-0 flex-1 items-center overflow-x-auto text-xs">
        {crumbs.map((crumb, i) => {
          const isLast = i === crumbs.length - 1;
          return (
            <span key={crumb.target} className="flex items-center">
              {i > 0 && (
                <ChevronRight className="h-3 w-3 shrink-0 text-muted-foreground/60" />
              )}
              <button
                type="button"
                onClick={() => onNavigate(crumb.target)}
                disabled={isLast}
                title={crumb.label}
                className={cn(
                  "max-w-[10rem] truncate rounded px-1 py-0.5",
                  isLast
                    ? "font-medium text-foreground"
                    : "text-muted-foreground hover:bg-accent/50",
                )}
              >
                {crumb.label}
              </button>
            </span>
          );
        })}
      </nav>
    </div>
  );
}

function FileList({
  root,
  currentPath,
  onNavigate,
}: {
  root: string;
  currentPath: string;
  onNavigate: (path: string) => void;
}) {
  const { openFile, activeId } = useEditorSession();
  // Keyed on currentPath: navigating rebuilds the listing and aborts the prior
  // in-flight page (StrictMode-safe in useDirectory — no manual fetch here).
  const { entries, isLoading, error, hasMore, loadMore } =
    useDirectory(currentPath);

  return (
    <nav className="flex w-72 shrink-0 flex-col overflow-hidden border-r bg-sidebar text-sm">
      <Breadcrumb
        root={root}
        currentPath={currentPath}
        onNavigate={onNavigate}
      />
      <div className="min-h-0 flex-1 overflow-y-auto p-1">
        {error && (
          <p className="px-2 py-1 text-xs text-destructive">{error.message}</p>
        )}
        {entries.map((entry) => (
          <FileRow
            key={entry.path}
            entry={entry}
            active={activeId === entry.path}
            onOpen={() => {
              // Directories drill in; files open in the editor.
              if (entry.isDirectory) onNavigate(entry.path);
              else void openFile(entry.path);
            }}
          />
        ))}
        {!isLoading && entries.length === 0 && !error && (
          <p className="px-2 py-1 text-xs text-muted-foreground">
            Empty folder
          </p>
        )}
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
      </div>
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
  const name = basename(entry.path);
  const Icon = entry.isDirectory ? Folder : FileText;
  // Directories show no size; both show a last-modified time when available.
  const size = entry.isDirectory ? "—" : formatFileSize(entry.fileSize);
  const modified = formatTimestamp(entry.lastModified);
  return (
    <button
      type="button"
      onClick={onOpen}
      className={cn(
        "flex w-full items-center gap-1.5 rounded px-2 py-1 text-left",
        active
          ? "bg-accent text-foreground"
          : "text-muted-foreground hover:bg-accent/50",
      )}
      title={entry.path}
    >
      <Icon className="h-3.5 w-3.5 shrink-0" />
      <span className="min-w-0 flex-1 truncate">{name}</span>
      <span className="shrink-0 text-right text-[0.6875rem] tabular-nums text-muted-foreground/80">
        {size}
      </span>
      {modified && (
        <span className="hidden shrink-0 text-right text-[0.6875rem] text-muted-foreground/60 lg:inline">
          {modified}
        </span>
      )}
    </button>
  );
}

function EditorSurface({
  results,
  onDownload,
}: {
  results: ReturnType<typeof useRunResults>;
  onDownload: (path: string) => Promise<void>;
}) {
  const { activeId, runActive, attachMonaco, readOnly } = useEditorSession();
  const isMarkdown = activeId?.toLowerCase().endsWith(".md") ?? false;
  const showResults = hasQueryRunner();

  const [downloading, setDownloading] = useState(false);
  const handleDownload = useCallback(async () => {
    if (!activeId) return;
    setDownloading(true);
    try {
      await onDownload(activeId);
    } finally {
      setDownloading(false);
    }
  }, [activeId, onDownload]);

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      {activeId && (
        <div className="flex items-center justify-end border-b px-3 py-1">
          <button
            type="button"
            onClick={handleDownload}
            disabled={downloading}
            className="flex items-center gap-1 rounded px-1.5 py-0.5 text-xs text-muted-foreground hover:bg-accent/50 disabled:opacity-50"
          >
            <Download className="h-3.5 w-3.5" />
            {downloading ? "Downloading…" : "Download"}
          </button>
        </div>
      )}
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
