// Editor session orchestration — the hub that ties together the tab reducer, the
// Monaco model registry, and autosave, and exposes the imperative API the rest
// of the editor UI calls (openFile / activate / close / reorder).
//
// Split of concerns (deliberate, mirrors the app's data layer):
//   - reducer state (tabs, order, active id, per-tab save status) lives here in
//     React and drives the tab strip;
//   - the live Monaco model + view state + saved-version baseline live in the
//     model registry (core/models.ts), a non-React singleton;
//   - autosave timers + the version-pinned flush live in ./autosave.
//
// MonacoHost registers the captured `monaco` + `editor` here on mount; opening a
// file needs `monaco` to create the model. File persistence goes through the
// injected `FileStore` (prop); execution goes through the injected `onRun` (the
// editor stays run-agnostic — the host wires it to a query service).

import type * as Monaco from "monaco-editor";
import {
  createContext,
  type ReactNode,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useReducer,
  useRef,
  useState,
} from "react";
import { type EditorLanguage, languageOf } from "../core/language";
import {
  disposeAll,
  disposeModel,
  ensureModel,
  getEntry,
} from "../core/models";
import { type Autosave, createAutosave } from "./autosave";
import type { FileStore } from "./fileStore";
import {
  initialSessionState,
  type OpenTab,
  sessionReducer,
  type TabId,
} from "./sessionReducer";

const CONTENT_TYPE_BY_LANG: Record<EditorLanguage, string> = {
  sql: "text/plain",
  markdown: "text/markdown",
  plaintext: "text/plain",
};

/** A request to run a buffer, handed to the host's `onRun`. */
export interface RunRequest {
  path: string;
  language: EditorLanguage;
  text: string;
}

interface EditorSessionValue {
  tabs: OpenTab[];
  activeId: TabId | null;
  /** True once the Monaco editor has mounted (openFile needs it). */
  editorReady: boolean;
  /** True when the file store cannot write (no `writeFile`): tabs open and edit
   *  in-buffer, but changes are never persisted and autosave is disabled. */
  readOnly: boolean;
  /** Open (or focus, if already open) a file in a tab. */
  openFile: (path: string) => Promise<void>;
  activate: (id: TabId) => void;
  close: (id: TabId) => Promise<void>;
  reorder: (from: number, to: number) => void;
  /** Force-save a tab (e.g. before running its query). */
  flush: (path: string) => Promise<void>;
  /** Flush then run the active tab's current buffer via the host's onRun. */
  runActive: () => Promise<void>;
  /** Set by MonacoHost once the editor has mounted. */
  attachMonaco: (
    monaco: typeof Monaco,
    editor: Monaco.editor.IStandaloneCodeEditor,
  ) => void;
}

const EditorSessionContext = createContext<EditorSessionValue | undefined>(
  undefined,
);

export interface EditorSessionProviderProps {
  /** The host-supplied file backend tabs are read from / saved to. */
  fileStore: FileStore;
  /** Execute a buffer (Cmd/Ctrl+Enter / runActive). The editor is run-agnostic;
   *  the host wires this to a query service and renders results. Omit to disable. */
  onRun?: (req: RunRequest) => void | Promise<void>;
  children: ReactNode;
}

