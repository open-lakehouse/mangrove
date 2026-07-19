# @open-lakehouse/editor

A reusable, embeddable Monaco editor for the Open Lakehouse UIs: SQL syntax
highlighting, catalog-aware autocompletion, live SQL diagnostics (validation),
markdown preview, and a run affordance — wrapped so a host application can drop
it into a file/volume view.

The package is a **leaf**: it never imports `@open-lakehouse/unity-catalog` (or
`@open-lakehouse/query` / `@open-lakehouse/data-grid`). Instead it exposes seams
the host pushes context into — catalog metadata for completion, a file store for
persistence, and a run callback for execution. This inversion is enforced by a
Biome `noRestrictedImports` rule (`node/biome.json`).

## Two layers

- **`.` (core)** — the Monaco lifecycle, owned cleanly in one place: run-once
  setup + worker wiring (`ensureMonacoSetup`), the model registry, the persistent
  `MonacoHost` editor surface, the theme bridge, the SQL language (highlight +
  diagnostics worker + catalog completion), and the `CatalogProvider` seam.
  Import this alone if you only need an editor surface.
- **`./session`** — a multi-tab file-editing shell built strictly on top of core:
  the tab reducer, per-tab debounced autosave over an injected `FileStore`, and
  markdown preview.
- **`./fixtures`** — a `fixtureCatalogProvider` for stories/tests/dev.

## Integrating into an application (READ THIS)

Most of the editor's worker/build wiring lives inside `ensureMonacoSetup()`, but
there are constraints the **consuming app** must honor:

### 1. Monaco version pin

Use **`monaco-editor@0.52.2` exactly**, not 0.55.x. `monaco-sql-languages@1.1`
is built and tested against 0.52.2 (its own devDep); its `pgsql.worker` relies
on monaco's `editor.worker.js#initialize`, an export removed in 0.55 — on 0.55.x
the SQL worker fails to initialize and completion/diagnostics silently fall back
to (broken) main-thread behavior (cf. DTStack monaco-sql-languages#213). If the
app ships its own monaco, dedupe it to a single 0.52.2 copy — do not let it float
forward. (When the registry exposes monaco-sql-languages ≥1.2, the tested pair
moves to monaco-editor 0.54.0; revisit then.)

### 2. Bundler / workers

The editor wires Monaco's workers with Vite's `?worker` import suffix (base
editor worker + the `pgsql` language worker) via `MonacoEnvironment.getWorker`.
On Vite you need **no monaco plugin** — avoid `vite-plugin-monaco-editor` (stale).
For a workspace-source consumer, widen `server.fs.allow` to span the source. Add
`optimizeDeps: { exclude: ["monaco-editor"] }` only if the dev server logs worker
pre-bundle errors. A non-Vite bundler (webpack) uses `getWorkerUrl` instead — see
monaco-sql-languages' `documents/integrate-esm.md`.

### 3. Catalog completion

Register a `CatalogProvider` before (or after — it's late-bound) the editor
mounts:

```ts
import { registerCatalogProvider } from "@open-lakehouse/editor";
registerCatalogProvider(myCatalogProvider);
```

Without one, completion degrades to keyword-only (it never breaks typing). The
SQL parse runs in the worker; the catalog-aware completion callback runs on the
main thread with the worker's parse context, so async catalog fetches are natural
— there is no worker↔main bridge to build.

### 4. Theme

Render inside `@open-lakehouse/ui-kit`'s `ThemeProvider` so the editor's
`useMonacoTheme` tracks light/dark.

### 5. React

React is a peer dependency. The app must resolve a single copy (dedupe) or hooks
break with "Invalid hook call".

### Known failure modes

- *"Could not create web worker … falling back to main thread"* / *"Missing
  requestHandler"* — a missing/mismatched `getWorker` label, or a
  monaco / monaco-sql-languages version mismatch (see §1).
- The `label` in `getWorker(_, label)` must equal `LanguageIdEnum.PG` (`"pgsql"`)
  or completion silently won't parse.
- Import the `pgsql.contribution` before creating the editor, or the language
  never registers. `ensureMonacoSetup()` handles this for you.
