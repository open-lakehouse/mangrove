// Public surface of `@open-lakehouse/editor/session` — the multi-tab file-editing
// shell built on top of the core layer (`@open-lakehouse/editor`).
//
// Import this only when you want the full session (tabs + autosave + markdown
// preview); a consumer that needs just an editor surface imports the core barrel
// alone and never pulls in the file-store/autosave/markdown code.

export {
  EditorSessionProvider,
  type EditorSessionProviderProps,
  type RunRequest,
  useEditorSession,
} from "./session/EditorSessionContext.js";
export {
  ConflictError,
  type FileStat,
  type FileStore,
  type ReadResult,
} from "./session/fileStore.js";
export { MarkdownPreview } from "./session/MarkdownPreview.js";
export type {
  OpenTab,
  SaveStatus,
  TabId,
} from "./session/sessionReducer.js";
export { TabStrip } from "./session/TabStrip.js";