export function EditorSessionProvider({
  fileStore,
  onRun,
  children,
}: EditorSessionProviderProps) {
  const [state, dispatch] = useReducer(sessionReducer, initialSessionState);
  const [editorReady, setEditorReady] = useState(false);

  const monacoRef = useRef<typeof Monaco | null>(null);
  const editorRef = useRef<Monaco.editor.IStandaloneCodeEditor | null>(null);
  // The content-change listener disposable, per open path.
  const listenersRef = useRef<Map<string, Monaco.IDisposable>>(new Map());
  const autosaveRef = useRef<Autosave | null>(null);
  // Etags by path, read by autosave's write-if-match (kept in a ref so the
  // autosave instance is stable across renders).
  const etagsRef = useRef<Map<string, string>>(new Map());
  // Latest onRun/fileStore in refs so stable callbacks always see the current
  // values without re-creating.
  const onRunRef = useRef(onRun);
  onRunRef.current = onRun;
  const fileStoreRef = useRef(fileStore);
  fileStoreRef.current = fileStore;
  // Read-only when the store can't write: no autosave, tabs are view/edit-only.
  const readOnly = !fileStore.writeFile;
  const readOnlyRef = useRef(readOnly);
  readOnlyRef.current = readOnly;

  // Build the autosave instance once; its callbacks dispatch into the reducer.
  // The store is read through the ref so a changed prop is picked up on the next
  // write without rebuilding the (timer-holding) autosave instance. The write
  // fn is forwarded only when the store has one (read-only stores omit it, and
  // autosave then no-ops).
  if (autosaveRef.current === null) {
    autosaveRef.current = createAutosave(
      {
        writeFile: fileStoreRef.current.writeFile
          ? (path, bytes, opts) =>
              // biome-ignore lint/style/noNonNullAssertion: guarded by the ternary above.
              fileStoreRef.current.writeFile!(path, bytes, opts)
          : undefined,
        readFile: (path) => fileStoreRef.current.readFile(path),
      },
      {
        onStatus: (path, saveStatus, error) =>
          dispatch({ type: "SET_STATUS", id: path, saveStatus, error }),
        onEtag: (path, etag) => {
          etagsRef.current.set(path, etag);
          dispatch({ type: "SET_ETAG", id: path, etag });
        },
        getEtag: (path) => etagsRef.current.get(path),
        contentType: (path) => CONTENT_TYPE_BY_LANG[languageOf(path)],
      },
    );
  }

  const attachMonaco = useCallback(
    (monaco: typeof Monaco, editor: Monaco.editor.IStandaloneCodeEditor) => {
      monacoRef.current = monaco;
      editorRef.current = editor;
      setEditorReady(true);
    },
    [],
  );

  const openFile = useCallback(async (path: string) => {
    // Already open → just activate (no refetch, no duplicate model).
    if (getEntry(path)) {
      dispatch({ type: "ACTIVATE_TAB", id: path });
      return;
    }
    const monaco = monacoRef.current;
    if (!monaco) return; // editor not mounted yet

    const { bytes, stat } = await fileStoreRef.current.readFile(path);
    const text = new TextDecoder().decode(bytes);
    const entry = ensureModel(monaco, path, text);
    etagsRef.current.set(path, stat.etag);

    // Mark dirty on edits; the autosave instance derives clean/dirty/saving.
    // Skipped in read-only mode — there's nothing to save, so no dirty cycle.
    if (!readOnlyRef.current) {
      const listener = entry.model.onDidChangeContent(() =>
        autosaveRef.current?.noteEdit(path),
      );
      listenersRef.current.get(path)?.dispose();
      listenersRef.current.set(path, listener);
    }

    const name = path.replace(/\/+$/, "").split("/").pop() ?? path;
    dispatch({
      type: "OPEN_TAB",
      tab: {
        id: path,
        path,
        name,
        language: languageOf(path),
        etag: stat.etag,
      },
    });
  }, []);

  const activate = useCallback(
    (id: TabId) => dispatch({ type: "ACTIVATE_TAB", id }),
    [],
  );

  const close = useCallback(async (id: TabId) => {
    // Best-effort flush before discarding the buffer.
    await autosaveRef.current?.flush(id);
    autosaveRef.current?.cancel(id);
    listenersRef.current.get(id)?.dispose();
    listenersRef.current.delete(id);
    etagsRef.current.delete(id);
    disposeModel(id);
    dispatch({ type: "CLOSE_TAB", id });
  }, []);

  const reorder = useCallback(
    (from: number, to: number) => dispatch({ type: "REORDER_TABS", from, to }),
    [],
  );

  const flush = useCallback(
    (path: string) => autosaveRef.current?.flush(path) ?? Promise.resolve(),
    [],
  );

  // Mirror of activeId for the stable runActive callback.
  const activeIdRef = useRef<TabId | null>(null);
  activeIdRef.current = state.activeId;

  // Save-on-run: flush the buffer, then hand its current text to the host. We run
  // what's in the model (the live buffer), so the flush is for persistence, not
  // to decide what executes.
  const runActive = useCallback(async () => {
    const path = activeIdRef.current;
    if (!path) return;
    const entry = getEntry(path);
    if (!entry || entry.model.isDisposed()) return;
    const text = entry.model.getValue();
    if (!text.trim()) return;
    await autosaveRef.current?.flush(path);
    await onRunRef.current?.({ path, language: languageOf(path), text });
  }, []);

  // Flush dirty buffers on browser unload.
  useEffect(() => {
    const onBeforeUnload = (e: BeforeUnloadEvent) => {
      const hasUnsaved = state.tabs.some(
        (t) => t.saveStatus === "dirty" || t.saveStatus === "saving",
      );
      if (hasUnsaved) {
        void autosaveRef.current?.flushAll();
        e.preventDefault();
      }
    };
    window.addEventListener("beforeunload", onBeforeUnload);
    return () => window.removeEventListener("beforeunload", onBeforeUnload);
  }, [state.tabs]);

  // On provider unmount, flush then dispose every model + listener.
  const autosave = autosaveRef.current;
  useEffect(() => {
    const listeners = listenersRef.current;
    return () => {
      void autosave?.flushAll().finally(() => {
        autosave?.dispose();
        for (const d of listeners.values()) d.dispose();
        listeners.clear();
        disposeAll();
      });
    };
  }, [autosave]);

  const value = useMemo<EditorSessionValue>(
    () => ({
      tabs: state.tabs,
      activeId: state.activeId,
      editorReady,
      readOnly,
      openFile,
      activate,
      close,
      reorder,
      flush,
      runActive,
      attachMonaco,
    }),
    [
      state.tabs,
      state.activeId,
      editorReady,
      readOnly,
      openFile,
      activate,
      close,
      reorder,
      flush,
      runActive,
      attachMonaco,
    ],
  );

  return (
    <EditorSessionContext.Provider value={value}>
      {children}
    </EditorSessionContext.Provider>
  );
}

export function useEditorSession(): EditorSessionValue {
  const ctx = useContext(EditorSessionContext);
  if (!ctx) {
    throw new Error(
      "useEditorSession must be used within an EditorSessionProvider",
    );
  }
  return ctx;
}
