// Per-tab autosave state machine.
//
// Each open tab gets a debounce timer; a content change (re)arms it to fire a
// flush after a quiet interval. All save triggers — debounce, Run, tab
// deactivate, beforeunload — converge on `flush(path)`.
//
// The race-safe part: `flush` snapshots the model's text and alternative version
// id BEFORE the write, and on success advances the saved baseline only to THAT
// id. Edits made during an in-flight write therefore keep the tab dirty rather
// than being silently marked saved. Disposed models are skipped.
//
// This module owns no React state; it reports transitions through the callbacks
// passed to `createAutosave`, which the session context turns into reducer
// dispatches. Persistence goes through the injected `FileStore` seam — the host
// supplies the concrete store (see fileStore.ts).

import { getEntry, isDirty, markSaved } from "../core/models";
import { ConflictError, type FileStore } from "./fileStore";
import type { SaveStatus } from "./sessionReducer";

const DEBOUNCE_MS = 1500;

export interface AutosaveCallbacks {
  /** Report a save-status transition for a tab. */
  onStatus: (path: string, status: SaveStatus, error?: string) => void;
  /** Report a new etag after a successful save. */
  onEtag: (path: string, etag: string) => void;
  /** The current etag for write-if-match, if known. */
  getEtag: (path: string) => string | undefined;
  /** Content type to write for a path (by extension). */
  contentType: (path: string) => string;
}

export interface Autosave {
  /** Called from the model's onDidChangeContent: mark dirty + arm the timer. */
  noteEdit(path: string): void;
  /** Force a save now (Run / deactivate / close). Resolves when done. */
  flush(path: string): Promise<void>;
  /** Flush every dirty tab (beforeunload / route leave). */
  flushAll(): Promise<void>;
  /** Cancel a tab's pending timer (on close). */
  cancel(path: string): void;
  /** Cancel all timers (on unmount). */
  dispose(): void;
}

export function createAutosave(
  fileStore: FileStore,
  cb: AutosaveCallbacks,
): Autosave {
  const timers = new Map<string, ReturnType<typeof setTimeout>>();
  // Tabs with a save in flight, so a concurrent flush() is a no-op rather than a
  // double write.
  const inFlight = new Set<string>();

  function cancel(path: string) {
    const t = timers.get(path);
    if (t !== undefined) {
      clearTimeout(t);
      timers.delete(path);
    }
  }

  function noteEdit(path: string) {
    cb.onStatus(path, isDirty(path) ? "dirty" : "clean");
    cancel(path);
    timers.set(
      path,
      setTimeout(() => {
        timers.delete(path);
        void flush(path);
      }, DEBOUNCE_MS),
    );
  }

  async function flush(path: string): Promise<void> {
    cancel(path);
    if (inFlight.has(path)) return;

    const entry = getEntry(path);
    if (!entry || entry.model.isDisposed()) return;
    if (!isDirty(path)) return;

    // Read-only store (no writeFile): nothing to persist — leave the tab dirty.
    if (!fileStore.writeFile) return;

    // Snapshot BEFORE the write so edits during the write keep the tab dirty.
    const versionAtFlush = entry.model.getAlternativeVersionId();
    const content = entry.model.getValue();

    inFlight.add(path);
    cb.onStatus(path, "saving");
    try {
      const stat = await fileStore.writeFile(
        path,
        new TextEncoder().encode(content),
        { ifMatchEtag: cb.getEtag(path), contentType: cb.contentType(path) },
      );
      cb.onEtag(path, stat.etag);
      // Advance the baseline only to the snapshot version; if the model moved on
      // during the write, it stays dirty and the next debounce/flush catches up.
      if (!entry.model.isDisposed()) {
        markSaved(path, versionAtFlush);
        cb.onStatus(path, isDirty(path) ? "dirty" : "saved");
      }
    } catch (err) {
      const message =
        err instanceof ConflictError
          ? "File changed on disk — reload or overwrite"
          : err instanceof Error
            ? err.message
            : String(err);
      cb.onStatus(path, "error", message);
    } finally {
      inFlight.delete(path);
    }
  }

  async function flushAll(): Promise<void> {
    await Promise.all([...timers.keys()].map((p) => flush(p)));
  }

  function dispose() {
    for (const t of timers.values()) clearTimeout(t);
    timers.clear();
  }

  return { noteEdit, flush, flushAll, cancel, dispose };
}
