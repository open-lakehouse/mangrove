# `node/` — JS/TS workspace

All packages are scoped `@open-lakehouse/*` and live in the root bun workspace.
Each package's README (and, for UI packages, its `DESIGN.md`) owns the detail;
this page is the map.

## Packages

| Package | Dir | Kind | Distribution |
|---|---|---|---|
| `@open-lakehouse/unity-catalog-node` | `client/` | Native (NAPI) Unity Catalog client for Node.js | published (GitHub Packages) |
| `@open-lakehouse/unity-catalog-client` | `unity-catalog-client/` | Typed REST client (`openapi-fetch`, framework-free) + React-Query hooks (`./react`) | published (GitHub Packages) |
| `@open-lakehouse/arrow-grid` | `arrow-grid/` | Arrow-backed virtualized result grid | published (GitHub Packages), [`DESIGN.md`](arrow-grid/DESIGN.md) |
| `@open-lakehouse/editor` | `editor/` | Multi-language Monaco editor (SQL diagnostics + catalog-aware completion, markdown) and a multi-tab file-editing shell | published (GitHub Packages), [`DESIGN.md`](editor/DESIGN.md) |
| `@open-lakehouse/query-runner` | `query-runner/` | Query-preview seam (`QueryRunner`) | published (GitHub Packages), [`DESIGN.md`](query-runner/DESIGN.md) |
| `@open-lakehouse/volume-files` | `volume-files/` | Volume-files seam (`FilesRunner`) | published (GitHub Packages), [`DESIGN.md`](volume-files/DESIGN.md) |
| `@open-lakehouse/files-connect` | `files-connect/` | ConnectRPC bindings for the files service | publish-ready, [`DESIGN.md`](files-connect/DESIGN.md) (private) |
| `@open-lakehouse/delta-log` | `delta-log/` | Delta-log views + log-query seam | published (GitHub Packages), [`DESIGN.md`](delta-log/DESIGN.md) |
| `@open-lakehouse/query-wasm` | `query-wasm/` | In-browser (wasm) `QueryRunner` | workspace-internal |
| `@open-lakehouse/files-wasm` | `files-wasm/` | In-browser (wasm) `FilesRunner` | workspace-internal |
| `@open-lakehouse/unity-catalog-ui` | `unity-catalog-ui/` | Unity Catalog browser components (presentation only) | published (GitHub Packages), [`DESIGN.md`](unity-catalog-ui/DESIGN.md) |
| `@open-lakehouse/uc-app` | `app/` | The bundled SPA served by `olai-uc-server`; composes everything | never published |

The primitives and the design-token contract come from trestle's
[`@open-lakehouse/ui-kit`](https://github.com/open-lakehouse/trestle/tree/main/node/ui-kit),
installed from its GitHub release tarball. The UI packages take it as a peer, so
the host app supplies the one copy.

## Dependency direction

```
ui-kit ◄── arrow-grid ◄── query-runner ◄──── query-wasm
   ▲           ▲                                   │
   │           └──── delta-log ◄───────────────────┘
   ├── editor
   │
volume-files ◄── files-wasm ──► files-connect
   ▲
unity-catalog-client
   ▲
unity-catalog-ui ──► (ui-kit, arrow-grid, editor, volume-files, delta-log, query-runner)
   ▲
uc-app ──► everything
```

Leaves never import composed packages. The rules are enforced by
`noRestrictedImports` overrides in [`biome.json`](biome.json): arrow-grid,
query-runner, volume-files, delta-log, editor and the wasm/connect packages
must not import `unity-catalog-ui` (and the editor must not import query-runner
or arrow-grid). Swappable
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
  the npm section of `.github/CONTRIBUTING.md`).

## Building and consuming

Publish-ready packages build to `dist/` with `bun run build` (all of them:
`bun run build:packages`; CI also lints the packed tarballs with
`bun run pack:check:all`). Their `exports` list a `source` condition first,
pointing at `src/`:

- **Inside this workspace** every consumer resolves `source` (TS
  `customConditions: ["source"]`, Vite `resolve.conditions` in `app/`,
  `bun test --conditions source`), so nothing needs building for dev, typecheck
  or tests.
- **Installed from the registry** consumers get `dist/`.
- **Linked from a sibling checkout** (`file:`, e.g. hydrofoil): either opt into
  the same `source` condition (Vite `resolve.conditions` + TS
  `customConditions`) to consume the TypeScript sources, or run
  `bun run build:packages` here first. Without either, the link resolves to a
  `dist/` that doesn't exist.

Every UI package's `DESIGN.md` is its consumer contract: peers, the host setup
(tokens + Tailwind `@source`), its seams, a usage example, and do/don't rules.
How the packages compose across services (state, seams, extensions) is in
[`ARCHITECTURE.md`](ARCHITECTURE.md).

## Theming

UI packages are headless with respect to theme: the host app defines the CSS
variables listed in [ui-kit's `DESIGN.md`](https://github.com/open-lakehouse/trestle/blob/ui-kit-v0.0.1/node/ui-kit/DESIGN.md) and points Tailwind
at the package sources. [`app/src/globals.css`](app/src/globals.css) is the
reference host.
