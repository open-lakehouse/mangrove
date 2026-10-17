# `node/` — JS/TS workspace

All packages are scoped `@open-lakehouse/*` and live in the root bun workspace.
Each package's README (and, for UI packages, its `DESIGN.md`) owns the detail;
this page is the map.

## Packages

| Package | Dir | Kind | Distribution |
|---|---|---|---|
| `@open-lakehouse/unity-catalog-node` | `client/` | Native (NAPI) Unity Catalog client for Node.js | published (GitHub Packages) |
| `@open-lakehouse/unity-catalog-client` | `unity-catalog-client/` | Typed REST client (`openapi-fetch`, framework-free) + React-Query hooks (`./react`) | published (GitHub Packages) |
| `@open-lakehouse/ui-kit` | `ui-kit/` | Headless, theme-agnostic primitives (Radix + CVA + Tailwind) and the design-token contract | workspace-internal |
| `@open-lakehouse/data-grid` | `data-grid/` | Arrow-backed virtualized result grid | workspace-internal |
| `@open-lakehouse/editor` | `editor/` | Monaco SQL editor with catalog-aware completion | workspace-internal |
| `@open-lakehouse/query` | `query/` | Query-preview seam (`QueryRunner`) | workspace-internal |
| `@open-lakehouse/files` | `files/` | Volume-files seam (`FilesRunner`) | workspace-internal |
| `@open-lakehouse/files-connect` | `files-connect/` | ConnectRPC bindings for the files service | workspace-internal |
| `@open-lakehouse/log-query` | `log-query/` | Delta-log views + log-query seam | workspace-internal |
| `@open-lakehouse/query-wasm` | `query-wasm/` | In-browser (wasm) `QueryRunner` | workspace-internal |
| `@open-lakehouse/files-wasm` | `files-wasm/` | In-browser (wasm) `FilesRunner` | workspace-internal |
| `@open-lakehouse/unity-catalog-ui` | `unity-catalog-ui/` | Unity Catalog browser components (presentation only) | workspace-internal |
| `@open-lakehouse/uc-app` | `app/` | The bundled SPA served by `olai-uc-server`; composes everything | never published |

## Dependency direction

```
ui-kit ◄── data-grid ◄── query ◄──────── query-wasm
   ▲           ▲                              │
   │           └──── log-query ◄──────────────┘
   ├── editor
   │
files ◄── files-wasm ──► files-connect
   ▲
unity-catalog-client
   ▲
unity-catalog-ui ──► (ui-kit, data-grid, editor, files, log-query, query)
   ▲
uc-app ──► everything
```

Leaves never import composed packages. The rules are enforced by
`noRestrictedImports` overrides in [`biome.json`](biome.json): data-grid, query,
files, log-query, editor and the wasm/connect packages must not import
`unity-catalog-ui` (and the editor must not import query or data-grid). Swappable
behavior crosses package boundaries only through seams: `QueryRunner`,
`FilesRunner`, `CatalogProvider`, and the UC client's fetch transport.

## Naming

- `unity-catalog-*` packages are Unity Catalog specific; the rest are
  UC-agnostic building blocks.
- The suffix says what kind of package it is: `-node` for the native Node
  binding, `-client` for the transport-level client, `-ui` for React components,
  `-wasm` / `-connect` for seam implementations.
- Workspace directory names match the unscoped package name (except `client/`,
  which holds the NAPI crate and `app/`).
- Published packages are released by release-plz through a crate each (see
  `.agents/rules/js-release-rule.mdc`).

## Theming

UI packages are headless with respect to theme: the host app defines the CSS
variables listed in [`ui-kit/DESIGN.md`](ui-kit/DESIGN.md) and points Tailwind
at the package sources. [`app/src/globals.css`](app/src/globals.css) is the
reference host.
