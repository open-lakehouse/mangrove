// Public surface of `@open-lakehouse/editor` — the CORE layer (Monaco lifecycle,
// SQL language, theme, and the catalog seam). The multi-tab session shell is a
// separate subexport, `@open-lakehouse/editor/session`; dev fixtures live at
// `@open-lakehouse/editor/fixtures`.
//
// This package is a LEAF: it never imports @open-lakehouse/unity-catalog (nor
// query / data-grid). The Unity Catalog package builds on it and pushes catalog
// context in via `registerCatalogProvider`; run/results are wired by the
// consumer via the `onRun` callback. Enforced by the Biome leaf rule in
// node/biome.json.

// The catalog-metadata seam (the swap point a host registers into).
export {
  type CatalogColumn,
  type CatalogProvider,
  getCatalogProvider,
  hasCatalogProvider,
  NoCatalogProviderError,
  registerCatalogProvider,
} from "./core/catalogProvider";
// The catalog-aware SQL completion service (already wired by ensureMonacoSetup;
// exported for hosts that build their own setup).
export { catalogCompletionService } from "./core/completionService";
// Path → language classification.
export {
  type EditorLanguage,
  extensionOf,
  languageOf,
  MONACO_LANGUAGE_ID,
} from "./core/language";
// The persistent editor surface.
export { MonacoHost, type MonacoHostProps } from "./core/MonacoHost";
// The Monaco model registry (used by the session layer and by core consumers
// that manage models directly).
export {
  disposeAll,
  disposeModel,
  ensureModel,
  getEntry,
  isDirty,
  type ModelEntry,
  markSaved,
  modelUri,
  saveViewState,
} from "./core/models";
// Run-once Monaco bootstrap (loader + workers + SQL features).
export { ensureMonacoSetup } from "./core/monaco-setup";
// Theme bridge to ui-kit's useTheme.
export { type MonacoThemeId, useMonacoTheme } from "./core/useMonacoTheme";

// NOTE: the multi-tab session shell (EditorSessionProvider, TabStrip,
// MarkdownPreview, the FileStore seam) is intentionally NOT re-exported here —
// import it from the `@open-lakehouse/editor/session` subexport so core-only
// consumers never pull the session layer's deps (marked/dompurify). See ./session.ts.
