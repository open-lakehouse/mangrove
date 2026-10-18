# Design contract — `@open-lakehouse/editor`

An embeddable Monaco editor for SQL and text files: highlighting, live SQL
diagnostics, catalog-aware completion, markdown preview and a run affordance. A
**leaf**: it never imports Unity Catalog, query or data-grid. The host pushes in
catalog metadata (`CatalogProvider`), persistence (`FileStore`) and execution
(`onRun`). [`README.md`](./README.md) has the integration details; this page is
the short contract.

## Install & peers

Dependencies include **exact pins** `monaco-editor@0.54.0` +
`monaco-sql-languages@1.2.1`. The pair only works together: never float monaco to
0.55+, and dedupe the host to a single monaco copy. Peers: `react` / `react-dom`
19.

## Host setup

1. **Vite.** Monaco's workers are wired with Vite `?worker` imports, which the
   published `dist/` keeps as-is, so the host must bundle with Vite (no monaco
   plugin needed). Other bundlers need the `getWorkerUrl` setup from
   monaco-sql-languages' docs instead.
2. **Theme.** Render inside ui-kit's `ThemeProvider`; the editor follows its
   light/dark mode.
3. **Tailwind.** Define the ui-kit tokens ([`ui-kit/DESIGN.md`](../ui-kit/DESIGN.md))
   and scan the package:

   ```css
   @source "../node_modules/@open-lakehouse/editor/dist";
   @source "../node_modules/@open-lakehouse/ui-kit/dist";
   ```

## Seams

| Seam | Where | Contract |
| --- | --- | --- |
| Catalog completion | `registerCatalogProvider(p)` | `CatalogProvider`: `catalogs()`, `schemas(c)`, `tables(c, s)`, `columns("c.s.t")`, all async. Without one, completion is keyword-only. `ucCatalogProvider` in unity-catalog-ui is the UC implementation. |
| Persistence | `<EditorSessionProvider fileStore>` (`./session`) | `FileStore`: `readFile`, optional `writeFile` (with `ifMatchEtag`; omit for read-only, which disables autosave), `ConflictError` on stale writes |
| Execution | `onRun` on `MonacoHost` / `EditorSessionProvider` | called on Cmd/Ctrl+Enter with the active buffer; the host runs it and renders results |

## Usage

```tsx
import { ensureMonacoSetup, ensureModel, MonacoHost, registerCatalogProvider } from "@open-lakehouse/editor";

ensureMonacoSetup(); // idempotent; once at startup
registerCatalogProvider(myCatalogProvider);

<MonacoHost
  activeId={path}
  onEditorMount={(monaco) => ensureModel(monaco, path, initialSql)}
  onRun={() => runSql(path)}
/>;
```

For multi-tab file editing with autosave and markdown preview, wrap in
`EditorSessionProvider` from `@open-lakehouse/editor/session` and render
`TabStrip`, `MonacoHost` and `MarkdownPreview`. `@open-lakehouse/editor/fixtures`
has a `fixtureCatalogProvider` for stories and tests.

## Rules

- **Do** bump `monaco-editor` and `monaco-sql-languages` together, and smoke-test
  completion and diagnostics afterwards; a mismatch fails silently (the worker
  falls back to the main thread).
- **Do** keep one `MonacoHost` and swap models on tab change; don't mount an
  editor per tab.
- **Don't** import Unity Catalog, query or data-grid here; provide them through
  the seams.
