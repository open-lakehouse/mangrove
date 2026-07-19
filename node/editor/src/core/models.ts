// Monaco model registry — the single, idempotent owner of `ITextModel`s and
// per-tab view state for the editor.
//
// Why this exists: the #1 source of "Monaco is buggy" pain is models being
// created twice (React 19 StrictMode double-invokes effects) or disposed while
// still in use ("model is disposed" throws). Every create/dispose goes through
// here, where create checks `getModel` first and dispose checks `isDisposed`.
// Models live for the lifetime of an open tab — they are disposed ONLY on tab
// close, never on tab switch (a switch just swaps which model the single editor
// displays, see MonacoHost).
//
// Keyed by file path. The registry is a module singleton (one editor surface per
// app); it holds no React state.

import type * as Monaco from "monaco-editor";
import { languageOf, MONACO_LANGUAGE_ID } from "./language";

export interface ModelEntry {
  model: Monaco.editor.ITextModel;
  /** Editor view state (cursor/scroll/selection) captured on tab deactivate. */
  viewState: Monaco.editor.ICodeEditorViewState | null;
  /**
   * `alternativeVersionId` at the last load/successful save. Dirty is derived by
   * comparing the model's current alternative version id to this baseline, so an
   * undo back to the saved state correctly clears dirty (no text diffing).
   */
  savedVersionId: number;
}

const entries = new Map<string, ModelEntry>();

/** A stable, unique model URI for a file path. */
export function modelUri(monaco: typeof Monaco, path: string): Monaco.Uri {
  return monaco.Uri.parse(`inmemory://editor/${encodeURI(path)}`);
}

/**
 * Get the entry for `path`, creating the model (with `initialText`) if absent.
 * Idempotent: a model already registered for the URI is reused, so a
 * double-invoked effect never creates duplicates.
 */
export function ensureModel(
  monaco: typeof Monaco,
  path: string,
  initialText: string,
): ModelEntry {
  const existing = entries.get(path);
  if (existing && !existing.model.isDisposed()) return existing;

  const uri = modelUri(monaco, path);
  const languageId = MONACO_LANGUAGE_ID[languageOf(path)];
  // Reuse a model Monaco already holds for this URI (StrictMode / re-open race),
  // otherwise create one.
  const model =
    monaco.editor.getModel(uri) ??
    monaco.editor.createModel(initialText, languageId, uri);

  const entry: ModelEntry = {
    model,
    viewState: null,
    savedVersionId: model.getAlternativeVersionId(),
  };
  entries.set(path, entry);
  return entry;
}

/** The registered entry for `path`, if any. */
export function getEntry(path: string): ModelEntry | undefined {
  return entries.get(path);
}

/** Whether `path`'s model has unsaved edits (current version ≠ saved baseline). */
export function isDirty(path: string): boolean {
  const entry = entries.get(path);
  if (!entry || entry.model.isDisposed()) return false;
  return entry.model.getAlternativeVersionId() !== entry.savedVersionId;
}

/** Mark `path` clean as of version `versionId` (its alternative version id). */
export function markSaved(path: string, versionId: number): void {
  const entry = entries.get(path);
  if (entry) entry.savedVersionId = versionId;
}

/** Stash the editor's view state for `path` (on tab deactivate). */
export function saveViewState(
  path: string,
  viewState: Monaco.editor.ICodeEditorViewState | null,
): void {
  const entry = entries.get(path);
  if (entry) entry.viewState = viewState;
}

/** Dispose `path`'s model and forget it (on tab close). Idempotent. */
export function disposeModel(path: string): void {
  const entry = entries.get(path);
  if (!entry) return;
  if (!entry.model.isDisposed()) entry.model.dispose();
  entries.delete(path);
}

/** Dispose every model (on editor unmount, after autosave has flushed). */
export function disposeAll(): void {
  for (const path of [...entries.keys()]) disposeModel(path);
}
